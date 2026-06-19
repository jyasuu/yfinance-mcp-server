mod format;
mod server;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, HeaderValue, Method};
use axum::middleware;
use axum::response::Response;
use rmcp::transport::{
    streamable_http_server::session::local::LocalSessionManager, StreamableHttpServerConfig,
    StreamableHttpService,
};
use rmcp::{transport::stdio, ServiceExt};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use yfinance_rs::YfClientBuilder;

use crate::server::YFinanceServer;

async fn ensure_mcp_accept(mut req: axum::http::Request<axum::body::Body>, next: middleware::Next) -> Response {
    if req.uri().path().starts_with("/mcp") && !req.headers().contains_key(header::ACCEPT) {
        req.headers_mut().insert(
            header::ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        );
    }
    next.run(req).await
}

fn build_client() -> Result<Arc<yfinance_rs::YfClient>, Box<dyn std::error::Error>> {
    let cache_ttl = std::env::var("YFINANCE_CACHE_TTL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(300));

    let timeout = std::env::var("YFINANCE_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(30));

    let max_retries = std::env::var("YFINANCE_MAX_RETRIES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(3);

    let client = YfClientBuilder::default()
        .timeout(timeout)
        .cache_ttl(cache_ttl)
        .retry_config(yfinance_rs::core::client::RetryConfig {
            max_retries,
            ..Default::default()
        })
        .build()?;

    Ok(Arc::new(client))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let client = build_client()?;

    if let Some(port) = std::env::var("YFINANCE_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
    {
        let service = StreamableHttpService::new(
            {
                let client = client.clone();
                move || Ok(YFinanceServer::new(client.clone()))
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: Some(Duration::from_secs(15)),
                ..Default::default()
            },
        );
        let mut router = axum::Router::new()
            .nest_service("/mcp", service)
            .layer(middleware::from_fn(ensure_mcp_accept));

        if let Ok(cors_origin) = std::env::var("YFINANCE_CORS_ORIGIN") {
            let cors = if cors_origin == "*" {
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(Any)
            } else {
                CorsLayer::new()
                    .allow_origin(cors_origin.parse::<HeaderValue>().expect("invalid CORS origin"))
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(Any)
            };
            router = router.layer(cors);
            tracing::info!("CORS enabled with origin: {}", cors_origin);
        }

        let addr = format!("0.0.0.0:{}", port);
        tracing::info!("Starting yfinance MCP server over HTTP on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, router).await?;
    } else {
        let server = YFinanceServer::new(client);
        tracing::info!("Starting yfinance MCP server over stdio...");
        server.serve(stdio()).await?.waiting().await?;
    }

    Ok(())
}
