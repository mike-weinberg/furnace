#!/usr/bin/env python3
"""
Benchmarking and profiling suite for schema inference.

Measures performance across schema complexity dimensions and compares with genson.
"""

import json
import time
import sys
from pathlib import Path
from typing import Dict, List, Any
import psutil
import os

sys.path.insert(0, str(Path(__file__).parent.parent / "lib"))

from infer_schema import infer_schema


class BenchmarkSuite:
    """Benchmarking and profiling for schema inference."""

    def __init__(self):
        """Initialize benchmark suite."""
        self.results = {}
        self.examples_dir = Path(__file__).parent.parent / "tests" / "examples"
        self.manifest_file = self.examples_dir / "manifest.json"

    def load_manifest(self) -> List[Dict[str, Any]]:
        """Load benchmark manifest."""
        with open(self.manifest_file) as f:
            return json.load(f)

    def benchmark_by_complexity(self):
        """Benchmark inference by schema complexity."""
        print("=== Benchmarking by Complexity ===\n")

        manifest = self.load_manifest()
        categories = {}

        for entry in manifest:
            cat = entry["category"]
            if cat not in categories:
                categories[cat] = []
            categories[cat].append(entry)

        for category, entries in sorted(categories.items()):
            print(f"{category}:")
            times = []

            for entry in entries[:10]:  # Sample 10 from each category
                schema_name = entry["name"]
                schema_file_path = entry["schema_file"]

                if schema_file_path.startswith("/"):
                    schema_file = Path(schema_file_path)
                else:
                    schema_file = self.examples_dir.parent.parent / schema_file_path

                output_file = schema_file.parent / "schema_with_examples.json"

                with open(output_file) as f:
                    data = json.load(f)

                examples = data["examples"]

                # Time the inference
                start = time.perf_counter()
                _ = infer_schema(examples)
                elapsed = time.perf_counter() - start

                times.append(elapsed)
                print(f"  {schema_name:<50} {elapsed*1000:8.2f}ms")

            avg = sum(times) / len(times)
            print(f"  Average: {avg*1000:8.2f}ms\n")

            self.results[category] = {"avg_ms": avg * 1000, "samples": len(times)}

    def benchmark_by_example_count(self):
        """Benchmark how performance scales with number of examples."""
        print("=== Benchmarking by Example Count ===\n")

        manifest = self.load_manifest()
        test_schema = None

        # Find a medium-complexity schema to test with
        for entry in manifest:
            if entry["category"] == "small+complex":
                test_schema = entry
                break

        if not test_schema is None:
            schema_file_path = test_schema["schema_file"]
            if schema_file_path.startswith("/"):
                schema_file = Path(schema_file_path)
            else:
                schema_file = self.examples_dir.parent.parent / schema_file_path

            output_file = schema_file.parent / "schema_with_examples.json"

            with open(output_file) as f:
                data = json.load(f)

            all_examples = data["examples"]

            # Test with varying numbers of examples
            for count in [1, 10, 25, 50, 100]:
                examples = all_examples[:count]

                start = time.perf_counter()
                _ = infer_schema(examples)
                elapsed = time.perf_counter() - start

                print(f"  {count:3d} examples: {elapsed*1000:8.2f}ms")

                self.results[f"count_{count}"] = elapsed * 1000

    def benchmark_memory_usage(self):
        """Benchmark memory usage during inference."""
        print("\n=== Memory Usage ===\n")

        manifest = self.load_manifest()
        process = psutil.Process(os.getpid())

        # Sample from each complexity category
        categories = {}
        for entry in manifest:
            cat = entry["category"]
            if cat not in categories:
                categories[cat] = []
            categories[cat].append(entry)

        for category, entries in categories.items():
            entry = entries[0]  # Just first one per category
            schema_name = entry["name"]
            schema_file_path = entry["schema_file"]

            if schema_file_path.startswith("/"):
                schema_file = Path(schema_file_path)
            else:
                schema_file = self.examples_dir.parent.parent / schema_file_path

            output_file = schema_file.parent / "schema_with_examples.json"

            with open(output_file) as f:
                data = json.load(f)

            examples = data["examples"]

            # Measure memory before
            process.memory_info()  # Dummy call to ensure measurement

            mem_start = process.memory_info().rss / (1024 * 1024)  # MB

            _ = infer_schema(examples)

            mem_end = process.memory_info().rss / (1024 * 1024)  # MB
            mem_used = mem_end - mem_start

            print(f"  {category:<20} {mem_used:8.2f} MB")

    def compare_with_genson(self):
        """Compare performance with genson library."""
        print("\n=== Comparison with GenSON ===\n")

        try:
            import genson
        except ImportError:
            print("  GenSON not installed, skipping comparison")
            return

        manifest = self.load_manifest()

        times_ours = []
        times_genson = []

        for entry in manifest[:5]:  # Test first 5 schemas
            schema_name = entry["name"]
            schema_file_path = entry["schema_file"]

            if schema_file_path.startswith("/"):
                schema_file = Path(schema_file_path)
            else:
                schema_file = self.examples_dir.parent.parent / schema_file_path

            output_file = schema_file.parent / "schema_with_examples.json"

            with open(output_file) as f:
                data = json.load(f)

            examples = data["examples"]

            # Our implementation
            start = time.perf_counter()
            _ = infer_schema(examples)
            time_ours = time.perf_counter() - start
            times_ours.append(time_ours)

            # GenSON
            try:
                start = time.perf_counter()
                builder = genson.SchemaBuilder()
                for ex in examples:
                    builder.add_object(ex)
                _ = builder.to_schema()
                time_genson = time.perf_counter() - start
                times_genson.append(time_genson)

                ratio = time_genson / time_ours if time_ours > 0 else 0
                print(f"  {schema_name:<40} Ours: {time_ours*1000:7.2f}ms  GenSON: {time_genson*1000:7.2f}ms  Ratio: {ratio:5.1f}x")
            except Exception as e:
                print(f"  {schema_name:<40} Ours: {time_ours*1000:7.2f}ms  GenSON: ERROR - {str(e)[:30]}")

        if times_genson and times_ours:
            avg_ours = sum(times_ours) / len(times_ours)
            avg_genson = sum(times_genson) / len(times_genson)
            ratio = avg_genson / avg_ours if avg_ours > 0 else 0
            print(f"\n  Average ratio (GenSON/Ours): {ratio:.1f}x")

    def run_all_benchmarks(self):
        """Run all benchmarks."""
        print("Starting Benchmarking Suite\n")
        print("=" * 70)

        self.benchmark_by_complexity()
        self.benchmark_by_example_count()
        self.benchmark_memory_usage()
        self.compare_with_genson()

        print("\n" + "=" * 70)
        print("Benchmarking Complete")

        return self.results


def main():
    """Run benchmarks."""
    suite = BenchmarkSuite()
    results = suite.run_all_benchmarks()


if __name__ == "__main__":
    main()
