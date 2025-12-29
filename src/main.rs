use clap::Parser;
use matmul_solver::{compute_workload, types, verify_correctness, add_timing_breakdown};
use std::fs;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input JSON file path
    #[arg(short, long)]
    input: Option<String>,

    /// Output JSON file path
    #[arg(short, long, default_value = "outputs/output.json")]
    output: String,

    /// Generate matrices from seed (hex string) instead of JSON file
    /// For seed dimensions: generates 16×50240 × 50240×16 matrices
    #[arg(long)]
    seed: Option<String>,

    /// Precision to use (fp32, fp16, int8, u8i8)
    /// Required when using --seed
    #[arg(long)]
    precision: Option<String>,

    /// Verify correctness by recomputing and checking hash
    #[arg(long)]
    verify: bool,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Time input parsing/generation
    let parse_start = Instant::now();
    
    let (input, parse_time_ms) = if let Some(seed_hex) = args.seed {
        // Generate matrices from seed
        let precision = args.precision.ok_or("--precision is required when using --seed")?;
        
        // Seed dimensions: 16×50240 × 50240×16
        let (matrix_a, matrix_b) = matmul_solver::generate_matrices_from_seed_hex(
            &seed_hex,
            16,      // rows_a
            50240,  // cols_a
            50240,  // rows_b
            16,     // cols_b
        )?;
        
        let parse_time = parse_start.elapsed().as_secs_f64() * 1000.0;
        
        let input = types::Input {
            matrix_a,
            matrix_b,
            precision,
            workload_type: Some("matmul".to_string()),
            metadata: None,
        };
        
        (input, parse_time)
    } else {
        // Read from JSON file
        let input_path = args.input.as_deref().unwrap_or("inputs/input.json");
        let input_str = fs::read_to_string(input_path)?;
        let input: types::Input = serde_json::from_str(&input_str)?;
        let parse_time = parse_start.elapsed().as_secs_f64() * 1000.0;
        (input, parse_time)
    };
    
    // Store input data for verification (before moving input)
    let matrix_a = input.matrix_a.clone();
    let matrix_b = input.matrix_b.clone();
    let precision = input.precision.clone();
    
    // Compute result (kernel_time is already measured inside)
    let mut output = compute_workload(input)?;
    
    // Add parse time to timing breakdown
    output = add_timing_breakdown(output, Some(parse_time_ms), None);
    
    // Time output serialization
    let serialize_start = Instant::now();
    let _output_str = serde_json::to_string_pretty(&output)?;
    let serialize_time_ms = serialize_start.elapsed().as_secs_f64() * 1000.0;
    
    // Add serialize time to timing breakdown
    output = add_timing_breakdown(output, Some(parse_time_ms), Some(serialize_time_ms));
    
    // Write output file (re-serialize with complete timing breakdown)
    let output_str = serde_json::to_string_pretty(&output)?;
    fs::write(&args.output, output_str)?;
    
    println!("Matrix multiplication completed successfully!");
    println!("Latency: {:.4} ms", output.metrics.latency_ms);
    println!("Throughput: {:.2} ops/sec", output.metrics.throughput_ops_per_sec);
    println!("Result hash: {}", output.result_hash);
    
    // Print timing breakdown if available
    if let Some(kernel_time) = output.metrics.kernel_time_ms {
        println!("\nTiming Breakdown:");
        if let Some(parse_time) = output.metrics.parse_time_ms {
            println!("  Parse time:     {:.4} ms", parse_time);
        }
        println!("  Kernel time:    {:.4} ms (matmul computation)", kernel_time);
        if let Some(serialize_time) = output.metrics.serialize_time_ms {
            println!("  Serialize time: {:.4} ms", serialize_time);
        }
    }
    
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

