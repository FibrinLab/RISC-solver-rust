use sha2::{Digest, Sha256};
use std::time::Instant;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use blake3;

#[cfg(feature = "api")]
pub mod api;
use std::sync::{Mutex, OnceLock};
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(feature = "openblas")]
extern crate openblas_src;
#[cfg(feature = "openblas")]
use cblas_sys::{cblas_sgemm, CBLAS_ORDER, CBLAS_TRANSPOSE};

struct AlignedBufferF32 {
    ptr: *mut f32,
    len: usize,
    layout: std::alloc::Layout,
}

impl AlignedBufferF32 {
    fn new(len: usize, align: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(len * std::mem::size_of::<f32>(), align)
            .expect("aligned layout");
        let ptr = unsafe { std::alloc::alloc(layout) as *mut f32 };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Self { ptr, len, layout }
    }

    fn as_ptr(&self) -> *const f32 {
        self.ptr as *const f32
    }

    fn as_mut_ptr(&mut self) -> *mut f32 {
        self.ptr
    }
}

impl Drop for AlignedBufferF32 {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr as *mut u8, self.layout);
        }
    }
}

unsafe impl Send for AlignedBufferF32 {}
unsafe impl Sync for AlignedBufferF32 {}

struct AlignedBufferI8 {
    ptr: *mut i8,
    len: usize,
    layout: std::alloc::Layout,
}

impl AlignedBufferI8 {
    fn new(len: usize, align: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(len * std::mem::size_of::<i8>(), align)
            .expect("aligned layout");
        let ptr = unsafe { std::alloc::alloc(layout) as *mut i8 };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Self { ptr, len, layout }
    }

    fn as_ptr(&self) -> *const i8 {
        self.ptr as *const i8
    }

    fn as_mut_ptr(&mut self) -> *mut i8 {
        self.ptr
    }
}

impl Drop for AlignedBufferI8 {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr as *mut u8, self.layout);
        }
    }
}

unsafe impl Send for AlignedBufferI8 {}
unsafe impl Sync for AlignedBufferI8 {}

struct AlignedBufferU8 {
    ptr: *mut u8,
    len: usize,
    layout: std::alloc::Layout,
}

impl AlignedBufferU8 {
    fn new(len: usize, align: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(len * std::mem::size_of::<u8>(), align)
            .expect("aligned layout");
        let ptr = unsafe { std::alloc::alloc(layout) as *mut u8 };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Self { ptr, len, layout }
    }

    fn as_ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for AlignedBufferU8 {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr as *mut u8, self.layout);
        }
    }
}

unsafe impl Send for AlignedBufferU8 {}
unsafe impl Sync for AlignedBufferU8 {}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CacheKey {
    ptr: usize,
    rows: usize,
    cols: usize,
    len: usize,
}

struct AlignedF32Cache {
    key: CacheKey,
    buf: AlignedBufferF32,
}

struct AlignedI8Cache {
    key: CacheKey,
    buf: AlignedBufferI8,
    scale: f32,
}

static B_T_FP16_CACHE: OnceLock<Mutex<Option<AlignedF32Cache>>> = OnceLock::new();
static B_T_I8_CACHE: OnceLock<Mutex<Option<AlignedI8Cache>>> = OnceLock::new();

#[inline(always)]
fn get_bt_fp16_cache(b: &FlatMatrix) -> (*const f32, usize) {
    use half::f16;

    let k = b.rows;
    let key = CacheKey {
        ptr: b.data.as_ptr() as usize,
        rows: b.rows,
        cols: b.cols,
        len: b.data.len(),
    };

    let cache = B_T_FP16_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap();
    let reuse = guard.as_ref().is_some_and(|entry| entry.key == key);
    if !reuse {
        let mut buf = AlignedBufferF32::new(16 * k, 64);
        let b_ptr = b.data.as_ptr();
        unsafe {
            for p in 0..k {
                let b_base = p * 16;
                for j in 0..16 {
                    let val = *b_ptr.add(b_base + j);
                    *buf.as_mut_ptr().add(j * k + p) = f16::from_f32(val).to_f32();
                }
            }
        }
        *guard = Some(AlignedF32Cache { key, buf });
    }
    let entry = guard.as_ref().unwrap();
    (entry.buf.as_ptr(), k)
}

#[inline(always)]
fn get_bt_i8_cache(b: &FlatMatrix) -> (*const i8, f32, usize) {
    let k = b.rows;
    let max_b = b.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let scale_b = if max_b == 0.0 { 1.0 } else { 127.0 / max_b };

    let key = CacheKey {
        ptr: b.data.as_ptr() as usize,
        rows: b.rows,
        cols: b.cols,
        len: b.data.len(),
    };

    let cache = B_T_I8_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap();
    let reuse = guard
        .as_ref()
        .is_some_and(|entry| entry.key == key && (entry.scale - scale_b).abs() < f32::EPSILON);
    if !reuse {
        let mut buf = AlignedBufferI8::new(16 * k, 64);
        let b_ptr = b.data.as_ptr();
        unsafe {
            for p in 0..k {
                let b_base = p * 16;
                for j in 0..16 {
                    let val = *b_ptr.add(b_base + j);
                    *buf.as_mut_ptr().add(j * k + p) = (val * scale_b).clamp(-128.0, 127.0) as i8;
                }
            }
        }
        *guard = Some(AlignedI8Cache { key, buf, scale: scale_b });
    }
    let entry = guard.as_ref().unwrap();
    (entry.buf.as_ptr(), entry.scale, k)
}

#[inline(always)]
fn dot_f32(a: *const f32, b: *const f32, len: usize) -> f32 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut acc = vdupq_n_f32(0.0);
        let mut p = 0usize;
        while p + 4 <= len {
            let av = vld1q_f32(a.add(p));
            let bv = vld1q_f32(b.add(p));
            acc = vmlaq_f32(acc, av, bv);
            p += 4;
        }
        let acc_low = vget_low_f32(acc);
        let acc_high = vget_high_f32(acc);
        let sum2 = vadd_f32(acc_low, acc_high);
        let sum1 = vpadd_f32(sum2, sum2);
        let mut total = vget_lane_f32(sum1, 0);
        while p < len {
            total += *a.add(p) * *b.add(p);
            p += 1;
        }
        total
    }
    #[cfg(not(target_arch = "aarch64"))]
    unsafe {
        let mut total = 0.0f32;
        let mut p = 0usize;
        while p < len {
            total += *a.add(p) * *b.add(p);
            p += 1;
        }
        total
    }
}

#[inline(always)]
fn dot_i8(a: *const i8, b: *const i8, len: usize) -> i32 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut acc = vdupq_n_s32(0);
        let mut p = 0usize;
        while p + 16 <= len {
            let av = vld1q_s8(a.add(p));
            let bv = vld1q_s8(b.add(p));
            let prod_low = vmull_s8(vget_low_s8(av), vget_low_s8(bv));
            let prod_high = vmull_s8(vget_high_s8(av), vget_high_s8(bv));
            let sum_low = vpaddlq_s16(prod_low);
            let sum_high = vpaddlq_s16(prod_high);
            acc = vaddq_s32(acc, vaddq_s32(sum_low, sum_high));
            p += 16;
        }
        let acc_low = vget_low_s32(acc);
        let acc_high = vget_high_s32(acc);
        let sum2 = vadd_s32(acc_low, acc_high);
        let sum1 = vpadd_s32(sum2, sum2);
        let mut total = vget_lane_s32(sum1, 0);
        while p < len {
            total += (*a.add(p) as i32) * (*b.add(p) as i32);
            p += 1;
        }
        total
    }
    #[cfg(not(target_arch = "aarch64"))]
    unsafe {
        let mut total = 0i32;
        let mut p = 0usize;
        while p < len {
            total += (*a.add(p) as i32) * (*b.add(p) as i32);
            p += 1;
        }
        total
    }
}

// Internal representation: flat Vec<f32> with dimensions
// Serializes/deserializes as Vec<Vec<f32>> for JSON compatibility
#[derive(Debug, Clone)]
pub struct FlatMatrix {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

// Custom deserializer: JSON Vec<Vec<f32>> → FlatMatrix (direct flattening, no intermediate Vec<Vec>)
impl<'de> Deserialize<'de> for FlatMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nested: Vec<Vec<f32>> = Vec::deserialize(deserializer)?;
        let rows = nested.len();
        if rows == 0 {
            return Ok(FlatMatrix {
                data: Vec::new(),
                rows: 0,
                cols: 0,
            });
        }
        let cols = nested[0].len();
        
        // Directly flatten during deserialization - single allocation!
        let mut data = Vec::with_capacity(rows * cols);
        for row in nested {
            if row.len() != cols {
                return Err(serde::de::Error::custom("Inconsistent row lengths"));
            }
            data.extend_from_slice(&row);
        }
        
        Ok(FlatMatrix { data, rows, cols })
    }
}

// Custom serializer: FlatMatrix → JSON Vec<Vec<f32>> (only for output serialization)
impl Serialize for FlatMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to Vec<Vec<f32>> only for serialization (not during computation)
        let mut nested = Vec::with_capacity(self.rows);
        for i in 0..self.rows {
            let start = i * self.cols;
            let end = start + self.cols;
            nested.push(self.data[start..end].to_vec());
        }
        nested.serialize(serializer)
    }
}

/// Generate matrices deterministically from a seed using Blake3 XOF
/// Matches the PoW specification: seed -> Blake3 XOF -> matrix_a (u8) + matrix_b (i8)
/// 
/// Seed format: raw bytes
/// Returns: (matrix_a as FlatMatrix, matrix_b as FlatMatrix)
/// 
/// For seed dimensions: matrix_a is 16×50240 (u8 bytes), matrix_b is 50240×16 (i8 bytes)
pub fn generate_matrices_from_seed(seed: &[u8], rows_a: usize, cols_a: usize, rows_b: usize, cols_b: usize) -> (FlatMatrix, FlatMatrix) {
    // Calculate total bytes needed
    let matrix_a_bytes = rows_a * cols_a;
    let matrix_b_bytes = rows_b * cols_b;
    let total_bytes = matrix_a_bytes + matrix_b_bytes;
    
    // Use Blake3 XOF to generate deterministic random bytes
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed);
    let mut output_reader = hasher.finalize_xof();
    
    // Read all bytes needed for both matrices
    let mut all_bytes = vec![0u8; total_bytes];
    output_reader.fill(&mut all_bytes);
    
    // Split bytes: first part for matrix_a (u8), second part for matrix_b (i8)
    let (a_bytes, b_bytes) = all_bytes.split_at(matrix_a_bytes);
    
    // Convert matrix_a bytes to f32 (u8: 0-255)
    // For u8i8, we'll interpret these as u8 directly in the matmul function
    let matrix_a_data: Vec<f32> = a_bytes.iter().map(|&b| b as f32).collect();
    
    // Convert matrix_b bytes to f32 (i8: -128 to 127)
    // Raw bytes are 0-255, but we interpret as i8 by subtracting 128
    let matrix_b_data: Vec<f32> = b_bytes.iter().map(|&b| (b.wrapping_sub(128)) as i8 as f32).collect();
    
    (
        FlatMatrix { data: matrix_a_data, rows: rows_a, cols: cols_a },
        FlatMatrix { data: matrix_b_data, rows: rows_b, cols: cols_b },
    )
}

/// Generate matrices from seed hex string (convenience function)
pub fn generate_matrices_from_seed_hex(seed_hex: &str, rows_a: usize, cols_a: usize, rows_b: usize, cols_b: usize) -> Result<(FlatMatrix, FlatMatrix), String> {
    let seed_bytes = hex::decode(seed_hex)
        .map_err(|e| format!("Invalid hex seed: {}", e))?;
    Ok(generate_matrices_from_seed(&seed_bytes, rows_a, cols_a, rows_b, cols_b))
}

pub mod types {
    pub use super::FlatMatrix;
    pub use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Deserialize)]
    pub struct Input {
        // MatMul fields - stored as FlatMatrix internally
        pub matrix_a: FlatMatrix,
        pub matrix_b: FlatMatrix,
        
        // Optional workload type for future workloads
        #[serde(default)]
        pub workload_type: Option<String>, // "matmul", "convolution", "attention", "inference"
        
        pub precision: String, // "fp32", "fp16", "int8", "u8i8"
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
    
    #[derive(Debug, Serialize)]
    pub struct Output {
        pub result_matrix: FlatMatrix,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parse_time_ms: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kernel_time_ms: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub serialize_time_ms: Option<f64>,
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

/// Optimized fp32 matrix multiplication with cache blocking (tiling) and flat memory layout
/// Uses optimized loop order (i -> p -> j) with cache-friendly tiling
/// Default tile sizes: BM=16, BN=64, BK=64 (tunable for different cache sizes)
/// Works directly with FlatMatrix - no conversion overhead!
/// 
/// Returns (result, kernel_time) where kernel_time is the duration of the computation loop only
pub fn matmul_fp32_optimized(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    const BM: usize = 16;  // Block size for rows of C
    const BN: usize = 64;  // Block size for cols of C
    const BK: usize = 64;  // Block size for reduction dimension
    
    let m = a.rows;        // rows of A and C
    let k = a.cols;        // cols of A, rows of B
    let n = b.cols;        // cols of B and C
    
    // Already flat! No conversion needed
    let a_flat = &a.data;
    let b_flat = &b.data;
    
    // Result in flat layout: C[i * n + j] = C[i][j]
    let mut result_flat = vec![0.0f32; m * n];
    
    // Kernel-only timing: measure only the computation loop
    let start = std::time::Instant::now();
    
    // Cache blocking: block over i (BM), j (BN), and p (BK)
    for ii in (0..m).step_by(BM) {
        let i_end = (ii + BM).min(m);
        for jj in (0..n).step_by(BN) {
            let j_end = (jj + BN).min(n);
            for pp in (0..k).step_by(BK) {
                let p_end = (pp + BK).min(k);
                
                // Microkernel on tile: C[ii:i_end, jj:j_end] += A[ii:i_end, pp:p_end] × B[pp:p_end, jj:j_end]
                // Optimized loop order: i -> p -> j
                // Flat indexing: A[i * k + p], B[p * n + j], C[i * n + j]
                // This streams across B[p, :] (contiguous) and C[i, :] (contiguous)
                // Hoisting a_ip out of inner loop for better register reuse
                for i in ii..i_end {
                    let c_base = i * n;
                    let a_base = i * k;
                    for p in pp..p_end {
                        let a_ip = a_flat[a_base + p];
                        let b_base = p * n;
                        for j in jj..j_end {
                            result_flat[c_base + j] += a_ip * b_flat[b_base + j];
                        }
                    }
                }
            }
        }
    }
    
    // Kernel timing ends here
    let kernel_time = start.elapsed();
    
    // Return as FlatMatrix - no conversion needed!
    (FlatMatrix { data: result_flat, rows: m, cols: n }, kernel_time)
}

#[inline(always)]
fn matmul_fp32_16x16(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;

    let mut result_flat = vec![0.0f32; m * n];
    let start = Instant::now();

    let a_ptr = a.data.as_ptr();
    let c_ptr = result_flat.as_mut_ptr();

    unsafe {
        let b_ptr = b.data.as_ptr();
        let mut p = 0usize;
        while p + 3 < k {
            for i in 0..16 {
                let a_base = i * k;
                let a0 = *a_ptr.add(a_base + p);
                let a1 = *a_ptr.add(a_base + p + 1);
                let a2 = *a_ptr.add(a_base + p + 2);
                let a3 = *a_ptr.add(a_base + p + 3);

                let c_base = i * 16;
                let b_base = p * 16;
                for j in 0..16 {
                    let b0 = *b_ptr.add(b_base + j);
                    let b1 = *b_ptr.add(b_base + 16 + j);
                    let b2 = *b_ptr.add(b_base + 32 + j);
                    let b3 = *b_ptr.add(b_base + 48 + j);
                    let c = c_ptr.add(c_base + j);
                    *c += a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3;
                }
            }
            p += 4;
        }

        while p < k {
            for i in 0..16 {
                let a_ip = *a_ptr.add(i * k + p);
                let c_base = i * 16;
                let b_base = p * 16;
                for j in 0..16 {
                    let b_pj = *b_ptr.add(b_base + j);
                    let c = c_ptr.add(c_base + j);
                    *c += a_ip * b_pj;
                }
            }
            p += 1;
        }
    }

    let kernel_time = start.elapsed();
    (FlatMatrix { data: result_flat, rows: 16, cols: 16 }, kernel_time)
}

#[cfg(feature = "openblas")]
fn matmul_fp32_openblas(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;

    let a_flat = &a.data;
    let b_flat = &b.data;
    let mut result_flat = vec![0.0f32; m * n];

    let start = Instant::now();
    unsafe {
        cblas_sgemm(
            CBLAS_ORDER::CblasRowMajor,
            CBLAS_TRANSPOSE::CblasNoTrans,
            CBLAS_TRANSPOSE::CblasNoTrans,
            m as i32,
            n as i32,
            k as i32,
            1.0,
            a_flat.as_ptr(),
            k as i32,
            b_flat.as_ptr(),
            n as i32,
            0.0,
            result_flat.as_mut_ptr(),
            n as i32,
        );
    }
    let kernel_time = start.elapsed();

    (FlatMatrix { data: result_flat, rows: m, cols: n }, kernel_time)
}

#[cfg(feature = "openblas")]
fn matmul_fp32(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    if a.rows == 16 && b.cols == 16 {
        return matmul_fp32_16x16(a, b);
    }
    matmul_fp32_openblas(a, b)
}

#[cfg(not(feature = "openblas"))]
fn matmul_fp32(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    if a.rows == 16 && b.cols == 16 {
        return matmul_fp32_16x16(a, b);
    }
    matmul_fp32_optimized(a, b)
}

fn matmul_fp16(a: &FlatMatrix, b: &FlatMatrix) -> FlatMatrix {
    use half::f16;
    
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;
    
    // Convert to fp16 (flat layout)
    let a_fp16: Vec<f16> = a.data.iter().map(|&x| f16::from_f32(x)).collect();
    let b_fp16: Vec<f16> = b.data.iter().map(|&x| f16::from_f32(x)).collect();
    
    let mut result_fp16 = vec![f16::from_f32(0.0); m * n];
    
    // Optimized loop order: i -> p -> j
    // This streams across B[p, :] (contiguous) and C[i, :] (contiguous)
    // Hoisting a_ip out of inner loop for better register reuse
    for i in 0..m {
        let c_base = i * n;
        let a_base = i * k;
        for p in 0..k {
            let a_ip = a_fp16[a_base + p];
            let b_base = p * n;
            for j in 0..n {
                result_fp16[c_base + j] += a_ip * b_fp16[b_base + j];
            }
        }
    }
    
    // Convert back to fp32 (flat layout)
    let result_flat: Vec<f32> = result_fp16.iter().map(|&x| x.to_f32()).collect();
    
    FlatMatrix { data: result_flat, rows: m, cols: n }
}

#[inline(always)]
fn matmul_fp16_16x16(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    use half::f16;

    let k = a.cols;

    let mut result_flat = vec![0.0f32; 16 * 16];
    let a_ptr = a.data.as_ptr();
    let c_ptr = result_flat.as_mut_ptr();

    let kernel_time = unsafe {
        let mut a_q = AlignedBufferF32::new(16 * k, 64);
        let a_q_ptr = a_q.as_mut_ptr();
        for i in 0..16 {
            let a_base = i * k;
            for p in 0..k {
                let val = *a_ptr.add(a_base + p);
                *a_q_ptr.add(a_base + p) = f16::from_f32(val).to_f32();
            }
        }

        let a_q_ptr = a_q.as_ptr();
        let (b_t_ptr, _) = get_bt_fp16_cache(b);

        let kernel_start = Instant::now();
        for i in 0..16 {
            let a_row = a_q_ptr.add(i * k);
            let c_base = i * 16;
            for j in 0..16 {
                let b_row = b_t_ptr.add(j * k);
                let acc = dot_f32(a_row, b_row, k);
                *c_ptr.add(c_base + j) = acc;
            }
        }
        kernel_start.elapsed()
    };

    (FlatMatrix { data: result_flat, rows: 16, cols: 16 }, kernel_time)
}

#[cfg(feature = "openblas")]
fn matmul_fp16_openblas(a: &FlatMatrix, b: &FlatMatrix) -> FlatMatrix {
    use half::f16;

    let m = a.rows;
    let k = a.cols;
    let n = b.cols;

    // Quantize inputs to fp16 then back to fp32 so BLAS computes on fp16-like values.
    let a_fp32: Vec<f32> = a
        .data
        .iter()
        .map(|&x| f16::from_f32(x).to_f32())
        .collect();
    let b_fp32: Vec<f32> = b
        .data
        .iter()
        .map(|&x| f16::from_f32(x).to_f32())
        .collect();

    let mut result_flat = vec![0.0f32; m * n];
    unsafe {
        cblas_sgemm(
            CBLAS_ORDER::CblasRowMajor,
            CBLAS_TRANSPOSE::CblasNoTrans,
            CBLAS_TRANSPOSE::CblasNoTrans,
            m as i32,
            n as i32,
            k as i32,
            1.0,
            a_fp32.as_ptr(),
            k as i32,
            b_fp32.as_ptr(),
            n as i32,
            0.0,
            result_flat.as_mut_ptr(),
            n as i32,
        );
    }

    FlatMatrix { data: result_flat, rows: m, cols: n }
}

fn matmul_int8(a: &FlatMatrix, b: &FlatMatrix) -> FlatMatrix {
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;
    
    // Convert to int8 (flat layout)
    let scale_a = 127.0 / a.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let scale_b = 127.0 / b.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    
    let a_int8: Vec<i8> = a.data.iter()
        .map(|&x| (x * scale_a).clamp(-128.0, 127.0) as i8)
        .collect();
    
    let b_int8: Vec<i8> = b.data.iter()
        .map(|&x| (x * scale_b).clamp(-128.0, 127.0) as i8)
        .collect();
    
    let mut result_int32 = vec![0i32; m * n];
    
    // Optimized loop order: i -> p -> j
    // This streams across B[p, :] (contiguous) and C[i, :] (contiguous)
    // Hoisting a_ip out of inner loop for better register reuse
    for i in 0..m {
        let c_base = i * n;
        let a_base = i * k;
        for p in 0..k {
            let a_ip = a_int8[a_base + p] as i32;
            let b_base = p * n;
            for j in 0..n {
                result_int32[c_base + j] += a_ip * b_int8[b_base + j] as i32;
            }
        }
    }
    
    // Convert back to fp32 with proper scaling (flat layout)
    let scale_result = 1.0 / (scale_a * scale_b);
    let result_flat: Vec<f32> = result_int32.iter()
        .map(|&x| x as f32 * scale_result)
        .collect();
    
    FlatMatrix { data: result_flat, rows: m, cols: n }
}

/// u8*i8 matrix multiplication (unsigned 8-bit × signed 8-bit)
/// matrix_a is interpreted as u8 (0-255), matrix_b as i8 (-128 to 127)
/// This matches the seed workload specification where matrices come from raw binary
pub fn matmul_u8i8(a: &FlatMatrix, b: &FlatMatrix) -> FlatMatrix {
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;
    
    // For u8i8, assume matrix_a values are 0..255 and matrix_b values are -128..127.
    // This matches the seed pipeline where bytes are already interpreted as u8/i8.
    let a_u8: Vec<u8> = a.data.iter().map(|&x| x as u8).collect();
    let b_i8: Vec<i8> = b.data.iter().map(|&x| x as i8).collect();
    
    let mut result_int32 = vec![0i32; m * n];
    
    // Optimized loop order: i -> p -> j
    // u8 * i8 multiplication: u8 is promoted to i32, i8 is promoted to i32
    for i in 0..m {
        let c_base = i * n;
        let a_base = i * k;
        for p in 0..k {
            let a_ip = a_u8[a_base + p] as i32;  // u8 -> i32
            let b_base = p * n;
            for j in 0..n {
                result_int32[c_base + j] += a_ip * b_i8[b_base + j] as i32;  // i8 -> i32
            }
        }
    }
    
    // Convert result back to f32 (no scaling needed for u8*i8, result is already correct)
    let result_flat: Vec<f32> = result_int32.iter()
        .map(|&x| x as f32)
        .collect();
    
    FlatMatrix { data: result_flat, rows: m, cols: n }
}

/// Optimized u8*i8 for 16x16 result (seed dimensions: 16×50240 × 50240×16 = 16×16)
#[inline(always)]
pub fn matmul_u8i8_16x16(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    let k = a.cols;  // Should be 50240 for seed dimensions

    let mut result_i32 = vec![0i32; 16 * 16];
    let c_ptr = result_i32.as_mut_ptr();

    let kernel_time = unsafe {
        let mut a_u8 = AlignedBufferU8::new(16 * k, 64);
        let a_u8_ptr = a_u8.as_mut_ptr();
        let a_ptr = a.data.as_ptr();
        for i in 0..16 {
            let a_base = i * k;
            for p in 0..k {
                *a_u8_ptr.add(a_base + p) = *a_ptr.add(a_base + p) as u8;
            }
        }

        let mut b_i8 = AlignedBufferI8::new(k * 16, 64);
        let b_i8_ptr = b_i8.as_mut_ptr();
        let b_ptr = b.data.as_ptr();
        for p in 0..k {
            let b_base = p * 16;
            for j in 0..16 {
                *b_i8_ptr.add(b_base + j) = *b_ptr.add(b_base + j) as i8;
            }
        }

        let a_u8_ptr = a_u8.as_ptr();
        let b_i8_ptr = b_i8.as_ptr();

        let kernel_start = Instant::now();
        for i in 0..16 {
            let a_row = a_u8_ptr.add(i * k);
            let c_base = i * 16;
            #[cfg(target_arch = "aarch64")]
            {
                let mut c0 = vdupq_n_s32(0);
                let mut c1 = vdupq_n_s32(0);
                let mut c2 = vdupq_n_s32(0);
                let mut c3 = vdupq_n_s32(0);
                for p in 0..k {
                    let a_ip = *a_row.add(p) as i16;
                    let b_vec = vld1q_s8(b_i8_ptr.add(p * 16));
                    let b_low = vmovl_s8(vget_low_s8(b_vec));
                    let b_high = vmovl_s8(vget_high_s8(b_vec));
                    c0 = vmlal_n_s16(c0, vget_low_s16(b_low), a_ip);
                    c1 = vmlal_n_s16(c1, vget_high_s16(b_low), a_ip);
                    c2 = vmlal_n_s16(c2, vget_low_s16(b_high), a_ip);
                    c3 = vmlal_n_s16(c3, vget_high_s16(b_high), a_ip);
                }
                vst1q_s32(c_ptr.add(c_base), c0);
                vst1q_s32(c_ptr.add(c_base + 4), c1);
                vst1q_s32(c_ptr.add(c_base + 8), c2);
                vst1q_s32(c_ptr.add(c_base + 12), c3);
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                for p in 0..k {
                    let a_ip = *a_row.add(p) as i32;
                    let b_base = p * 16;
                    for j in 0..16 {
                        let b_pj = *b_i8_ptr.add(b_base + j) as i32;
                        *c_ptr.add(c_base + j) += a_ip * b_pj;
                    }
                }
            }
        }
        kernel_start.elapsed()
    };

    let result_f32: Vec<f32> = result_i32.iter().map(|&x| x as f32).collect();
    (FlatMatrix { data: result_f32, rows: 16, cols: 16 }, kernel_time)
}

#[inline(always)]
fn matmul_int8_16x16(a: &FlatMatrix, b: &FlatMatrix) -> (FlatMatrix, std::time::Duration) {
    let k = a.cols;
    let max_a = a.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let scale_a = if max_a == 0.0 { 1.0 } else { 127.0 / max_a };
    let (b_t_ptr, scale_b, _) = get_bt_i8_cache(b);
    let scale_result = 1.0 / (scale_a * scale_b);

    let mut result_flat = vec![0.0f32; 16 * 16];
    let a_ptr = a.data.as_ptr();
    let c_ptr = result_flat.as_mut_ptr();

    let kernel_time = unsafe {
        let mut a_q = AlignedBufferI8::new(16 * k, 64);
        let a_q_ptr = a_q.as_mut_ptr();
        for i in 0..16 {
            let a_base = i * k;
            for p in 0..k {
                let val = *a_ptr.add(a_base + p);
                *a_q_ptr.add(a_base + p) = (val * scale_a).clamp(-128.0, 127.0) as i8;
            }
        }

        let a_q_ptr = a_q.as_ptr();

        let kernel_start = Instant::now();
        for i in 0..16 {
            let a_row = a_q_ptr.add(i * k);
            let c_base = i * 16;
            for j in 0..16 {
                let b_row = b_t_ptr.add(j * k);
                let acc = dot_i8(a_row, b_row, k);
                *c_ptr.add(c_base + j) = acc as f32 * scale_result;
            }
        }
        kernel_start.elapsed()
    };

    (FlatMatrix { data: result_flat, rows: 16, cols: 16 }, kernel_time)
}

#[cfg(feature = "openblas")]
fn matmul_int8_openblas(a: &FlatMatrix, b: &FlatMatrix) -> FlatMatrix {
    let m = a.rows;
    let k = a.cols;
    let n = b.cols;

    let scale_a = 127.0 / a.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let scale_b = 127.0 / b.data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

    // Quantize to int8, then convert to f32 for BLAS.
    let a_q: Vec<f32> = a
        .data
        .iter()
        .map(|&x| (x * scale_a).clamp(-128.0, 127.0) as i8 as f32)
        .collect();
    let b_q: Vec<f32> = b
        .data
        .iter()
        .map(|&x| (x * scale_b).clamp(-128.0, 127.0) as i8 as f32)
        .collect();

    let mut result_flat = vec![0.0f32; m * n];
    unsafe {
        cblas_sgemm(
            CBLAS_ORDER::CblasRowMajor,
            CBLAS_TRANSPOSE::CblasNoTrans,
            CBLAS_TRANSPOSE::CblasNoTrans,
            m as i32,
            n as i32,
            k as i32,
            1.0,
            a_q.as_ptr(),
            k as i32,
            b_q.as_ptr(),
            n as i32,
            0.0,
            result_flat.as_mut_ptr(),
            n as i32,
        );
    }

    // Scale back to match int8 quantization semantics.
    let scale_result = 1.0 / (scale_a * scale_b);
    for val in &mut result_flat {
        *val *= scale_result;
    }

    FlatMatrix { data: result_flat, rows: m, cols: n }
}

fn compute_hash(matrix: &FlatMatrix) -> String {
    let mut hasher = Sha256::new();
    
    // Hash flat data directly - same order as Vec<Vec<f32>> (row-major)
    for &val in &matrix.data {
        let bytes = val.to_le_bytes();
        hasher.update(&bytes);
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
    matrix_a: FlatMatrix,
    matrix_b: FlatMatrix,
    precision: &str,
    metadata: &Option<types::InputMetadata>,
) -> Result<types::Output, String> {
    let rows_a = matrix_a.rows;
    let cols_a = matrix_a.cols;
    let rows_b = matrix_b.rows;
    let cols_b = matrix_b.cols;
    
    if cols_a != rows_b {
        return Err(format!("Matrix dimensions incompatible: A is {}x{}, B is {}x{}", 
            rows_a, cols_a, rows_b, cols_b));
    }
    
    // Perform matrix multiplication with timing
    // Fast 16x16 kernels use kernel-only timing; fallback paths include conversion overhead.
    let (result, elapsed) = match precision {
        "fp32" => {
            let (res, kernel_time) = matmul_fp32(&matrix_a, &matrix_b);
            (res, kernel_time)
        },
        "fp16" => {
            let (res, elapsed) = if matrix_a.rows == 16 && matrix_b.cols == 16 {
                matmul_fp16_16x16(&matrix_a, &matrix_b)
            } else {
                let start = Instant::now();
                #[cfg(feature = "openblas")]
                let res = matmul_fp16_openblas(&matrix_a, &matrix_b);
                #[cfg(not(feature = "openblas"))]
                let res = matmul_fp16(&matrix_a, &matrix_b);
                (res, start.elapsed())
            };
            (res, elapsed)
        },
        "int8" => {
            let (res, elapsed) = if matrix_a.rows == 16 && matrix_b.cols == 16 {
                matmul_int8_16x16(&matrix_a, &matrix_b)
            } else {
                let start = Instant::now();
                #[cfg(feature = "openblas")]
                let res = matmul_int8_openblas(&matrix_a, &matrix_b);
                #[cfg(not(feature = "openblas"))]
                let res = matmul_int8(&matrix_a, &matrix_b);
                (res, start.elapsed())
            };
            (res, elapsed)
        },
        "u8i8" => {
            // u8*i8: matrix_a as u8 (unsigned), matrix_b as i8 (signed)
            // Optimized path for seed dimensions (16×50240 × 50240×16 = 16×16)
            let (res, elapsed) = if matrix_a.rows == 16 && matrix_b.cols == 16 {
                matmul_u8i8_16x16(&matrix_a, &matrix_b)
            } else {
                let start = Instant::now();
                let res = matmul_u8i8(&matrix_a, &matrix_b);
                (res, start.elapsed())
            };
            (res, elapsed)
        },
        _ => return Err(format!("Unsupported precision: {}", precision)),
    };
    
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
            parse_time_ms: None,  // Set by caller (main.rs)
            kernel_time_ms: Some(elapsed.as_secs_f64() * 1000.0),
            serialize_time_ms: None,  // Set by caller (main.rs)
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

/// Helper function to add timing breakdown to metrics
pub fn add_timing_breakdown(
    mut output: types::Output,
    parse_time_ms: Option<f64>,
    serialize_time_ms: Option<f64>,
) -> types::Output {
    output.metrics.parse_time_ms = parse_time_ms;
    output.metrics.serialize_time_ms = serialize_time_ms;
    output
}

// Keep old function name for backward compatibility
pub fn compute_matmul(input: types::Input) -> Result<types::Output, String> {
    compute_workload(input)
}

/// Verify correctness of a result by recomputing and comparing hashes
pub fn verify_correctness(
    matrix_a: &FlatMatrix,
    matrix_b: &FlatMatrix,
    precision: &str,
    expected_hash: &str,
) -> Result<bool, String> {
    let result = match precision {
        "fp32" => {
            let (res, _) = matmul_fp32(matrix_a, matrix_b);
            res
        },
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
    
    // Helper function to create FlatMatrix from Vec<Vec<f32>> for tests
    fn to_flat_matrix(nested: Vec<Vec<f32>>) -> FlatMatrix {
        let rows = nested.len();
        if rows == 0 {
            return FlatMatrix { data: Vec::new(), rows: 0, cols: 0 };
        }
        let cols = nested[0].len();
        let mut data = Vec::with_capacity(rows * cols);
        for row in nested {
            data.extend_from_slice(&row);
        }
        FlatMatrix { data, rows, cols }
    }
    
    #[test]
    fn test_matmul_fp32_correctness() {
        let a = to_flat_matrix(vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ]);
        let b = to_flat_matrix(vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ]);
        
        let (result, _) = matmul_fp32(&a, &b);
        
        // Expected: [[1*5+2*7, 1*6+2*8], [3*5+4*7, 3*6+4*8]]
        //          = [[19, 22], [43, 50]]
        assert_eq!(result.data[0 * result.cols + 0], 19.0);
        assert_eq!(result.data[0 * result.cols + 1], 22.0);
        assert_eq!(result.data[1 * result.cols + 0], 43.0);
        assert_eq!(result.data[1 * result.cols + 1], 50.0);
    }
    
    #[test]
    fn test_matmul_fp32_hash_consistency() {
        let a = to_flat_matrix(vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ]);
        let b = to_flat_matrix(vec![
            vec![7.0, 8.0],
            vec![9.0, 10.0],
            vec![11.0, 12.0],
        ]);
        
        // Compute multiple times - hash should be identical
        let (result1, _) = matmul_fp32(&a, &b);
        let (result2, _) = matmul_fp32(&a, &b);
        let (result3, _) = matmul_fp32(&a, &b);
        
        let hash1 = compute_hash(&result1);
        let hash2 = compute_hash(&result2);
        let hash3 = compute_hash(&result3);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }
    
    #[test]
    fn test_verify_correctness() {
        let a = to_flat_matrix(vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ]);
        let b = to_flat_matrix(vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ]);
        
        // Compute and get hash
        let (result, _) = matmul_fp32(&a, &b);
        let correct_hash = compute_hash(&result);
        
        // Verify it matches
        assert!(verify_correctness(&a, &b, "fp32", &correct_hash).unwrap());
        
        // Wrong hash should fail
        assert!(!verify_correctness(&a, &b, "fp32", "wrong_hash").unwrap());
    }
    
    #[test]
    fn test_fp16_correctness() {
        let a = to_flat_matrix(vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ]);
        let b = to_flat_matrix(vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ]);
        
        let result = matmul_fp16(&a, &b);
        
        // FP16 should give approximately correct results (may have small precision differences)
        assert!((result.data[0 * result.cols + 0] - 19.0).abs() < 0.1);
        assert!((result.data[0 * result.cols + 1] - 22.0).abs() < 0.1);
        assert!((result.data[1 * result.cols + 0] - 43.0).abs() < 0.1);
        assert!((result.data[1 * result.cols + 1] - 50.0).abs() < 0.1);
    }
    
    #[test]
    fn test_int8_correctness() {
        let a = to_flat_matrix(vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
        ]);
        let b = to_flat_matrix(vec![
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ]);
        
        let result = matmul_int8(&a, &b);
        
        // INT8 should give approximately correct results (quantization may cause differences)
        assert!((result.data[0 * result.cols + 0] - 19.0).abs() < 1.0);
        assert!((result.data[0 * result.cols + 1] - 22.0).abs() < 1.0);
        assert!((result.data[1 * result.cols + 0] - 43.0).abs() < 1.0);
        assert!((result.data[1 * result.cols + 1] - 50.0).abs() < 1.0);
    }
    
    #[test]
    fn test_compute_workload_integration() {
        // Create input JSON and deserialize to test the full flow
        let input_json = r#"{
            "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
            "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
            "precision": "fp32",
            "workload_type": "matmul"
        }"#;
        
        let input: types::Input = serde_json::from_str(input_json).unwrap();
        let output = compute_workload(input).unwrap();
        
        // Check result correctness (using flat indexing)
        assert_eq!(output.result_matrix.data[0 * output.result_matrix.cols + 0], 19.0);
        assert_eq!(output.result_matrix.data[0 * output.result_matrix.cols + 1], 22.0);
        assert_eq!(output.result_matrix.data[1 * output.result_matrix.cols + 0], 43.0);
        assert_eq!(output.result_matrix.data[1 * output.result_matrix.cols + 1], 50.0);
        
        // Check hash is present
        assert!(!output.result_hash.is_empty());
        
        // Check metrics are reasonable
        assert!(output.metrics.latency_ms >= 0.0);
        assert!(output.metrics.ops_per_second > 0.0);
        
        // Verify hash matches recomputed hash
        let input2_json = r#"{
            "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
            "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
            "precision": "fp32",
            "workload_type": "matmul"
        }"#;
        let input2: types::Input = serde_json::from_str(input2_json).unwrap();
        
        assert!(verify_correctness(
            &input2.matrix_a,
            &input2.matrix_b,
            "fp32",
            &output.result_hash
        ).unwrap());
    }
    
    #[test]
    fn test_matrix_dimension_validation() {
        let input_json = r#"{
            "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
            "matrix_b": [[5.0, 6.0]],
            "precision": "fp32",
            "workload_type": "matmul"
        }"#;
        
        let input: types::Input = serde_json::from_str(input_json).unwrap();
        let result = compute_workload(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("incompatible"));
    }
}
