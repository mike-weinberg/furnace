//! JSON Schema Inference Library
//!
//! Infers JSON schemas from sample documents.
//! Ported from the Python implementation for performance.

use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use regex::Regex;

// Pre-compiled regex patterns for performance
static ISO_DATETIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(.\d+)?(Z|[+-]\d{2}:\d{2})?$").unwrap()
});

static ISO_DATE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap()
});

static ISO_TIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{2}:\d{2}:\d{2}(.\d+)?$").unwrap()
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

static IPV4_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{1,3}\.){3}\d{1,3}$").unwrap()
});

static IPV6_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4})$").unwrap()
});

/// Main entry point: Infer a JSON schema from a list of example documents
pub fn infer_schema(examples: &[Value]) -> Value {
    if examples.is_empty() {
        return json!({
            "type": "object",
            "properties": {}
        });
    }

    // Infer schema from all examples by merging
    let inferred_schemas: Vec<Value> = examples
        .iter()
        .map(|ex| infer_from_single_example(ex))
        .collect();

    merge_schemas(&inferred_schemas)
}

/// Infer schema from a single example value
fn infer_from_single_example(example: &Value) -> Value {
    match example {
        Value::Null => json!({ "type": "null" }),
        Value::Bool(_) => json!({ "type": "boolean" }),
        Value::Object(obj) => infer_object_schema(obj),
        Value::Array(arr) => infer_array_schema(arr),
        Value::String(s) => {
            let mut schema = json!({ "type": "string" });
            if let Some(fmt) = detect_format(s) {
                if let Value::Object(ref mut obj) = schema {
                    obj.insert("format".to_string(), Value::String(fmt));
                }
            }
            schema
        }
        Value::Number(n) => {
            if n.is_i64() {
                json!({ "type": "integer" })
            } else {
                json!({ "type": "number" })
            }
        }
    }
}

/// Infer schema for an object
fn infer_object_schema(obj: &Map<String, Value>) -> Value {
    let mut properties = Map::new();

    for (key, value) in obj.iter() {
        properties.insert(key.clone(), infer_from_single_example(value));
    }

    json!({
        "type": "object",
        "properties": properties
    })
}

/// Infer schema for an array
fn infer_array_schema(arr: &[Value]) -> Value {
    if arr.is_empty() {
        return json!({ "type": "array" });
    }

    let item_schemas: Vec<Value> = arr
        .iter()
        .map(|item| infer_from_single_example(item))
        .collect();

    let merged_items = merge_schemas(&item_schemas);

    json!({
        "type": "array",
        "items": merged_items
    })
}

/// Merge multiple schemas into a single schema that describes all of them
fn merge_schemas(schemas: &[Value]) -> Value {
    if schemas.is_empty() {
        return json!({});
    }

    if schemas.len() == 1 {
        return schemas[0].clone();
    }

    // Fast path: collect types without creating intermediate sets
    let mut type_counter: HashMap<String, usize> = HashMap::new();
    let mut has_null = false;

    for schema in schemas {
        if let Some(type_value) = schema.get("type") {
            match type_value {
                Value::Array(types) => {
                    for t in types {
                        if let Value::String(type_str) = t {
                            if type_str == "null" {
                                has_null = true;
                            } else {
                                *type_counter.entry(type_str.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                Value::String(type_str) => {
                    if type_str == "null" {
                        has_null = true;
                    } else {
                        *type_counter.entry(type_str.clone()).or_insert(0) += 1;
                    }
                }
                _ => {}
            }
        }
    }

    let mut merged = Map::new();
    let num_types = type_counter.len();

    // Handle type merging
    if num_types == 0 {
        if has_null {
            merged.insert("type".to_string(), Value::String("null".to_string()));
        }
        return Value::Object(merged);
    }

    if num_types == 1 {
        let merged_type = type_counter.keys().next().unwrap().clone();
        merged.insert("type".to_string(), Value::String(merged_type.clone()));

        // Merge type-specific properties
        match merged_type.as_str() {
            "object" => {
                return merge_object_schemas(schemas, merged);
            }
            "array" => {
                return merge_array_schemas(schemas, merged);
            }
            "string" | "number" | "integer" => {
                return merge_scalar_schemas(schemas, merged, &merged_type);
            }
            _ => {}
        }
    } else {
        // Multiple types - use anyOf
        merged.insert(
            "anyOf".to_string(),
            Value::Array(schemas.to_vec()),
        );
    }

    // Add nullable if needed
    if has_null && num_types > 0 {
        if let Some(Value::String(current_type)) = merged.get("type") {
            merged.insert(
                "type".to_string(),
                Value::Array(vec![
                    Value::String(current_type.clone()),
                    Value::String("null".to_string()),
                ]),
            );
        } else if let Some(Value::Array(any_of)) = merged.get("anyOf") {
            let mut new_any_of = any_of.clone();
            new_any_of.push(json!({ "type": "null" }));
            merged.insert("anyOf".to_string(), Value::Array(new_any_of));
        }
    }

    Value::Object(merged)
}

/// Merge object schemas - union all properties, track required
fn merge_object_schemas(schemas: &[Value], mut base: Map<String, Value>) -> Value {
    let mut properties: HashMap<String, Vec<Value>> = HashMap::new();
    let mut common_keys: Option<HashSet<String>> = None;

    for schema in schemas {
        if let Some(Value::Object(schema_props)) = schema.get("properties") {
            let schema_keys: HashSet<String> = schema_props.keys().cloned().collect();

            // Track keys present in all schemas
            common_keys = match common_keys {
                None => Some(schema_keys.clone()),
                Some(keys) => Some(keys.intersection(&schema_keys).cloned().collect()),
            };

            // Merge property schemas
            for (key, prop_schema) in schema_props.iter() {
                properties
                    .entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(prop_schema.clone());
            }
        }
    }

    // Merge all property schemas at once
    let mut merged_props = Map::new();
    for (k, v) in properties.iter() {
        merged_props.insert(k.clone(), merge_schemas(v));
    }

    base.insert("properties".to_string(), Value::Object(merged_props));

    // Mark keys that appear in all examples as required
    if let Some(keys) = common_keys {
        if !keys.is_empty() {
            let mut sorted_keys: Vec<String> = keys.into_iter().collect();
            sorted_keys.sort();
            base.insert("required".to_string(), Value::Array(
                sorted_keys.into_iter().map(Value::String).collect()
            ));
        }
    }

    Value::Object(base)
}

/// Merge array schemas
fn merge_array_schemas(schemas: &[Value], mut base: Map<String, Value>) -> Value {
    let mut item_schemas = Vec::new();

    for schema in schemas {
        if let Some(items) = schema.get("items") {
            item_schemas.push(items.clone());
        }
    }

    if !item_schemas.is_empty() {
        base.insert("items".to_string(), merge_schemas(&item_schemas));
    }

    Value::Object(base)
}

/// Merge scalar (string, number, integer) schemas
fn merge_scalar_schemas(
    schemas: &[Value],
    mut base: Map<String, Value>,
    schema_type: &str,
) -> Value {
    // Try to detect common format for strings
    if schema_type == "string" {
        let mut formats = HashSet::new();
        for schema in schemas {
            if let Some(Value::String(fmt)) = schema.get("format") {
                formats.insert(fmt.clone());
            }
        }

        // Only set format if all non-null examples agree
        if formats.len() == 1 {
            if let Some(fmt) = formats.into_iter().next() {
                base.insert("format".to_string(), Value::String(fmt));
            }
        }
    }

    Value::Object(base)
}

/// Detect if a string matches a known format
fn detect_format(value: &str) -> Option<String> {
    let len = value.len();

    // Fast path checks first - these are O(1) or O(len)
    if len == 0 {
        return None;
    }

    // URI - fast byte check
    if len > 6 && (value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("ftp://")
        || value.starts_with("file://"))
    {
        return Some("uri".to_string());
    }

    // ISO Date - fixed length with fast pattern
    if len == 10 && value.as_bytes()[4] == b'-' && value.as_bytes()[7] == b'-' {
        if is_iso_date(value) {
            return Some("date".to_string());
        }
    }

    // Email - common pattern check before regex
    if len > 5 && len < 255 && value.contains('@') {
        if is_email(value) {
            return Some("email".to_string());
        }
    }

    // UUID - fixed length
    if len == 36 && value.as_bytes()[8] == b'-' {
        if is_uuid(value) {
            return Some("uuid".to_string());
        }
    }

    // DateTime - check length and T separator before regex
    if len >= 19 && value.as_bytes()[10] == b'T' {
        if is_iso_datetime(value) {
            return Some("date-time".to_string());
        }
    }

    // Time - colon separator
    if len >= 8 && value.contains(':') {
        if is_iso_time(value) {
            return Some("time".to_string());
        }
    }

    // IPv4 - simple dot count check
    if len < 16 && value.contains('.') {
        if is_ipv4(value) {
            return Some("ipv4".to_string());
        }
    }

    // IPv6 - must have colons
    if value.contains(':') {
        if is_ipv6(value) {
            return Some("ipv6".to_string());
        }
    }

    None
}

fn is_iso_datetime(s: &str) -> bool {
    ISO_DATETIME_REGEX.is_match(s)
}

fn is_iso_date(s: &str) -> bool {
    ISO_DATE_REGEX.is_match(s)
}

fn is_iso_time(s: &str) -> bool {
    ISO_TIME_REGEX.is_match(s)
}

fn is_email(s: &str) -> bool {
    EMAIL_REGEX.is_match(s)
}

fn is_uuid(s: &str) -> bool {
    UUID_REGEX.is_match(&s.to_lowercase())
}

fn is_ipv4(s: &str) -> bool {
    if !IPV4_REGEX.is_match(s) {
        return false;
    }

    s.split('.')
        .all(|part| part.parse::<u8>().is_ok())
}

fn is_ipv6(s: &str) -> bool {
    IPV6_REGEX.is_match(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let example = json!({
            "name": "Alice",
            "age": 30
        });

        let schema = infer_schema(&[example]);
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_array_schema() {
        let example = json!([1, 2, 3]);
        let schema = infer_schema(&[example]);
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("array"));
    }

    #[test]
    fn test_merge_types() {
        let examples = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob"}),
        ];

        let schema = infer_schema(&examples);
        if let Some(required) = schema.get("required") {
            let req_vec = required.as_array().unwrap();
            assert_eq!(req_vec.len(), 1); // Only "name" is required
        }
    }

    #[test]
    fn test_detect_format_email() {
        assert_eq!(detect_format("test@example.com"), Some("email".to_string()));
    }

    #[test]
    fn test_detect_format_uuid() {
        assert_eq!(
            detect_format("550e8400-e29b-41d4-a716-446655440000"),
            Some("uuid".to_string())
        );
    }

    #[test]
    fn test_detect_format_date() {
        assert_eq!(detect_format("2021-01-01"), Some("date".to_string()));
    }
}
