mod data;
mod handlers;
mod models;

use axum::{routing::post, Router};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/forecast", post(handlers::forecast_handler))
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Server running at http://localhost:3000");
    axum::serve(listener, app).await.expect("server error");
}
