/// Example: Processing paginated API responses
/// This simulates fetching multiple pages from an API and melting them all into tables
use furnace::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    println!("=== Processing Paginated API Responses ===\n");

    // Simulate 3 pages of API responses
    let page1 = json!({
        "page": 1,
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "country": "USA",
                "orders": [
                    {"id": 100, "product": "Laptop", "amount": 999.99},
                    {"id": 101, "product": "Mouse", "amount": 29.99}
                ]
            },
            {
                "id": 2,
                "name": "Bob",
                "country": "UK",
                "orders": [
                    {"id": 200, "product": "Keyboard", "amount": 79.99}
                ]
            }
        ]
    });

    let page2 = json!({
        "page": 2,
        "users": [
            {
                "id": 3,
                "name": "Carol",
                "country": "Canada",
                "orders": [
                    {"id": 300, "product": "Monitor", "amount": 399.99},
                    {"id": 301, "product": "Webcam", "amount": 89.99}
                ]
            }
        ]
    });

    let page3 = json!({
        "page": 3,
        "users": [
            {
                "id": 4,
                "name": "Dave",
                "country": "Australia",
                "orders": []  // User with no orders
            }
        ]
    });

    // Set up the melter and writer
    let melter = JsonMelter::new(MeltConfig::default());
    let mut writer = EntityWriter::new_file_writer("./output")?;

    // Process each page
    let pages = vec![page1, page2, page3];
    for (i, page) in pages.into_iter().enumerate() {
        println!("Processing page {}...", i + 1);
        let entities = melter.melt(page)?;
        println!("  Extracted {} entities", entities.len());
        writer.write_entities(entities)?;
    }

    writer.flush()?;

    println!("\n✓ All pages processed!");
    println!("\nOutput files in ./output/:");
    println!("  • root.jsonl           - Page metadata");
    println!("  • root_users.jsonl     - All users from all pages");
    println!("  • root_users_orders.jsonl - All orders with user_id foreign key");

    println!("\n📊 Summary Statistics:");

    // Read back and count
    use std::io::BufRead;
    let users = std::fs::File::open("./output/root_users.jsonl")?;
    let user_count = std::io::BufReader::new(users).lines().count();

    let orders = std::fs::File::open("./output/root_users_orders.jsonl")?;
    let order_count = std::io::BufReader::new(orders).lines().count();

    println!("  Total users: {}", user_count);
    println!("  Total orders: {}", order_count);

    println!("\n💡 Try these queries:");
    println!("  # View all users");
    println!("  cat ./output/root_users.jsonl | jq");
    println!();
    println!("  # Find orders over $100");
    println!("  cat ./output/root_users_orders.jsonl | jq 'select(.amount > 100)'");
    println!();
    println!("  # Count orders per user");
    println!("  cat ./output/root_users_orders.jsonl | jq -r .orders_id | sort | uniq -c");

    Ok(())
}
