use anyhow::{anyhow, Result};
use std::env;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::info;
use utils::RabbitMQ;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    println!("Hello, world!");

    let amq_uri = env::var("RABBITMQ").map_err(|e| anyhow!("RABBITMQ env not set"))?;

    let amq = RabbitMQ::new(&amq_uri, "foxeye.crawler", "foxeye.parser").await?;
    info!("starting consumer");

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let asd = "hel;p";

    // let a = tokio::task::spawn(async move {
    //     amq.consume(&amq.consumer_tag, true, tx.clone())
    //         .await
    // }).await;

    // let a =      amq.consume(&amq.consumer_tag, true, tx.clone()).await;
    //
    //

    /*
    println!("{a:?}");
    loop {
        if let Ok(msg) = rx.try_recv() {
            println!("message recived from queue: {msg}");
        }
    }
    */

    amq.consume(&amq.consumer_tag, true, tx)
        .await
        .expect("TODO: panic message");

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    println!("message received from queue: {}", msg);
                }
            }
            _ = shutdown_signal() => break,
        }
    }

    amq.connection.close().await?;
    amq.channel.close().await?;
    info!("Shutting down .. bye bye ..");

    Ok(())
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
