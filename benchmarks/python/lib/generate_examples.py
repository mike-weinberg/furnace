#!/usr/bin/env python3
"""
Generate synthetic JSON examples from schemas with maximum entropy.
Goal: 100 examples per schema, maximizing diversity while respecting constraints.
"""

import json
import random
import string
from pathlib import Path
from typing import Any, Dict, List, Optional
from datetime import datetime, timedelta
import uuid


class SchemaExampleGenerator:
    """Generate diverse JSON examples from JSON Schema."""

    def __init__(self, seed: int = 42):
        """Initialize with random seed for reproducibility."""
        random.seed(seed)
        self.generation_count = 0

    def generate_examples(self, schema: Dict[str, Any], count: int = 100) -> List[Dict[str, Any]]:
        """Generate multiple diverse examples from schema."""
        examples = []
        for i in range(count):
            # Vary randomness slightly for each example
            random.seed(42 + i)
            self.generation_count = i
            example = self.generate_from_schema(schema)
            examples.append(example)
        return examples

    def generate_from_schema(self, schema: Dict[str, Any]) -> Any:
        """Generate a single value conforming to schema."""
        # Handle $ref (simplified - doesn't resolve external refs)
        if "$ref" in schema:
            return self._generate_placeholder(schema)

        # Handle type
        schema_type = schema.get("type")

        if isinstance(schema_type, list):
            # Multiple possible types - pick one randomly
            schema_type = random.choice(schema_type)

        if schema_type == "object":
            return self._generate_object(schema)
        elif schema_type == "array":
            return self._generate_array(schema)
        elif schema_type == "string":
            return self._generate_string(schema)
        elif schema_type == "number":
            return self._generate_number(schema)
        elif schema_type == "integer":
            return self._generate_integer(schema)
        elif schema_type == "boolean":
            return random.choice([True, False])
        elif schema_type == "null":
            return None
        elif "enum" in schema:
            return random.choice(schema["enum"])
        elif "const" in schema:
            return schema["const"]
        elif "properties" in schema:
            # No type specified but has properties - assume object
            return self._generate_object(schema)
        elif "items" in schema:
            # No type specified but has items - assume array
            return self._generate_array(schema)
        else:
            # No type info - generate random value
            return self._generate_random_value()

    def _generate_object(self, schema: Dict[str, Any]) -> Dict[str, Any]:
        """Generate an object conforming to schema."""
        obj = {}
        properties = schema.get("properties", {})
        required = schema.get("required", [])

        # Generate required properties
        for prop in required:
            if prop in properties:
                obj[prop] = self.generate_from_schema(properties[prop])
            else:
                obj[prop] = self._generate_random_value()

        # Randomly include optional properties (60% chance)
        for prop, prop_schema in properties.items():
            if prop not in required and random.random() < 0.6:
                obj[prop] = self.generate_from_schema(prop_schema)

        # Handle additionalProperties
        additional = schema.get("additionalProperties")
        if additional is True:
            # Add some random properties (0-3)
            for _ in range(random.randint(0, 3)):
                key = self._generate_random_string(5, 10)
                obj[key] = self._generate_random_value()
        elif isinstance(additional, dict):
            # Add properties conforming to schema
            for _ in range(random.randint(0, 2)):
                key = self._generate_random_string(5, 10)
                obj[key] = self.generate_from_schema(additional)

        return obj

    def _generate_array(self, schema: Dict[str, Any]) -> List[Any]:
        """Generate an array conforming to schema."""
        min_items = schema.get("minItems", 0)
        max_items = schema.get("maxItems", 10)
        items_schema = schema.get("items")

        # Vary array length across examples
        if self.generation_count < 33:
            length = min_items
        elif self.generation_count < 66:
            length = (min_items + max_items) // 2
        else:
            length = min(max_items, min_items + random.randint(0, 5))

        length = max(min_items, min(max_items, length))

        if items_schema is None:
            # No schema - any items
            return [self._generate_random_value() for _ in range(length)]
        elif isinstance(items_schema, dict):
            # Single schema for all items
            return [self.generate_from_schema(items_schema) for _ in range(length)]
        elif isinstance(items_schema, list):
            # Tuple validation - each position has its own schema
            return [self.generate_from_schema(s) for s in items_schema[:length]]
        else:
            return []

    def _generate_string(self, schema: Dict[str, Any]) -> str:
        """Generate a string conforming to schema."""
        # Check for format
        format_type = schema.get("format")

        if format_type == "date-time":
            return self._generate_datetime()
        elif format_type == "date":
            return self._generate_date()
        elif format_type == "time":
            return self._generate_time()
        elif format_type == "email":
            return self._generate_email()
        elif format_type == "uri":
            return self._generate_uri()
        elif format_type == "uuid":
            return str(uuid.uuid4())
        elif format_type == "ipv4":
            return f"{random.randint(1,255)}.{random.randint(0,255)}.{random.randint(0,255)}.{random.randint(0,255)}"
        elif format_type == "ipv6":
            return ":".join(f"{random.randint(0,65535):04x}" for _ in range(8))
        elif "enum" in schema:
            return random.choice(schema["enum"])
        elif "const" in schema:
            return schema["const"]
        elif "pattern" in schema:
            # Simplified - just generate random string (proper implementation would use regex)
            return self._generate_random_string(5, 15)
        else:
            # Regular string
            min_len = schema.get("minLength", 1)
            max_len = schema.get("maxLength", 50)
            return self._generate_random_string(min_len, max_len)

    def _generate_number(self, schema: Dict[str, Any]) -> float:
        """Generate a number conforming to schema."""
        minimum = schema.get("minimum", -1000)
        maximum = schema.get("maximum", 1000)
        exclusive_min = schema.get("exclusiveMinimum", False)
        exclusive_max = schema.get("exclusiveMaximum", False)

        if exclusive_min:
            minimum += 0.01
        if exclusive_max:
            maximum -= 0.01

        return round(random.uniform(minimum, maximum), 2)

    def _generate_integer(self, schema: Dict[str, Any]) -> int:
        """Generate an integer conforming to schema."""
        minimum = schema.get("minimum", -1000)
        maximum = schema.get("maximum", 1000)
        exclusive_min = schema.get("exclusiveMinimum", False)
        exclusive_max = schema.get("exclusiveMaximum", False)

        if exclusive_min:
            minimum += 1
        if exclusive_max:
            maximum -= 1

        return random.randint(int(minimum), int(maximum))

    def _generate_random_value(self) -> Any:
        """Generate a random value of any type."""
        types = [
            lambda: random.randint(0, 100),
            lambda: round(random.uniform(0, 100), 2),
            lambda: self._generate_random_string(5, 15),
            lambda: random.choice([True, False]),
            lambda: None,
        ]
        return random.choice(types)()

    def _generate_random_string(self, min_len: int, max_len: int) -> str:
        """Generate a random string."""
        length = random.randint(min_len, min(max_len, min_len + 20))

        # Vary string type
        choice = random.random()
        if choice < 0.3:
            # Words
            words = ['lorem', 'ipsum', 'dolor', 'sit', 'amet', 'consectetur', 'adipiscing']
            return ' '.join(random.choices(words, k=min(5, length // 5 + 1)))[:length]
        elif choice < 0.6:
            # Alphanumeric
            return ''.join(random.choices(string.ascii_letters + string.digits, k=length))
        else:
            # Letters only
            return ''.join(random.choices(string.ascii_lowercase, k=length))

    def _generate_datetime(self) -> str:
        """Generate an ISO datetime string."""
        base = datetime(2020, 1, 1)
        random_dt = base + timedelta(days=random.randint(0, 1825), hours=random.randint(0, 23))
        return random_dt.isoformat() + "Z"

    def _generate_date(self) -> str:
        """Generate an ISO date string."""
        base = datetime(2020, 1, 1)
        random_date = base + timedelta(days=random.randint(0, 1825))
        return random_date.strftime("%Y-%m-%d")

    def _generate_time(self) -> str:
        """Generate an ISO time string."""
        return f"{random.randint(0,23):02d}:{random.randint(0,59):02d}:{random.randint(0,59):02d}"

    def _generate_email(self) -> str:
        """Generate a random email."""
        names = ['alice', 'bob', 'charlie', 'diana', 'eve', 'frank']
        domains = ['example.com', 'test.org', 'demo.net', 'sample.io']
        return f"{random.choice(names)}{random.randint(1,999)}@{random.choice(domains)}"

    def _generate_uri(self) -> str:
        """Generate a random URI."""
        schemes = ['http', 'https', 'ftp']
        domains = ['example.com', 'test.org', 'demo.net']
        paths = ['api', 'v1', 'data', 'resource']
        return f"{random.choice(schemes)}://{random.choice(domains)}/{'/'.join(random.sample(paths, 2))}"

    def _generate_placeholder(self, schema: Dict[str, Any]) -> Any:
        """Generate a placeholder for unresolved refs."""
        return {"_ref": schema.get("$ref"), "_placeholder": True}


def main():
    """Generate examples for all downloaded schemas."""
    # Get examples directory
    examples_dir = Path(__file__).parent.parent / "tests" / "examples"
    manifest_file = examples_dir / "manifest.json"

    if not manifest_file.exists():
        print("Error: manifest.json not found. Run fetch_schemas.py first.")
        return

    with open(manifest_file) as f:
        manifest = json.load(f)

    generator = SchemaExampleGenerator()

    print(f"Generating 100 examples for {len(manifest)} schemas...\n")

    for idx, entry in enumerate(manifest):
        name = entry["name"]
        # Handle both relative and absolute paths
        schema_file_path = entry["schema_file"]
        if schema_file_path.startswith("/"):
            schema_file = Path(schema_file_path)
        else:
            schema_file = examples_dir.parent.parent / schema_file_path

        print(f"[{idx+1}/{len(manifest)}] {name}")

        # Load schema
        with open(schema_file) as f:
            schema = json.load(f)

        # Generate examples
        examples = generator.generate_examples(schema, count=100)

        # Save combined file
        output = {
            "schema": schema,
            "examples": examples
        }

        output_file = schema_file.parent / f"{schema_file.stem}_with_examples.json"
        with open(output_file, "w") as f:
            json.dump(output, f, indent=2)

        print(f"  ✓ Generated 100 examples -> {output_file.name}")

    print(f"\n✓ Generated {len(manifest) * 100} total examples!")


if __name__ == "__main__":
    main()
