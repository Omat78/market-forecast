use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct PricePoint {
    pub date: String,
    pub close: f64,
}

/// Pulls daily closing prices from stooq's free CSV endpoint. No API key
/// needed. Works for most stock/ETF/index/FX tickers, e.g. "aapl", "spy",
/// "^spx", "eurusd".
pub async fn fetch_daily_closes(symbol: &str) -> Result<Vec<PricePoint>> {
    let symbol = symbol.trim().to_lowercase();
    let valid = !symbol.is_empty()
        && symbol.len() <= 12
        && symbol
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '^');
    if !valid {
        return Err(anyhow!("'{}' doesn't look like a valid ticker", symbol));
    }

    let url = format!("https://stooq.com/q/d/l/?s={}&i=d", symbol);
    let text = reqwest::get(&url).await?.text().await?;

    let mut points = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue; // header row: Date,Open,High,Low,Close,Volume
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 5 {
            continue;
        }
        if let Ok(close) = cols[4].parse::<f64>() {
            points.push(PricePoint {
                date: cols[0].to_string(),
                close,
            });
        }
    }

    if points.len() < 30 {
        return Err(anyhow!(
            "not enough data came back for '{}' ({} rows) — double check the ticker",
            symbol,
            points.len()
        ));
    }

    Ok(points)
}
