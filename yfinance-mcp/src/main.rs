mod format;
mod server;

use std::sync::Arc;
use std::time::Duration;

use rmcp::{ServiceExt, transport::stdio};
use rmcp::transport::{
    StreamableHttpService, StreamableHttpServerConfig,
    streamable_http_server::session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

use yfinance_rs::YfClientBuilder;

use crate::server::YFinanceServer;

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
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let http_port = std::env::var("YFINANCE_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok());

    if let Some(port) = http_port {
        let client = build_client()?;
        let ct = CancellationToken::new();
        let service = StreamableHttpService::new(
            {
                let client = client.clone();
                move || Ok(YFinanceServer::new(client.clone()))
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: Some(Duration::from_secs(15)),
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );
        let router = axum::Router::new().nest_service("/mcp", service);
        let addr = format!("0.0.0.0:{}", port);
        tracing::info!("Starting yfinance MCP server over HTTP on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, router).await?;
    } else {
        let client = build_client()?;
        let server = YFinanceServer::new(client);
        tracing::info!("Starting yfinance MCP server over stdio...");
        server.serve(stdio()).await?.waiting().await?;
    }

    Ok(())
}
