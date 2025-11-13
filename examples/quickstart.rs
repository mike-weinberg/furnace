/// Quickstart example - the simplest possible usage
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    println!("=== JSON Melt Quick Start ===\n");

    // Step 1: Your JSON data
    let my_data = json!({
        "id": 1,
        "username": "alice",
        "email": "alice@example.com",
        "posts": [
            {
                "id": 100,
                "title": "My First Post",
                "content": "Hello, world!",
                "tags": ["intro", "welcome"]
            },
            {
                "id": 101,
                "title": "Second Post",
                "content": "More content here",
                "tags": ["update", "news"]
            }
        ]
    });

    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&my_data)?);

    // Step 2: Create a melter
    let melter = JsonMelter::new(MeltConfig::default());

    // Step 3: Melt the JSON into entities
    let entities = melter.melt(my_data)?;

    println!("Extracted {} entities:\n", entities.len());

    // Step 4: Look at what we got
    for (i, entity) in entities.iter().enumerate() {
        println!("Entity {}: {}", i + 1, entity.entity_type);
        println!("{}\n", serde_json::to_string_pretty(&entity.data)?);
    }

    // Step 5: Write to files
    println!("Writing to .jsonl files...");
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    println!("\n✓ Done! Created files:");
    println!("  • root.jsonl              - User data");
    println!("  • root_posts.jsonl        - Posts (with posts_id foreign key)");
    println!("  • root_posts_tags.jsonl   - Tags (with tags_id foreign key)");

    println!("\nTry these commands:");
    println!("  cat root.jsonl");
    println!("  cat root_posts.jsonl");
    println!("  cat root_posts_tags.jsonl");

    Ok(())
}
