use axum::{extract::Json, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::data::fetch_daily_closes;
use crate::models::*;

#[derive(Deserialize)]
pub struct ForecastRequest {
    pub symbol: String,
    pub model: String,
    #[serde(default = "default_horizon")]
    pub horizon: usize,
}

fn default_horizon() -> usize {
    10
}

#[derive(Serialize)]
pub struct ForecastResponse {
    pub symbol: String,
    pub model: String,
    pub dates: Vec<String>,
    pub historical: Vec<f64>,
    pub forecast: Vec<f64>,
    pub lower: Vec<f64>,
    pub upper: Vec<f64>,
    pub metrics: Metrics,
    pub naive_rmse: f64,
    pub beats_naive: bool,
}

pub async fn forecast_handler(
    Json(req): Json<ForecastRequest>,
) -> Result<Json<ForecastResponse>, (StatusCode, String)> {
    let horizon = req.horizon.clamp(1, 60);

    let points = fetch_daily_closes(&req.symbol)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let series: Vec<f64> = points.iter().map(|p| p.close).collect();
    let dates: Vec<String> = points.iter().map(|p| p.date.clone()).collect();

    let test_len = (series.len() / 5).clamp(5, 40);

    let (forecast, metrics): (Vec<f64>, Metrics) = match req.model.as_str() {
        "naive" => {
            let m = backtest(&series, test_len, |train| naive_forecast(train, 1)[0]);
            (naive_forecast(&series, horizon), m)
        }
        "moving_average" => {
            let window = 10;
            let m = backtest(&series, test_len, move |train| {
                moving_average_forecast(train, window, 1)[0]
            });
            (moving_average_forecast(&series, window, horizon), m)
        }
        "ses" => {
            let alpha = 0.3;
            let m = backtest(&series, test_len, move |train| {
                ses_forecast(train, alpha, 1)[0]
            });
            (ses_forecast(&series, alpha, horizon), m)
        }
        "holt" => {
            let (alpha, beta) = (0.3, 0.1);
            let m = backtest(&series, test_len, move |train| {
                holt_linear_forecast(train, alpha, beta, 1)[0]
            });
            (holt_linear_forecast(&series, alpha, beta, horizon), m)
        }
        "linear_regression" => {
            let m = backtest(&series, test_len, |train| {
                linear_regression_forecast(train, 1)[0]
            });
            (linear_regression_forecast(&series, horizon), m)
        }
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("unknown model '{other}'. Use one of: naive, moving_average, ses, holt, linear_regression"),
            ));
        }
    };

    let naive_metrics = backtest(&series, test_len, |train| naive_forecast(train, 1)[0]);

    // Confidence band grows with sqrt(horizon), the standard random-walk
    // assumption for compounding uncertainty over multiple steps.
    let sigma1 = metrics.rmse.max(1e-6);
    let mut lower = Vec::with_capacity(horizon);
    let mut upper = Vec::with_capacity(horizon);
    for h in 1..=horizon {
        let width = 1.96 * sigma1 * (h as f64).sqrt();
        lower.push(forecast[h - 1] - width);
        upper.push(forecast[h - 1] + width);
    }

    Ok(Json(ForecastResponse {
        symbol: req.symbol.to_uppercase(),
        model: req.model,
        dates,
        historical: series,
        forecast,
        lower,
        upper,
        beats_naive: metrics.rmse < naive_metrics.rmse,
        naive_rmse: naive_metrics.rmse,
        metrics,
    }))
}
