use serde_json::json;

fn main() {
    // Test 1: Simple object
    let examples = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"name": "Bob", "age": 25}),
    ];

    let schema_old = furnace::infer_schema(&examples);
    let schema_new = furnace::infer_schema_streaming(&examples);

    println!("=== Test 1: Simple Object ===");
    println!("Old schema: {}", serde_json::to_string_pretty(&schema_old).unwrap());
    println!("\nNew schema: {}", serde_json::to_string_pretty(&schema_new).unwrap());

    // Test 2: Optional fields
    let examples = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"name": "Bob"}),
    ];

    let schema_old = furnace::infer_schema(&examples);
    let schema_new = furnace::infer_schema_streaming(&examples);

    println!("\n=== Test 2: Optional Fields ===");
    println!("Old schema: {}", serde_json::to_string_pretty(&schema_old).unwrap());
    println!("\nNew schema: {}", serde_json::to_string_pretty(&schema_new).unwrap());

    // Test 3: Array of objects
    let examples = vec![
        json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]),
    ];

    let schema_old = furnace::infer_schema(&examples);
    let schema_new = furnace::infer_schema_streaming(&examples);

    println!("\n=== Test 3: Array of Objects ===");
    println!("Old schema: {}", serde_json::to_string_pretty(&schema_old).unwrap());
    println!("\nNew schema: {}", serde_json::to_string_pretty(&schema_new).unwrap());

    // Test 4: Format detection
    let examples = vec![
        json!("test@example.com"),
        json!("another@test.org"),
    ];

    let schema_old = furnace::infer_schema(&examples);
    let schema_new = furnace::infer_schema_streaming(&examples);

    println!("\n=== Test 4: Format Detection (Email) ===");
    println!("Old schema: {}", serde_json::to_string_pretty(&schema_old).unwrap());
    println!("\nNew schema: {}", serde_json::to_string_pretty(&schema_new).unwrap());
}
