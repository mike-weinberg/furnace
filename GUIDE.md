# Furnace User Guide

Complete guide to using Furnace for JSON melting and schema inference.

## Table of Contents

1. [Quick Start](#quick-start)
2. [JSON Melting](#json-melting)
   - [Basic Usage](#basic-melting-usage)
   - [High-Performance (PlannedMelter)](#high-performance-melting)
   - [Streaming Large Files](#streaming-large-files)
3. [Schema Inference](#schema-inference)
4. [Understanding Output](#understanding-output)
5. [Configuration](#configuration)
6. [Real-World Examples](#real-world-examples)
7. [Performance](#performance)

---

## Quick Start

### Installation

```toml
[dependencies]
furnace = "0.1.0"
```

### Run Examples

```bash
# JSON melting
cargo run --example quickstart
cargo run --example api_pagination

# Schema inference
cargo run --example schema_builder_usage

# Performance benchmarks
cargo run --release --bin melt_performance_benchmark
cargo run --release --bin schema_inference_benchmark
```

---

## JSON Melting

Convert nested JSON into flat relational tables with foreign keys.

### Basic Melting Usage

```rust
use furnace::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let data = json!({
        "id": 1,
        "name": "Alice",
        "posts": [
            {"id": 10, "title": "First Post"},
            {"id": 11, "title": "Second Post"}
        ]
    });

    let melter = JsonMelter::new(MeltConfig::default());
    let entities = melter.melt(data)?;

    // Write to .jsonl files
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    Ok(())
}
```

**Output:**
- `root.jsonl`: `{"id": 1, "name": "Alice"}`
- `root_posts.jsonl`: `{"id": 10, "title": "First Post", "posts_id": "1"}`

### High-Performance Melting

For processing many similar records (e.g., paginated API responses), use `PlannedMelter`:

```rust
use furnace::{PlannedMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // Step 1: Sample 10-100 records to build extraction plan
    let samples = vec![
        json!({"id": 1, "name": "Alice", "posts": [{"id": 10, "title": "Post"}]}),
        json!({"id": 2, "name": "Bob", "posts": [{"id": 20, "title": "Post"}]}),
        // ... more samples
    ];

    // Step 2: Create planned melter (analyzes structure once)
    let melter = PlannedMelter::from_examples(&samples, MeltConfig::default())?;

    // Step 3: Process thousands of records with pre-computed plan
    let mut writer = EntityWriter::new_file_writer(".")?;

    for page in api_responses {
        let entities = melter.melt(page)?;  // 40% faster!
        writer.write_entities(entities)?;
    }

    writer.flush()?;
    Ok(())
}
```

**Performance:** PlannedMelter is **40-50% faster** by eliminating runtime decisions.

**When to use:**
- Paginated API responses
- Log streams with consistent structure
- Database exports
- Any scenario with >100 similar records

### Streaming Large Files

Process JSON Lines files without loading everything into memory:

```rust
use furnace::{melt_json, EntityWriter, MeltConfig};
use std::fs::File;
use std::io::BufReader;

fn main() -> anyhow::Result<()> {
    let file = File::open("large_data.jsonl")?;
    let reader = BufReader::new(file);
    let mut writer = EntityWriter::new_file_writer("./output")?;

    melt_json(reader, &mut writer, MeltConfig::default())?;

    Ok(())
}
```

### Single Output Stream

For cases where you want all entities in one stream with metadata:

```rust
use furnace::{JsonMelter, SingleWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let melter = JsonMelter::new(MeltConfig::default());
    let data = json!({"id": 1, "posts": [{"id": 10}]});

    let entities = melter.melt(data)?;

    let mut output = Vec::new();
    let mut writer = SingleWriter::new(&mut output);
    writer.write_entities(entities)?;

    // Each line includes: _entity_type, _entity_id, _parent_type, _parent_id
    Ok(())
}
```

---

## Schema Inference

Automatically infer JSON Schemas from examples with format detection.

### Basic Schema Inference

```rust
use furnace::infer_schema_streaming;
use serde_json::json;

fn main() {
    let examples = vec![
        json!({"name": "Alice", "email": "alice@example.com", "age": 30}),
        json!({"name": "Bob", "email": "bob@example.com", "age": 25}),
    ];

    let schema = infer_schema_streaming(&examples);

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

**Output includes:**
- `type`: "object"
- `required`: ["name", "email", "age"]
- `properties` with types for each field
- `format`: "email" for email field (**unique to Furnace!**)

### Format Detection

Furnace automatically detects these formats (not available in genson-rs):

- **date**: ISO 8601 dates (e.g., "2024-01-15")
- **time**: ISO 8601 times (e.g., "14:30:00")
- **email**: Email addresses
- **uuid**: UUIDs
- **ipv4**: IPv4 addresses
- **ipv6**: IPv6 addresses

### Streaming Schema Builder

For large datasets, use the streaming API:

```rust
use furnace::SchemaBuilder;
use serde_json::json;

fn main() {
    let mut builder = SchemaBuilder::new();

    // Process examples one at a time
    builder.add_value(&json!({"name": "Alice", "age": 30}));
    builder.add_value(&json!({"name": "Bob", "age": 25}));
    builder.add_value(&json!({"name": "Charlie"}));  // age is optional

    let schema = builder.into_schema();

    // required will be ["name"] (age is optional)
}
```

---

## Understanding Output

### Entity Naming

Entity types are named based on their JSON path:

| JSON Path | Entity Type | File |
|-----------|-------------|------|
| Root object | `root` | `root.jsonl` |
| `users` array | `root_users` | `root_users.jsonl` |
| `users[].posts` | `root_users_posts` | `root_users_posts.jsonl` |
| `users[].posts[].comments` | `root_users_posts_comments` | `root_users_posts_comments.jsonl` |

### Foreign Keys

Foreign keys maintain relationships between entities:

```json
// root.jsonl
{"id": 1, "name": "Alice"}

// root_posts.jsonl
{"id": 10, "title": "Post", "posts_id": "1"}  ← Links to parent
```

**Format:** `{parent_field_name}_id`
**Generated IDs:** Use format `_gen_N` when no ID exists in data

### Arrays of Scalars

```json
{"tags": ["rust", "json", "data"]}
```

Becomes:

```json
// root_tags.jsonl
{"value": "rust", "_idx": 0, "tags_id": "..."}
{"value": "json", "_idx": 1, "tags_id": "..."}
{"value": "data", "_idx": 2, "tags_id": "..."}
```

---

## Configuration

### MeltConfig Options

```rust
use furnace::MeltConfig;

let config = MeltConfig {
    // Maximum nesting depth to extract (default: 10)
    max_depth: 5,

    // Prefix for generated foreign key columns (default: "")
    fk_prefix: "".to_string(),

    // Suffix for ID columns (default: "_id")
    id_prefix: "_fk".to_string(),

    // Separator for nested entity names (default: "_")
    separator: "__".to_string(),  // root__users__posts

    // Include parent IDs in child entities (default: true)
    include_parent_ids: true,

    // Fields to always treat as scalars (never extract)
    scalar_fields: vec![
        "metadata".to_string(),
        "config".to_string(),
    ],
};

let melter = JsonMelter::new(config);
```

---

## Real-World Examples

### Example 1: Process Paginated API

```rust
use furnace::{PlannedMelter, EntityWriter, MeltConfig};
use serde_json::Value;

async fn process_api() -> anyhow::Result<()> {
    // Fetch first page to build plan
    let samples = fetch_samples(10).await?;
    let melter = PlannedMelter::from_examples(&samples, MeltConfig::default())?;

    let mut writer = EntityWriter::new_file_writer("./api_data")?;

    // Process all pages
    let mut page = 1;
    loop {
        let response = fetch_page(page).await?;
        let data: Vec<Value> = serde_json::from_value(response["data"].clone())?;

        if data.is_empty() {
            break;
        }

        for record in data {
            let entities = melter.melt(record)?;
            writer.write_entities(entities)?;
        }

        page += 1;
    }

    writer.flush()?;
    Ok(())
}
```

### Example 2: Query with jq

```bash
# Find users from USA
cat root_users.jsonl | jq 'select(.country == "USA")'

# Count posts per user
cat root_posts.jsonl | jq -r .posts_id | sort | uniq -c

# Get all tags
cat root_posts_tags.jsonl | jq -r .value | sort | uniq
```

### Example 3: Load into SQLite

```bash
# Install sqlite-utils
pip install sqlite-utils

# Import all tables
sqlite-utils insert data.db users root_users.jsonl --nl
sqlite-utils insert data.db posts root_posts.jsonl --nl
sqlite-utils insert data.db tags root_posts_tags.jsonl --nl

# Query with SQL
sqlite3 data.db "
  SELECT u.name, COUNT(p.id) as post_count
  FROM users u
  LEFT JOIN posts p ON p.posts_id = u.id
  GROUP BY u.id
"
```

### Example 4: Analyze with Python/Pandas

```python
import pandas as pd

# Load tables
users = pd.read_json('root_users.jsonl', lines=True)
posts = pd.read_json('root_posts.jsonl', lines=True)

# Join on foreign key
merged = posts.merge(users, left_on='posts_id', right_on='id')

# Analyze
print(merged.groupby('name')['title'].count())
```

---

## Performance

### Melting Performance

**Benchmark:** 1000 complex records (users with posts, tags, friends)

| Approach | Time | Per Record | Speedup |
|----------|------|------------|---------|
| JsonMelter (unplanned) | 12.2ms | 12.2μs | baseline |
| PlannedMelter | 8.8ms | 8.8μs | **1.4x faster** |
| Plan generation | 1.3ms | one-time | amortized |

**Run benchmark:**
```bash
cargo run --release --bin melt_performance_benchmark
```

### Schema Inference Performance

**Benchmark:** 100 real-world schemas

| Implementation | Average Time | vs genson-rs |
|----------------|--------------|--------------|
| Furnace | 1.12ms | 1.08x slower |
| genson-rs | 1.04ms | baseline |
| Python GenSON | 0.36ms | 2.89x faster |

**Correctness:** ✅ 100/100 schemas pass validation

**Run benchmark:**
```bash
cargo run --release --bin schema_inference_benchmark
cargo run --release --bin schema_correctness_validation
```

### When Performance Matters

**Use PlannedMelter when:**
- Processing >100 similar records
- Consistent data structure (API responses, logs, exports)
- Performance is critical

**Use JsonMelter when:**
- One-off data exploration
- Heterogeneous data
- Small datasets (<100 records)
- Unknown/varying structures

---

## Tips & Best Practices

### 1. Start Simple
Use default config first, inspect output, then customize:
```rust
let melter = JsonMelter::new(MeltConfig::default());
```

### 2. Inspect Output
Run examples to understand what files are created:
```bash
cargo run --example quickstart
ls -la *.jsonl
cat root.jsonl | jq
```

### 3. Use jq for Exploration
```bash
# Pretty print
cat root_users.jsonl | jq

# Filter
cat root_users.jsonl | jq 'select(.age > 25)'

# Extract field
cat root_users.jsonl | jq -r .name
```

### 4. Check Foreign Keys
Look for `*_id` fields to understand relationships:
```bash
cat root_posts.jsonl | jq '.posts_id'
```

### 5. Validate Schema
Use inferred schemas to validate new data:
```rust
let schema = infer_schema_streaming(&examples);
// Use schema with a JSON Schema validator library
```

---

## Troubleshooting

### Problem: Too many files created
**Solution:** Increase `max_depth` to limit extraction depth, or add fields to `scalar_fields`

### Problem: Missing relationships
**Solution:** Check `include_parent_ids` is `true` in config

### Problem: Out of memory
**Solution:** Use streaming API (`melt_json` function) for large files

### Problem: Wrong entity types
**Solution:** Customize `separator` in config to change naming

### Problem: Performance is slow
**Solution:** Use `PlannedMelter` for homogeneous data

---

## Design Principles

Furnace follows tidy data principles (see `tidy-data-principles.md`):

1. **Each variable is a column**: Scalar values become columns
2. **Each observation is a row**: Each entity instance is one row
3. **Each type of observational unit is a table**: Different entity types get separate tables

---

## Further Reading

- **[README.md](README.md)** - Overview and API reference
- **[tidy-data-principles.md](tidy-data-principles.md)** - Background on data organization
- **[schema_inference/PERFORMANCE_SUMMARY.md](schema_inference/PERFORMANCE_SUMMARY.md)** - Detailed optimization analysis
- **Examples in `examples/` directory** - Working code samples
