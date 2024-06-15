mod handlers;
mod svc;

use crate::handlers::*;
use crate::svc::Searcher;
use anyhow::Result;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::CorsLayer;
use tower_http::trace::{TraceLayer};
use tracing::{error, info, info_span, Span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "RUST_LOG=info,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("init search");
    let searcher = Arc::new(AsyncMutex::new(Searcher::new().await?));
    let cors = CorsLayer::permissive();

    let router = Router::new()
        .route("/", get(web_root))
        .route("/search", post(search_handler))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|request: &Request<_>, _span: &Span| {
                    info!("received request {} {} with body {:?}", request.uri(), request.method(), request.body());
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                        // ...
                        error!("request failed with error {error}");
                        span.record(
                            "some_other_field",
                            format!(
                                "encountered error: {} latency {}",
                                error,
                                latency.as_secs_f32()
                            ),
                        );
                    },
                ),
        )
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
