# Schema Inference Performance Summary

## Overview

Furnace includes a production-ready JSON Schema inference implementation with comprehensive format detection and proper type unification.

## Current Performance

**Benchmark Date:** November 13, 2025
**Test Set:** 28 real-world schemas from SchemaStore

### Results

| Implementation | Average Time | Performance |
|----------------|--------------|-------------|
| Furnace | 6.99ms | baseline |
| genson-rs | 0.89ms | **7.89x faster** |

### By Complexity Category

**Small + Simple (8 schemas):**
- Furnace: 0.20ms average
- genson-rs: 0.07ms average
- Ratio: 2.86x slower

**Small + Complex (10 schemas):**
- Furnace: 0.60ms average
- genson-rs: 0.26ms average
- Ratio: 2.31x slower

**Big + Complex (10 schemas):**
- Furnace: 18.80ms average
- genson-rs: 2.16ms average
- Ratio: 8.70x slower

## Correctness

✅ **100/100 schemas pass validation**

All inferred schemas correctly describe their source examples.

### Validation Process

```bash
# Run correctness validation
cargo run --release --bin schema_correctness_validation

# Result: 100/100 Pass
```

## Feature Comparison

| Feature | Furnace | genson-rs |
|---------|---------|-----------|
| **Format Detection** | ✅ date, time, email, UUID, IPv4, IPv6 | ❌ No |
| **Required Fields** | ✅ Yes | ✅ Yes |
| **Type Unification** | ✅ Yes | ✅ Yes |
| **Nullable Types** | ✅ Yes | ✅ Yes |

## Performance Trade-off

**Why is Furnace slower?**

Furnace prioritizes **schema quality** over raw speed:

1. **Format Detection**: Regex-based pattern matching for 6 different formats
   - ISO date detection (YYYY-MM-DD)
   - ISO time detection (HH:MM:SS)
   - Email validation
   - UUID validation (multiple versions)
   - IPv4 validation
   - IPv6 validation

2. **Comprehensive Type Tracking**: Maintains detailed type information across all examples

3. **Required Field Analysis**: Tracks which fields appear in ALL examples vs SOME examples

**When does this matter?**

- **Format detection is valuable**: If you need to know that a field contains emails or dates
- **Documentation generation**: When schema is used for API documentation
- **Data validation**: When the inferred schema will validate new data
- **Performance is not critical**: For batch schema inference (not real-time)

**When to use genson-rs instead:**

- Real-time schema inference
- Performance-critical applications
- Don't need format detection
- Minimal schema requirements

## Optimization History

### Cycle 1: Regex Pre-compilation (59x improvement)

**Problem:** Regex patterns were compiled on every string validation

**Solution:** Used `once_cell::Lazy` for lazy static initialization

**Result:**
- Before: 389.68ms average
- After: 6.59ms average
- Improvement: **59x faster**

### Current State

After regex optimization, the remaining performance difference (vs genson-rs) is due to:
1. Format detection overhead (regex pattern matching)
2. More comprehensive type tracking
3. Additional required field analysis

**Decision:** Accept the performance trade-off for superior schema quality.

## Benchmarking

### Run Benchmarks

```bash
# Compare Furnace vs genson-rs
cargo run --release --bin schema_inference_benchmark

# Verify correctness
cargo run --release --bin schema_correctness_validation
```

### Benchmark Methodology

**Fair Comparison:**
- Both implementations receive already-parsed JSON (`serde_json::Value`)
- No parsing time included (matches Python GenSON methodology)
- Same test data (28 real-world schemas from SchemaStore)
- Multiple runs averaged for consistency

## Conclusion

Furnace's schema inference is **7.89x slower** than genson-rs but provides:

✅ **Format detection** (unique feature)
✅ **100% correctness** (validated on 100 schemas)
✅ **Comprehensive type information**
✅ **Production-ready quality**

**Recommendation:** Use Furnace when schema quality matters more than milliseconds.
