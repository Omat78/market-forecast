let chart;

async function runForecast() {
  const symbol = document.getElementById('symbol').value.trim();
  const model = document.getElementById('model').value;
  const horizon = parseInt(document.getElementById('horizon').value, 10) || 10;
  const statusEl = document.getElementById('status');

  statusEl.textContent = 'Loading...';
  try {
    const res = await fetch('/api/forecast', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ symbol, model, horizon }),
    });
    if (!res.ok) {
      throw new Error(await res.text());
    }
    const data = await res.json();
    render(data);
    statusEl.textContent = '';
  } catch (err) {
    statusEl.textContent = 'Error: ' + err.message;
  }
}

function render(data) {
  const histLen = data.historical.length;
  const labels = [
    ...data.dates,
    ...data.forecast.map((_, i) => `+${i + 1}d`),
  ];
  const histData = [...data.historical, ...Array(data.forecast.length).fill(null)];
  const forecastData = [...Array(histLen).fill(null), ...data.forecast];
  const lowerData = [...Array(histLen).fill(null), ...data.lower];
  const upperData = [...Array(histLen).fill(null), ...data.upper];

  const ctx = document.getElementById('chart').getContext('2d');
  if (chart) chart.destroy();
  chart = new Chart(ctx, {
    type: 'line',
    data: {
      labels,
      datasets: [
        { label: 'Historical', data: histData, borderColor: '#2563eb', pointRadius: 0, borderWidth: 2 },
        { label: 'Forecast', data: forecastData, borderColor: '#dc2626', borderDash: [6, 4], pointRadius: 0, borderWidth: 2 },
        { label: 'Upper bound (95%)', data: upperData, borderColor: 'rgba(220,38,38,0.25)', pointRadius: 0, fill: '+1' },
        { label: 'Lower bound (95%)', data: lowerData, borderColor: 'rgba(220,38,38,0.25)', pointRadius: 0 },
      ],
    },
    options: {
      animation: false,
      interaction: { mode: 'index', intersect: false },
      scales: { x: { display: false } },
      plugins: { legend: { position: 'bottom' } },
    },
  });

  document.getElementById('metrics').innerHTML = `
    <p><strong>Walk-forward backtest, 1-day-ahead (${data.model}):</strong>
      MAE ${data.metrics.mae.toFixed(3)} · RMSE ${data.metrics.rmse.toFixed(3)} · MAPE ${data.metrics.mape.toFixed(2)}%</p>
    <p><strong>Naive random-walk RMSE:</strong> ${data.naive_rmse.toFixed(3)} —
      this model <strong>${data.beats_naive ? 'beat' : 'did NOT beat'}</strong> the naive baseline
      on this stock's recent history. If it didn't beat naive, the model is adding
      complexity without adding accuracy — a common outcome in market forecasting.</p>
  `;
}

document.getElementById('run').addEventListener('click', runForecast);
