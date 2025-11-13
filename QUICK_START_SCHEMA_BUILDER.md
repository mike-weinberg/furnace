# SchemaBuilder Quick Start

## Installation

The `SchemaBuilder` is included in the `json-melt` crate:

```rust
use json_melt::{SchemaBuilder, infer_schema_streaming};
use serde_json::json;
```

## Quick Examples

### Example 1: Basic Usage

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({"name": "Alice", "age": 30}));
    builder.add_value(&json!({"name": "Bob", "age": 25}));

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

Output:
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age": {"type": "integer"}
  },
  "required": ["name", "age"]
}
```

### Example 2: Convenience Function

```rust
use json_melt::infer_schema_streaming;
use serde_json::json;

fn main() {
    let examples = vec![
        json!({"id": 1, "email": "alice@example.com"}),
        json!({"id": 2, "email": "bob@example.com"}),
    ];

    let schema = infer_schema_streaming(&examples);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

Output:
```json
{
  "type": "object",
  "properties": {
    "id": {"type": "integer"},
    "email": {
      "type": "string",
      "format": "email"
    }
  },
  "required": ["id", "email"]
}
```

### Example 3: Streaming from File

```rust
use json_melt::SchemaBuilder;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let file = File::open("data.jsonl")?;
    let reader = BufReader::new(file);

    let mut builder = SchemaBuilder::new();

    for line in reader.lines() {
        let value: Value = serde_json::from_str(&line?)?;
        builder.add_value(&value);
    }

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema)?);

    Ok(())
}
```

### Example 4: Optional Fields

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({"name": "Alice", "age": 30, "email": "alice@example.com"}));
    builder.add_value(&json!({"name": "Bob", "age": 25})); // email missing

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

Output:
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age": {"type": "integer"},
    "email": {
      "type": "string",
      "format": "email"
    }
  },
  "required": ["name", "age"]  // Only fields present in ALL samples
}
```

### Example 5: Arrays

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ]));

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

Output:
```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "id": {"type": "integer"},
      "name": {"type": "string"}
    },
    "required": ["id", "name"]
  }
}
```

### Example 6: Nested Objects

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({
        "user": {
            "name": "Alice",
            "contact": {
                "email": "alice@example.com"
            }
        }
    }));

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

### Example 7: Format Detection

The builder automatically detects common string formats:

```rust
use json_melt::infer_schema_streaming;
use serde_json::json;

fn main() {
    // Email
    let schema = infer_schema_streaming(&vec![
        json!("user@example.com"),
        json!("admin@test.org")
    ]);
    // -> {"type": "string", "format": "email"}

    // UUID
    let schema = infer_schema_streaming(&vec![
        json!("550e8400-e29b-41d4-a716-446655440000")
    ]);
    // -> {"type": "string", "format": "uuid"}

    // Date
    let schema = infer_schema_streaming(&vec![
        json!("2021-01-01"),
        json!("2021-12-31")
    ]);
    // -> {"type": "string", "format": "date"}

    // DateTime
    let schema = infer_schema_streaming(&vec![
        json!("2021-01-01T00:00:00Z")
    ]);
    // -> {"type": "string", "format": "date-time"}

    // URI
    let schema = infer_schema_streaming(&vec![
        json!("https://example.com"),
        json!("http://test.org")
    ]);
    // -> {"type": "string", "format": "uri"}
}
```

Supported formats:
- `email`
- `uuid`
- `date` (ISO 8601: YYYY-MM-DD)
- `date-time` (ISO 8601 with time)
- `time` (HH:MM:SS)
- `uri` (http://, https://, ftp://, file://)
- `ipv4`
- `ipv6`

### Example 8: Nullable Fields

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!("hello"));
    builder.add_value(&json!(null));

    let schema = builder.build();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

Output:
```json
{
  "type": ["string", "null"]
}
```

## API Reference

### `SchemaBuilder`

```rust
pub struct SchemaBuilder { /* ... */ }

impl SchemaBuilder {
    // Create a new empty builder
    pub fn new() -> Self;

    // Add a value to accumulate statistics
    pub fn add_value(&mut self, value: &Value);

    // Build the final schema from accumulated statistics
    pub fn build(self) -> Value;
}
```

### `infer_schema_streaming`

```rust
// Convenience function for inferring schema from a slice of examples
pub fn infer_schema_streaming(examples: &[Value]) -> Value;
```

## Performance Tips

1. **Use streaming**: Process values one at a time instead of collecting all first
2. **Release mode**: Always benchmark in release mode (`cargo run --release`)
3. **Batch size**: Process in batches if memory is limited
4. **Reuse builders**: Create one builder per schema, don't recreate

## Common Patterns

### Pattern 1: Stream Processing

```rust
fn process_stream<R: BufRead>(reader: R) -> anyhow::Result<Value> {
    let mut builder = SchemaBuilder::new();

    for line in reader.lines() {
        let value: Value = serde_json::from_str(&line?)?;
        builder.add_value(&value);
    }

    Ok(builder.build())
}
```

### Pattern 2: API Response

```rust
async fn infer_api_schema(api_url: &str, limit: usize) -> anyhow::Result<Value> {
    let mut builder = SchemaBuilder::new();

    for page in 0..limit {
        let response = fetch_page(api_url, page).await?;
        for item in response.items {
            builder.add_value(&item);
        }
    }

    Ok(builder.build())
}
```

### Pattern 3: Database Export

```rust
fn infer_table_schema(conn: &Connection, table: &str) -> anyhow::Result<Value> {
    let mut builder = SchemaBuilder::new();

    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table))?;
    let rows = stmt.query_map([], |row| {
        // Convert row to JSON value
        Ok(row_to_json(row))
    })?;

    for row in rows {
        builder.add_value(&row?);
    }

    Ok(builder.build())
}
```

## See Also

- [SCHEMA_BUILDER.md](SCHEMA_BUILDER.md) - Full documentation
- [examples/schema_builder_usage.rs](examples/schema_builder_usage.rs) - Comprehensive examples
- [examples/schema_performance.rs](examples/schema_performance.rs) - Performance benchmarks
