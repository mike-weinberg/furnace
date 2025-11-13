#!/usr/bin/env python3
"""
Fetch diverse JSON schemas from SchemaStore for testing schema inference.
Goal: Download 100 schemas with maximum structural diversity.
"""

import json
import requests
import os
from pathlib import Path
from typing import List, Dict, Any
import hashlib
import time


CATALOG_URL = "https://www.schemastore.org/api/json/catalog.json"
OUTPUT_DIR = Path(__file__).parent.parent / "tests" / "examples"


def fetch_catalog() -> List[Dict[str, Any]]:
    """Fetch the SchemaStore catalog."""
    print(f"Fetching catalog from {CATALOG_URL}...")
    response = requests.get(CATALOG_URL)
    response.raise_for_status()
    catalog = response.json()
    return catalog.get("schemas", [])


def download_schema(url: str) -> Dict[str, Any]:
    """Download a single schema from URL."""
    try:
        response = requests.get(url, timeout=10)
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"  Error downloading {url}: {e}")
        return None


def estimate_complexity(schema: Dict[str, Any]) -> Dict[str, int]:
    """
    Estimate schema complexity along multiple dimensions.
    Returns: {size, depth, property_count, array_count, object_count}
    """
    schema_str = json.dumps(schema)

    def count_depth(obj, current_depth=0):
        if isinstance(obj, dict):
            if not obj:
                return current_depth
            return max(count_depth(v, current_depth + 1) for v in obj.values())
        elif isinstance(obj, list):
            if not obj:
                return current_depth
            return max(count_depth(item, current_depth + 1) for item in obj)
        return current_depth

    def count_features(obj):
        props = 0
        arrays = 0
        objects = 0

        if isinstance(obj, dict):
            objects += 1
            if "properties" in obj:
                props += len(obj.get("properties", {}))
            if "items" in obj:
                arrays += 1
            for v in obj.values():
                sub = count_features(v)
                props += sub[0]
                arrays += sub[1]
                objects += sub[2]
        elif isinstance(obj, list):
            for item in obj:
                sub = count_features(item)
                props += sub[0]
                arrays += sub[1]
                objects += sub[2]

        return (props, arrays, objects)

    props, arrays, objects = count_features(schema)

    return {
        "size": len(schema_str),
        "depth": count_depth(schema),
        "property_count": props,
        "array_count": arrays,
        "object_count": objects
    }


def categorize_schema(complexity: Dict[str, int]) -> str:
    """
    Categorize schema into one of four quadrants:
    - small+simple
    - small+complex
    - big+simple
    - big+complex
    """
    size_threshold = 10000  # bytes
    complexity_threshold = 20  # combined metric

    is_big = complexity["size"] > size_threshold
    complexity_score = (
        complexity["depth"] * 2 +
        complexity["property_count"] +
        complexity["array_count"] * 2 +
        complexity["object_count"]
    )
    is_complex = complexity_score > complexity_threshold

    if is_big and is_complex:
        return "big+complex"
    elif is_big and not is_complex:
        return "big+simple"
    elif not is_big and is_complex:
        return "small+complex"
    else:
        return "small+simple"


def select_diverse_schemas(catalog: List[Dict], target_count: int = 100) -> List[Dict]:
    """
    Select diverse schemas to maximize coverage across complexity dimensions.
    Target: 25 schemas from each quadrant.
    """
    # Download and analyze all schemas
    analyzed = []

    print(f"\nAnalyzing {len(catalog)} schemas from catalog...")
    for idx, entry in enumerate(catalog[:min(300, len(catalog))]):  # Limit initial scan
        url = entry.get("url")
        if not url:
            continue

        print(f"  [{idx+1}] {entry.get('name', 'unknown')}")
        schema = download_schema(url)
        if schema is None:
            continue

        complexity = estimate_complexity(schema)
        category = categorize_schema(complexity)

        analyzed.append({
            "name": entry.get("name", "unknown"),
            "url": url,
            "schema": schema,
            "complexity": complexity,
            "category": category
        })

        # Rate limiting
        time.sleep(0.1)

    # Select diverse set
    categories = {"small+simple": [], "small+complex": [], "big+simple": [], "big+complex": []}
    for item in analyzed:
        categories[item["category"]].append(item)

    # Print distribution
    print("\n=== Schema Distribution ===")
    for cat, items in categories.items():
        print(f"{cat}: {len(items)} schemas")

    # Sample from each category
    per_category = target_count // 4
    selected = []

    for cat, items in categories.items():
        # Take up to per_category from each, or all if fewer
        selected.extend(items[:min(per_category, len(items))])

    # If we're short, fill from largest categories
    if len(selected) < target_count:
        remaining = target_count - len(selected)
        all_remaining = [item for cat, items in categories.items()
                        for item in items[per_category:]]
        selected.extend(all_remaining[:remaining])

    print(f"\n=== Selected {len(selected)} schemas ===")
    return selected


def save_schemas(schemas: List[Dict]):
    """Save schemas to test directory structure."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    manifest = []

    for item in schemas:
        # Create directory per schema
        name = item["name"].replace("/", "_").replace(" ", "_")
        schema_dir = OUTPUT_DIR / name
        schema_dir.mkdir(exist_ok=True)

        # Save schema (will be used to generate examples later)
        schema_file = schema_dir / "schema.json"
        with open(schema_file, "w") as f:
            json.dump(item["schema"], f, indent=2)

        manifest.append({
            "name": name,
            "category": item["category"],
            "complexity": item["complexity"],
            "url": item["url"],
            "schema_file": str(schema_file)
        })

        print(f"  Saved: {name} ({item['category']})")

    # Save manifest
    manifest_file = OUTPUT_DIR / "manifest.json"
    with open(manifest_file, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"\nManifest saved to {manifest_file}")


def main():
    """Main execution."""
    catalog = fetch_catalog()
    print(f"Found {len(catalog)} schemas in catalog")

    selected = select_diverse_schemas(catalog, target_count=100)

    save_schemas(selected)
    print("\n✓ Schema collection complete!")


if __name__ == "__main__":
    main()
