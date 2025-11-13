use json_melt::{EntityWriter, JsonMelter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    println!("JSON Melt Example: API Response Processing\n");

    // Example 1: Simple user with posts
    println!("=== Example 1: User with Posts ===\n");
    let example1 = json!({
        "id": 1,
        "name": "Alice Johnson",
        "email": "alice@example.com",
        "posts": [
            {
                "id": 101,
                "title": "Introduction to Rust",
                "content": "Rust is a systems programming language...",
                "tags": ["rust", "programming", "tutorial"]
            },
            {
                "id": 102,
                "title": "Data Processing with JSON",
                "content": "Working with JSON in Rust...",
                "tags": ["rust", "json", "data"]
            }
        ]
    });

    process_example(example1)?;

    println!("\n=== Example 2: E-commerce Order ===\n");
    let example2 = json!({
        "order_id": "ORD-2024-001",
        "customer": {
            "id": 42,
            "name": "Bob Smith",
            "address": {
                "street": "123 Main St",
                "city": "Springfield",
                "country": "USA"
            }
        },
        "items": [
            {
                "product_id": "PROD-001",
                "name": "Laptop",
                "quantity": 1,
                "price": 999.99
            },
            {
                "product_id": "PROD-002",
                "name": "Mouse",
                "quantity": 2,
                "price": 29.99
            }
        ],
        "shipping": {
            "method": "express",
            "tracking_number": "TRK123456",
            "status": "in_transit"
        }
    });

    process_example(example2)?;

    println!("\n=== Example 3: GitHub-style API Response ===\n");
    let example3 = json!({
        "repository": {
            "id": 12345,
            "name": "awesome-project",
            "owner": {
                "id": 999,
                "login": "rustacean",
                "type": "User"
            },
            "stargazers_count": 1500,
            "watchers_count": 250
        },
        "issues": [
            {
                "number": 42,
                "title": "Bug in parser",
                "state": "open",
                "labels": [
                    {"name": "bug", "color": "red"},
                    {"name": "high-priority", "color": "orange"}
                ],
                "comments": [
                    {
                        "id": 1001,
                        "user": "contributor1",
                        "body": "I can reproduce this issue"
                    },
                    {
                        "id": 1002,
                        "user": "maintainer",
                        "body": "Thanks for the report!"
                    }
                ]
            },
            {
                "number": 43,
                "title": "Feature request: streaming support",
                "state": "open",
                "labels": [
                    {"name": "enhancement", "color": "green"}
                ],
                "comments": []
            }
        ]
    });

    process_example(example3)?;

    println!("\n=== Files Created ===");
    println!("Check the current directory for .jsonl files:");
    println!("- root.jsonl");
    println!("- root_posts.jsonl");
    println!("- root_posts_tags.jsonl");
    println!("- root_items.jsonl");
    println!("- root_issues.jsonl");
    println!("- root_issues_labels.jsonl");
    println!("- root_issues_comments.jsonl");
    println!("\nEach file contains one JSON object per line (JSON Lines format)");

    Ok(())
}

fn process_example(json: serde_json::Value) -> anyhow::Result<()> {
    let config = MeltConfig::default();
    let melter = JsonMelter::new(config);

    // Melt the JSON into entities
    let entities = melter.melt(json)?;

    println!("Extracted {} entities:\n", entities.len());

    // Print each entity
    for (i, entity) in entities.iter().enumerate() {
        println!("Entity {}: {}", i + 1, entity.entity_type);
        println!("  Data: {}", serde_json::to_string_pretty(&entity.data)?);

        if let Some(ref id) = entity.id {
            println!("  ID: {}", id.0);
        }

        if let Some(ref parent) = entity.parent {
            println!("  Parent: {} (id: {})", parent.entity_type, parent.id.0);
        }

        println!();
    }

    // Write to files
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    Ok(())
}
