# SchemaBuilder - Streaming Schema Inference

## Overview

`SchemaBuilder` is an optimized implementation of JSON schema inference using a streaming accumulator pattern inspired by genson-rs's architecture. Unlike the original `infer_schema` implementation which creates intermediate schemas for each value and merges them, `SchemaBuilder` accumulates statistics and builds the final schema only once at the end.

## Key Features

- **Streaming accumulator pattern**: Process values one at a time without creating intermediate schemas
- **Efficient memory usage**: Uses HashMaps and HashSets for tracking instead of creating full schema objects
- **Zero-copy where possible**: Uses references instead of cloning values
- **Type-specific builders**: Specialized builders for objects, arrays, and strings
- **Format detection**: Automatic detection of string formats (email, UUID, date, datetime, URI, etc.)
- **Required field tracking**: Uses set intersection to determine which fields appear in all samples
- **Recursive support**: Handles nested objects and arrays

## Performance

Compared to the original `infer_schema` implementation:

- **Small datasets (100 objects)**: ~8.3x faster
- **Medium datasets (1,000 objects)**: ~5.6x faster
- **Large datasets (5,000 objects)**: ~5.8x faster
- **Nested datasets**: ~8.0x faster
- **Overall average**: **~7x faster**

Compared to genson-rs:
- **~1.55x slower** on average (which is quite good considering we use `serde_json::Value` instead of `simd_json::BorrowedValue`)

## Architecture

### Core Components

1. **`SchemaBuilder`**: Main accumulator that tracks:
   - Type counts across all samples
   - Total sample count
   - Type-specific sub-builders (string, array, object)

2. **`StringStats`**: Tracks string formats:
   - Format counts for each detected format
   - Only returns format if all strings match the same format

3. **`ArrayBuilder`**: Accumulates array statistics:
   - Uses a nested `SchemaBuilder` for items
   - Processes all array elements across all samples

4. **`ObjectBuilder`**: Tracks object properties:
   - Map of property names to their builders
   - Tracks which properties appear in each sample
   - Computes required fields using set intersection

### Type System

The implementation uses an internal `JsonType` enum for efficient type tracking:

```rust
enum JsonType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}
```

## API Usage

### Basic Usage

```rust
use json_melt::SchemaBuilder;
use serde_json::json;

let mut builder = SchemaBuilder::new();
builder.add_value(&json!({"name": "Alice", "age": 30}));
builder.add_value(&json!({"name": "Bob", "age": 25}));

let schema = builder.build();
```

### Streaming from Iterator

```rust
let values = vec![/* ... */];
let mut builder = SchemaBuilder::new();

for value in &values {
    builder.add_value(value);
}

let schema = builder.build();
```

### Convenience Function

```rust
use json_melt::infer_schema_streaming;

let examples = vec![
    json!({"name": "Alice"}),
    json!({"name": "Bob"}),
];

let schema = infer_schema_streaming(&examples);
```

## Implementation Details

### Building Process

The `build()` method follows this strategy:

1. **Empty check**: Return empty object if no samples processed
2. **Single type case**: Build schema for that type directly
3. **Nullable case**: If exactly 2 types and one is null, make the other nullable
4. **Multiple types**: Create a type array with all seen types

### Required Field Detection

For objects, required fields are determined by:

1. Track which properties appear in each sample
2. Compute intersection of all property sets
3. Fields in the intersection are required (appear in ALL samples)
4. Sort alphabetically for consistent output

### Format Detection

String formats are detected using optimized checks:

1. **Fast-path checks**: Length and byte checks before regex
2. **URI**: Prefix matching (http://, https://, etc.)
3. **Date**: Fixed length (10 chars) with dash positions
4. **Email**: Contains @ before regex
5. **UUID**: Fixed length (36 chars) with dash at position 8
6. **DateTime**: Length ≥19 with 'T' at position 10
7. **IPv4/IPv6**: Contains dots/colons before regex

Formats are only added to the schema if ALL string values match the same format.

### Memory Efficiency

The implementation avoids unnecessary allocations:

- Uses `&Value` references in `add_value()`
- Builds sub-schemas recursively without cloning
- Only creates the final JSON schema once in `build()`
- Uses `Box<SchemaBuilder>` for recursive structures to keep size small

## Testing

The module includes comprehensive tests:

- Empty builder
- Simple types (string, number, boolean)
- Objects with required fields
- Optional field detection
- Arrays of primitives and objects
- Nested objects
- Nullable fields
- Format detection (email, UUID, date, datetime)
- Streaming function wrapper

Run tests:

```bash
cargo test --lib schema_builder
```

## Examples

Three examples demonstrate the functionality:

1. **schema_builder_usage.rs**: Comprehensive API examples
2. **schema_comparison.rs**: Compare output with original implementation
3. **schema_performance.rs**: Performance benchmarks

Run examples:

```bash
cargo run --example schema_builder_usage
cargo run --example schema_comparison
cargo run --example schema_performance --release
```

## Benchmarking

Compare with genson-rs:

```bash
cargo build --release --bin schema_builder_benchmark
./target/release/schema_builder_benchmark
```

## Future Optimizations

Potential improvements:

1. **Switch to simd-json**: Use `BorrowedValue` instead of `serde_json::Value`
2. **Parallel processing**: Process samples in parallel using rayon
3. **String interning**: Reduce memory for repeated strings
4. **Format detection caching**: Cache regex results for identical strings
5. **Separate type builders**: Track multiple types simultaneously instead of falling back to type array

## Comparison with Original Implementation

| Aspect | Original (`infer_schema`) | New (`SchemaBuilder`) |
|--------|---------------------------|----------------------|
| Pattern | Create + Merge | Accumulate + Build |
| Intermediate objects | Yes (one per value) | No |
| Memory usage | Higher | Lower |
| Speed | Baseline | ~7x faster |
| API | Function-based | Builder-based |
| Streaming | No | Yes |

## When to Use

Use `SchemaBuilder` when:

- Processing large datasets
- Streaming data from external sources
- Memory efficiency is important
- You need incremental schema building

Use `infer_schema` when:

- Simplicity is preferred over performance
- Dataset is very small (< 10 samples)
- You have pre-collected examples array

## License

Same as the parent project.
