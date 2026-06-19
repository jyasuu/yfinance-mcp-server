---
name: stock-analysis
description: Use ONLY when the user asks to analyze, evaluate, research, or get a summary/trend/outlook for a stock, ticker, or company. Trigger keywords: analyze, trend, outlook, summary, evaluate, research, "what's happening with", "how is * doing", "deep dive", "stock report", fundamentals, technicals. Do NOT trigger for simple quote lookups or price checks.
---

# Stock Analysis

When the user asks you to analyze a stock, generate an HTML report using `generate_report`, then present a brief summary.

## Workflow

### 1. Identify the symbol

If the user gives a company name (e.g. "Apple", "TSMC"), look it up:

```
Tool: search_tickers
Input: {"query": "<company name>"}
```

Use the most relevant result's symbol.

**Suffix rules** (when search_tickers returns nothing for a known market):
- Taiwan: numeric symbols need `.TW` suffix (e.g. `0050.TW`, `2330.TW`)
- Hong Kong: add `.HK` suffix (e.g. `0700.HK`, `0005.HK`)
- London: add `.L` suffix (e.g. `BP.L`, `HSBA.L`)
- If the first suffix fails, try another or ask the user.

### 2. Generate the report

Call `generate_report` with the symbol (and optional range):

```
Tool: generate_report
Input: {"symbol": "<SYMBOL>", "range": "6mo"}
```

The tool returns a JSON with the file path and (in HTTP mode) a URL.

### 3. Present to the user

State the file path/URL and give a 2-3 sentence summary:

```
### <Company Name> (<SYMBOL>) — Stock Analysis

**Report:** <file path or URL>

<2-3 sentence verdict synthesizing price action, fundamentals, analyst consensus, and news sentiment. Call out if news aligns or conflicts with the technical/fundamental picture.>
```

### 4. Use good judgment

- If the stock is clearly trending up with strong volume, bullish analyst consensus, and positive news — say so with supporting evidence.
- If data conflicts (price down but analysts bullish), highlight the tension.
- If a tool errors (e.g. no recommendations/news for a small cap or ETF), say "No recommendations data available" — **do not** show the raw MCP error code or URL.
- If `get_news` returns empty for an ETF, try fetching news for its top holding instead (e.g. for `0050.TW`, try `2330.TW`).
