mod handlers;
mod svc;

use crate::handlers::*;
use crate::svc::Searcher;
use anyhow::Result;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("init search");
    let searcher = Arc::new(AsyncMutex::new(Searcher::new().await?));

    let router = Router::new()
        .route("/", get(web_root))
        .route("/search", get(search_handler))
        .with_state(searcher);

    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    info!("searcher is listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, router).await?;

    Ok(())
}
