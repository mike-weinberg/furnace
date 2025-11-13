use serde_json::json;
use std::time::Instant;

fn main() {
    println!("=== Performance Comparison: Original vs Streaming ===\n");

    // Test 1: Small dataset
    let small_examples: Vec<_> = (0..100)
        .map(|i| json!({"id": i, "name": format!("User {}", i), "age": 20 + (i % 50)}))
        .collect();

    let start = Instant::now();
    let _ = json_melt::infer_schema(&small_examples);
    let old_small = start.elapsed();

    let start = Instant::now();
    let _ = json_melt::infer_schema_streaming(&small_examples);
    let new_small = start.elapsed();

    println!("Small dataset (100 objects):");
    println!("  Original:  {:?}", old_small);
    println!("  Streaming: {:?}", new_small);
    println!("  Speedup:   {:.2}x\n", old_small.as_secs_f64() / new_small.as_secs_f64());

    // Test 2: Medium dataset
    let medium_examples: Vec<_> = (0..1000)
        .map(|i| json!({
            "id": i,
            "name": format!("User {}", i),
            "age": 20 + (i % 50),
            "email": format!("user{}@example.com", i),
            "active": i % 2 == 0,
        }))
        .collect();

    let start = Instant::now();
    let _ = json_melt::infer_schema(&medium_examples);
    let old_medium = start.elapsed();

    let start = Instant::now();
    let _ = json_melt::infer_schema_streaming(&medium_examples);
    let new_medium = start.elapsed();

    println!("Medium dataset (1,000 objects):");
    println!("  Original:  {:?}", old_medium);
    println!("  Streaming: {:?}", new_medium);
    println!("  Speedup:   {:.2}x\n", old_medium.as_secs_f64() / new_medium.as_secs_f64());

    // Test 3: Nested objects
    let nested_examples: Vec<_> = (0..500)
        .map(|i| json!({
            "id": i,
            "user": {
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i),
            },
            "posts": [
                {"id": i * 10, "title": format!("Post {}", i * 10)},
                {"id": i * 10 + 1, "title": format!("Post {}", i * 10 + 1)},
            ]
        }))
        .collect();

    let start = Instant::now();
    let _ = json_melt::infer_schema(&nested_examples);
    let old_nested = start.elapsed();

    let start = Instant::now();
    let _ = json_melt::infer_schema_streaming(&nested_examples);
    let new_nested = start.elapsed();

    println!("Nested dataset (500 objects with nesting):");
    println!("  Original:  {:?}", old_nested);
    println!("  Streaming: {:?}", new_nested);
    println!("  Speedup:   {:.2}x\n", old_nested.as_secs_f64() / new_nested.as_secs_f64());

    // Test 4: Large dataset
    let large_examples: Vec<_> = (0..5000)
        .map(|i| json!({
            "id": i,
            "name": format!("User {}", i),
            "age": 20 + (i % 50),
            "email": format!("user{}@example.com", i),
            "active": i % 2 == 0,
            "created_at": "2021-01-01T00:00:00Z",
        }))
        .collect();

    let start = Instant::now();
    let _ = json_melt::infer_schema(&large_examples);
    let old_large = start.elapsed();

    let start = Instant::now();
    let _ = json_melt::infer_schema_streaming(&large_examples);
    let new_large = start.elapsed();

    println!("Large dataset (5,000 objects):");
    println!("  Original:  {:?}", old_large);
    println!("  Streaming: {:?}", new_large);
    println!("  Speedup:   {:.2}x\n", old_large.as_secs_f64() / new_large.as_secs_f64());

    // Overall average
    let avg_speedup = (
        old_small.as_secs_f64() / new_small.as_secs_f64() +
        old_medium.as_secs_f64() / new_medium.as_secs_f64() +
        old_nested.as_secs_f64() / new_nested.as_secs_f64() +
        old_large.as_secs_f64() / new_large.as_secs_f64()
    ) / 4.0;

    println!("=== Summary ===");
    println!("Average speedup: {:.2}x", avg_speedup);
}
