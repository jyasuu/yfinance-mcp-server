mod format;
mod server;

use std::sync::Arc;
use std::time::Duration;

use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

use yfinance_rs::YfClientBuilder;

use crate::server::YFinanceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

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

    let client = Arc::new(client);
    let server = YFinanceServer::new(client);

    tracing::info!("Starting yfinance MCP server over stdio...");
    server.serve(stdio()).await?.waiting().await?;

    Ok(())
}
