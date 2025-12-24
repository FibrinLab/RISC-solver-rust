use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

// Import types and functions from lib.rs
use matmul_solver::compute_matmul;
use matmul_solver::types::{Input, Output};

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

async fn solve_handler(
    State(_state): State<Arc<()>>,
    Json(input): Json<Input>,
) -> Result<Json<Output>, (StatusCode, Json<ErrorResponse>)> {
    match compute_matmul(input) {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() {
    // Get port from environment variable or default to 8080
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    
    let addr = format!("0.0.0.0:{}", port);
    
    let app = Router::new()
        .route("/solve", post(solve_handler))
        .route("/health", axum::routing::get(health_handler))
        .route("/", axum::routing::get(|| async { "MatMul Solver API - POST to /solve" }))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(Arc::new(()));
    
    println!("🚀 MatMul Solver API server starting on {}", addr);
    println!("📡 Endpoints:");
    println!("   GET  /health - Health check");
    println!("   POST /solve  - Solve matrix multiplication");
    println!("   GET  /       - API info");
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

