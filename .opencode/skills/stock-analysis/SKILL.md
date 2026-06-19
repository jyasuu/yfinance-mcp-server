---
name: stock-analysis
description: Use ONLY when the user asks to analyze, evaluate, research, or get a summary/trend/outlook for a stock, ticker, or company. Trigger keywords: analyze, trend, outlook, summary, evaluate, research, "what's happening with", "how is * doing", "deep dive", "stock report", fundamentals, technicals. Do NOT trigger for simple quote lookups or price checks.
---

# Stock Analysis

When the user asks you to analyze a stock, produce a structured report by calling the yfinance MCP tools in this order.

## Workflow

### 1. Identify the symbol

If the user gives a company name (e.g. "Apple", "TSMC"), ask for the ticker or look it up:

```
Tool: search_tickers
Input: {"query": "<company name>"}
```

Use the most relevant result's symbol.

### 2. Gather data (parallel calls)

Fire these concurrently:

```
Tool: get_info
Input: {"symbol": "<SYMBOL>"}
```

```
Tool: get_fast_info
Input: {"symbol": "<SYMBOL>"}
```

```
Tool: get_historical_data
Input: {"symbol": "<SYMBOL>", "range": "6mo", "interval": "1d"}
```

```
Tool: get_recommendations
Input: {"symbol": "<SYMBOL>"}
```

```
Tool: get_news
Input: {"symbol": "<SYMBOL>"}
```

### 3. Compile the report

Present the findings in this structure:

```
## 📊 <Company Name> (<SYMBOL>) — Stock Analysis

**Price:** $XXX.XX  |  **Prev Close:** $XXX.XX  |  **Day Range:** $X.XX – $X.XX
**50-day MA:** $XXX.XX  |  **200-day MA:** $XXX.XX  |  **Volume:** X,XXX,XXX

### Trend (from 6-month chart)
- **Direction:** Up / Down / Sideways
- **Key Support:** ~$XXX  |  **Key Resistance:** ~$XXX
- **Notable:** (brief observation on the chart pattern, volume spikes, etc.)

### Fundamentals
- **Sector/Industry:** ...
- **Market Cap:** $X.XB
- **Key Ratios:** P/E, EPS, Dividend Yield, etc. (pulled from info)

### Analyst Consensus
- **Strong Buy:** X  |  **Buy:** X  |  **Hold:** X  |  **Sell:** X  |  **Strong Sell:** X
- **Price Target:** High $XXX / Low $XXX / Mean $XXX

### Recent News
- (Top 3 headlines with brief 1-line takeaway)

### Summary
1-2 sentence verdict on the stock based on the combined data.
```

### 4. Use good judgment

- If the stock is clearly trending up with strong volume, bullish analyst consensus, and positive news — say so with supporting evidence.
- If data conflicts (price down but analysts bullish), highlight the tension.
- If a tool errors (e.g. no recommendations data for a small cap), note it and continue with what you have.
- For Taiwan stocks, use `.TW` suffix (e.g. `2330.TW` for TSMC). For Hong Kong, use `.HK`.
