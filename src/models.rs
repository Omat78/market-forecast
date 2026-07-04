use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Metrics {
    pub mae: f64,
    pub rmse: f64,
    pub mape: f64,
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

/// Naive random-walk baseline: tomorrow = today. This is the benchmark
/// every other model here has to beat to be worth anything.
pub fn naive_forecast(series: &[f64], horizon: usize) -> Vec<f64> {
    let last = *series.last().unwrap_or(&0.0);
    vec![last; horizon]
}

/// Simple moving average of the last `window` points, held flat.
pub fn moving_average_forecast(series: &[f64], window: usize, horizon: usize) -> Vec<f64> {
    let window = window.min(series.len()).max(1);
    let slice = &series[series.len() - window..];
    let avg = mean(slice);
    vec![avg; horizon]
}

/// Simple exponential smoothing (level only), held flat.
pub fn ses_forecast(series: &[f64], alpha: f64, horizon: usize) -> Vec<f64> {
    if series.is_empty() {
        return vec![0.0; horizon];
    }
    let mut level = series[0];
    for &x in &series[1..] {
        level = alpha * x + (1.0 - alpha) * level;
    }
    vec![level; horizon]
}

/// Holt's linear trend method (double exponential smoothing).
pub fn holt_linear_forecast(series: &[f64], alpha: f64, beta: f64, horizon: usize) -> Vec<f64> {
    if series.len() < 2 {
        return naive_forecast(series, horizon);
    }
    let mut level = series[0];
    let mut trend = series[1] - series[0];
    for &x in &series[1..] {
        let last_level = level;
        level = alpha * x + (1.0 - alpha) * (level + trend);
        trend = beta * (level - last_level) + (1.0 - beta) * trend;
    }
    (1..=horizon)
        .map(|h| level + h as f64 * trend)
        .collect()
}

/// Ordinary least squares linear regression on index -> value, extrapolated.
pub fn linear_regression_forecast(series: &[f64], horizon: usize) -> Vec<f64> {
    let n = series.len() as f64;
    if series.is_empty() {
        return vec![0.0; horizon];
    }
    let xs: Vec<f64> = (0..series.len()).map(|i| i as f64).collect();
    let x_mean = mean(&xs);
    let y_mean = mean(series);
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..series.len() {
        num += (xs[i] - x_mean) * (series[i] - y_mean);
        den += (xs[i] - x_mean).powi(2);
    }
    let slope = if den.abs() < 1e-9 { 0.0 } else { num / den };
    let intercept = y_mean - slope * x_mean;
    (0..horizon)
        .map(|h| intercept + slope * (n + h as f64))
        .collect()
}

/// Walk-forward backtest: for each of the last `test_len` points, predict it
/// one step ahead using only the data before it, then compare to what
/// actually happened. This is the honest way to score a forecasting method.
pub fn backtest<F>(series: &[f64], test_len: usize, predict_one: F) -> Metrics
where
    F: Fn(&[f64]) -> f64,
{
    let max_test = series.len().saturating_sub(2);
    let test_len = test_len.min(max_test).max(1);
    let start = series.len() - test_len;

    let mut errors = Vec::with_capacity(test_len);
    let mut pct_errors = Vec::with_capacity(test_len);

    for i in start..series.len() {
        let train = &series[..i];
        if train.is_empty() {
            continue;
        }
        let pred = predict_one(train);
        let actual = series[i];
        errors.push((pred - actual).abs());
        if actual.abs() > 1e-9 {
            pct_errors.push(((pred - actual) / actual).abs());
        }
    }

    let mae = mean(&errors);
    let rmse = (errors.iter().map(|e| e.powi(2)).sum::<f64>() / errors.len().max(1) as f64).sqrt();
    let mape = if pct_errors.is_empty() {
        0.0
    } else {
        mean(&pct_errors) * 100.0
    };
    Metrics { mae, rmse, mape }
}
