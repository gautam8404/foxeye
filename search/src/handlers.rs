use crate::svc::{SearchInput, SearchResult, Searcher};
use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use tracing::log::info;

pub async fn web_root() -> &'static str {
    "Hello, World! from embedder"
}

// #[axum::debug_handler]
pub async fn search_handler(
    State(searcher): State<Arc<Mutex<Searcher>>>,
    Json(input): Json<SearchInput>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    info!("received request {:#?}", input);
    let mut search = searcher.lock().await;
    let res = search.search(input).await.map_err(|e| {
        error!("search_handler: error while searching: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json::from(res))
}
