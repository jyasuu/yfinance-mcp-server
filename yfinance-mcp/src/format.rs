pub fn json_md(title: &str, json: &serde_json::Value) -> String {
    let json_str = serde_json::to_string_pretty(json).unwrap_or_default();
    format!("## {}\n\n```json\n{}\n```\n", title, json_str)
}

pub fn json_md_with_table(
    title: &str,
    json: &serde_json::Value,
    headers: &[&str],
    rows: &[Vec<String>],
) -> String {
    let mut out = format!(
        "## {}\n\n```json\n{}\n```\n\n",
        title,
        serde_json::to_string_pretty(json).unwrap_or_default()
    );
    if !rows.is_empty() {
        out.push_str("| ");
        out.push_str(&headers.join(" | "));
        out.push_str(" |\n| ");
        out.push_str(
            &headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | "),
        );
        out.push_str(" |\n");
        for row in rows {
            out.push_str("| ");
            out.push_str(
                &row.iter()
                    .map(|c| escape_md(c))
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
            out.push_str(" |\n");
        }
    }
    out
}

fn escape_md(s: &str) -> String {
    s.replace('|', "\\|")
}

fn val(js: &serde_json::Value, path: &str) -> serde_json::Value {
    js.pointer(path).cloned().unwrap_or(serde_json::Value::Null)
}

fn extract_num(js: &serde_json::Value) -> Option<f64> {
    match js {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        serde_json::Value::Object(m) => m.get("amount").and_then(|a| match a {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }),
        _ => None,
    }
}

fn str_val(js: &serde_json::Value, path: &str) -> String {
    let v = val(js, path);
    v.as_str()
        .map(|s| s.to_string())
        .or_else(|| extract_num(&v).map(format_f64))
        .unwrap_or_else(|| "N/A".to_string())
}

fn num_val(js: &serde_json::Value, path: &str) -> String {
    let v = val(js, path);
    match v {
        serde_json::Value::Null => "N/A".to_string(),
        _ => extract_num(&v)
            .map(format_f64)
            .or_else(|| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "N/A".to_string()),
    }
}

fn format_f64(n: f64) -> String {
    if n == (n as i64) as f64 {
        format!("{:.0}", n)
    } else if n.abs() > 1.0 {
        format!("{:.2}", n)
    } else {
        format!("{:.4}", n)
    }
}

fn fmt_market_cap(v: &serde_json::Value) -> String {
    let n = match extract_num(v) {
        Some(f) => f,
        None => return "N/A".to_string(),
    };
    if n >= 1_000_000_000_000.0 {
        format!("${:.2}T", n / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000.0 {
        format!("${:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("${:.2}M", n / 1_000_000.0)
    } else {
        format!("${:.0}", n)
    }
}

fn fmt_change(price: f64, prev_close: f64) -> String {
    if prev_close == 0.0 {
        return "N/A".to_string();
    }
    let chg = price - prev_close;
    let pct = (chg / prev_close) * 100.0;
    let sign = if chg >= 0.0 { "+" } else { "" };
    format!("{}${:.2} ({}{:.2}%)", sign, chg, sign, pct)
}

fn fmt_ts_ms(ms: i64) -> String {
    use chrono::{DateTime, Utc};
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    match DateTime::<Utc>::from_timestamp(secs, nsecs) {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => ms.to_string(),
    }
}

pub fn report_html(
    symbol: &str,
    info: &serde_json::Value,
    fast_info: &serde_json::Value,
    candles: &serde_json::Value,
    news: &serde_json::Value,
) -> String {
    let name = str_val(info, "/profile/name");
    let sector = str_val(info, "/profile/sector");
    let industry = str_val(info, "/profile/industry");
    let currency = str_val(info, "/snapshot/currency");
    let day_high = num_val(info, "/snapshot/day_high");
    let day_low = num_val(info, "/snapshot/day_low");
    let volume = num_val(info, "/snapshot/volume");
    let market_cap = fmt_market_cap(&val(info, "/key_statistics/market_cap"));
    let pe = num_val(info, "/key_statistics/pe_trailing_twelve_months");
    let eps = num_val(info, "/key_statistics/eps_trailing_twelve_months");
    let beta = num_val(info, "/key_statistics/beta");
    let high_52w = num_val(info, "/key_statistics/fifty_two_week_high");
    let low_52w = num_val(info, "/key_statistics/fifty_two_week_low");
    let exchange = str_val(info, "/profile/exchange");

    let price = extract_num(&val(fast_info, "/snapshot/last"))
        .map(format_f64)
        .or_else(|| extract_num(&val(fast_info, "/snapshot/previous_close")).map(format_f64))
        .unwrap_or_else(|| num_val(info, "/snapshot/previous_close"));

    let fast_last = extract_num(&val(fast_info, "/snapshot/last"));
    let fast_prev = extract_num(&val(fast_info, "/snapshot/previous_close"));
    let change_str = match (fast_last, fast_prev) {
        (Some(p), Some(pc)) => fmt_change(p, pc),
        _ => "N/A".to_string(),
    };

    let ma_50d = num_val(fast_info, "/moving_averages/fifty_day");
    let ma_200d = num_val(fast_info, "/moving_averages/two_hundred_day");

    let pt_high = num_val(info, "/price_target/high");
    let pt_mean = num_val(info, "/price_target/mean");
    let pt_low = num_val(info, "/price_target/low");

    let rs = val(info, "/recommendation_summary");
    let sb = num_val(&rs, "/strong_buy");
    let buy = num_val(&rs, "/buy");
    let hold = num_val(&rs, "/hold");
    let sell = num_val(&rs, "/sell");
    let ss = num_val(&rs, "/strong_sell");

    let name_display = if name != "N/A" {
        format!("{} ({})", name, symbol)
    } else {
        symbol.to_string()
    };

    let mut h = String::new();
    h.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    h.push_str(&format!("<title>{} Stock Report</title>\n", symbol));
    h.push_str("<script src=\"https://cdn.tailwindcss.com\"></script>\n");
    h.push_str("</head>\n<body class=\"bg-gray-50\">\n");
    h.push_str("<div class=\"max-w-5xl mx-auto p-6\">\n");

    // Header
    h.push_str(&format!(
        "<div class=\"bg-white rounded-xl shadow-sm p-6 mb-6 border border-gray-100\">\n\
         <h1 class=\"text-3xl font-bold text-gray-900\">{}</h1>\n",
        name_display
    ));
    if exchange != "N/A" || sector != "N/A" {
        h.push_str("<div class=\"flex flex-wrap gap-2 mt-2\">\n");
        if exchange != "N/A" {
            h.push_str(&format!("<span class=\"px-2 py-1 bg-blue-100 text-blue-700 text-xs font-medium rounded\">{}</span>\n", exchange));
        }
        if sector != "N/A" {
            h.push_str(&format!("<span class=\"px-2 py-1 bg-green-100 text-green-700 text-xs font-medium rounded\">{}</span>\n", sector));
        }
        if industry != "N/A" {
            h.push_str(&format!("<span class=\"px-2 py-1 bg-purple-100 text-purple-700 text-xs font-medium rounded\">{}</span>\n", industry));
        }
        h.push_str("</div>\n");
    }
    h.push_str("</div>\n");

    // Key Metrics grid
    h.push_str("<div class=\"grid grid-cols-2 md:grid-cols-4 gap-4 mb-6\">\n");
    h.push_str(&metric_card(
        "Price",
        &format!("{} {}", price, currency),
        &change_str,
    ));
    h.push_str(&metric_card(
        "Market Cap",
        &market_cap,
        &format!("P/E: {}", pe),
    ));
    h.push_str(&metric_card(
        "52W Range",
        &format!("{} - {}", low_52w, high_52w),
        &format!("Beta: {}", beta),
    ));
    h.push_str(&metric_card("Volume", &volume, &format!("EPS: {}", eps)));
    h.push_str("</div>\n");

    // Technicals + Analyst row
    h.push_str("<div class=\"grid grid-cols-1 md:grid-cols-2 gap-6 mb-6\">\n");

    // Technical Analysis card
    h.push_str("<div class=\"bg-white rounded-xl shadow-sm p-6 border border-gray-100\">\n");
    h.push_str("<h2 class=\"text-lg font-semibold text-gray-900 mb-4\">Technical Analysis</h2>\n");
    if ma_50d != "N/A" || ma_200d != "N/A" {
        h.push_str("<table class=\"w-full text-sm\">\n<tbody>\n");
        if ma_50d != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">50-Day MA</td><td class=\"py-1 text-right font-medium\">{}</td></tr>\n", ma_50d));
        }
        if ma_200d != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">200-Day MA</td><td class=\"py-1 text-right font-medium\">{}</td></tr>\n", ma_200d));
        }
        if day_high != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Day Range</td><td class=\"py-1 text-right font-medium\">{} - {}</td></tr>\n", day_low, day_high));
        }
        h.push_str("</tbody>\n</table>\n");
    } else {
        h.push_str("<p class=\"text-gray-400 text-sm\">No technical data available</p>\n");
    }
    h.push_str("</div>\n");

    // Analyst Consensus card
    h.push_str("<div class=\"bg-white rounded-xl shadow-sm p-6 border border-gray-100\">\n");
    h.push_str("<h2 class=\"text-lg font-semibold text-gray-900 mb-4\">Analyst Consensus</h2>\n");
    let has_pt = pt_high != "N/A" || pt_mean != "N/A" || pt_low != "N/A";
    let has_rec = sb != "N/A" || buy != "N/A" || hold != "N/A" || sell != "N/A" || ss != "N/A";
    if has_pt {
        h.push_str("<table class=\"w-full text-sm\">\n<tbody>\n");
        if pt_high != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Target High</td><td class=\"py-1 text-right font-medium\">{}</td></tr>\n", pt_high));
        }
        if pt_mean != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Target Mean</td><td class=\"py-1 text-right font-medium\">{} {}</td></tr>\n", pt_mean, currency));
        }
        if pt_low != "N/A" {
            h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Target Low</td><td class=\"py-1 text-right font-medium\">{}</td></tr>\n", pt_low));
        }
        h.push_str("</tbody>\n</table>\n");
    }
    if has_rec {
        h.push_str(
            "<h3 class=\"text-sm font-medium text-gray-700 mt-4 mb-2\">Recommendations</h3>\n",
        );
        h.push_str("<table class=\"w-full text-sm\">\n<tbody>\n");
        h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Strong Buy</td><td class=\"py-1 text-right font-medium text-green-600\">{}</td></tr>\n", sb));
        h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Buy</td><td class=\"py-1 text-right font-medium text-green-500\">{}</td></tr>\n", buy));
        h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Hold</td><td class=\"py-1 text-right font-medium text-yellow-500\">{}</td></tr>\n", hold));
        h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Sell</td><td class=\"py-1 text-right font-medium text-red-500\">{}</td></tr>\n", sell));
        h.push_str(&format!("<tr><td class=\"py-1 text-gray-500\">Strong Sell</td><td class=\"py-1 text-right font-medium text-red-600\">{}</td></tr>\n", ss));
        h.push_str("</tbody>\n</table>\n");
    }
    if !has_pt && !has_rec {
        h.push_str("<p class=\"text-gray-400 text-sm\">No analyst data available</p>\n");
    }
    h.push_str("</div>\n");

    h.push_str("</div>\n");

    // News section
    h.push_str("<div class=\"bg-white rounded-xl shadow-sm p-6 border border-gray-100 mb-6\">\n");
    h.push_str("<h2 class=\"text-lg font-semibold text-gray-900 mb-4\">Recent News</h2>\n");
    if let Some(articles) = news.as_array().filter(|a| !a.is_empty()) {
        for article in articles.iter().take(10) {
            let title = article
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let publisher = article
                .get("publisher")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let link = article.get("link").and_then(|v| v.as_str());
            let date = article
                .get("published_at")
                .and_then(|v| v.as_i64())
                .map(fmt_ts_ms)
                .unwrap_or_default();
            h.push_str("<div class=\"py-3 border-b border-gray-100 last:border-0\">\n");
            if let Some(url) = link {
                h.push_str(&format!("<a href=\"{}\" target=\"_blank\" class=\"text-blue-600 hover:text-blue-800 font-medium\">{}</a>\n", url, title));
            } else {
                h.push_str(&format!("<span class=\"font-medium\">{}</span>\n", title));
            }
            h.push_str("<div class=\"flex gap-3 text-xs text-gray-400 mt-1\">\n");
            if !publisher.is_empty() {
                h.push_str(&format!("<span>{}</span>\n", publisher));
            }
            h.push_str(&format!("<span>{}</span>\n", date));
            h.push_str("</div>\n</div>\n");
        }
    } else {
        h.push_str("<p class=\"text-gray-400 text-sm\">No recent news available</p>\n");
    }
    h.push_str("</div>\n");

    // Price History table
    h.push_str("<div class=\"bg-white rounded-xl shadow-sm p-6 border border-gray-100\">\n");
    h.push_str(
        "<h2 class=\"text-lg font-semibold text-gray-900 mb-4\">Recent Price History</h2>\n",
    );
    if let Some(c) = candles.as_array().filter(|a| !a.is_empty()) {
        let last_n = if c.len() > 30 {
            &c[c.len() - 30..]
        } else {
            &c[..]
        };
        h.push_str("<div class=\"overflow-x-auto\">\n");
        h.push_str("<table class=\"w-full text-sm\">\n<thead>\n<tr class=\"text-left text-gray-500 border-b\">\n");
        h.push_str("<th class=\"py-2 pr-4\">Date</th><th class=\"py-2 pr-4 text-right\">Open</th><th class=\"py-2 pr-4 text-right\">High</th><th class=\"py-2 pr-4 text-right\">Low</th><th class=\"py-2 pr-4 text-right\">Close</th><th class=\"py-2 pr-4 text-right\">Volume</th>\n");
        h.push_str("</tr>\n</thead>\n<tbody>\n");
        for candle in last_n.iter().rev() {
            let ts = candle.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
            let ts_display = fmt_ts_ms(ts);
            let open = candle
                .pointer("/ohlc/open")
                .and_then(extract_num)
                .map(format_f64)
                .unwrap_or_default();
            let high = candle
                .pointer("/ohlc/high")
                .and_then(extract_num)
                .map(format_f64)
                .unwrap_or_default();
            let low = candle
                .pointer("/ohlc/low")
                .and_then(extract_num)
                .map(format_f64)
                .unwrap_or_default();
            let close = candle
                .pointer("/ohlc/close")
                .and_then(extract_num)
                .map(format_f64)
                .unwrap_or_default();
            let vol = candle
                .get("volume")
                .and_then(extract_num)
                .map(format_f64)
                .unwrap_or_default();
            h.push_str(&format!(
                "<tr class=\"border-b border-gray-50 hover:bg-gray-50\">\n\
                <td class=\"py-2 pr-4 whitespace-nowrap\">{}</td>\n\
                <td class=\"py-2 pr-4 text-right\">{}</td>\n\
                <td class=\"py-2 pr-4 text-right\">{}</td>\n\
                <td class=\"py-2 pr-4 text-right\">{}</td>\n\
                <td class=\"py-2 pr-4 text-right font-medium\">{}</td>\n\
                <td class=\"py-2 pr-4 text-right text-gray-500\">{}</td>\n\
            </tr>\n",
                ts_display, open, high, low, close, vol
            ));
        }
        h.push_str("</tbody>\n</table>\n</div>\n");
    } else {
        h.push_str("<p class=\"text-gray-400 text-sm\">No price history available</p>\n");
    }
    h.push_str("</div>\n");

    h.push_str(
        "<p class=\"text-center text-xs text-gray-400 mt-8\">Generated by yfinance-mcp</p>\n",
    );
    h.push_str("</div>\n</body>\n</html>\n");
    h
}

fn metric_card(label: &str, value: &str, sub: &str) -> String {
    let sub_html = if sub.is_empty() || sub == "N/A" {
        String::new()
    } else {
        format!("<p class=\"text-xs text-gray-400 mt-1\">{}</p>\n", sub)
    };
    format!(
        "<div class=\"bg-white rounded-xl shadow-sm p-4 border border-gray-100\">\n\
         <p class=\"text-xs text-gray-400 uppercase tracking-wide\">{}</p>\n\
         <p class=\"text-xl font-bold text-gray-900 mt-1\">{}</p>\n\
         {}\
         </div>\n",
        label, value, sub_html
    )
}
