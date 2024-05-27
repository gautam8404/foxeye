mod amq;
mod config;
mod crawler;
mod robots;

use crate::crawler::Crawler;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("yo");

    let a = Crawler::new().await;
    println!("{:#?}", a);
}
