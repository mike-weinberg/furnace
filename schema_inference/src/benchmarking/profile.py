#!/usr/bin/env python3
"""
Profiling suite for identifying bottlenecks in schema inference.

Uses timing analysis to identify which functions consume the most time.
"""

import json
import sys
import time
from pathlib import Path
from collections import defaultdict

sys.path.insert(0, str(Path(__file__).parent.parent / "lib"))

from infer_schema import infer_schema


def profile_inference():
    """Profile the inference function with timing analysis."""
    # Load test data
    examples_dir = Path(__file__).parent.parent / "tests" / "examples"

    # Find a complex schema to profile
    example_files = list(examples_dir.glob("**/schema_with_examples.json"))

    print(f"Found {len(example_files)} example files\n")
    print("=== Profile Results ===\n")

    # Just test first 3 files
    for test_file in example_files[:3]:
        with open(test_file) as f:
            data = json.load(f)

        examples = data["examples"]

        print(f"Schema: {test_file.parent.name}")
        print(f"Examples: {len(examples)}")

        # Warm up
        _ = infer_schema(examples)

        # Time 5 runs
        times = []
        for _ in range(5):
            start = time.perf_counter()
            _ = infer_schema(examples)
            elapsed = time.perf_counter() - start
            times.append(elapsed)

        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)

        print(f"  Min:  {min_time*1000:.2f}ms")
        print(f"  Avg:  {avg_time*1000:.2f}ms")
        print(f"  Max:  {max_time*1000:.2f}ms")
        print()


if __name__ == "__main__":
    profile_inference()
