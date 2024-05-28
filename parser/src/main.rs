use anyhow::{anyhow, Result};
use std::env;
use tokio::sync::Notify;
use tracing::info;
use utils::RabbitMQ;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    println!("Hello, world!");

    let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!("RABBITMQ env not set"))?;

    let amq = RabbitMQ::new(&amq_uri, "foxeye.crawler", "foxeye.parser").await?;
    info!("starting consumer");

    amq.consume(&amq.consumer_tag, true)
        .await
        .expect("TODO: panic message");
    let guard = Notify::new();
    guard.notified().await;

    Ok(())
}
