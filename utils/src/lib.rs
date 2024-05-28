pub mod amq;

pub use amq::RabbitMQ;

use serde::Serialize;
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

#[derive(Debug, Clone, Serialize)]
pub struct CrawlMessage {
    id: String,
    content: String,
    depth: u32,
}

impl CrawlMessage {
    pub fn new(id: String, content: String, depth: u32) -> CrawlMessage {
        CrawlMessage { id, content, depth }
    }
}
