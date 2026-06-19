use std::sync::Arc;

use rmcp::{
    handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters, model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};

use yfinance_rs::{Interval, Range, Ticker, YfClient};

use crate::format;

fn parse_range(s: Option<&str>) -> Range {
    match s {
        Some("1d") | None => Range::D1,
        Some("5d") => Range::D5,
        Some("1mo") => Range::M1,
        Some("3mo") => Range::M3,
        Some("6mo") => Range::M6,
        Some("ytd") => Range::Ytd,
        Some("1y") => Range::Y1,
        Some("2y") => Range::Y2,
        Some("5y") => Range::Y5,
        Some("10y") => Range::Y10,
        Some("max") => Range::Max,
        Some(_) => Range::M6,
    }
}

fn parse_interval(s: Option<&str>) -> Option<Interval> {
    match s {
        Some("1m") => Some(Interval::I1m),
        Some("2m") => Some(Interval::I2m),
        Some("5m") => Some(Interval::I5m),
        Some("15m") => Some(Interval::I15m),
        Some("30m") => Some(Interval::I30m),
        Some("1h") => Some(Interval::I1h),
        Some("1d") | None => Some(Interval::D1),
        Some("5d") => Some(Interval::D5),
        Some("1wk") => Some(Interval::W1),
        Some("1mo") => Some(Interval::M1),
        Some("3mo") => Some(Interval::M3),
        Some(_) => None,
    }
}

fn json_val<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_default()
}

fn fmt_action_date(action: &yfinance_rs::Action) -> String {
    match action {
        yfinance_rs::Action::Dividend { date, .. }
        | yfinance_rs::Action::Split { date, .. }
        | yfinance_rs::Action::CapitalGain { date, .. } => date.to_string(),
        _ => "N/A".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct YFinanceServer {
    client: Arc<YfClient>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl YFinanceServer {
    pub fn new(client: Arc<YfClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    fn ticker(&self, symbol: &str) -> Ticker {
        Ticker::new(&self.client, symbol)
    }

    async fn exec<T>(
        f: impl std::future::Future<Output = Result<T, yfinance_rs::core::error::YfError>>,
    ) -> Result<T, McpError> {
        f.await
            .map_err(|e| McpError::internal_error(e.to_string(), None::<serde_json::Value>))
    }

    // ── Core Market Data ──────────────────────────────────────────

    #[tool(description = "Get a real-time quote for a ticker symbol")]
    async fn get_quote(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let quote = Self::exec(ticker.quote()).await?;
        let j = json_val(&quote);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Quote: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get a lightweight quote snapshot with moving averages")]
    async fn get_fast_info(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let info = Self::exec(ticker.fast_info()).await?;
        let j = json_val(&info);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Fast Info: {}", args.symbol), &j),
        )]))
    }

    #[tool(
        description = "Get comprehensive aggregate info for a ticker (quote + profile + analysis)"
    )]
    async fn get_info(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let info = Self::exec(ticker.info()).await?;
        let j = json_val(&info);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Info: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get historical OHLCV price data for a ticker")]
    async fn get_historical_data(
        &self,
        Parameters(args): Parameters<HistoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let range = Some(parse_range(args.range.as_deref()));
        let interval = parse_interval(args.interval.as_deref());
        let candles =
            Self::exec(ticker.history(range, interval, args.prepost.unwrap_or(false))).await?;
        let j = json_val(&candles);
        let headers = vec!["Date", "Open", "High", "Low", "Close", "Volume"];
        let rows: Vec<Vec<String>> = candles
            .iter()
            .map(|c| {
                vec![
                    c.ts.to_string(),
                    json_val(&c.ohlc.open).to_string(),
                    json_val(&c.ohlc.high).to_string(),
                    json_val(&c.ohlc.low).to_string(),
                    json_val(&c.ohlc.close).to_string(),
                    json_val(&c.volume).to_string(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(
                &format!(
                    "Historical Data: {} ({} candles)",
                    args.symbol,
                    candles.len()
                ),
                &j,
                &headers,
                &rows,
            ),
        )]))
    }

    #[tool(description = "Get corporate actions (dividends, splits, capital gains)")]
    async fn get_corporate_actions(
        &self,
        Parameters(args): Parameters<CorporateActionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let actions = Self::exec(ticker.actions(Some(parse_range(args.range.as_deref())))).await?;
        let j = json_val(&actions);
        let headers = vec!["Type", "Date", "Details"];
        let rows: Vec<Vec<String>> = actions
            .iter()
            .map(|a| {
                let (typ, details) = match a {
                    yfinance_rs::Action::Dividend { date: _, amount } => {
                        ("Dividend".into(), json_val(amount).to_string())
                    }
                    yfinance_rs::Action::Split {
                        date: _,
                        numerator,
                        denominator,
                    } => ("Split".into(), format!("{}:{}", numerator, denominator)),
                    yfinance_rs::Action::CapitalGain { date: _, gain } => {
                        ("Capital Gain".into(), json_val(gain).to_string())
                    }
                    _ => ("Unknown".into(), "N/A".into()),
                };
                vec![typ, fmt_action_date(a), details]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(
                &format!("Corporate Actions: {}", args.symbol),
                &j,
                &headers,
                &rows,
            ),
        )]))
    }

    #[tool(description = "Get key valuation, dividend, volume, and risk statistics")]
    async fn get_key_statistics(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let stats = Self::exec(ticker.key_statistics()).await?;
        let j = json_val(&stats);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Key Statistics: {}", args.symbol), &j),
        )]))
    }

    // ── Multi-Symbol ──────────────────────────────────────────────

    #[tool(description = "Get quotes for multiple ticker symbols at once")]
    async fn get_batch_quotes(
        &self,
        Parameters(args): Parameters<BatchQuotesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let symbols: Vec<String> = args.symbols.clone();
        let quotes = Self::exec(yfinance_rs::quote::quotes(&self.client, symbols)).await?;
        let j = json_val(&quotes);
        let headers = vec!["Symbol", "Price", "Bid", "Ask", "Volume"];
        let rows: Vec<Vec<String>> = quotes
            .iter()
            .map(|q| {
                vec![
                    q.instrument.symbol.as_str().to_string(),
                    json_val(&q.price).to_string(),
                    json_val(&q.bid).to_string(),
                    json_val(&q.ask).to_string(),
                    json_val(&q.day_volume).to_string(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table("Batch Quotes", &j, &headers, &rows),
        )]))
    }

    #[tool(description = "Download historical data for multiple symbols concurrently")]
    async fn download_data(
        &self,
        Parameters(args): Parameters<DownloadArgs>,
    ) -> Result<CallToolResult, McpError> {
        let symbols: Vec<&str> = args.symbols.iter().map(|s| s.as_str()).collect();
        let range = parse_range(args.range.as_deref());
        let interval = parse_interval(args.interval.as_deref()).unwrap_or(Interval::D1);
        let results = Self::exec(async {
            yfinance_rs::DownloadBuilder::new(&self.client)
                .symbols(symbols)
                .range(range)
                .interval(interval)
                .run()
                .await
        })
        .await?;
        let j = json_val(&results.entries);
        let mut md = format!(
            "## Multi-Symbol Download\n\n```json\n{}\n```\n",
            serde_json::to_string_pretty(&j).unwrap_or_default()
        );
        for entry in &results.entries {
            md.push_str(&format!("\n### {}\n\n| Date | Open | High | Low | Close | Volume |\n|------|------|------|------|-------|--------|\n", entry.instrument.symbol.as_str()));
            for c in &entry.history.candles {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    c.ts,
                    json_val(&c.ohlc.open),
                    json_val(&c.ohlc.high),
                    json_val(&c.ohlc.low),
                    json_val(&c.ohlc.close),
                    json_val(&c.volume),
                ));
            }
        }
        Ok(CallToolResult::success(vec![Content::text(md)]))
    }

    // ── Financial Statements ──────────────────────────────────────

    #[tool(description = "Get income statement (annual or quarterly)")]
    async fn get_income_statement(
        &self,
        Parameters(args): Parameters<StatementArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = if args.quarterly.unwrap_or(false) {
            Self::exec(ticker.quarterly_income_stmt(None)).await?
        } else {
            Self::exec(ticker.income_stmt(None)).await?
        };
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(
                &format!(
                    "Income Statement: {} ({})",
                    args.symbol,
                    if args.quarterly.unwrap_or(false) {
                        "quarterly"
                    } else {
                        "annual"
                    }
                ),
                &j,
            ),
        )]))
    }

    #[tool(description = "Get balance sheet (annual or quarterly)")]
    async fn get_balance_sheet(
        &self,
        Parameters(args): Parameters<StatementArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = if args.quarterly.unwrap_or(false) {
            Self::exec(ticker.quarterly_balance_sheet(None)).await?
        } else {
            Self::exec(ticker.balance_sheet(None)).await?
        };
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(
                &format!(
                    "Balance Sheet: {} ({})",
                    args.symbol,
                    if args.quarterly.unwrap_or(false) {
                        "quarterly"
                    } else {
                        "annual"
                    }
                ),
                &j,
            ),
        )]))
    }

    #[tool(description = "Get cash flow statement (annual or quarterly)")]
    async fn get_cashflow(
        &self,
        Parameters(args): Parameters<StatementArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = if args.quarterly.unwrap_or(false) {
            Self::exec(ticker.quarterly_cashflow(None)).await?
        } else {
            Self::exec(ticker.cashflow(None)).await?
        };
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(
                &format!(
                    "Cash Flow: {} ({})",
                    args.symbol,
                    if args.quarterly.unwrap_or(false) {
                        "quarterly"
                    } else {
                        "annual"
                    }
                ),
                &j,
            ),
        )]))
    }

    #[tool(description = "Get earnings data for a ticker")]
    async fn get_earnings(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = Self::exec(ticker.earnings(None)).await?;
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Earnings: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get analyst earnings estimates and trends")]
    async fn get_earnings_trend(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = Self::exec(ticker.earnings_trend(None)).await?;
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Earnings Trend: {}", args.symbol), &j),
        )]))
    }

    // ── Analysis ─────────────────────────────────────────────────

    #[tool(description = "Get analyst recommendation history for a ticker")]
    async fn get_recommendations(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let recs = Self::exec(ticker.recommendations()).await?;
        let j = json_val(&recs);
        let headers = vec!["Period", "Strong Buy", "Buy", "Hold", "Sell", "Strong Sell"];
        let rows: Vec<Vec<String>> = recs
            .iter()
            .map(|r| {
                vec![
                    r.period.to_string(),
                    r.strong_buy.map(|v| v.to_string()).unwrap_or_default(),
                    r.buy.map(|v| v.to_string()).unwrap_or_default(),
                    r.hold.map(|v| v.to_string()).unwrap_or_default(),
                    r.sell.map(|v| v.to_string()).unwrap_or_default(),
                    r.strong_sell.map(|v| v.to_string()).unwrap_or_default(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(
                &format!("Recommendations: {}", args.symbol),
                &j,
                &headers,
                &rows,
            ),
        )]))
    }

    #[tool(description = "Get summary of latest analyst recommendations")]
    async fn get_recommendations_summary(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let summary = Self::exec(ticker.recommendations_summary()).await?;
        let j = json_val(&summary);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Recommendations Summary: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get analyst price targets for a ticker")]
    async fn get_price_target(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let pt = Self::exec(ticker.analyst_price_target(None)).await?;
        let j = json_val(&pt);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Price Target: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get history of analyst upgrades and downgrades")]
    async fn get_upgrades_downgrades(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let data = Self::exec(ticker.upgrades_downgrades()).await?;
        let j = json_val(&data);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Upgrades/Downgrades: {}", args.symbol), &j),
        )]))
    }

    // ── Holders ──────────────────────────────────────────────────

    #[tool(description = "Get top institutional holders for a ticker")]
    async fn get_institutional_holders(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let holders = Self::exec(ticker.institutional_holders()).await?;
        let j = json_val(&holders);
        let headers = vec!["Holder", "Shares", "% Held", "Value", "Date Reported"];
        let rows: Vec<Vec<String>> = holders
            .iter()
            .map(|h| {
                vec![
                    h.holder.clone(),
                    h.shares
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    json_val(&h.pct_held).to_string(),
                    json_val(&h.value).to_string(),
                    h.date_reported.to_string(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(
                &format!("Institutional Holders: {}", args.symbol),
                &j,
                &headers,
                &rows,
            ),
        )]))
    }

    #[tool(description = "Get major holders breakdown for a ticker")]
    async fn get_major_holders(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let holders = Self::exec(ticker.major_holders()).await?;
        let j = json_val(&holders);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Major Holders: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get recent insider transactions for a ticker")]
    async fn get_insider_transactions(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let txns = Self::exec(ticker.insider_transactions()).await?;
        let j = json_val(&txns);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Insider Transactions: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get top mutual fund holders for a ticker")]
    async fn get_mutual_fund_holders(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let holders = Self::exec(ticker.mutual_fund_holders()).await?;
        let j = json_val(&holders);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Mutual Fund Holders: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get insider roster (company insiders and their holdings)")]
    async fn get_insider_roster(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let roster = Self::exec(ticker.insider_roster_holders()).await?;
        let j = json_val(&roster);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Insider Roster: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get net insider share purchase activity summary")]
    async fn get_net_share_purchase_activity(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let activity = Self::exec(ticker.net_share_purchase_activity()).await?;
        let j = json_val(&activity);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Net Share Purchase Activity: {}", args.symbol), &j),
        )]))
    }

    // ── Options ──────────────────────────────────────────────────

    #[tool(description = "Get available option expiration dates for a ticker")]
    async fn get_option_expirations(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let dates = Self::exec(ticker.options()).await?;
        let j = json_val(&dates);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Option Expirations: {}", args.symbol), &j),
        )]))
    }

    #[tool(
        description = "Get the full option chain (calls and puts) for a ticker, optionally for a specific expiration date"
    )]
    async fn get_option_chain(
        &self,
        Parameters(args): Parameters<OptionChainArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let date_ts = args.date.as_ref().and_then(|d| {
            use chrono::NaiveDate;
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .ok()
                .map(|nd| nd.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
        });
        let chain = Self::exec(ticker.option_chain(date_ts)).await?;
        let j = json_val(&chain);
        let calls: Vec<_> = chain.calls().collect();
        let puts: Vec<_> = chain.puts().collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(
                &format!(
                    "Option Chain: {} ({} calls, {} puts)",
                    args.symbol,
                    calls.len(),
                    puts.len()
                ),
                &j,
            ),
        )]))
    }

    // ── News & Profile ───────────────────────────────────────────

    #[tool(description = "Get latest news articles for a ticker")]
    async fn get_news(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let articles = Self::exec(ticker.news()).await?;
        let j = json_val(&articles);
        let headers = vec!["Title", "Publisher", "Date", "Link"];
        let rows: Vec<Vec<String>> = articles
            .iter()
            .map(|a| {
                vec![
                    a.title.clone(),
                    a.publisher.clone().unwrap_or_default(),
                    a.published_at.to_string(),
                    a.link.clone().unwrap_or_default(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(
                &format!("News: {} ({} articles)", args.symbol, articles.len()),
                &j,
                &headers,
                &rows,
            ),
        )]))
    }

    #[tool(description = "Get company, ETF, or fund profile information")]
    async fn get_profile(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let profile = Self::exec(ticker.profile()).await?;
        let j = json_val(&profile);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Profile: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get corporate calendar (earnings dates, dividend dates)")]
    async fn get_calendar(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let cal = Self::exec(ticker.calendar()).await?;
        let j = json_val(&cal);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Calendar: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Get ESG (Environmental, Social, Governance) scores for a ticker")]
    async fn get_sustainability(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let sus = Self::exec(ticker.sustainability()).await?;
        let j = json_val(&sus);
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("Sustainability: {}", args.symbol), &j),
        )]))
    }

    #[tool(description = "Look up the ISIN for a ticker symbol")]
    async fn get_isin(
        &self,
        Parameters(args): Parameters<SymbolArg>,
    ) -> Result<CallToolResult, McpError> {
        let ticker = self.ticker(&args.symbol);
        let isin = Self::exec(ticker.isin()).await?;
        let j = serde_json::json!({ "isin": isin });
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md(&format!("ISIN: {}", args.symbol), &j),
        )]))
    }

    // ── Search ───────────────────────────────────────────────────

    #[tool(description = "Search for ticker symbols by name or keyword")]
    async fn search_tickers(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let results = Self::exec(yfinance_rs::search::search(&self.client, &args.query)).await?;
        let j = json_val(&results);
        let headers = vec!["Symbol", "Name", "Exchange", "Type"];
        let rows: Vec<Vec<String>> = results
            .results
            .iter()
            .map(|r| {
                vec![
                    r.instrument.symbol.as_str().to_string(),
                    r.name.clone().unwrap_or_default(),
                    json_val(&r.instrument.exchange).to_string(),
                    json_val(&r.instrument.kind).to_string(),
                ]
            })
            .collect();
        Ok(CallToolResult::success(vec![Content::text(
            format::json_md_with_table(&format!("Search: {}", args.query), &j, &headers, &rows),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for YFinanceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Yahoo Finance MCP Server. Provides stock quotes, historical data, financial statements, \
                analyst ratings, options chains, holder information, news, ESG scores, and ticker search. \
                Use get_quote for real-time prices, get_historical_data for OHLCV candles, \
                get_info for comprehensive ticker data, and search_tickers to find symbols."
                    .to_string(),
            ),
        }
    }
}

// ── Parameter Structs ──────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SymbolArg {
    /// The ticker symbol (e.g. AAPL, MSFT, GOOGL)
    pub symbol: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HistoryArgs {
    /// The ticker symbol
    pub symbol: String,
    /// Time range: 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y, 2y, 5y, 10y, max (default: 6mo)
    pub range: Option<String>,
    /// Interval: 1m, 2m, 5m, 15m, 30m, 1h, 1d, 5d, 1wk, 1mo, 3mo (default: 1d)
    pub interval: Option<String>,
    /// Include pre/post market data for intraday intervals (default: false)
    pub prepost: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CorporateActionsArgs {
    /// The ticker symbol
    pub symbol: String,
    /// Time range: 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y, 2y, 5y, 10y, max (default: max)
    pub range: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BatchQuotesArgs {
    /// List of ticker symbols (e.g. ["AAPL", "MSFT", "GOOGL"])
    pub symbols: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DownloadArgs {
    /// List of ticker symbols
    pub symbols: Vec<String>,
    /// Time range: 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y, 2y, 5y, 10y, max (default: 6mo)
    pub range: Option<String>,
    /// Interval: 1d, 1wk, 1mo (default: 1d)
    pub interval: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatementArgs {
    /// The ticker symbol
    pub symbol: String,
    /// If true, return quarterly data instead of annual (default: false)
    pub quarterly: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OptionChainArgs {
    /// The ticker symbol
    pub symbol: String,
    /// Optional expiration date in YYYY-MM-DD format. If omitted, uses nearest expiration.
    pub date: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchArgs {
    /// The search query (company name or keyword)
    pub query: String,
}
