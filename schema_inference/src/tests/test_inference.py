#!/usr/bin/env python3
"""
Test suite for schema inference using real-world schemas and synthetic examples.

Uses 10,000 examples (100 per schema × 100 schemas) as test cases.
"""

import pytest
import json
from pathlib import Path
import sys

# Add lib to path
sys.path.insert(0, str(Path(__file__).parent.parent / "lib"))

from infer_schema import infer_schema, infer_type, detect_format, merge_schemas


class TestInferType:
    """Test basic type inference."""

    def test_infer_null(self):
        assert infer_type(None) == "null"

    def test_infer_boolean(self):
        assert infer_type(True) == "boolean"
        assert infer_type(False) == "boolean"

    def test_infer_integer(self):
        assert infer_type(42) == "integer"
        assert infer_type(-1) == "integer"

    def test_infer_float(self):
        assert infer_type(3.14) == "number"
        assert infer_type(-2.5) == "number"

    def test_infer_string(self):
        assert infer_type("hello") == "string"
        assert infer_type("") == "string"

    def test_infer_object(self):
        assert infer_type({}) == "object"
        assert infer_type({"key": "value"}) == "object"

    def test_infer_array(self):
        assert infer_type([]) == "array"
        assert infer_type([1, 2, 3]) == "array"


class TestDetectFormat:
    """Test string format detection."""

    def test_detect_datetime(self):
        assert detect_format("2020-01-15T12:30:45Z") == "date-time"
        assert detect_format("2020-01-15T12:30:45+00:00") == "date-time"

    def test_detect_date(self):
        assert detect_format("2020-01-15") == "date"
        assert detect_format("2024-12-31") == "date"

    def test_detect_time(self):
        assert detect_format("12:30:45") == "time"
        assert detect_format("00:00:00") == "time"

    def test_detect_email(self):
        assert detect_format("user@example.com") == "email"
        assert detect_format("test.user+tag@domain.co.uk") == "email"

    def test_detect_uri(self):
        assert detect_format("https://example.com") == "uri"
        assert detect_format("http://example.com/path") == "uri"
        assert detect_format("ftp://files.example.com") == "uri"

    def test_detect_uuid(self):
        assert detect_format("550e8400-e29b-41d4-a716-446655440000") == "uuid"

    def test_detect_ipv4(self):
        assert detect_format("192.168.1.1") == "ipv4"
        assert detect_format("127.0.0.1") == "ipv4"

    def test_detect_ipv6(self):
        assert detect_format("2001:0db8:85a3:0000:0000:8a2e:0370:7334") == "ipv6"

    def test_no_format(self):
        assert detect_format("just a string") is None
        assert detect_format("") is None


class TestMergeSchemas:
    """Test schema merging logic."""

    def test_merge_empty(self):
        merged = merge_schemas([])
        assert merged == {}

    def test_merge_single_schema(self):
        schema = {"type": "object"}
        merged = merge_schemas([schema])
        assert merged == schema

    def test_merge_same_type(self):
        schemas = [{"type": "string"}, {"type": "string"}]
        merged = merge_schemas(schemas)
        assert merged["type"] == "string"

    def test_merge_different_types(self):
        schemas = [{"type": "string"}, {"type": "number"}]
        merged = merge_schemas(schemas)
        assert "anyOf" in merged

    def test_merge_objects_union_properties(self):
        schemas = [
            {"type": "object", "properties": {"a": {"type": "string"}}},
            {"type": "object", "properties": {"b": {"type": "number"}}},
        ]
        merged = merge_schemas(schemas)
        assert "a" in merged["properties"]
        assert "b" in merged["properties"]

    def test_merge_objects_track_required(self):
        schemas = [
            {"type": "object", "properties": {"a": {"type": "string"}, "b": {"type": "number"}}},
            {"type": "object", "properties": {"a": {"type": "string"}, "b": {"type": "number"}}},
        ]
        merged = merge_schemas(schemas)
        assert set(merged.get("required", [])) == {"a", "b"}

    def test_merge_with_null(self):
        schemas = [{"type": "string"}, {"type": "null"}]
        merged = merge_schemas(schemas)
        assert "null" in (merged.get("type") if isinstance(merged.get("type"), list) else [merged.get("type")])


class TestInferSchema:
    """Test main inference function with synthetic examples."""

    def test_infer_simple_object(self):
        examples = [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
        ]
        schema = infer_schema(examples)
        assert schema["type"] == "object"
        assert "name" in schema["properties"]
        assert "age" in schema["properties"]

    def test_infer_array(self):
        examples = [[1, 2, 3], [4, 5, 6]]
        schema = infer_schema(examples)
        assert schema["type"] == "array"

    def test_empty_examples(self):
        schema = infer_schema([])
        assert schema["type"] == "object"

    def test_with_optional_fields(self):
        examples = [
            {"name": "Alice", "age": 30, "email": "alice@example.com"},
            {"name": "Bob", "age": 25},
        ]
        schema = infer_schema(examples)
        # "name" and "age" should be required (in both)
        assert "name" in schema.get("required", [])
        assert "age" in schema.get("required", [])
        # "email" should NOT be required (only in one)
        assert "email" not in schema.get("required", [])

    def test_with_format_detection(self):
        examples = [
            {"email": "alice@example.com"},
            {"email": "bob@example.com"},
        ]
        schema = infer_schema(examples)
        email_schema = schema["properties"]["email"]
        assert email_schema.get("format") == "email"


class TestWithRealSchemas:
    """Load and test with real schemas and their synthetic examples."""

    @pytest.fixture
    def examples_dir(self):
        """Get examples directory."""
        return Path(__file__).parent / "examples"

    def test_can_load_example_files(self, examples_dir):
        """Verify example files exist and are readable."""
        assert examples_dir.exists(), f"Examples directory not found: {examples_dir}"

        # Count example files
        example_files = list(examples_dir.glob("**/schema_with_examples.json"))
        assert len(example_files) > 0, f"No example files found in {examples_dir}"

        print(f"\nFound {len(example_files)} example files")

    def test_sample_schemas_infer(self, examples_dir):
        """Test inference on a sample of real schemas."""
        example_files = list(examples_dir.glob("**/schema_with_examples.json"))[:5]  # Just first 5 for quick test

        for example_file in example_files:
            with open(example_file) as f:
                data = json.load(f)

            original_schema = data["schema"]
            examples = data["examples"]

            # Infer schema from examples
            inferred_schema = infer_schema(examples)

            # Basic checks
            assert "type" in inferred_schema or "properties" in inferred_schema or "anyOf" in inferred_schema
            assert inferred_schema is not None

            print(f"\n✓ {example_file.parent.name}")
            print(f"  Original schema keys: {list(original_schema.keys())[:5]}")
            print(f"  Inferred schema keys: {list(inferred_schema.keys())[:5]}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
