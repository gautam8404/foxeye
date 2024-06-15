use anyhow::{anyhow, Result};
use std::env;
use tokio::sync::Notify;
use utils::RabbitMQ;

use crate::parser::Parser;

mod config;
mod parser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    println!("Hello, world!");

    let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!(format!("RABBITMQ env not set {e}")))?;

    let amq = RabbitMQ::new(
        &amq_uri,
        "foxeye.parser",
        "consumer.parser",
        "crawler.to.parser",
        "crawler.parser.exchange",
    )
    .await?;

    let parser = Parser::new().await.unwrap();
    parser.send_missing_ids().await?; // call once jic
    let guard = Notify::new();

    amq.consume(&amq.consumer_tag, parser.auto_ack, parser)
        .await?;
    guard.notified().await;

    Ok(())
}
