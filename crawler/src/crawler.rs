use crate::amq::RabbitMQ;
use crate::config::{Sites, SitesConfig};
use anyhow::{anyhow, Result};
use db::Db;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::env;
use tracing::{error, info};
use url::Url;

#[derive(Debug, Clone)]
pub struct Crawler {
    client: Client,
    db: Db,
    site_map: HashMap<String, Sites>, // host url -> site config
    url_queue: Vec<Url>,              // url queue
    amq: RabbitMQ,
}

impl Crawler {
    pub async fn new() -> Result<Crawler> {
        let config = SitesConfig::load_config().await?;
        info!("sites loaded: {}", config.len());

        let mut site_map = HashMap::new();
        let mut url_queue = vec![];

        for site in config {
            if let Some(host) = site.url.host() {
                url_queue.push(site.url.clone());
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
    // check if url is in redis if yes skip
    // else send request to url and get response
    // save url in redis cache for 7 days
    // get all urls from the page
    // parse url and append host to them if they start from "/"
    // save parse url in db url queue
    // send response content to parser using rabbitmq
}
