#[cfg(feature = "api")]
use matmul_solver::api;

#[cfg(feature = "api")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);
    
    api::api::run_api_server(port).await?;
    Ok(())
}

#[cfg(not(feature = "api"))]
fn main() {
    eprintln!("API feature is not enabled. Build with --features api");
    std::process::exit(1);
}

