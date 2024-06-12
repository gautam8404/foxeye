use std::env;

use anyhow::{anyhow, Error, Result};
use scraper::{Html, Selector};
use sqlx::Acquire;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{error, info};
use ulid::Ulid;
use url::Url;

use crate::config::SiteConfig;
use db::Db;
use utils::amqprs::channel::{BasicAckArguments, Channel};
use utils::amqprs::{BasicProperties, Deliver};
use utils::async_trait::async_trait;
use utils::{amqprs::consumer::AsyncConsumer, async_trait, CrawlMessage, RabbitMQ};

pub struct Parser {
    db: Db,
    amq: RabbitMQ,
    config: SiteConfig,
    pub auto_ack: bool,
}

impl Parser {
    pub async fn new() -> Result<Self> {
        let db = Db::new(5).await?;

        let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!("RABBITMQ env not set"))?;
        let amq = RabbitMQ::new(
            &amq_uri,
            "foxeye.embedder",
            "consumer.embedder",
            "parser.to.embedder",
            "parser.embedder.exchange",
        )
        .await?;
        let config = SiteConfig::load_config()?;

        Ok(Self {
            db,
            amq,
            config,
            auto_ack: true,
        })
    }

    async fn get_document(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let doc = self.db.get_cache(id).await?;

        Ok(doc)
    }

    fn parse_document(&self, doc: String, host: Url) -> Result<(Vec<Url>, String)> {
        let mut document = Html::parse_document(&doc);

        let script_selector = Selector::parse("script").unwrap();
        let style_selector = Selector::parse("style").unwrap();
        let body = Selector::parse("body").unwrap();
        let title = Selector::parse("title").unwrap();
        let href = Selector::parse("a").unwrap();

        let ids = document
            .select(&script_selector)
            .chain(document.select(&style_selector))
            .map(|p| p.id())
            .collect::<Vec<_>>();

        for id in ids {
            if let Some(node) = &mut document.tree.get_mut(id) {
                node.detach();
            }
        }

        let title = document.select(&title).next();
        let body = document.select(&body).next();
        let url_hrefs = document
            .select(&href)
            .filter_map(|e| e.value().attr("href"))
            .map(|m| m.to_string())
            .collect::<Vec<_>>();

        let title = if let Some(title) = title {
            title.text().collect::<Vec<_>>().join(" ")
        } else {
            String::new()
        };

        let body = if let Some(body) = body {
            body.text().collect::<Vec<_>>().join(" ")
        } else {
            String::new()
        };

        let mut urls = vec![];

        for url in url_hrefs {
            if url.starts_with("/") {
                if let Ok(u) = host.join(&url) {
                    urls.push(u);
                    continue;
                }
            }

            if let Ok(u) = Url::parse(&url) {
                urls.push(u);
            }
        }

        if body.is_empty() {
            return Err(Error::msg("parse_document: body not found"));
        }

        Ok((urls, format!("{title} {body}")))
    }

    async fn save_urls(&self, urls: Vec<Url>, depth: i32) -> Result<()> {
        let (urls, hosts): (Vec<_>, Vec<_>) = urls
            .iter()
            .filter_map(|u| {
                if let Some(host) = u.host() {
                    if self.config.is_allowed(host.to_string()) {
                        return Some((u.to_string(), host.to_string()));
                    }
                }
                None
            })
            .unzip();

        let depths = vec![depth + 1; urls.len()];

        let mut pool = self.db.get_pg().await?;

        let res = sqlx::query!(
            "
            INSERT INTO crawler_queue (url, host, depth)
                SELECT * FROM 
                UNNEST($1::text[], $2::text[], $3::int[])
                ON CONFLICT DO NOTHING",
            &urls[..],
            &hosts[..],
            &depths[..]
        )
        .execute(pool.acquire().await?)
        .await
        .map_err(|e| Error::msg(format!("save_urls: error while saving url {e}")))?;

        info!(
            "wrote {} urls to queue, rows affected: {}",
            urls.len(),
            res.rows_affected()
        );

        Ok(())
    }

    async fn save_document(&self, doc: String, url: Url) -> Result<String> {
        let mut pool = self.db.get_pg().await?;
        let id = Ulid::new().to_string();
        let url = url.to_string();

        let rec = sqlx::query!(
            r#"
            INSERT INTO document (doc_id, url, content)
            VALUES ($1, $2, $3)
            ON CONFLICT (url)
            DO UPDATE SET content=$2
            RETURNING doc_id 
            "#,
            id,
            url,
            doc
        )
        .fetch_one(pool.acquire().await?)
        .await?;

        info!("save_document: saved document {id} to db");
        Ok(rec.doc_id)
    }

    pub async fn parse(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::msg("parse: redis id is empty"));
        }
        let doc = self.get_document(id).await?;

        if doc.is_none() {
            return Err(Error::msg(format!("parse: document not found for id {id}")));
        }

        let crawl_message = serde_json::from_slice::<CrawlMessage>(&doc.unwrap())?;
        let host = Url::parse(&crawl_message.url)?;

        let (urls, doc) = self.parse_document(crawl_message.content, host.clone())?;

        let id = self.save_document(doc, host).await?;
        self.save_urls(urls, crawl_message.depth as i32).await?;
        info!("sending {id} to embedder");
        self.amq.publish(id).await?;
        Ok(())
    }
}

#[async_trait]
impl AsyncConsumer for Parser {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // ack explicitly if manual ack
        if !self.auto_ack {
            info!("ack to delivery {} on channel {}", deliver, channel);
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).await.unwrap();
        }

        let id = String::from_utf8(content).unwrap();
        info!("received id from crawler, parsing now {id}");
        let now = Instant::now();
        if let Err(e) = self.parse(&id).await {
            error!("amq consumer::parser error while parsing id {id}: {e}");
        } else {
            info!("parsed {id} in {}", now.elapsed().as_secs_f32());
        }
    }
}