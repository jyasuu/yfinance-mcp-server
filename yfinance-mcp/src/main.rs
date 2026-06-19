mod format;
mod server;

use std::path::PathBuf;
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
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use yfinance_rs::YfClientBuilder;

use crate::server::YFinanceServer;

async fn ensure_mcp_accept(
    mut req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
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
        .unwrap_or(5);

    let retry_base = std::env::var("YFINANCE_RETRY_BASE_DELAY")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(2));

    let retry_max = std::env::var("YFINANCE_RETRY_MAX_DELAY")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(30));

    use yfinance_rs::core::client::Backoff;
    let retry_config = yfinance_rs::core::client::RetryConfig {
        max_retries,
        backoff: Backoff::Exponential {
            base: retry_base,
            factor: 2.0,
            max: retry_max,
            jitter: true,
        },
        ..Default::default()
    };

    let client = YfClientBuilder::default()
        .timeout(timeout)
        .cache_ttl(cache_ttl)
        .retry_config(retry_config)
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

    let reports_dir = std::env::var("YFINANCE_REPORTS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./yfinance-reports"));
    tokio::fs::create_dir_all(&reports_dir).await?;
    tracing::info!("Reports directory: {}", reports_dir.display());

    if let Some(port) = std::env::var("YFINANCE_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
    {
        let base_url = std::env::var("YFINANCE_BASE_URL")
            .ok()
            .unwrap_or_else(|| format!("http://localhost:{}", port));
        let http_base_url = Some(base_url.clone());

        let service = StreamableHttpService::new(
            {
                let client = client.clone();
                let reports_dir = reports_dir.clone();
                let http_base_url = http_base_url.clone();
                move || {
                    Ok(YFinanceServer::new(
                        client.clone(),
                        reports_dir.clone(),
                        http_base_url.clone(),
                    ))
                }
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
            .nest_service("/reports", ServeDir::new(reports_dir))
            .layer(middleware::from_fn(ensure_mcp_accept));

        if let Ok(cors_origin) = std::env::var("YFINANCE_CORS_ORIGIN") {
            let cors = if cors_origin == "*" {
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(Any)
            } else {
                CorsLayer::new()
                    .allow_origin(
                        cors_origin
                            .parse::<HeaderValue>()
                            .expect("invalid CORS origin"),
                    )
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(Any)
            };
            router = router.layer(cors);
            tracing::info!("CORS enabled with origin: {}", cors_origin);
        }

        let addr = format!("0.0.0.0:{}", port);
        tracing::info!("Starting yfinance MCP server over HTTP on {}", addr);
        tracing::info!("Reports available at {}/reports/", base_url);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, router).await?;
    } else {
        let server = YFinanceServer::new(client, reports_dir, None);
        tracing::info!("Starting yfinance MCP server over stdio...");
        server.serve(stdio()).await?.waiting().await?;
    }

    Ok(())
}
