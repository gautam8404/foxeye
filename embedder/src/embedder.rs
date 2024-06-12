use crate::embed::candle_embed::{CandleEmbed, CandleEmbedBuilder};
use crate::embed::models::Model;
use anyhow::{anyhow, Result};
use candle::Device;
use db::Db;
use std::env;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info};
use utils::RabbitMQ;

pub struct Embedder {
    db: Db,
    amq: RabbitMQ,
    embed: CandleEmbed,
}

impl Embedder {
    pub async fn new() -> Result<Self> {
        let db = Db::new(5).await?;
        let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!("RABBITMQ env not set"))?;
        let amq = RabbitMQ::new(&amq_uri, "foxeye.parser", "foxeye.embedder").await?;
        let embed = CandleEmbedBuilder::new()
            .padding(true)
            .model(Model::UaeLargeV1)
            .mean_pooling(false)
            .device(Device::new_cuda(0).unwrap())
            .build()
            .await?;

        Ok(Self { db, amq, embed })
    }

    pub async fn embed_loop(&self) {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        self.amq
            .basic_consume(&self.amq.consumer_tag, true, tx)
            .await
            .expect("failed to init queue");

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    if let Some(msg) = msg {
                        info!("message received from queue: {}", msg);
                        // if let Err(e) = self.parse(&msg).await {
                        //     error!("parser_loop::parse: error while parsing id: {msg} {e}");
                        // }
                    }
                }
                _ = shutdown_signal() => break,
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
