use sha2::{Digest, Sha256};
use std::time::Instant;

pub mod types {
    pub use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Input {
        // MatMul fields (backward compatible - current format)
        pub matrix_a: Vec<Vec<f32>>,
        pub matrix_b: Vec<Vec<f32>>,
        
        // Optional workload type for future workloads
        #[serde(default)]
        pub workload_type: Option<String>, // "matmul", "convolution", "attention", "inference"
        
        pub precision: String, // "fp32", "fp16", "int8"
        #[serde(default)]
        pub metadata: Option<InputMetadata>,
        
        // Future workload-specific fields will be added here when schemas are provided
        // For example:
        // pub convolution_params: Option<ConvolutionParams>,
        // pub attention_params: Option<AttentionParams>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InputMetadata {
        pub compiler_flags: Option<String>,
        pub libraries: Option<Vec<String>>,
        pub cache_enabled: Option<bool>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub result_matrix: Vec<Vec<f32>>,
        pub result_hash: String,
        pub metrics: Metrics,
        pub metadata: OutputMetadata,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Metrics {
        pub latency_ms: f64,
        pub throughput_ops_per_sec: f64,
        pub ops_per_second: f64,
        pub memory_usage_mb: Option<f64>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct OutputMetadata {
        pub precision: String,
        pub matrix_a_shape: (usize, usize),
        pub matrix_b_shape: (usize, usize),
        pub result_shape: (usize, usize),
        pub compiler_flags: Option<String>,
        pub libraries: Option<Vec<String>>,
    }
}

pub fn matmul_fp32_optimized(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
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

// Shared computation function that can be used by both CLI and API
pub fn compute_workload(input: types::Input) -> Result<types::Output, String> {
    let workload_type = input.workload_type.as_deref().unwrap_or("matmul");
    
    match workload_type {
        "matmul" => {
            compute_matmul_internal(input.matrix_a, input.matrix_b, &input.precision, &input.metadata)
        }
        // Future workloads will be handled here when schemas are provided:
        // "convolution" => { compute_convolution(...) }
        // "attention" => { compute_attention(...) }
        // "inference" => { compute_inference(...) }
        _ => Err(format!("Unsupported workload type: {}. Currently only 'matmul' is supported.", workload_type)),
    }
}

fn compute_matmul_internal(
    matrix_a: Vec<Vec<f32>>,
    matrix_b: Vec<Vec<f32>>,
    precision: &str,
    metadata: &Option<types::InputMetadata>,
) -> Result<types::Output, String> {
    let rows_a = matrix_a.len();
    let cols_a = matrix_a[0].len();
    let rows_b = matrix_b.len();
    let cols_b = matrix_b[0].len();
    
    if cols_a != rows_b {
        return Err(format!("Matrix dimensions incompatible: A is {}x{}, B is {}x{}", 
            rows_a, cols_a, rows_b, cols_b));
    }
    
    // Perform matrix multiplication with timing
    let start = Instant::now();
    let result = match precision {
        "fp32" => matmul_fp32_optimized(&matrix_a, &matrix_b),
        "fp16" => matmul_fp16(&matrix_a, &matrix_b),
        "int8" => matmul_int8(&matrix_a, &matrix_b),
        _ => return Err(format!("Unsupported precision: {}", precision)),
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
    Ok(types::Output {
        result_matrix: result,
        result_hash,
        metrics: types::Metrics {
            latency_ms,
            throughput_ops_per_sec,
            ops_per_second,
            memory_usage_mb,
        },
        metadata: types::OutputMetadata {
            precision: precision.to_string(),
            matrix_a_shape: (rows_a, cols_a),
            matrix_b_shape: (rows_b, cols_b),
            result_shape: (rows_a, cols_b),
            compiler_flags: metadata.as_ref().and_then(|m| m.compiler_flags.clone()),
            libraries: metadata.as_ref().and_then(|m| m.libraries.clone()),
        },
    })
}

// Keep old function name for backward compatibility
pub fn compute_matmul(input: types::Input) -> Result<types::Output, String> {
    compute_workload(input)
}

/// Verify correctness of a result by recomputing and comparing hashes
pub fn verify_correctness(
    matrix_a: &[Vec<f32>],
    matrix_b: &[Vec<f32>],
    precision: &str,
    expected_hash: &str,
) -> Result<bool, String> {
    let result = match precision {
        "fp32" => matmul_fp32_optimized(matrix_a, matrix_b),
        "fp16" => matmul_fp16(matrix_a, matrix_b),
        "int8" => matmul_int8(matrix_a, matrix_b),
        _ => return Err(format!("Unsupported precision: {}", precision)),
    };
    
    let computed_hash = compute_hash(&result);
    Ok(computed_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matmul_fp32_correctness() {
        let a = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ];
        let b = vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];
        
        let result = matmul_fp32_optimized(&a, &b);
        
        // Expected: [[1*5+2*7, 1*6+2*8], [3*5+4*7, 3*6+4*8]]
        //          = [[19, 22], [43, 50]]
        assert_eq!(result[0][0], 19.0);
        assert_eq!(result[0][1], 22.0);
        assert_eq!(result[1][0], 43.0);
        assert_eq!(result[1][1], 50.0);
    }
    
    #[test]
    fn test_matmul_fp32_hash_consistency() {
        let a = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ];
        let b = vec![
            vec![7.0, 8.0],
            vec![9.0, 10.0],
            vec![11.0, 12.0],
        ];
        
        // Compute multiple times - hash should be identical
        let result1 = matmul_fp32_optimized(&a, &b);
        let result2 = matmul_fp32_optimized(&a, &b);
        let result3 = matmul_fp32_optimized(&a, &b);
        
        let hash1 = compute_hash(&result1);
        let hash2 = compute_hash(&result2);
        let hash3 = compute_hash(&result3);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }
    
    #[test]
    fn test_verify_correctness() {
        let a = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ];
        let b = vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];
        
        // Compute and get hash
        let result = matmul_fp32_optimized(&a, &b);
        let correct_hash = compute_hash(&result);
        
        // Verify it matches
        assert!(verify_correctness(&a, &b, "fp32", &correct_hash).unwrap());
        
        // Wrong hash should fail
        assert!(!verify_correctness(&a, &b, "fp32", "wrong_hash").unwrap());
    }
    
    #[test]
    fn test_fp16_correctness() {
        let a = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ];
        let b = vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];
        
        let result = matmul_fp16(&a, &b);
        
        // FP16 should give approximately correct results (may have small precision differences)
        assert!((result[0][0] - 19.0).abs() < 0.1);
        assert!((result[0][1] - 22.0).abs() < 0.1);
        assert!((result[1][0] - 43.0).abs() < 0.1);
        assert!((result[1][1] - 50.0).abs() < 0.1);
    }
    
    #[test]
    fn test_int8_correctness() {
        let a = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ];
        let b = vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];
        
        let result = matmul_int8(&a, &b);
        
        // INT8 should give approximately correct results (quantization may cause differences)
        assert!((result[0][0] - 19.0).abs() < 1.0);
        assert!((result[0][1] - 22.0).abs() < 1.0);
        assert!((result[1][0] - 43.0).abs() < 1.0);
        assert!((result[1][1] - 50.0).abs() < 1.0);
    }
    
    #[test]
    fn test_compute_workload_integration() {
        let input = types::Input {
            matrix_a: vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
            ],
            matrix_b: vec![
                vec![5.0, 6.0],
                vec![7.0, 8.0],
            ],
            precision: "fp32".to_string(),
            workload_type: Some("matmul".to_string()),
            metadata: None,
        };
        
        let output = compute_workload(input).unwrap();
        
        // Check result correctness
        assert_eq!(output.result_matrix[0][0], 19.0);
        assert_eq!(output.result_matrix[0][1], 22.0);
        assert_eq!(output.result_matrix[1][0], 43.0);
        assert_eq!(output.result_matrix[1][1], 50.0);
        
        // Check hash is present
        assert!(!output.result_hash.is_empty());
        
        // Check metrics are reasonable
        assert!(output.metrics.latency_ms >= 0.0);
        assert!(output.metrics.ops_per_second > 0.0);
        
        // Verify hash matches recomputed hash
        let input2 = types::Input {
            matrix_a: vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
            ],
            matrix_b: vec![
                vec![5.0, 6.0],
                vec![7.0, 8.0],
            ],
            precision: "fp32".to_string(),
            workload_type: Some("matmul".to_string()),
            metadata: None,
        };
        
        assert!(verify_correctness(
            &input2.matrix_a,
            &input2.matrix_b,
            &input2.precision,
            &output.result_hash
        ).unwrap());
    }
    
    #[test]
    fn test_matrix_dimension_validation() {
        let input = types::Input {
            matrix_a: vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
            ],
            matrix_b: vec![
                vec![5.0, 6.0], // Wrong dimensions - should be 2x2
            ],
            precision: "fp32".to_string(),
            workload_type: Some("matmul".to_string()),
            metadata: None,
        };
        
        let result = compute_workload(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("incompatible"));
    }
}

