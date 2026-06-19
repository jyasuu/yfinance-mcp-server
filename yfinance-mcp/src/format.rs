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
    let mut out = format!("## {}\n\n```json\n{}\n```\n\n", title, serde_json::to_string_pretty(json).unwrap_or_default());
    if !rows.is_empty() {
        out.push_str("| ");
        out.push_str(&headers.join(" | "));
        out.push_str(" |\n| ");
        out.push_str(&headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
        out.push_str(" |\n");
        for row in rows {
            out.push_str("| ");
            out.push_str(&row.iter().map(|c| escape_md(c)).collect::<Vec<_>>().join(" | "));
            out.push_str(" |\n");
        }
    }
    out
}

fn escape_md(s: &str) -> String {
    s.replace('|', "\\|")
}
