#!/usr/bin/env python3
"""
JSON Schema Inference Library

Infers JSON schemas from sample documents using a TDD approach.
Decomposed into testable subfunctions.
"""

from typing import Any, Dict, List, Optional, Set
from collections import defaultdict
import json
import re


def infer_schema(examples: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    Main entry point: Infer a JSON schema from a list of example documents.

    Args:
        examples: List of JSON objects to analyze

    Returns:
        A JSON Schema Draft 7 schema that describes the structure
    """
    if not examples:
        return {"type": "object", "properties": {}}

    # Infer schema from all examples by merging
    inferred_schemas = [_infer_from_single_example(ex) for ex in examples]
    merged = merge_schemas(inferred_schemas)

    return merged


def _infer_from_single_example(example: Any) -> Dict[str, Any]:
    """
    Infer schema from a single example value.

    Args:
        example: A single JSON value

    Returns:
        A partial schema describing this value
    """
    if example is None:
        return {"type": "null"}

    value_type = infer_type(example)

    if value_type == "object":
        return _infer_object_schema(example)
    elif value_type == "array":
        return _infer_array_schema(example)
    elif value_type in ("string", "number", "integer", "boolean"):
        schema = {"type": value_type}

        # Add format detection for strings
        if value_type == "string":
            fmt = detect_format(example)
            if fmt:
                schema["format"] = fmt

        return schema
    else:
        return {}


def _infer_object_schema(obj: Dict[str, Any]) -> Dict[str, Any]:
    """Infer schema for an object."""
    schema = {"type": "object", "properties": {}}

    for key, value in obj.items():
        schema["properties"][key] = _infer_from_single_example(value)

    return schema


def _infer_array_schema(arr: List[Any]) -> Dict[str, Any]:
    """Infer schema for an array."""
    if not arr:
        return {"type": "array"}

    # Infer items schema from all elements
    item_schemas = [_infer_from_single_example(item) for item in arr]
    merged_items = merge_schemas(item_schemas)

    return {"type": "array", "items": merged_items}


def merge_schemas(schemas: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    Merge multiple schemas into a single schema that describes all of them.

    This is the key algorithm for handling diverse examples.
    """
    if not schemas:
        return {}

    if len(schemas) == 1:
        return schemas[0]

    # Fast path: collect types without creating intermediate sets
    type_counter = {}
    has_null = False

    for schema in schemas:
        if "type" in schema:
            t = schema["type"]
            if isinstance(t, list):
                for tt in t:
                    if tt == "null":
                        has_null = True
                    else:
                        type_counter[tt] = type_counter.get(tt, 0) + 1
            else:
                if t == "null":
                    has_null = True
                else:
                    type_counter[t] = type_counter.get(t, 0) + 1

    merged = {}
    num_types = len(type_counter)

    # Handle type merging
    if num_types == 0:
        # All null or empty
        if has_null:
            merged["type"] = "null"
        return merged
    elif num_types == 1:
        merged_type = list(type_counter.keys())[0]
        merged["type"] = merged_type

        # Merge type-specific properties
        if merged_type == "object":
            merged = _merge_object_schemas(schemas, merged)
        elif merged_type == "array":
            merged = _merge_array_schemas(schemas, merged)
        elif merged_type in ("string", "number", "integer"):
            merged = _merge_scalar_schemas(schemas, merged, merged_type)
    else:
        # Multiple types - use anyOf
        merged["anyOf"] = schemas

    # Add nullable if needed
    if has_null and num_types > 0:
        if "type" in merged:
            merged["type"] = [merged["type"], "null"]
        elif "anyOf" in merged:
            merged["anyOf"].append({"type": "null"})

    return merged


def _merge_object_schemas(schemas: List[Dict[str, Any]], base: Dict) -> Dict[str, Any]:
    """Merge object schemas - union all properties, track required."""
    # Use defaultdict to avoid repeated lookups
    properties = defaultdict(list)
    common_keys = None

    for schema in schemas:
        if "properties" in schema:
            schema_props = schema["properties"]
            schema_keys = set(schema_props.keys())

            # Track keys present in all schemas
            if common_keys is None:
                common_keys = schema_keys.copy()
            else:
                common_keys &= schema_keys

            # Merge property schemas (no repeated lookups)
            for key, prop_schema in schema_props.items():
                properties[key].append(prop_schema)

    # Merge all property schemas at once
    merged_props = {}
    for k, v in properties.items():
        merged_props[k] = merge_schemas(v)

    base["properties"] = merged_props

    # Mark keys that appear in all examples as required
    if common_keys:
        base["required"] = sorted(list(common_keys))

    return base


def _merge_array_schemas(schemas: List[Dict[str, Any]], base: Dict) -> Dict[str, Any]:
    """Merge array schemas."""
    item_schemas = []

    for schema in schemas:
        if "items" in schema:
            item_schemas.append(schema["items"])

    if item_schemas:
        base["items"] = merge_schemas(item_schemas)

    return base


def _merge_scalar_schemas(
    schemas: List[Dict[str, Any]], base: Dict, schema_type: str
) -> Dict[str, Any]:
    """Merge scalar (string, number, integer) schemas."""
    # For now, just keep the base type
    # Could add format, range constraints, etc.

    # Try to detect common format for strings
    if schema_type == "string":
        formats = set()
        for schema in schemas:
            if "format" in schema:
                formats.add(schema["format"])

        # Only set format if all non-null examples agree
        if len(formats) == 1:
            base["format"] = list(formats)[0]

    return base


def infer_type(value: Any) -> str:
    """
    Infer the JSON type of a single value.

    Returns one of: null, boolean, object, array, number, integer, string
    """
    if value is None:
        return "null"
    elif isinstance(value, bool):
        return "boolean"
    elif isinstance(value, dict):
        return "object"
    elif isinstance(value, list):
        return "array"
    elif isinstance(value, int) and not isinstance(value, bool):
        return "integer"
    elif isinstance(value, float):
        return "number"
    elif isinstance(value, str):
        return "string"
    else:
        return "unknown"


def detect_format(value: str) -> Optional[str]:
    """
    Detect if a string matches a known format.

    Returns: format name (date, time, date-time, email, uri, uuid, ipv4, ipv6)
             or None if no match
    """
    if not isinstance(value, str) or not value:
        return None

    # URI - fast path
    if value.startswith(("http://", "https://", "ftp://", "file://")):
        return "uri"

    length = len(value)

    # Email - common and quick to check
    if "@" in value and length > 5 and _is_email(value):
        return "email"

    # UUID - fixed length
    if length == 36 and "-" in value and _is_uuid(value):
        return "uuid"

    # Date/Time - common lengths help
    if length == 10 and _is_iso_date(value):
        return "date"
    elif length >= 19 and _is_iso_datetime(value):
        return "date-time"
    elif length >= 8 and ":" in value and _is_iso_time(value):
        return "time"

    # IP addresses
    if "." in value and _is_ipv4(value):
        return "ipv4"
    elif ":" in value and _is_ipv6(value):
        return "ipv6"

    return None


def _is_iso_datetime(s: str) -> bool:
    """Check if string is ISO 8601 datetime."""
    pattern = r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(.\d+)?(Z|[+-]\d{2}:\d{2})?$'
    return bool(re.match(pattern, s))


def _is_iso_date(s: str) -> bool:
    """Check if string is ISO 8601 date."""
    pattern = r'^\d{4}-\d{2}-\d{2}$'
    return bool(re.match(pattern, s))


def _is_iso_time(s: str) -> bool:
    """Check if string is ISO 8601 time."""
    pattern = r'^\d{2}:\d{2}:\d{2}(.\d+)?$'
    return bool(re.match(pattern, s))


def _is_email(s: str) -> bool:
    """Check if string is an email."""
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, s))


def _is_uri(s: str) -> bool:
    """Check if string is a URI."""
    return s.startswith(("http://", "https://", "ftp://", "file://"))


def _is_uuid(s: str) -> bool:
    """Check if string is a UUID."""
    pattern = r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    return bool(re.match(pattern, s, re.IGNORECASE))


def _is_ipv4(s: str) -> bool:
    """Check if string is IPv4."""
    pattern = r'^(\d{1,3}\.){3}\d{1,3}$'
    if not re.match(pattern, s):
        return False

    parts = s.split(".")
    return all(0 <= int(p) <= 255 for p in parts)


def _is_ipv6(s: str) -> bool:
    """Check if string is IPv6."""
    pattern = r'^(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4})$'
    return bool(re.match(pattern, s))
