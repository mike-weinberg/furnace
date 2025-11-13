# Furnace

A high-performance Rust library for JSON processing: **melt** nested JSON into relational tables and **infer** JSON Schemas with format detection.

## Overview

Furnace combines two powerful JSON processing capabilities:

1. **JSON Melting**: Convert nested JSON structures into flat, relational tables with foreign keys
2. **Schema Inference**: Automatically infer JSON Schemas from examples with format detection

Perfect for:
- Processing paginated API responses into queryable datasets
- Converting nested JSON into database-friendly formats
- Normalizing complex JSON structures while maintaining relationships
- Generating accurate JSON Schemas for documentation and validation
- Stream processing large JSON files without loading everything into memory

## Features

### JSON Melting
- **Automatic Entity Detection**: Identifies objects and arrays that should become separate tables
- **Schema-Guided Extraction**: Use `PlannedMelter` for 40-50% faster processing of homogeneous data
- **Foreign Key Tracking**: Maintains relationships between parent and child entities
- **ID Generation**: Automatically generates IDs when not present in the data
- **JSON Lines Output**: Outputs each entity type to its own `.jsonl` file
- **Streaming Processing**: Handles large datasets efficiently

### Schema Inference
- **Format Detection**: Identifies date, time, email, UUID, IPv4, IPv6 formats (not in genson-rs!)
- **Required Fields**: Tracks which fields are always present
- **Type Unification**: Properly merges schemas from multiple examples
- **High Performance**: Near-competitive with genson-rs (1.08x slower, 100% correctness)
- **Production Ready**: Validated on 100 real-world schemas

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
furnace = "0.1.0"
```

## Quick Start

### JSON Melting (Basic)

```rust
use furnace::{EntityWriter, JsonMelter, MeltConfig};
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

### JSON Melting (High-Performance)

For processing many similar records (like API pagination), use `PlannedMelter`:

```rust
use furnace::{PlannedMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // Sample 10 records to build extraction plan
    let samples = vec![
        json!({"id": 1, "name": "Alice", "posts": [{"id": 10, "title": "Post"}]}),
        json!({"id": 2, "name": "Bob", "posts": [{"id": 20, "title": "Post"}]}),
        // ... 8 more samples
    ];

    // Create planned melter (analyzes structure once)
    let melter = PlannedMelter::from_examples(&samples, MeltConfig::default())?;

    // Process thousands of records with pre-computed plan (40% faster!)
    for page in api_responses {
        let entities = melter.melt(page)?;
        // ... write entities
    }

    Ok(())
}
```

**Performance**: PlannedMelter is 40-50% faster by eliminating runtime decisions.

### Schema Inference

```rust
use furnace::infer_schema_streaming;
use serde_json::json;

let examples = vec![
    json!({"name": "Alice", "email": "alice@example.com", "age": 30}),
    json!({"name": "Bob", "email": "bob@example.com", "age": 25}),
];

let schema = infer_schema_streaming(&examples);

// Schema includes:
// - type: "object"
// - required: ["name", "email", "age"]
// - properties with types
// - format: "email" for email field (unique to Furnace!)
```

## How JSON Melting Works

Furnace applies these rules to transform JSON:

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
use furnace::MeltConfig;

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

furnace::melt_json(reader, &mut writer, MeltConfig::default())?;
```

### Single Output Stream

For cases where you want all entities in one file with metadata:

```rust
use furnace::SingleWriter;

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

## Performance: Planned vs Unplanned Melting

Furnace offers two melting approaches:

### JsonMelter (Unplanned)
- Makes extraction decisions at runtime for each record
- Great for heterogeneous data or one-off processing
- No setup cost

### PlannedMelter (Schema-Guided)
- Analyzes sample data once to build extraction plan
- Processes subsequent records with pre-computed decisions
- **40-50% faster** for homogeneous data streams

**Benchmark Results** (1000 complex records):

| Approach | Time | Per Record | Speedup |
|----------|------|------------|---------|
| JsonMelter (unplanned) | 12.2ms | 12.2μs | baseline |
| PlannedMelter (extraction) | 8.8ms | 8.8μs | **1.4x faster** |
| Plan generation (one-time) | 1.3ms | - | amortized |

**When to use PlannedMelter:**
- Processing paginated API responses
- Log file streams with consistent structure
- Database exports with uniform schema
- Any scenario with >100 similar records

**How it works:**
1. Analyze 10-100 sample records with schema inference
2. Generate pre-computed extraction rules (which fields to extract, array types, etc.)
3. Process remaining records using the plan (no conditionals!)
4. Plan generation cost is amortized over thousands of records

Run the benchmark yourself:
```bash
cargo run --release --bin melt_performance_benchmark
```

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

## Memory Safety: Processing Large Files

Furnace is designed for streaming processing of large JSON datasets. However, the `--planned` mode has specific requirements:

### Key Safeguards

**Streaming by Default:**
```bash
# Safe for any file size - processes line-by-line with constant memory usage
furnace-melt large_file.jsonl --output-dir ./output
```

**Planned Mode (Schema-Guided):**
```bash
# Optimal for files > 1GB: samples first N records, builds plan, processes rest
# File path MUST be provided for full-file processing
furnace-melt --planned large_file.jsonl --output-dir ./output

# With stdin/pipe, only first N records are processed (memory-safe but limited)
cat large_file.jsonl | furnace-melt --planned > output.jsonl
# ⚠ Warning: Using --planned mode with stdin. Only first 100 records are processed.
```

### Why This Matters

Previous versions had a critical bug where `--planned` mode with file input would load the entire file into memory before processing, causing system lockup on files >1GB. This has been **fixed** - the tool now:

1. **First pass**: Samples only the first N records (default: 100) to build extraction plan
2. **Second pass**: Re-opens the file and processes all records with constant memory usage
3. **Stdin limitation**: If piping data via stdin, only the sampled records can be processed (stdin can't be re-read)

### Recommendations

| Use Case | Recommended Command |
|----------|-------------------|
| Small file (< 100MB) | `furnace-melt file.json` (default) |
| Large file (> 1GB) | `furnace-melt --planned file.jsonl` (best performance) |
| Streaming pipeline | `cmd \| furnace-melt` (no `--planned` for full processing) |
| Unknown data size | Use `--planned` with file path for safety |

### Technical Details

- **Memory per record**: ~10-50 KB depending on nesting depth (JSON parsing)
- **Sampling phase**: Only `sample_size` records (default 100) held in memory
- **Processing phase**: Each record processed and released independently
- **Output buffering**: Small buffer (4KB) per entity type being written

For very large individual records (>100MB each), you may still need to adjust system limits, but typical JSON records process safely.

## Contributing

This is a learning project demonstrating:
- Tidy data principles applied to JSON
- Streaming data processing in Rust
- Entity extraction and normalization

## License

MIT

## Schema Inference Module

This project includes a production-ready JSON schema inference library ported from Python to Rust with careful optimization and fair benchmarking.

### Performance Summary

**Fair Benchmark (already-parsed input):**
- Our implementation: **6.99ms average**
- genson-rs: **0.89ms average**
- Ratio: **7.89x slower**
- Validation: ✅ 100/100 schemas pass correctness tests

**Trade-off:** We're slower but provide superior schema quality with format detection, better required field tracking, and proper type unification - features not available in genson-rs.

![Performance Summary](https://raw.githubusercontent.com/mike-weinberg/furnace/main/docs/performance_graphs.png?t=1763021108)

![Optimization Timeline](https://raw.githubusercontent.com/mike-weinberg/furnace/main/docs/optimization_timeline.png?t=1763021108)

### Key Results

| Metric | Value |
|--------|-------|
| **Correctness** | ✅ 100/100 schemas pass validation |
| **Performance vs genson-rs** | 7.89x slower |
| **Quality advantage** | Format detection (date, time, email, UUID, IPv4, IPv6) |
| **Required fields** | Proper tracking across all examples |
| **Type unification** | Comprehensive merging of schemas |

### Optimization History

1. **Cycle 1: Pre-compile Regex Patterns** ✅
   - Identified 99% of overhead in regex compilation
   - Used `once_cell::Lazy` for lazy static initialization
   - **Result: 59x improvement** (389.68ms → 6.59ms)

2. **Current State**
   - Fair benchmarking against genson-rs (both receive already-parsed input)
   - **Result: 7.89x slower than genson-rs**
   - Trade-off accepted for superior schema quality features

### Features

- Full JSON Schema Draft 7 support
- **Format detection**: date, time, email, UUID, IPv4, IPv6 (not supported by genson-rs or GenSON)
- Required field tracking
- Proper type unification and merging
- Production-ready Rust implementation

### Comparison with genson-rs

Furnace produces richer, more descriptive schemas than genson-rs:

| Feature | Furnace | genson-rs |
|---------|---------|-----------|
| **Format detection** | ✅ Detects date, time, email, UUID, IPv4, IPv6 | ❌ Not supported |
| **Required fields** | ✅ Yes | ✅ Yes |
| **Type unification** | ✅ Yes | ✅ Yes |
| **Performance** | 6.99ms average | 0.89ms average (7.89x faster) |

**Why the difference?** genson-rs intentionally minimizes feature scope to stay lightweight. Furnace prioritizes richer schema output with format detection, making it ideal for data exploration and documentation where detailed format information is valuable.

### Documentation

See [`docs/PERFORMANCE_SUMMARY.md`](docs/PERFORMANCE_SUMMARY.md) for comprehensive analysis including:
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

### Testing & Correctness Validation

The schema inference implementation is validated through integration tests that verify inferred schemas correctly describe the data they were inferred from:

**Run Python integration tests:**
```bash
cd schema_inference
source venv/bin/activate
python3 -m pytest src/tests/test_integration.py::TestIntegrationWithRealSchemas::test_infer_and_validate_all_schemas -v
```

**Run Rust correctness validation:**
```bash
cargo run --release --bin schema_correctness_validation
```

**Results:**
- Python tests: ✅ 100/100 schemas pass
- Rust tests: ✅ 100/100 schemas pass

**What this validates:**
1. Inferred schemas can validate all examples they were inferred from
2. Schema inference works correctly across all 100 real-world schemas
3. Implementation is compatible with Python reference implementation

**Important Note on Schema Types:**
The test data includes two types of schemas:
- **Hand-written schemas** (`schema.json`): Prescriptive schemas that define validation rules
- **Inferred schemas** (generated from examples): Descriptive schemas that describe what data looks like

Schema inference tools produce *descriptive* schemas, not *prescriptive* validation schemas. These serve different purposes and should not be compared directly. The correctness tests above validate the right thing: that inferred schemas describe their source examples accurately.

---

## See Also

- [Tidy Data Principles](tidy-data-principles.md) - Background on data organization
- [Schema Inference Performance Report](docs/PERFORMANCE_SUMMARY.md) - Detailed optimization analysis
- Hadley Wickham's "Tidy Data" paper
- R's `tidyr` package for similar functionality
