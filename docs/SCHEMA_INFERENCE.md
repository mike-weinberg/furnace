# Schema Inference Module

JSON Schema inference with comprehensive format detection, ported from Python to Rust.

## Quick Start

```bash
# Run benchmarks
cargo run --release --bin schema_inference_benchmark

# Verify correctness
cargo run --release --bin schema_correctness_validation
```

## Features

- **Format Detection**: date, time, email, UUID, IPv4, IPv6 (not in genson-rs!)
- **Required Field Tracking**: Identifies fields present in all examples
- **Type Unification**: Properly merges schemas from multiple examples
- **100% Correctness**: Validated on 100 real-world schemas

## Performance

**Current:** 6.99ms average (7.89x slower than genson-rs)
**Trade-off:** Richer schema quality with format detection

See [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) for detailed analysis.

## Correctness Validation

✅ 100/100 schemas pass validation

The schema inference implementation is validated through integration tests that verify inferred schemas correctly describe the data they were inferred from.

**Run validation:**
```bash
cargo run --release --bin schema_correctness_validation
```

## Project Structure

```
schema_inference/
├── README.md                      # This file
├── PERFORMANCE_SUMMARY.md         # Detailed performance analysis
├── src/
│   ├── tests/
│   │   └── examples/              # 100 real-world test schemas
│   └── benchmarking/
│       └── benchmark.py           # Python reference benchmarks
└── performance_graphs.png         # Visual performance comparison
```

## Documentation

- [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) - Comprehensive performance analysis
- [../GUIDE.md](../GUIDE.md) - Usage guide for schema inference
- [../README.md](../README.md) - Main project documentation
