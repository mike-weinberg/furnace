//! Optimized Schema Builder with Streaming Accumulator Pattern
//!
//! This module implements a streaming schema inference approach inspired by genson-rs.
//! Instead of creating intermediate schemas and merging them, it accumulates statistics
//! and builds the final schema only once at the end.

use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use regex::Regex;

// Re-use the same regex patterns from schema_inference
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

/// Type identifier for JSON values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum JsonType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

impl JsonType {
    fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => JsonType::Null,
            Value::Bool(_) => JsonType::Boolean,
            Value::Number(n) => {
                if n.is_i64() {
                    JsonType::Integer
                } else {
                    JsonType::Number
                }
            }
            Value::String(_) => JsonType::String,
            Value::Array(_) => JsonType::Array,
            Value::Object(_) => JsonType::Object,
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            JsonType::Null => "null",
            JsonType::Boolean => "boolean",
            JsonType::Integer => "integer",
            JsonType::Number => "number",
            JsonType::String => "string",
            JsonType::Array => "array",
            JsonType::Object => "object",
        }
    }
}

/// Builder for accumulating statistics about strings
#[derive(Debug)]
struct StringStats {
    format_counts: HashMap<String, usize>,
    total_count: usize,
}

impl StringStats {
    fn new() -> Self {
        StringStats {
            format_counts: HashMap::new(),
            total_count: 0,
        }
    }

    fn add_string(&mut self, s: &str) {
        self.total_count += 1;
        if let Some(format) = detect_format(s) {
            *self.format_counts.entry(format).or_insert(0) += 1;
        }
    }

    fn get_format(&self) -> Option<String> {
        // Only return format if all strings had the same format
        if self.format_counts.len() == 1 {
            if let Some((format, count)) = self.format_counts.iter().next() {
                if *count == self.total_count {
                    return Some(format.clone());
                }
            }
        }
        None
    }
}

/// Builder for accumulating statistics about arrays
#[derive(Debug)]
struct ArrayBuilder {
    items_builder: Box<SchemaBuilder>,
}

impl ArrayBuilder {
    fn new() -> Self {
        ArrayBuilder {
            items_builder: Box::new(SchemaBuilder::new()),
        }
    }

    fn add_array(&mut self, arr: &[Value]) {
        for item in arr {
            self.items_builder.add_value(item);
        }
    }

    fn build(self) -> Map<String, Value> {
        let mut schema = Map::new();
        schema.insert("type".to_string(), Value::String("array".to_string()));

        // Only add items if we saw any
        if self.items_builder.sample_count > 0 {
            schema.insert("items".to_string(), self.items_builder.build());
        }

        schema
    }
}

/// Builder for accumulating statistics about objects
#[derive(Debug)]
struct ObjectBuilder {
    // Map from property name to its builder
    properties: HashMap<String, SchemaBuilder>,
    // Track which properties appeared in each sample
    property_appearances: Vec<HashSet<String>>,
}

impl ObjectBuilder {
    fn new() -> Self {
        ObjectBuilder {
            properties: HashMap::new(),
            property_appearances: Vec::new(),
        }
    }

    fn add_object(&mut self, obj: &Map<String, Value>) {
        let mut current_keys = HashSet::new();

        for (key, value) in obj.iter() {
            current_keys.insert(key.clone());
            self.properties
                .entry(key.clone())
                .or_insert_with(SchemaBuilder::new)
                .add_value(value);
        }

        self.property_appearances.push(current_keys);
    }

    fn build(self) -> Map<String, Value> {
        let mut schema = Map::new();
        schema.insert("type".to_string(), Value::String("object".to_string()));

        // Build property schemas
        let mut properties = Map::new();
        for (key, builder) in self.properties {
            properties.insert(key, builder.build());
        }
        schema.insert("properties".to_string(), Value::Object(properties));

        // Determine required fields (those that appear in ALL samples)
        if !self.property_appearances.is_empty() {
            let mut required: Option<HashSet<String>> = None;

            for appearance in &self.property_appearances {
                required = match required {
                    None => Some(appearance.clone()),
                    Some(req) => Some(req.intersection(appearance).cloned().collect()),
                };
            }

            if let Some(req_set) = required {
                if !req_set.is_empty() {
                    let mut req_vec: Vec<String> = req_set.into_iter().collect();
                    req_vec.sort();
                    schema.insert(
                        "required".to_string(),
                        Value::Array(req_vec.into_iter().map(Value::String).collect()),
                    );
                }
            }
        }

        schema
    }
}

/// Main schema builder that accumulates statistics
#[derive(Debug)]
pub struct SchemaBuilder {
    // Count of each type seen
    type_counts: HashMap<JsonType, usize>,
    // Total number of samples processed
    sample_count: usize,
    // Type-specific builders
    string_stats: Option<StringStats>,
    array_builder: Option<ArrayBuilder>,
    object_builder: Option<ObjectBuilder>,
}

impl SchemaBuilder {
    /// Create a new empty schema builder
    pub fn new() -> Self {
        SchemaBuilder {
            type_counts: HashMap::new(),
            sample_count: 0,
            string_stats: None,
            array_builder: None,
            object_builder: None,
        }
    }

    /// Add a value to the builder, accumulating statistics
    pub fn add_value(&mut self, value: &Value) {
        self.sample_count += 1;
        let json_type = JsonType::from_value(value);
        *self.type_counts.entry(json_type).or_insert(0) += 1;

        // Accumulate type-specific statistics
        match value {
            Value::String(s) => {
                let stats = self.string_stats.get_or_insert_with(StringStats::new);
                stats.add_string(s);
            }
            Value::Array(arr) => {
                let builder = self.array_builder.get_or_insert_with(ArrayBuilder::new);
                builder.add_array(arr);
            }
            Value::Object(obj) => {
                let builder = self.object_builder.get_or_insert_with(ObjectBuilder::new);
                builder.add_object(obj);
            }
            _ => {}
        }
    }

    /// Build the final JSON schema from accumulated statistics
    pub fn build(self) -> Value {
        if self.sample_count == 0 {
            return json!({});
        }

        // Handle single type case (most common)
        if self.type_counts.len() == 1 {
            let json_type = *self.type_counts.keys().next().unwrap();
            return self.build_single_type_schema(json_type);
        }

        // Handle null + one other type
        if self.type_counts.len() == 2 && self.type_counts.contains_key(&JsonType::Null) {
            // Find the non-null type
            let non_null_type = self.type_counts
                .keys()
                .find(|&&t| t != JsonType::Null)
                .copied();

            if let Some(json_type) = non_null_type {
                let mut schema = self.build_single_type_schema(json_type);

                // Make it nullable
                if let Value::Object(ref mut obj) = schema {
                    if let Some(Value::String(type_str)) = obj.get("type") {
                        obj.insert(
                            "type".to_string(),
                            Value::Array(vec![
                                Value::String(type_str.clone()),
                                Value::String("null".to_string()),
                            ]),
                        );
                    }
                }

                return schema;
            }
        }

        // Multiple types - need to use anyOf
        // This is a simplified version; full implementation would require
        // separate builders for each type
        let mut types: Vec<String> = self.type_counts
            .keys()
            .map(|t| t.to_str().to_string())
            .collect();
        types.sort();

        json!({
            "type": Value::Array(types.into_iter().map(Value::String).collect::<Vec<_>>())
        })
    }

    /// Build schema for a single type
    fn build_single_type_schema(self, json_type: JsonType) -> Value {
        let mut schema = Map::new();
        schema.insert("type".to_string(), Value::String(json_type.to_str().to_string()));

        match json_type {
            JsonType::String => {
                if let Some(stats) = self.string_stats {
                    if let Some(format) = stats.get_format() {
                        schema.insert("format".to_string(), Value::String(format));
                    }
                }
            }
            JsonType::Array => {
                if let Some(builder) = self.array_builder {
                    return Value::Object(builder.build());
                }
            }
            JsonType::Object => {
                if let Some(builder) = self.object_builder {
                    return Value::Object(builder.build());
                }
            }
            _ => {}
        }

        Value::Object(schema)
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
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

/// Infer schema from multiple examples using the streaming builder
pub fn infer_schema_streaming(examples: &[Value]) -> Value {
    let mut builder = SchemaBuilder::new();

    for example in examples {
        builder.add_value(example);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let builder = SchemaBuilder::new();
        let schema = builder.build();
        assert_eq!(schema, json!({}));
    }

    #[test]
    fn test_simple_string() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!("hello"));
        builder.add_value(&json!("world"));

        let schema = builder.build();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("string"));
    }

    #[test]
    fn test_simple_number() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!(42));
        builder.add_value(&json!(100));

        let schema = builder.build();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("integer"));
    }

    #[test]
    fn test_simple_object() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!({"name": "Alice", "age": 30}));
        builder.add_value(&json!({"name": "Bob", "age": 25}));

        let schema = builder.build();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));

        // Check properties exist
        let properties = schema.get("properties").and_then(|v| v.as_object()).unwrap();
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("age"));

        // Check required fields
        let required = schema.get("required").and_then(|v| v.as_array()).unwrap();
        assert_eq!(required.len(), 2);
    }

    #[test]
    fn test_optional_fields() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!({"name": "Alice", "age": 30}));
        builder.add_value(&json!({"name": "Bob"}));

        let schema = builder.build();

        // Only "name" should be required
        let required = schema.get("required").and_then(|v| v.as_array()).unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "name");
    }

    #[test]
    fn test_array_schema() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!([1, 2, 3]));
        builder.add_value(&json!([4, 5]));

        let schema = builder.build();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("array"));

        // Check items schema
        let items = schema.get("items").unwrap();
        assert_eq!(items.get("type").and_then(|v| v.as_str()), Some("integer"));
    }

    #[test]
    fn test_nullable_field() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!("hello"));
        builder.add_value(&json!(null));

        let schema = builder.build();

        // Should have both string and null types
        let type_val = schema.get("type").unwrap();
        if let Value::Array(types) = type_val {
            assert_eq!(types.len(), 2);
            assert!(types.contains(&Value::String("string".to_string())));
            assert!(types.contains(&Value::String("null".to_string())));
        } else {
            panic!("Expected array type, got: {:?}", type_val);
        }
    }

    #[test]
    fn test_format_detection_email() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!("test@example.com"));
        builder.add_value(&json!("another@test.org"));

        let schema = builder.build();
        assert_eq!(schema.get("format").and_then(|v| v.as_str()), Some("email"));
    }

    #[test]
    fn test_format_detection_uuid() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!("550e8400-e29b-41d4-a716-446655440000"));

        let schema = builder.build();
        assert_eq!(schema.get("format").and_then(|v| v.as_str()), Some("uuid"));
    }

    #[test]
    fn test_format_detection_date() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!("2021-01-01"));
        builder.add_value(&json!("2021-12-31"));

        let schema = builder.build();
        assert_eq!(schema.get("format").and_then(|v| v.as_str()), Some("date"));
    }

    #[test]
    fn test_nested_objects() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!({
            "user": {
                "name": "Alice",
                "email": "alice@example.com"
            }
        }));
        builder.add_value(&json!({
            "user": {
                "name": "Bob",
                "email": "bob@example.com"
            }
        }));

        let schema = builder.build();
        let properties = schema.get("properties").and_then(|v| v.as_object()).unwrap();
        let user_schema = properties.get("user").and_then(|v| v.as_object()).unwrap();

        assert_eq!(user_schema.get("type").and_then(|v| v.as_str()), Some("object"));

        let user_props = user_schema.get("properties").and_then(|v| v.as_object()).unwrap();
        assert!(user_props.contains_key("name"));
        assert!(user_props.contains_key("email"));
    }

    #[test]
    fn test_array_of_objects() {
        let mut builder = SchemaBuilder::new();
        builder.add_value(&json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]));

        let schema = builder.build();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("array"));

        let items = schema.get("items").and_then(|v| v.as_object()).unwrap();
        assert_eq!(items.get("type").and_then(|v| v.as_str()), Some("object"));

        let item_props = items.get("properties").and_then(|v| v.as_object()).unwrap();
        assert!(item_props.contains_key("id"));
        assert!(item_props.contains_key("name"));
    }

    #[test]
    fn test_streaming_function() {
        let examples = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];

        let schema = infer_schema_streaming(&examples);
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
    }
}
