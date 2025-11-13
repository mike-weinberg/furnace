# Schema Inference Performance Benchmarking Guide

This project includes comprehensive benchmarks to compare different schema inference implementations.

## Available Benchmarks

### 1. Individual Implementation Benchmarks

Run each implementation separately:

```bash
# Old merge-based implementation
cargo run --release --bin schema_inference_benchmark

# New streaming implementation (SchemaBuilder)
cargo run --release --bin schema_builder_benchmark

# Genson-rs (third-party library)
cargo run --release --bin genson_benchmark
```

### 2. Comprehensive Comparison Benchmark

Run all three implementations side-by-side with detailed comparisons:

```bash
cargo run --release --bin schema_comparison_benchmark
```

This benchmark provides:
- Detailed timing for each test case across all three implementations
- Statistical summaries (average, min, max)
- Performance comparison ratios
- Top 5 improvements from old to streaming implementation
- Performance breakdown by complexity category
- Clear verdict on which implementation performs best

## Understanding the Results

### Implementations Compared

1. **Old (infer_schema)**: Merge-based approach that creates intermediate schemas and merges them
2. **Streaming (SchemaBuilder)**: Accumulator pattern that collects statistics and builds schema once
3. **Genson-rs**: Third-party library using simd_json for performance

### Key Metrics

- **Average time**: Mean time across all test cases
- **Min/Max time**: Range of performance across different schema complexities
- **Speedup ratio**: How many times faster one implementation is vs another
- **Improvement %**: Percentage speed increase/decrease

### Categories

Test cases are organized by complexity:
- **big+complex**: Large schemas with deep nesting
- **small+complex**: Small schemas with complex structures
- **small+simple**: Simple, flat schemas

## Sample Output

```
═══════════════════════════════════════════════════════════════════════════
                          SUMMARY STATISTICS                               
═══════════════════════════════════════════════════════════════════════════

Total benchmarks run: 28

┌─────────────────┬──────────────┬──────────────┬──────────────┐
│ Metric          │ Old (ms)     │ Stream (ms)  │ Genson (ms)  │
├─────────────────┼──────────────┼──────────────┼──────────────┤
│ Average         │         7.60 │         1.55 │         1.06 │
│ Minimum         │         0.03 │         0.01 │         0.00 │
│ Maximum         │       168.64 │        32.79 │        17.46 │
└─────────────────┴──────────────┴──────────────┴──────────────┘
```

## Interpreting Results

- **Streaming vs Old**: Shows improvement from refactoring to accumulator pattern
- **Streaming vs Genson-rs**: Shows how our implementation compares to a specialized library
- **Performance by Category**: Reveals which schema types benefit most from each approach

## Test Data

All benchmarks use the same test data from `schema_inference/src/tests/examples/`, which includes:
- Real-world schema examples from various JSON Schema Store entries
- Different complexity levels to test performance across various scenarios
- Multiple examples per schema to test schema merging/accumulation
