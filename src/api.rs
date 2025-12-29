#[cfg(feature = "api")]
pub mod api {
    use axum::{
        extract::State,
        http::StatusCode,
        response::Json,
        routing::post,
        Router,
    };
    use tower_http::cors::CorsLayer;
    use crate::{compute_workload, types, add_timing_breakdown};
    use std::sync::Arc;
    use std::time::Instant;

    // Shared state for the API
    pub struct AppState {
        // Can be used for caching or other state if needed
    }

    // Request body for /compute endpoint
    #[derive(serde::Deserialize)]
    pub struct ComputeRequest {
        // Option 1: Provide matrices directly
        pub matrix_a: Option<Vec<Vec<f32>>>,
        pub matrix_b: Option<Vec<Vec<f32>>>,
        
        // Option 2: Generate from seed (deterministic)
        pub seed: Option<String>,
        
        pub precision: String,
        pub workload_type: Option<String>,
    }

    // POST /compute - Accept matrix input (JSON or seed) and return result
    async fn compute_handler(
        State(_state): State<Arc<AppState>>,
        Json(req): Json<ComputeRequest>,
    ) -> Result<Json<types::Output>, (StatusCode, String)> {
        let parse_start = Instant::now();
        
        let input = if let Some(seed_hex) = req.seed {
            // Generate from seed (deterministic)
            let (matrix_a, matrix_b) = crate::generate_matrices_from_seed_hex(
                &seed_hex,
                16, 50240, 50240, 16,  // Seed dimensions
            ).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            
            types::Input {
                matrix_a,
                matrix_b,
                precision: req.precision,
                workload_type: req.workload_type.or(Some("matmul".to_string())),
                metadata: None,
            }
        } else {
            // Use provided matrices
            let matrix_a = req.matrix_a.ok_or_else(|| (StatusCode::BAD_REQUEST, "matrix_a is required when not using seed".to_string()))?;
            let matrix_b = req.matrix_b.ok_or_else(|| (StatusCode::BAD_REQUEST, "matrix_b is required when not using seed".to_string()))?;
            
            // Convert Vec<Vec<f32>> to FlatMatrix
            let rows_a = matrix_a.len();
            let cols_a = if rows_a > 0 { matrix_a[0].len() } else { 0 };
            let mut a_data = Vec::with_capacity(rows_a * cols_a);
            for row in matrix_a {
                a_data.extend_from_slice(&row);
            }
            
            let rows_b = matrix_b.len();
            let cols_b = if rows_b > 0 { matrix_b[0].len() } else { 0 };
            let mut b_data = Vec::with_capacity(rows_b * cols_b);
            for row in matrix_b {
                b_data.extend_from_slice(&row);
            }
            
            types::Input {
                matrix_a: crate::FlatMatrix { data: a_data, rows: rows_a, cols: cols_a },
                matrix_b: crate::FlatMatrix { data: b_data, rows: rows_b, cols: cols_b },
                precision: req.precision,
                workload_type: req.workload_type.or(Some("matmul".to_string())),
                metadata: None,
            }
        };
        
        let parse_time_ms = parse_start.elapsed().as_secs_f64() * 1000.0;
        
        let mut output = match compute_workload(input) {
            Ok(output) => output,
            Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
        };
        
        // Add parse time
        output = add_timing_breakdown(output, Some(parse_time_ms), None);
        
        // Time serialization
        let serialize_start = Instant::now();
        let _ = serde_json::to_string(&output);
        let serialize_time_ms = serialize_start.elapsed().as_secs_f64() * 1000.0;
        output = add_timing_breakdown(output, Some(parse_time_ms), Some(serialize_time_ms));
        
        Ok(Json(output))
    }

    // GET /health - Health check endpoint
    async fn health_handler() -> &'static str {
        "OK"
    }

    pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let state = Arc::new(AppState {});

        let app = Router::new()
            .route("/compute", post(compute_handler))
            .route("/health", axum::routing::get(health_handler))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("API server listening on port {}", port);
        println!("Endpoints:");
        println!("  POST /compute - Submit matrix computation");
        println!("  GET  /health  - Health check");
        axum::serve(listener, app).await?;
        Ok(())
    }
}

