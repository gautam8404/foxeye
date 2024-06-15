use crate::embedder::Embedder;
use anyhow::{anyhow, Result};
use std::env;
use tokio::sync::Notify;
use utils::RabbitMQ;

mod embed;
mod embedder;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!(format!("RABBITMQ env not set {e}")))?;

    let amq = RabbitMQ::new(
        &amq_uri,
        "foxeye.embedder",
        "consumer.embedder",
        "parser.to.embedder",
        "parser.embedder.exchange",
    )
    .await?;

    let embedder = Embedder::new().await?;
    let guard = Notify::new();

    amq.consume(&amq.consumer_tag, embedder.auto_ack, embedder)
        .await?;

    guard.notified().await;

    Ok(())
}
