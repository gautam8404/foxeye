pub mod amq;

pub use amq::RabbitMQ;
pub use amqprs;
pub use async_trait;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlUrl {
    pub url: Url,
    pub depth: u32,
}

impl CrawlUrl {
    pub fn new(url: Url, depth: u32) -> CrawlUrl {
        CrawlUrl { url, depth }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlMessage {
    pub id: String,
    pub content: String,
    pub depth: u32,
    pub url: String,
}

impl CrawlMessage {
    pub fn new(id: String, content: String, depth: u32, url: String) -> CrawlMessage {
        CrawlMessage {
            id,
            content,
            depth,
            url,
        }
    }
}
