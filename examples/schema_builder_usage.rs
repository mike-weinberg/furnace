/// Example demonstrating the SchemaBuilder API
use furnace::SchemaBuilder;
use serde_json::json;

fn main() {
    println!("=== SchemaBuilder Usage Examples ===\n");

    // Example 1: Basic usage with add_value
    println!("Example 1: Building a schema incrementally");
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({"name": "Alice", "age": 30}));
    builder.add_value(&json!({"name": "Bob", "age": 25}));
    builder.add_value(&json!({"name": "Charlie", "age": 35}));

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Example 2: Streaming from an iterator
    println!("Example 2: Processing a stream of values");
    let values = vec![
        json!({"id": 1, "email": "user1@example.com"}),
        json!({"id": 2, "email": "user2@example.com"}),
        json!({"id": 3, "email": "user3@example.com"}),
    ];

    let mut builder = SchemaBuilder::new();
    for value in &values {
        builder.add_value(value);
    }

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Example 3: Handling optional fields
    println!("Example 3: Optional fields detection");
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({
        "name": "Alice",
        "age": 30,
        "email": "alice@example.com"
    }));
    builder.add_value(&json!({
        "name": "Bob",
        "age": 25
        // email is missing
    }));
    builder.add_value(&json!({
        "name": "Charlie",
        "age": 35,
        "email": "charlie@example.com"
    }));

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());
    println!("Note: Only 'name' and 'age' are required (appear in all samples)\n");

    // Example 4: Nested objects
    println!("Example 4: Nested objects");
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!({
        "user": {
            "name": "Alice",
            "contact": {
                "email": "alice@example.com",
                "phone": "+1234567890"
            }
        }
    }));
    builder.add_value(&json!({
        "user": {
            "name": "Bob",
            "contact": {
                "email": "bob@example.com",
                "phone": "+0987654321"
            }
        }
    }));

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Example 5: Arrays
    println!("Example 5: Arrays with uniform items");
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!([1, 2, 3, 4, 5]));
    builder.add_value(&json!([10, 20, 30]));

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Example 6: Format detection
    println!("Example 6: String format detection");
    let examples = vec![
        ("Emails", vec![
            json!("alice@example.com"),
            json!("bob@test.org"),
        ]),
        ("UUIDs", vec![
            json!("550e8400-e29b-41d4-a716-446655440000"),
            json!("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
        ]),
        ("Dates", vec![
            json!("2021-01-01"),
            json!("2021-12-31"),
        ]),
        ("Date-times", vec![
            json!("2021-01-01T00:00:00Z"),
            json!("2021-12-31T23:59:59Z"),
        ]),
    ];

    for (name, values) in examples {
        let mut builder = SchemaBuilder::new();
        for value in &values {
            builder.add_value(value);
        }
        let schema = builder.build();
        println!("{}: {}", name, serde_json::to_string(&schema).unwrap());
    }
    println!();

    // Example 7: Nullable fields
    println!("Example 7: Nullable fields");
    let mut builder = SchemaBuilder::new();

    builder.add_value(&json!("hello"));
    builder.add_value(&json!(null));
    builder.add_value(&json!("world"));

    let schema = builder.build();
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Example 8: Convenience function
    println!("Example 8: Using the convenience function");
    let examples = vec![
        json!({"status": "active", "count": 10}),
        json!({"status": "inactive", "count": 5}),
    ];

    let schema = furnace::infer_schema_streaming(&examples);
    println!("Schema: {}\n", serde_json::to_string_pretty(&schema).unwrap());
}
