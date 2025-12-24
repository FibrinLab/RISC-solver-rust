use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Input {
    matrix_a: Vec<Vec<f32>>,
    matrix_b: Vec<Vec<f32>>,
    precision: String, // "fp32", "fp16", "int8"
    #[serde(default)]
    metadata: Option<InputMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputMetadata {
    compiler_flags: Option<String>,
    libraries: Option<Vec<String>>,
    cache_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    result_matrix: Vec<Vec<f32>>,
    result_hash: String,
    metrics: Metrics,
    metadata: OutputMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics {
    latency_ms: f64,
    throughput_ops_per_sec: f64,
    ops_per_second: f64,
    memory_usage_mb: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputMetadata {
    precision: String,
    matrix_a_shape: (usize, usize),
    matrix_b_shape: (usize, usize),
    result_shape: (usize, usize),
    compiler_flags: Option<String>,
    libraries: Option<Vec<String>>,
}

fn matmul_fp32_optimized(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let rows_a = a.len();
    let cols_a = a[0].len();
    let cols_b = b[0].len();
    
    let mut result = vec![vec![0.0f32; cols_b]; rows_a];
    
    // Cache-friendly: transpose B for better memory access
    let b_transposed: Vec<Vec<f32>> = (0..cols_b)
        .map(|j| (0..cols_a).map(|i| b[i][j]).collect())
        .collect();
    
    for i in 0..rows_a {
        for j in 0..cols_b {
            let mut sum = 0.0f32;
            for k in 0..cols_a {
                sum += a[i][k] * b_transposed[j][k];
            }
            result[i][j] = sum;
        }
    }
    
    result
}

fn matmul_fp16(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    use half::f16;
    
    // Convert to fp16, compute, convert back
    let a_fp16: Vec<Vec<f16>> = a.iter()
        .map(|row| row.iter().map(|&x| f16::from_f32(x)).collect())
        .collect();
    
    let b_fp16: Vec<Vec<f16>> = b.iter()
        .map(|row| row.iter().map(|&x| f16::from_f32(x)).collect())
        .collect();
    
    let rows_a = a_fp16.len();
    let cols_a = a_fp16[0].len();
    let cols_b = b_fp16[0].len();
    
    let mut result_fp16 = vec![vec![f16::from_f32(0.0); cols_b]; rows_a];
    
    for i in 0..rows_a {
        for j in 0..cols_b {
            let mut sum = f16::from_f32(0.0);
            for k in 0..cols_a {
                sum += a_fp16[i][k] * b_fp16[k][j];
            }
            result_fp16[i][j] = sum;
        }
    }
    
    // Convert back to fp32
    result_fp16.iter()
        .map(|row| row.iter().map(|&x| x.to_f32()).collect())
        .collect()
}

fn matmul_int8(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    // Convert to int8, compute, convert back
    let scale_a = 127.0 / a.iter()
        .flat_map(|row| row.iter())
        .map(|&x| x.abs())
        .fold(0.0f32, f32::max);
    
    let scale_b = 127.0 / b.iter()
        .flat_map(|row| row.iter())
        .map(|&x| x.abs())
        .fold(0.0f32, f32::max);
    
    let a_int8: Vec<Vec<i8>> = a.iter()
        .map(|row| row.iter()
            .map(|&x| (x * scale_a).clamp(-128.0, 127.0) as i8)
            .collect())
        .collect();
    
    let b_int8: Vec<Vec<i8>> = b.iter()
        .map(|row| row.iter()
            .map(|&x| (x * scale_b).clamp(-128.0, 127.0) as i8)
            .collect())
        .collect();
    
    let rows_a = a_int8.len();
    let cols_a = a_int8[0].len();
    let cols_b = b_int8[0].len();
    
    let mut result_int32 = vec![vec![0i32; cols_b]; rows_a];
    
    for i in 0..rows_a {
        for j in 0..cols_b {
            let mut sum = 0i32;
            for k in 0..cols_a {
                sum += a_int8[i][k] as i32 * b_int8[k][j] as i32;
            }
            result_int32[i][j] = sum;
        }
    }
    
    // Convert back to fp32 with proper scaling
    let scale_result = 1.0 / (scale_a * scale_b);
    result_int32.iter()
        .map(|row| row.iter()
            .map(|&x| x as f32 * scale_result)
            .collect())
        .collect()
}

fn compute_hash(matrix: &[Vec<f32>]) -> String {
    let mut hasher = Sha256::new();
    
    for row in matrix {
        for &val in row {
            let bytes = val.to_le_bytes();
            hasher.update(&bytes);
        }
    }
    
    hex::encode(hasher.finalize())
}

fn estimate_memory_usage(rows_a: usize, cols_a: usize, rows_b: usize, cols_b: usize) -> f64 {
    // Rough estimate: input matrices + output matrix (all as f32)
    let input_size = (rows_a * cols_a + rows_b * cols_b) * 4; // 4 bytes per f32
    let output_size = rows_a * cols_b * 4;
    (input_size + output_size) as f64 / (1024.0 * 1024.0) // Convert to MB
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input
    let input_str = fs::read_to_string(&args.input)?;
    let input: Input = serde_json::from_str(&input_str)?;
    
    let rows_a = input.matrix_a.len();
    let cols_a = input.matrix_a[0].len();
    let rows_b = input.matrix_b.len();
    let cols_b = input.matrix_b[0].len();
    
    if cols_a != rows_b {
        return Err(format!("Matrix dimensions incompatible: A is {}x{}, B is {}x{}", 
            rows_a, cols_a, rows_b, cols_b).into());
    }
    
    // Perform matrix multiplication with timing
    let start = Instant::now();
    let result = match input.precision.as_str() {
        "fp32" => matmul_fp32_optimized(&input.matrix_a, &input.matrix_b),
        "fp16" => matmul_fp16(&input.matrix_a, &input.matrix_b),
        "int8" => matmul_int8(&input.matrix_a, &input.matrix_b),
        _ => return Err(format!("Unsupported precision: {}", input.precision).into()),
    };
    let elapsed = start.elapsed();
    
    // Compute metrics
    let latency_ms = elapsed.as_secs_f64() * 1000.0;
    let total_ops = (rows_a * cols_a * cols_b) as f64; // Multiply-add operations
    let ops_per_second = total_ops / elapsed.as_secs_f64();
    let throughput_ops_per_sec = ops_per_second;
    
    // Compute result hash
    let result_hash = compute_hash(&result);
    
    // Estimate memory usage
    let memory_usage_mb = Some(estimate_memory_usage(rows_a, cols_a, rows_b, cols_b));
    
    // Build output
    let output = Output {
        result_matrix: result,
        result_hash,
        metrics: Metrics {
            latency_ms,
            throughput_ops_per_sec,
            ops_per_second,
            memory_usage_mb,
        },
        metadata: OutputMetadata {
            precision: input.precision.clone(),
            matrix_a_shape: (rows_a, cols_a),
            matrix_b_shape: (rows_b, cols_b),
            result_shape: (rows_a, cols_b),
            compiler_flags: input.metadata.as_ref().and_then(|m| m.compiler_flags.clone()),
            libraries: input.metadata.as_ref().and_then(|m| m.libraries.clone()),
        },
    };
    
    // Write output
    let output_str = serde_json::to_string_pretty(&output)?;
    fs::write(&args.output, output_str)?;
    
    println!("Matrix multiplication completed successfully!");
    println!("Latency: {:.4} ms", latency_ms);
    println!("Throughput: {:.2} ops/sec", throughput_ops_per_sec);
    println!("Result hash: {}", output.result_hash);
    
    Ok(())
}

