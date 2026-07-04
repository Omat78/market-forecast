# Market Forecast Sandbox (Rust)

A small fullstack app for exploring classic time series forecasting models
against real historical price data and just as importantly against a
naive "tomorrow = today" baseline. 
It's built to make the unreliability of
market forecasting visible, not to hide it.

**Stack:** Rust backend (Axum) + plain HTML/JS frontend (Chart.js via CDN).
No build tooling needed beyond Cargo.

## What it does

1. You pick a ticker, a model, and a forecast horizon.
2. The backend pulls free daily closing prices for that ticker from stooq.com.
3. It runs a walk-forward backtest: for each of the last N days, predict that day using only the data before it, then compare prediction vs. reality.
4. It shows you the forecast, a 95% band that widens with `sqrt(horizon)` and critically the same backtest run with the naive random walk model, so you can see whether the fancier model actually earned its complexity.

Models included: naive (random walk), moving average, simple exponential
smoothing, Holt's linear trend, and OLS linear regression. These are standard textbook methods, not anything with genuine market edge real quantitative trading uses far more data (order flow, fundamentals, alternative data) and still struggles to consistently beat naive baselines net of costs.

## Running it

```bash
cd market-forecast
cargo run
```

Then open http://localhost:3000

## Project layout

```
market-forecast/
├── Cargo.toml
├── src/
│   ├── main.rs       # Axum server setup, routing
│   ├── handlers.rs    # /api/forecast endpoint
│   ├── data.rs        # fetches historical prices from stooq
│   └── models.rs      # forecasting algorithms + backtesting
└── static/
    ├── index.html
    ├── style.css
    └── app.js          # calls the API, renders the chart
```

## Extending it

- Swap `data.rs` to read from a CSV upload instead of stooq, if you want to
work with data that isn't publicly available.
- Add ARIMA/GARCH via a crate like `augurs` or shell out to a Python
  process if you need heavier statistical machinery.
- Add a second "model" that's literally coin-flip random, to further drive
  home how much of the naive model's performance is just "markets are hard
  to beat," not "this specific baseline is smart."

## The honest caveat

Nothing here should inform real trading decisions. Backtested accuracy on
past prices says very little about future accuracy — markets are close to
efficient, meaning most public information is already priced in, and the
methods here don't use anything markets don't already know. Treat this as a
learning tool for forecasting mechanics and backtesting discipline, not a
signal generator.
