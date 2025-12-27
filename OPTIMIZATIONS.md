# Optimization Log

Track optimizations and benchmark results over time.

## Format

Each entry should include:
- **Date/Time**: When the optimization was made
- **Change**: What was changed
- **Benchmark Results**: Before/after metrics
- **Impact**: Performance improvement observed

---

## Optimization History

### Performance Baseline Notes

**Local Machine (Native x86_64/ARM):**
- Latency: ~0.011 ms (native execution)
- Fast due to native CPU architecture

**Docker (RISC-V Emulated):**
- Latency: ~0.152 ms (QEMU emulation)
- ~13x slower due to RISC-V emulation on non-RISC-V hardware
- **Note:** This is expected! Docker image targets RISC-V, so on x86_64/ARM it uses QEMU emulation
- **For hackathon:** Benchmarking will be on actual RISC-V hardware, so Docker performance on local machine is not representative

### 2025-01-XX - Initial Implementation
- **Change**: Basic MatMul implementation with cache-friendly transpose
- **Benchmark**: [Add benchmark results here]
- **Metrics**: 
  - Latency: X ms
  - Throughput: X ops/sec
  - Memory: X MB

### [Next Optimization]
- **Change**: [Description]
- **Benchmark**: [Results]
- **Impact**: [Improvement %]

---

## Benchmarking Workload

Use consistent test cases for fair comparison:

```json
{
  "matrix_a": [[...]],  // Standard size
  "matrix_b": [[...]],  // Standard size  
  "precision": "fp32"
}
```

## Running Benchmarks

```bash
# Run benchmark and log results
./benchmark.sh >> OPTIMIZATIONS.md
```


### 2025-12-26 17:33:06 UTC
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency**: 0.023125 ms
- **Throughput**: 172972972.972973 ops/sec
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:34:05 UTC
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency**: 0.021208 ms
- **Throughput**: 188608072.4254998 ops/sec
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:34:44 UTC
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency**: 0.157542 ms
- **Throughput**: 25390054.715567913 ops/sec
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:34:55 UTC
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency**: 0.015542000000000002 ms
- **Throughput**: 257367134.21696046 ops/sec
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:40:54 UTC (N=3 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.013875, median=0.014375, p90=0.015167, max=0.015167
- **Throughput (ops/sec)**: min=263730467.46225357, median=278260869.5652174, p90=288288288.2882883, max=288288288.2882883
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:41:34 UTC (N=50 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.011542, median=0.013562, p90=0.037, max=0.35554199999999997
- **Throughput (ops/sec)**: min=11250428.922602674, median=294942456.18201876, p90=327653997.378768, max=346560388.1476347
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 17:42:40 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.011292, median=0.014188000000000001, p90=0.034666999999999996, max=2.049375
- **Throughput (ops/sec)**: min=1951814.5776151265, median=281929007.8297148, p90=325414904.0026033, max=354233085.3701736
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:00:36 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.010958, median=0.021917, p90=0.026958, max=0.039292
- **Throughput (ops/sec)**: min=101801893.51521938, median=182507400.154834, p90=295399158.1123994, max=365030114.9844862
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:01:02 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.011250000000000001, median=0.019521, p90=0.025334, max=0.061875
- **Throughput (ops/sec)**: min=64646464.646464646, median=204907772.60811034, p90=291800408.52057195, max=355555555.5555555
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`



### 2024-12-XX - Loop Order + Cache Blocking Optimization

**Changes:**
- **Loop order optimization**: Changed from `i -> j -> k` to `i -> p -> j` (where p is the reduction dimension)
  - Hoists `a_ip = A[i,p]` out of inner loop for better register reuse
  - Streams across `B[p, :]` (contiguous) and `C[i, :]` (contiguous)
  - Eliminates the need for transposing B matrix (saves memory allocation)
  
- **Cache blocking (tiling)**: Added tiling for fp32 MatMul
  - Block sizes: BM=16 (rows of C), BN=64 (cols of C), BK=64 (reduction dimension)
  - Reuses hot data in L1 cache
  - Applied optimized loop order within each tile
  
- **Applied to all precisions**: 
  - fp32: Full cache blocking with optimized loop order
  - fp16: Optimized loop order (conversion overhead may still dominate)
  - int8: Optimized loop order (conversion overhead may still dominate)

**Expected impact:**
- Significant latency reduction for large matrices (cache-friendly access patterns)
- Better register reuse from hoisting `a_ip`
- Improved cache hit rates from blocking
- Reduced memory allocation (no B transpose needed)

**Note**: Run benchmark to measure actual performance improvements vs. baseline

### 2025-12-26 21:04:36 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.0024159999999999997, median=0.004521000000000001, p90=0.007625, max=0.011834
- **Throughput (ops/sec)**: min=338009126.24640864, median=884779098.7817408, p90=1432664756.4469914, max=1655629139.0728478
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:05:11 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.002375, median=0.004646, p90=0.0077919999999999994, max=0.026166
- **Throughput (ops/sec)**: min=152870136.81877246, median=860973250.9453958, p90=1433178072.3754926, max=1684210526.3157895
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:06:15 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.002417, median=0.0048330000000000005, p90=0.0077919999999999994, max=0.627084
- **Throughput (ops/sec)**: min=6378730.760153345, median=827643285.7438444, p90=1333333333.3333333, max=1654944145.6350849
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`



### 2024-12-XX - B Packing + Microkernel Optimization

**Changes:**
- **B matrix packing**: Pack each B tile into contiguous buffer before microkernel
  - Layout: B_pack[k * BN + j] = B[pp + k][jj + j] 
  - Eliminates awkward strides and enables vector loads on RISC-V
  - Packed once per (pp, jj) tile and reused
  
- **Microkernel with unrolling**: Compute MR×NR blocks (MR=4, NR=8)
  - Unrolls p loop by 4 (P_UNROLL=4) to keep accumulators in registers
  - Processes 4×8 = 32 elements at a time within each tile
  - Reduces register spills and improves instruction-level parallelism
  
- **Memory access pattern**:
  - A: accessed row-wise (contiguous within row)
  - B: packed into contiguous buffer (perfect streaming)
  - C: written back once per microkernel block

**Implementation details:**
- MR = 4 (microkernel rows), NR = 8 (microkernel columns)
- P_UNROLL = 4 (p loop unrolling factor)
- Pack buffer allocated once and reused
- All operations within microkernel keep values in registers when possible

**Expected impact:**
- Significant performance improvement on RISC-V (enables vectorization)
- Better cache utilization from packed B access
- Reduced register spills from unrolling
- Improved instruction-level parallelism

**Note**: Run benchmark to measure actual performance improvements vs. previous version

### 2025-12-26 21:10:49 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.0115, median=0.019770500000000003, p90=0.025625, max=0.159917
- **Throughput (ops/sec)**: min=25012975.481030785, median=202348907.4254132, p90=290023201.8561485, max=347826086.95652175
- **Memory**: 0.0030517578125 MB
- **Hash**: `7b173eb9cb4f52f7ad29b84c57cba6dbc154a0a5e57c41d4e4ff1127d0346d94`


### 2025-12-26 21:11:24 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.011708, median=0.0221665, p90=0.025667, max=0.04575
- **Throughput (ops/sec)**: min=87431693.98907104, median=180452484.69696134, p90=294485754.25163805, max=341646737.27365905
- **Memory**: 0.0030517578125 MB
- **Hash**: `7b173eb9cb4f52f7ad29b84c57cba6dbc154a0a5e57c41d4e4ff1127d0346d94`


### 2025-12-26 21:14:09 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.0024159999999999997, median=0.004521000000000001, p90=0.006834, max=0.010708
- **Throughput (ops/sec)**: min=373552484.12401944, median=884779098.7817408, p90=1454545454.5454545, max=1655629139.0728478
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:20:41 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.002667, median=0.0055005, p90=0.008624999999999999, max=0.133
- **Throughput (ops/sec)**: min=30075187.96992481, median=727248015.1074026, p90=1185185185.1851852, max=1499812523.4345706
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:23:49 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.001833, median=0.003292, p90=0.004583, max=0.025833000000000002
- **Throughput (ops/sec)**: min=154840707.62203383, median=1215066828.6755772, p90=1811594202.8985507, max=2182214948.1723948
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:31:47 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.0018750000000000001, median=0.0032294999999999997, p90=0.003708, max=0.007542
- **Throughput (ops/sec)**: min=530363298.8597189, median=1238631732.866697, p90=1777777777.7777777, max=2133333333.3333333
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:37:13 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.00925, median=0.016375, p90=0.020583, max=0.028584
- **Throughput (ops/sec)**: min=139938427.0920795, median=244375870.29522723, p90=396707329.1679064, max=432432432.4324325
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:37:39 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  20,
  20
], B=[
  20,
  10
]
- **Latency (ms)**: min=0.009875, median=0.0191045, p90=0.021124999999999998, max=0.031791
- **Throughput (ops/sec)**: min=125821773.45789689, median=209374995.7189611, p90=278260869.5652174, max=405063291.1392405
- **Memory**: 0.0030517578125 MB
- **Hash**: `32b27a05167a0e88361342e34a9dcad678925693e94c28ea3c437028fc58436a`


### 2025-12-26 21:54:30 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=4.397333, median=5.599562, p90=5.970291, max=6.5225420000000005
- **Throughput (ops/sec)**: min=1971844719.4360726, median=2296865560.4358425, p90=2707362654.6191993, max=2924827389.692798
- **Memory**: 6.1337890625 MB
- **Hash**: `406d959ff8d44880e12abc5334ceabec7e754d4eec74c01648dae20088efc78b`


### 2025-12-26 22:04:49 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=4.378209, median=5.629854, p90=5.987875, max=6.502542
- **Throughput (ops/sec)**: min=1977909562.1373918, median=2284507151.8739204, p90=2725458783.6406016, max=2937603024.43305
- **Memory**: 6.1337890625 MB
- **Hash**: `406d959ff8d44880e12abc5334ceabec7e754d4eec74c01648dae20088efc78b`


### 2025-12-26 22:10:59 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=0.983833, median=1.3613955, p90=4.918708, max=14.970291999999999
- **Throughput (ops/sec)**: min=859130870.6603719, median=9447391188.567879, p90=11488978158.19963, max=13072787759.711252
- **Memory**: 6.1337890625 MB
- **Hash**: `800847aef907b7b2603c7814c4e47ef3b586506ff73c9eed11def0e861af06b7`


### 2025-12-26 22:11:44 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=0.965209, median=1.2406044999999999, p90=5.490458, max=24.362583
- **Throughput (ops/sec)**: min=527917749.93644965, median=10368391604.680494, p90=12108212327.529072, max=13325031159.054672
- **Memory**: 6.1337890625 MB
- **Hash**: `800847aef907b7b2603c7814c4e47ef3b586506ff73c9eed11def0e861af06b7`


### 2025-12-26 22:17:24 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=0.424916, median=0.4286875, p90=0.980959, max=2.113584
- **Throughput (ops/sec)**: min=6085133119.857077, median=30001904607.926083, p90=30247408327.218678, max=30268194184.262302
- **Memory**: 6.1337890625 MB
- **Hash**: `78badc3e28acab83c741116ae313b1ce71e8681d8c0513b125ba7e9b86027b2a`


### 2025-12-26 22:22:48 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=2.646208, median=2.6912085, p90=7.670167, max=10.492417
- **Throughput (ops/sec)**: min=1225784297.364468, median=4779057662.983688, p90=4826811549.8747835, max=4860328439.79007
- **Memory**: 6.1337890625 MB
- **Hash**: `78badc3e28acab83c741116ae313b1ce71e8681d8c0513b125ba7e9b86027b2a`


### 2025-12-26 22:23:36 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=0.424875, median=0.4281455, p90=0.977375, max=3.759875
- **Throughput (ops/sec)**: min=3420709465.0753016, median=30039881942.174774, p90=30235533352.92389, max=30271115033.83348
- **Memory**: 6.1337890625 MB
- **Hash**: `78badc3e28acab83c741116ae313b1ce71e8681d8c0513b125ba7e9b86027b2a`


### 2025-12-26 22:26:22 UTC (N=100 runs)
- **Precision**: fp16
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=4.925458, median=5.7831455, p90=11.872167, max=18.594791999999998
- **Throughput (ops/sec)**: min=691668936.1193178, median=2223952398.981576, p90=2557370347.290187, max=2611217068.544692
- **Memory**: 6.1337890625 MB
- **Hash**: `dc936cd446cf7580298d3d42dc92b5c764886df438d3597e92cb0e2d61b8666f`


### 2025-12-26 22:27:34 UTC (N=100 runs)
- **Precision**: int8
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=7.73325, median=8.015937000000001, p90=13.365, max=27.567416
- **Throughput (ops/sec)**: min=466544996.4552354, median=1604483708.3112607, p90=1656690496.2082653, max=1663135163.09443
- **Memory**: 6.1337890625 MB
- **Hash**: `d51104857b9b2488e6d0b2c82820a81530f1cb27164c4e84f9dab2eef16f0c39`


### 2025-12-26 22:28:25 UTC (N=100 runs)
- **Precision**: fp32
- **Matrix Size**: A=[
  16,
  50240
], B=[
  50240,
  16
]
- **Latency (ms)**: min=0.425041, median=0.4280625, p90=0.97875, max=1.618125
- **Throughput (ops/sec)**: min=7948359984.550019, median=30045706011.382324, p90=30241434120.557167, max=30259292632.945995
- **Memory**: 6.1337890625 MB
- **Hash**: `78badc3e28acab83c741116ae313b1ce71e8681d8c0513b125ba7e9b86027b2a`

