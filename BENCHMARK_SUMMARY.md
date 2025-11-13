# Schema Inference Benchmark - Comprehensive Comparison

## Overview

A comprehensive benchmarking suite has been created to compare three different schema inference implementations side-by-side. This benchmark provides detailed performance analysis across real-world test cases.

## Key Files Created

### Main Benchmark
- **`src/bin/schema_comparison_benchmark.rs`** (336 lines)
  - Compares all three implementations on identical test data
  - Provides detailed timing, statistics, and improvement ratios
  - Generates comprehensive performance reports

### Supporting Files
- **`BENCHMARK_GUIDE.md`** - How to run and interpret the benchmarks
- **`benchmark_results.txt`** - Latest benchmark results snapshot

## Benchmark Results Summary

### Performance Comparison

| Implementation | Average Time | vs Old | vs Genson-rs |
|----------------|-------------|--------|--------------|
| **Old (merge-based)** | 7.16 ms | baseline | 7.33x slower |
| **Streaming (new)** | 1.20 ms | **5.95x faster** | 1.23x slower |
| **Genson-rs** | 0.98 ms | 7.33x faster | baseline |

### Key Findings

1. **Streaming Implementation is 5.95x faster than Old**
   - 495% speed improvement
   - Biggest wins on complex schemas (up to 8.69x faster)
   - Consistent improvements across all complexity categories

2. **Comparison to Genson-rs**
   - Streaming is 1.23x slower than Genson-rs
   - Only 18.8% behind the highly-optimized third-party library
   - Excellent result considering Genson-rs uses simd_json

3. **Category Performance**
   - Big+Complex: 6.26x faster (18.91ms → 3.02ms)
   - Small+Complex: 3.21x faster (0.97ms → 0.30ms)
   - Small+Simple: 3.46x faster (0.21ms → 0.06ms)

### Top Performance Improvements

1. **Application_Accelerator**: 8.69x faster
2. **.adonisrc.json**: 7.15x faster
3. **Airlock_Microgateway**: 7.00x faster
4. **cibuildwheel**: 5.40x faster
5. **Chromia_Seeder_Root_Config**: 5.21x faster

## Benchmark Features

### Comprehensive Analysis
- Detailed per-schema timing across all implementations
- Statistical summaries (average, min, max)
- Performance ratios and improvement percentages
- Category-based breakdown
- Top performers list

### Clear Presentation
- Beautiful formatted tables with box-drawing characters
- Color-coded results (✓ for improvements, ✗ for gaps)
- Percentage improvements clearly displayed
- Final verdict summary

### Fair Comparison
- Same test data for all implementations
- Data conversion overhead excluded from timing
- Multiple runs across different complexity levels
- Real-world schemas from JSON Schema Store

## How to Run

```bash
# Run comprehensive comparison
cargo run --release --bin schema_comparison_benchmark

# Run individual benchmarks
cargo run --release --bin schema_inference_benchmark  # Old implementation
cargo run --release --bin schema_builder_benchmark    # New streaming
cargo run --release --bin genson_benchmark            # Genson-rs
```

## Implementation Details

### Three Implementations Compared

1. **Old (infer_schema)**
   - Location: `src/schema_inference.rs`
   - Approach: Creates intermediate schemas, merges them
   - Pros: Straightforward, easy to understand
   - Cons: Slower due to multiple allocations and merges

2. **Streaming (SchemaBuilder)**
   - Location: `src/schema_builder.rs`
   - Approach: Accumulator pattern, single-pass build
   - Pros: 5.95x faster, memory efficient
   - Cons: Slightly more complex code

3. **Genson-rs (third-party)**
   - Library: `genson-rs` with `simd_json`
   - Approach: Highly optimized with SIMD operations
   - Pros: Fastest implementation
   - Cons: External dependency, different API

## Conclusion

The new **Streaming (SchemaBuilder)** implementation provides:
- **Massive improvement** over the old merge-based approach (5.95x faster)
- **Near-competitive performance** with specialized third-party library (only 1.23x slower)
- **Consistent wins** across all complexity categories
- **Excellent choice** for production use - great balance of performance and maintainability

The benchmark clearly demonstrates that the refactoring effort was successful, delivering substantial performance improvements while maintaining code quality and correctness.
