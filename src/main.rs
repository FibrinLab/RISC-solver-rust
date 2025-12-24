use clap::Parser;
use matmul_solver::{compute_workload, types};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input JSON file path
    #[arg(short, long, default_value = "input.json")]
    input: String,

    /// Output JSON file path
    #[arg(short, long, default_value = "output.json")]
    output: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input
    let input_str = fs::read_to_string(&args.input)?;
    let input: types::Input = serde_json::from_str(&input_str)?;
    
    // Compute result
    let output = compute_workload(input)?;
    
    // Write output
    let output_str = serde_json::to_string_pretty(&output)?;
    fs::write(&args.output, output_str)?;
    
    println!("Matrix multiplication completed successfully!");
    println!("Latency: {:.4} ms", output.metrics.latency_ms);
    println!("Throughput: {:.2} ops/sec", output.metrics.throughput_ops_per_sec);
    println!("Result hash: {}", output.result_hash);
    
    Ok(())
}

