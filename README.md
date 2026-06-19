# yfinance-mcp-server

MCP server wrapping [yfinance-rs](https://crates.io/crates/yfinance-rs) v0.9, exposing 25+ Yahoo Finance tools over stdio transport.

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
| `YFINANCE_MAX_RETRIES` | `3` | Max retries on failure |

## Tools

### Quotes & Info

| Tool | Parameters | Description |
|---|---|---|
| `yf_quote` | `symbol` | Current quote (price, change, volume, etc.) |
| `yf_fast_info` | `symbol` | Fast-access info snapshot |
| `yf_info` | `symbol` | Full instrument metadata |
| `yf_profile` | `symbol` | Company profile (sector, industry, description) |
| `yf_calendar` | `symbol` | Earnings calendar dates |
| `yf_sustainability` | `symbol` | ESG risk scores |
| `yf_isin` | `symbol` | ISIN lookup by ticker |

### Historical Data

| Tool | Parameters | Description |
|---|---|---|
| `yf_historical_data` | `symbol`, `interval`(opt), `range`(opt) | OHLCV candles over time |
| `yf_download_data` | `symbols`, `interval`(opt), `range`(opt) | Multi-symbol historical download |

### Financials

| Tool | Parameters | Description |
|---|---|---|
| `yf_income_stmt` | `symbol` | Income statement |
| `yf_balance_sheet` | `symbol` | Balance sheet |
| `yf_cashflow` | `symbol` | Cash flow statement |
| `yf_earnings` | `symbol` | Earnings history |
| `yf_earnings_trend` | `symbol` | Earnings trend estimates |
| `yf_key_statistics` | `symbol` | Key statistics summary |

### Recommendations & Price Targets

| Tool | Parameters | Description |
|---|---|---|
| `yf_recommendations` | `symbol` | Analyst recommendations |
| `yf_recommendations_summary` | `symbol` | Recommendations summary breakdown |
| `yf_price_target` | `symbol` | Price target data |
| `yf_upgrades_downgrades` | `symbol` | Upgrade/downgrade history |

### Holders

| Tool | Parameters | Description |
|---|---|---|
| `yf_institutional_holders` | `symbol` | Institutional holders |
| `yf_major_holders` | `symbol` | Major holders breakdown |
| `yf_insider_holders` | `symbol` | Insider holdings |
| `yf_mutual_fund_holders` | `symbol` | Mutual fund holders |
| `yf_insider_roster` | `symbol` | Insider roster |
| `yf_net_share_purchase` | `symbol` | Net insider share purchase activity |
| `yf_corporate_actions` | `symbol` | Corporate actions |

### Batch / Multi-Symbol

| Tool | Parameters | Description |
|---|---|---|
| `yf_batch_quotes` | `symbols` (comma-sep) | Quotes for multiple symbols |

### Options

| Tool | Parameters | Description |
|---|---|---|
| `yf_option_expirations` | `symbol` | Available option expiration dates |
| `yf_option_chain` | `symbol`, `date` (YYYY-MM-DD) | Full option chain for a given expiration |

### Search & News

| Tool | Parameters | Description |
|---|---|---|
| `yf_news` | `symbol` | Recent news for symbol |
| `yf_search_tickers` | `query` | Search tickers by keyword |

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

## Remote MCP (workaround)

This server uses `rmcp` v0.16 with stdio transport. For HTTP/SSE transport, wrap with the [MCP Inspector](https://github.com/modelcontextprotocol/inspector):

```sh
npx @modelcontextprotocol/inspector /path/to/yfinance-mcp
```