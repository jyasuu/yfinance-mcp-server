# yfinance-mcp-server

MCP server wrapping [yfinance-rs](https://crates.io/crates/yfinance-rs) v0.9, exposing 30+ Yahoo Finance tools over stdio or HTTP/SSE transport.

## Usage

```json
{
  "mcpServers": {
    "yfinance": {
      "command": "/path/to/yfinance-mcp"
    }
  }
}
```

## Build

```sh
cargo build --release
```

## Configuration (environment variables)

| Variable | Default | Description |
|---|---|---|
| `YFINANCE_CACHE_TTL` | `300` | Cache TTL in seconds |
| `YFINANCE_TIMEOUT` | `30` | Request timeout in seconds |
| `YFINANCE_MAX_RETRIES` | `5` | Max retries on rate-limit / server errors |
| `YFINANCE_RETRY_BASE_DELAY` | `2` | Initial retry backoff delay in seconds |
| `YFINANCE_RETRY_MAX_DELAY` | `30` | Maximum retry backoff delay in seconds |
| `YFINANCE_CORS_ORIGIN` | (none) | CORS origin for HTTP mode (`*` for any, or specific origin) |
| `YFINANCE_REPORTS_DIR` | `./yfinance-reports` | Directory for `generate_report` HTML output |
| `YFINANCE_BASE_URL` | `http://localhost:<port>` | Base URL for report links in HTTP mode |
| `YFINANCE_AI_BASE_URL` | (none) | AI chat completions endpoint for `analyze_news` (e.g. `https://opencode.ai/zen/v1/chat/completions`) |
| `YFINANCE_AI_MODEL` | (none) | Model name for AI analysis (e.g. `big-pickle`) |
| `YFINANCE_AI_API_KEY` | (none) | API key for the AI endpoint (not required by all endpoints) |

## Tools

### Quotes & Info

| Tool | Parameters | Description |
|---|---|---|
| `get_quote` | `symbol` | Current quote (price, change, volume, etc.) |
| `get_fast_info` | `symbol` | Fast-access info snapshot |
| `get_info` | `symbol` | Full instrument metadata |
| `get_profile` | `symbol` | Company profile (sector, industry, description) |
| `get_calendar` | `symbol` | Earnings calendar dates |
| `get_sustainability` | `symbol` | ESG risk scores |
| `get_isin` | `symbol` | ISIN lookup by ticker |

### Historical Data

| Tool | Parameters | Description |
|---|---|---|
| `get_historical_data` | `symbol`, `interval`(opt), `range`(opt) | OHLCV candles over time |
| `download_data` | `symbols`, `interval`(opt), `range`(opt) | Multi-symbol historical download |

### Financials

| Tool | Parameters | Description |
|---|---|---|
| `get_income_statement` | `symbol`, `quarterly`(opt) | Income statement |
| `get_balance_sheet` | `symbol`, `quarterly`(opt) | Balance sheet |
| `get_cashflow` | `symbol`, `quarterly`(opt) | Cash flow statement |
| `get_earnings` | `symbol` | Earnings history |
| `get_earnings_trend` | `symbol` | Earnings trend estimates |
| `get_key_statistics` | `symbol` | Key statistics summary |

### Recommendations & Price Targets

| Tool | Parameters | Description |
|---|---|---|
| `get_recommendations` | `symbol` | Analyst recommendations |
| `get_recommendations_summary` | `symbol` | Recommendations summary breakdown |
| `get_price_target` | `symbol` | Price target data |
| `get_upgrades_downgrades` | `symbol` | Upgrade/downgrade history |

### Holders

| Tool | Parameters | Description |
|---|---|---|
| `get_institutional_holders` | `symbol` | Institutional holders |
| `get_major_holders` | `symbol` | Major holders breakdown |
| `get_insider_transactions` | `symbol` | Recent insider transactions |
| `get_mutual_fund_holders` | `symbol` | Mutual fund holders |
| `get_insider_roster` | `symbol` | Insider roster |
| `get_net_share_purchase_activity` | `symbol` | Net insider share purchase activity |
| `get_corporate_actions` | `symbol` | Corporate actions |

### Batch / Multi-Symbol

| Tool | Parameters | Description |
|---|---|---|
| `get_batch_quotes` | `symbols` | Quotes for multiple symbols |

### Options

| Tool | Parameters | Description |
|---|---|---|
| `get_option_expirations` | `symbol` | Available option expiration dates |
| `get_option_chain` | `symbol`, `date`(opt, YYYY-MM-DD) | Full option chain for a given expiration |

### Search & News

| Tool | Parameters | Description |
|---|---|---|
| `get_news` | `symbol` | Recent news for symbol |
| `analyze_news` | `symbol` | AI-powered news sentiment analysis (requires `YFINANCE_AI_BASE_URL`) |
| `search_tickers` | `query` | Search tickers by keyword |

### Reports

| Tool | Parameters | Description |
|---|---|---|
| `generate_report` | `symbol`, `range`(opt) | Generate HTML stock summary report with Tailwind CSS |

## Output Format

Every tool response includes a JSON block followed by a Markdown table:

```
{"symbol": "AAPL", "price": 150.25, ...}

| Field | Value |
|---|---|
| symbol | AAPL |
| price | 150.25 |
| ... | ... |
```

## Docker

```sh
docker pull ghcr.io/jyasuu/yfinance-mcp:latest
docker run ghcr.io/jyasuu/yfinance-mcp:latest
```

Images are published on tag (semver + `latest`) and on `main` branch pushes.

## AI News Analysis

The `analyze_news` tool fetches news for a symbol and sends it to an AI endpoint for sentiment/thematic analysis. Configure via environment variables:

```sh
YFINANCE_AI_BASE_URL="https://opencode.ai/zen/v1/chat/completions" \
YFINANCE_AI_MODEL="big-pickle" \
./yfinance-mcp
```

The tool is **hidden** from the tool list when `YFINANCE_AI_BASE_URL` is not set. API key is optional (not all endpoints require it).

### opencode.json

```json
"yfinance": {
  "type": "local",
  "command": ["./yfinance-mcp/target/release/yfinance-mcp"],
  "enabled": true,
  "environment": {
    "YFINANCE_AI_BASE_URL": "https://opencode.ai/zen/v1/chat/completions",
    "YFINANCE_AI_MODEL": "big-pickle"
  }
}
```

## Stock Analysis Skill

An opencode skill is included at `.opencode/skills/stock-analysis/SKILL.md`. It provides a structured stock analysis workflow: symbol identification → `generate_report` → file path/URL + brief summary. Triggered on keywords like "analyze", "trend", "outlook".

## HTTP / SSE Transport

Set `YFINANCE_HTTP_PORT` to run as an HTTP+SSE server (Streamable HTTP):

```sh
YFINANCE_HTTP_PORT=8080 ./yfinance-mcp
```

**Windows** (auto-downloads binary if missing):
```cmd
run-http.cmd
```
```powershell
.\run-http.ps1
```

Listens on `0.0.0.0:<port>` with SSE endpoint at `/mcp` and reports served at `/reports/`. Accept header must include both `application/json` and `text/event-stream`.

Example client config:

```json
{
  "mcpServers": {
    "yfinance": {
      "type": "remote",
      "url": "http://localhost:8080/mcp"
    }
  }
}
```