# JSON Melt

A Rust library for streaming JSON into relational (melted) tabular data. Converts nested JSON structures from APIs into normalized tables with proper foreign key relationships.

## Overview

JSON Melt automatically detects entities within JSON data and splits them into separate relational tables, making it easy to:
- Process paginated API responses into queryable datasets
- Convert nested JSON into database-friendly formats
- Normalize complex JSON structures while maintaining relationships
- Stream process large JSON files without loading everything into memory

## Features

- **Automatic Entity Detection**: Identifies objects and arrays that should become separate tables
- **Foreign Key Tracking**: Maintains relationships between parent and child entities
- **ID Generation**: Automatically generates IDs when not present in the data
- **JSON Lines Output**: Outputs each entity type to its own `.jsonl` file
- **Streaming Processing**: Handles large datasets efficiently
- **Configurable**: Control extraction depth, field names, and extraction rules

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
json-melt = "0.1.0"
```

## Quick Start

```rust
use json_melt::{EntityWriter, JsonMelter, MeltConfig};
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

    let config = MeltConfig::default();
    let melter = JsonMelter::new(config);

    // Extract entities
    let entities = melter.melt(data)?;

    // Write to files
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    Ok(())
}
```

This creates two files:
- `root.jsonl`: Contains the root entity with `id` and `name`
- `root_posts.jsonl`: Contains posts with `posts_id` foreign key

## How It Works

JSON Melt applies these rules to transform JSON:

1. **Arrays are always extracted** as separate entity types
2. **Objects with IDs or multiple fields** become separate entities
3. **Foreign keys are automatically added** to maintain relationships
4. **Entity types are named** based on their JSON path (e.g., `users_posts`)
5. **Missing IDs are generated** to ensure all entities are identifiable

### Example Transformation

**Input JSON:**
```json
{
  "user": {
    "id": 1,
    "name": "Alice",
    "posts": [
      {"id": 101, "title": "Hello", "tags": ["intro", "welcome"]},
      {"id": 102, "title": "Update", "tags": ["news"]}
    ]
  }
}
```

**Output Entities:**

`root.jsonl`:
```json
{}
```

`root_user.jsonl`:
```json
{"id": 1, "name": "Alice", "user_id": "_gen_1"}
```

`root_user_posts.jsonl`:
```json
{"id": 101, "title": "Hello", "posts_id": "1"}
{"id": 102, "title": "Update", "posts_id": "1"}
```

`root_user_posts_tags.jsonl`:
```json
{"value": "intro", "_idx": 0, "tags_id": "101"}
{"value": "welcome", "_idx": 1, "tags_id": "101"}
{"value": "news", "_idx": 0, "tags_id": "102"}
```

## Configuration

Customize the melting process with `MeltConfig`:

```rust
use json_melt::MeltConfig;

let config = MeltConfig {
    max_depth: 10,              // Maximum nesting level to extract
    fk_prefix: "".to_string(),  // Prefix for foreign key columns
    id_prefix: "_id".to_string(), // Suffix for ID columns
    separator: "_".to_string(),  // Separator for nested entity names
    include_parent_ids: true,    // Add parent foreign keys
    scalar_fields: vec![         // Fields to never extract
        "metadata".to_string(),
    ],
};
```

## Examples

See the `examples/` directory for complete examples:

```bash
cargo run --example api_example
```

This demonstrates:
- User with posts and tags
- E-commerce order with nested customer and items
- GitHub-style API response with issues, labels, and comments

## Use Cases

### API Response Processing

```rust
// Process multiple pages of API responses
let melter = JsonMelter::new(MeltConfig::default());
let mut writer = EntityWriter::new_file_writer("./output")?;

for page in api_responses {
    let entities = melter.melt(page)?;
    writer.write_entities(entities)?;
}

writer.flush()?;
```

### JSON Lines Streaming

```rust
use std::io::BufReader;
use std::fs::File;

let file = File::open("data.jsonl")?;
let reader = BufReader::new(file);
let mut writer = EntityWriter::new_file_writer("./output")?;

json_melt::melt_json(reader, &mut writer, MeltConfig::default())?;
```

### Single Output Stream

For cases where you want all entities in one file with metadata:

```rust
use json_melt::SingleWriter;

let mut output = Vec::new();
let mut writer = SingleWriter::new(&mut output);

writer.write_entities(entities)?;

// Each line includes _entity_type, _entity_id, _parent_type, _parent_id
```

## Entity Naming

Entity types are named based on their path in the JSON:

| JSON Path | Entity Type |
|-----------|-------------|
| Root object | `root` |
| `users` array | `root_users` |
| `users[].posts` array | `root_users_posts` |
| `users[].posts[].comments` | `root_users_posts_comments` |

## Foreign Keys

Foreign keys are automatically added to maintain relationships:

- **Format**: `{parent_field_name}_id`
- **Example**: Post has `user_id` linking to parent user
- **Generated IDs**: Use format `_gen_N` when no ID exists

## Design Principles

Following tidy data principles (see `tidy-data-principles.md`):

1. **Each variable is a column**: Scalar values become columns
2. **Each observation is a row**: Each entity instance is one row
3. **Each type of observational unit is a table**: Different entity types get separate tables

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover:
- Simple objects
- Nested arrays
- Scalar arrays
- Foreign key generation
- Writer functionality

## Performance

- **Streaming**: Processes JSON line-by-line
- **Memory efficient**: Doesn't load entire dataset into memory
- **Lazy evaluation**: Only processes what's needed
- **Async ready**: Core logic is synchronous but can be wrapped for async use

## Limitations

- **No schema inference**: Doesn't validate consistency across records
- **Simple ID detection**: Only looks for `id` field (case-sensitive)
- **File-based output**: Primary writer outputs to files (use `SingleWriter` for other sinks)

## Contributing

This is a learning project demonstrating:
- Tidy data principles applied to JSON
- Streaming data processing in Rust
- Entity extraction and normalization

## License

MIT

## Schema Inference Module

This project also includes an optimized JSON schema inference library (ported from Python to Rust).

### Performance Highlights

**6.01x faster than genson-rs!** After optimization:

![Performance Summary](https://raw.githubusercontent.com/mike-weinberg/furnace/main/schema_inference/performance_graphs.png)

![Optimization Timeline](https://raw.githubusercontent.com/mike-weinberg/furnace/main/schema_inference/optimization_timeline.png)

### Key Results

| Metric | Value |
|--------|-------|
| **Speedup vs genson-rs** | 6.01x faster |
| **Python → Rust Optimization** | 59x improvement |
| **Optimization Cycles** | 2 cycles completed |
| **Test Coverage** | 100 real-world schemas |

### Optimization Journey

1. **Cycle 1: Pre-compile Regex Patterns**
   - Identified 99% of overhead in regex compilation
   - Used `once_cell::Lazy` for lazy static initialization
   - **Result: 59x improvement** (389.68ms → 6.59ms)

2. **Cycle 2: Early Byte Validation**
   - Added fast byte position checks before regex matching
   - Optimized UUID, date, and datetime detection
   - **Result: 6.01x faster than genson-rs** (6.59ms → 7.22ms)

### Features

- Full JSON Schema Draft 7 support
- Format detection: date, time, email, UUID, IPv4, IPv6
- Required field tracking
- Proper type unification and merging
- Production-ready Rust implementation

### Documentation

See [`schema_inference/PERFORMANCE_SUMMARY.md`](schema_inference/PERFORMANCE_SUMMARY.md) for comprehensive analysis including:
- Detailed performance comparisons
- Optimization methodology
- Benchmarking results
- Future optimization opportunities

### Benchmarks

Run the performance benchmarks:

```bash
# Rust vs genson-rs
cargo run --release --bin schema_inference_benchmark

# genson-rs baseline
cargo run --release --bin genson_benchmark

# Python reference
cd schema_inference
source venv/bin/activate
python3 src/benchmarking/benchmark.py
```

---

## See Also

- [Tidy Data Principles](tidy-data-principles.md) - Background on data organization
- [Schema Inference Performance Report](schema_inference/PERFORMANCE_SUMMARY.md) - Detailed optimization analysis
- Hadley Wickham's "Tidy Data" paper
- R's `tidyr` package for similar functionality
