use clap::Parser;
use matmul_solver::{compute_workload, types, verify_correctness};
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

    /// Verify correctness by recomputing and checking hash
    #[arg(long)]
    verify: bool,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input
    let input_str = fs::read_to_string(&args.input)?;
    let input: types::Input = serde_json::from_str(&input_str)?;
    
    // Store input data for verification (before moving input)
    let matrix_a = input.matrix_a.clone();
    let matrix_b = input.matrix_b.clone();
    let precision = input.precision.clone();
    
    // Compute result
    let output = compute_workload(input)?;
    
    // Write output
    let output_str = serde_json::to_string_pretty(&output)?;
    fs::write(&args.output, output_str)?;
    
    println!("Matrix multiplication completed successfully!");
    println!("Latency: {:.4} ms", output.metrics.latency_ms);
    println!("Throughput: {:.2} ops/sec", output.metrics.throughput_ops_per_sec);
    println!("Result hash: {}", output.result_hash);
    
    // Verify correctness if requested
    if args.verify {
        match verify_correctness(&matrix_a, &matrix_b, &precision, &output.result_hash) {
            Ok(true) => {
                println!("✅ Correctness verified: Hash matches recomputed result");
            }
            Ok(false) => {
                eprintln!("❌ Correctness check failed: Hash mismatch!");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("⚠️  Verification error: {}", e);
            }
        }
    }
    
    // Note about latency variance
    println!("\nNote: Latency may vary between runs due to system load, CPU scheduling, and cache effects.");
    println!("      For consistent benchmarking, run multiple iterations and average the results.");
    
    Ok(())
}

