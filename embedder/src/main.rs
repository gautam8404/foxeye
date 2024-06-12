use crate::embedder::Embedder;

mod embed;
mod embedder;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let embedder = Embedder::new().await.unwrap();

    embedder.embed_loop().await;
}
