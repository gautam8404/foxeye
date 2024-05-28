use std::collections::HashMap;
use std::env;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use sqlx::Acquire;
use tracing::{error, info, warn};
use ulid::Ulid;
use url::Url;

use db::Db;

use crate::config::{Sites, SitesConfig, FOXEYE_USER_AGENT};
use utils::{CrawlMessage, CrawlUrl, RabbitMQ};

#[derive(Debug, Clone)]
pub struct Crawler {
    client: Client,
    db: Db,
    site_map: HashMap<String, Sites>, // host url -> site config
    url_queue: Vec<CrawlUrl>,         // url queue
    amq: RabbitMQ,
}

impl Crawler {
    const MAX_QUEUE_SIZE: usize = 100;
    const MAX_DEPTH: u32 = 10;
    pub async fn new() -> Result<Crawler> {
        let config = SitesConfig::load_config().await?;
        info!("sites loaded: {}", config.len());

        let mut site_map = HashMap::new();
        let mut url_queue = vec![];

        for site in config {
            if let Some(host) = site.url.host() {
                let crl = CrawlUrl {
                    url: site.url.clone(),
                    depth: 0,
                };
                url_queue.push(crl);
                site_map.insert(host.to_string(), site);
            }
        }
        let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!("RABBITMQ env not set"))?;

        let amq = RabbitMQ::new(&amq_uri, "foxeye.crawler", "foxeye.parser").await?;

        Ok(Crawler {
            client: Client::new(),
            db: Db::new(5).await?,
            site_map,
            url_queue,
            amq,
        })
    }

    // crawling strat
    // check if self.url_queue is empty (max 100 urls are allowed at once)
    // if its empty get url from db url queue using DELETE FROM url RETURNING * LIMIT 100;
    // check if url host is in self.site_map which is a hashmap of configured site to be crawled
    // check if urls is allowed according to robots.txt
    // check timer if rate limit has passed
    // check if url depth has reached
    // check if url is in redis if yes skip
    // else send request to url and get response
    // save url in redis cache for 7 days
    // get all urls from the page
    // parse url and append host to them if they start from "/"
    // save parse url in db url queue
    // save response content in redis and assign a key
    // send key to parser using rabbitmq

    async fn populate_urls(&mut self) -> Result<()> {
        if self.url_queue.len() >= 100 {
            return Ok(());
        }

        let num_urls = Self::MAX_QUEUE_SIZE - self.url_queue.len();
        let stmt = format!(r#"
            DELETE FROM crawler_queue
            WHERE url_id IN (
            SELECT url_id
            FROM crawler_queue
            ORDER BY created_at ASC
            LIMIT {} )
            RETURNING url, depth
        "#, Self::MAX_QUEUE_SIZE);

        let mut pool = self.db.get_pg().await?;
        let urls = sqlx::query_as::<_, (String, i32)>(&stmt)
            .bind(num_urls as i8)
            .fetch_all(pool.acquire().await?)
            .await?;
        
        let urls = urls.iter().filter_map(|(url, depth)| {
            if let Ok(url) = Url::parse(url) {
                return Some(CrawlUrl {
                    url,
                    depth: *depth as u32,
                });
            }

            None
        });

        self.url_queue.extend(urls);

        Ok(())
    }

    pub async fn crawl_loop(&mut self) {
        loop {
            if self.url_queue.is_empty() {
                info!("crawl_loop: url queue is empty, trying to populate");
                if let Err(e) = self.populate_urls().await {
                    error!("crawl: error while populating urls {}", e)
                }
            }
            let mut index = 0;
            while index < self.url_queue.len() {
                let crl = self.url_queue.get(index).unwrap().clone();
                let url_str = crl.url.to_string();

                info!(
                    "crawl_loop: crawling url {} on depth {}",
                    url_str, crl.depth
                );

                if let Err(e) = self.crawl(crl.url, crl.depth).await {
                    error!(
                        "crawl_loop: error while crawling url: {} on depth: {}, e: {}",
                        url_str, crl.depth, e
                    );
                }

                index += 1
            }
            
            self.url_queue.clear();
            tokio::time::sleep(Duration::new(3,0)).await;
        }
    }

    pub async fn check_valid(&mut self, url: &Url, depth: u32) -> Result<(bool, &str)> {
        let host = url.host();
        let key = url.to_string();

        if host.is_none() {
            return Ok((false, "No host found"));
        }
        let host = host.clone().unwrap().to_string();

        // check if url host is in self.site_map
        let site = self.site_map.get(&host);
        if site.is_none() {
            return Ok((false, "host not found in configured sites"));
        }
        let mut site = site.unwrap().to_owned();

        // check if url depth has reached
        if let Some(site_depth) = site.depth {
            if depth >= site_depth {
                return Ok((false, "site depth reached"));
            }
        }

        // check if urls is allowed according to robots.txt
        if !site.is_allowed(url.path()) {
            return Ok((false, "not allowed by robots.txt"));
        }

        // check if timer rate limit has passed
        if !site.timer.can_send() {
            // add url back to queue
            self.url_queue.push(CrawlUrl::new(url.clone(), depth));
            return Ok((false, "rate limit exceeded"))
        }

        // check if url is in redis if yes skip
        let exists = self.db.exists(&key).await?;
        if exists {
            return Ok((false, "url exists in redis"));
        }

        Ok((true, "all checks passed"))
    }

    pub async fn crawl(&mut self, url: Url, depth: u32) -> Result<()> {
        let (valid, reason) = self.check_valid(&url, depth).await?;
        if !valid {
            warn!("crawl: invalid url {url} at depth {depth}, reason: {reason}");
            return Ok(());
        }

        // send request
        let res = self
            .client
            .get(url)
            .header(USER_AGENT, FOXEYE_USER_AGENT)
            .send()
            .await?;

        let res = res.text().await?;
        let id = Ulid::new().to_string();

        let message = CrawlMessage::new(id.clone(), res, depth);
        let message = serde_json::to_string(&message)?;
        let mins_10 = 60 * 10;

        self.db
            .set_cache(&id, message.into_bytes(), Some(mins_10))
            .await?;
        info!("saved crawled content in redis with id: {id}");
        self.amq.publish(id).await?;
        info!("sent id in amq");

        Ok(())
    }
}
