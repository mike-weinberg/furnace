# Python Reference Implementation

This directory contains the original Python implementation of JSON schema inference, kept for reference and comparison purposes.

## Setup

```bash
cd benchmarks/python
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Running

```bash
python3 benchmarking/benchmark.py
```

## Purpose

The Python implementation serves as:
1. **Reference implementation** - Original algorithm before Rust port
2. **Correctness validation** - Verify Rust implementation matches Python behavior
3. **Performance baseline** - Compare Python vs Rust performance

The production Rust implementation is in `src/schema/`.
