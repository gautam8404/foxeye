mod config;
mod crawler;
mod robots;

use crate::crawler::Crawler;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("starting crawler");

    let mut crawler = Crawler::new().await.unwrap();
    crawler.crawl_loop().await;
}
