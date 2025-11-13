# JSON Melt Usage Guide

A practical guide to using the json-melt library for converting nested JSON into relational tables.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
json-melt = { path = "." }  # Use path for local development
anyhow = "1.0"              # For error handling
serde_json = "1.0"          # For JSON manipulation
```

## Basic Concepts

**What does "melting" mean?**

Melting takes nested JSON and splits it into separate tables (entities), like converting:

```
User { posts: [Post, Post] }
```

Into:

```
users.jsonl:     {id: 1, name: "Alice"}
users_posts.jsonl: {id: 10, title: "Post 1", user_id: 1}
                   {id: 11, title: "Post 2", user_id: 1}
```

## Usage Patterns

### Pattern 1: Process a Single JSON Object

The simplest case - you have one JSON object and want to split it into relational tables.

```rust
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // Your JSON data
    let data = json!({
        "id": 1,
        "username": "alice",
        "email": "alice@example.com",
        "posts": [
            {
                "id": 100,
                "title": "My first post",
                "content": "Hello world!",
                "likes": 42
            },
            {
                "id": 101,
                "title": "Second post",
                "content": "More content",
                "likes": 15
            }
        ]
    });

    // Create a melter with default configuration
    let melter = JsonMelter::new(MeltConfig::default());

    // Extract entities
    let entities = melter.melt(data)?;

    // Print what we found
    println!("Found {} entities:", entities.len());
    for entity in &entities {
        println!("  - {}: {} fields",
            entity.entity_type,
            entity.data.len()
        );
    }

    // Write to .jsonl files
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    println!("\nFiles created:");
    println!("  root.jsonl        - Main user data");
    println!("  root_posts.jsonl  - Posts with user_id foreign key");

    Ok(())
}
```

**Output files:**

`root.jsonl`:
```json
{"id":1,"username":"alice","email":"alice@example.com"}
```

`root_posts.jsonl`:
```json
{"id":100,"title":"My first post","content":"Hello world!","likes":42,"posts_id":"1"}
{"id":101,"title":"Second post","content":"More content","likes":15,"posts_id":"1"}
```

Note the `posts_id` field - this is the foreign key back to the parent!

### Pattern 2: Process Multiple JSON Objects (JSON Lines)

When you have multiple JSON objects (like paginated API responses), process them one at a time.

```rust
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let melter = JsonMelter::new(MeltConfig::default());
    let mut writer = EntityWriter::new_file_writer("./output")?;

    // Simulate multiple API responses
    let responses = vec![
        json!({"id": 1, "name": "Alice", "posts": [{"id": 10, "title": "Post 1"}]}),
        json!({"id": 2, "name": "Bob", "posts": [{"id": 20, "title": "Post 2"}]}),
        json!({"id": 3, "name": "Carol", "posts": [{"id": 30, "title": "Post 3"}]}),
    ];

    // Process each response
    for response in responses {
        let entities = melter.melt(response)?;
        writer.write_entities(entities)?;
    }

    writer.flush()?;

    println!("Processed 3 API responses into relational tables");
    Ok(())
}
```

**Result:** All users go to `root.jsonl`, all posts go to `root_posts.jsonl`.

### Pattern 3: Stream from a JSON Lines File

Reading from a file with one JSON object per line.

```rust
use json_melt::{melt_json, EntityWriter, MeltConfig};
use std::fs::File;
use std::io::BufReader;

fn main() -> anyhow::Result<()> {
    // Open input file
    let file = File::open("input.jsonl")?;
    let reader = BufReader::new(file);

    // Create output writer
    let mut writer = EntityWriter::new_file_writer("./output")?;

    // Process the entire file
    melt_json(reader, &mut writer, MeltConfig::default())?;

    println!("Successfully processed input.jsonl");
    Ok(())
}
```

First, create a test file:
```bash
cat > input.jsonl << 'EOF'
{"user_id": 1, "name": "Alice", "orders": [{"id": 100, "total": 50.00}]}
{"user_id": 2, "name": "Bob", "orders": [{"id": 101, "total": 75.50}]}
EOF
```

### Pattern 4: Inspect Entities Before Writing

Sometimes you want to see what was extracted before writing to files.

```rust
use json_melt::{JsonMelter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let data = json!({
        "repository": {
            "id": 12345,
            "name": "awesome-project",
            "stars": 1500
        },
        "issues": [
            {
                "number": 1,
                "title": "Bug report",
                "comments": [
                    {"id": 10, "text": "I can reproduce"},
                    {"id": 11, "text": "Me too"}
                ]
            }
        ]
    });

    let melter = JsonMelter::new(MeltConfig::default());
    let entities = melter.melt(data)?;

    // Inspect each entity
    for entity in entities {
        println!("\n=== Entity: {} ===", entity.entity_type);

        if let Some(id) = &entity.id {
            println!("ID: {}", id.0);
        }

        if let Some(parent) = &entity.parent {
            println!("Parent: {} (id: {})", parent.entity_type, parent.id.0);
        }

        println!("Data: {}", serde_json::to_string_pretty(&entity.data)?);
    }

    Ok(())
}
```

### Pattern 5: Custom Configuration

Control how melting works with `MeltConfig`.

```rust
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let config = MeltConfig {
        max_depth: 5,                    // Only extract 5 levels deep
        fk_prefix: "".to_string(),       // No prefix on foreign keys
        id_prefix: "_fk".to_string(),    // Use "_fk" suffix (posts_fk instead of posts_id)
        separator: "__".to_string(),     // Use double underscore (root__posts)
        include_parent_ids: true,        // Include foreign keys (true by default)
        scalar_fields: vec![             // Never extract these fields
            "metadata".to_string(),
            "config".to_string(),
        ],
    };

    let melter = JsonMelter::new(config);

    let data = json!({
        "id": 1,
        "name": "Test",
        "metadata": {"large": "object", "with": "many", "fields": "here"},  // Kept inline
        "items": [{"id": 10, "name": "Item 1"}]  // Extracted
    });

    let entities = melter.melt(data)?;
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    // Entity types will be: root, root__items
    // Foreign keys will be: items_fk

    Ok(())
}
```

### Pattern 6: Single Output Stream (All Entities in One File)

Instead of multiple files, put everything in one stream with metadata.

```rust
use json_melt::{JsonMelter, SingleWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let melter = JsonMelter::new(MeltConfig::default());

    let data = json!({
        "id": 1,
        "name": "Alice",
        "posts": [{"id": 10, "title": "Post"}]
    });

    let entities = melter.melt(data)?;

    // Write to stdout (or any writer)
    let mut writer = SingleWriter::new(std::io::stdout());
    writer.write_entities(entities)?;

    Ok(())
}
```

**Output** (each entity includes metadata):
```json
{"id":1,"name":"Alice","_entity_type":"root","_entity_id":"1"}
{"id":10,"title":"Post","posts_id":"1","_entity_type":"root_posts","_entity_id":"10","_parent_type":"root","_parent_id":"1"}
```

## Real-World Example: Processing a GitHub API Response

```rust
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // Simulated GitHub API response
    let github_response = json!({
        "id": 12345,
        "name": "rust-lang/rust",
        "description": "The Rust Programming Language",
        "stargazers_count": 80000,
        "open_issues": [
            {
                "number": 42,
                "title": "Compiler bug",
                "state": "open",
                "assignees": [
                    {"id": 1, "login": "developer1"},
                    {"id": 2, "login": "developer2"}
                ],
                "labels": [
                    {"name": "bug", "color": "red"},
                    {"name": "P-high", "color": "orange"}
                ]
            },
            {
                "number": 43,
                "title": "Feature request",
                "state": "open",
                "assignees": [
                    {"id": 3, "login": "developer3"}
                ],
                "labels": [
                    {"name": "enhancement", "color": "green"}
                ]
            }
        ]
    });

    let melter = JsonMelter::new(MeltConfig::default());
    let entities = melter.melt(github_response)?;

    println!("Extracted {} entities:", entities.len());
    for entity in &entities {
        println!("  - {}", entity.entity_type);
    }

    let mut writer = EntityWriter::new_file_writer("./github_data")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    println!("\nCreated relational tables:");
    println!("  root.jsonl                      - Repository info");
    println!("  root_open_issues.jsonl          - All issues");
    println!("  root_open_issues_assignees.jsonl - Issue assignees");
    println!("  root_open_issues_labels.jsonl    - Issue labels");

    Ok(())
}
```

Now you can query these files separately:
- Find all high-priority bugs: `grep '"P-high"' root_open_issues_labels.jsonl`
- Count issues per developer: Process `root_open_issues_assignees.jsonl`
- Get repository stats: Read `root.jsonl`

## Understanding Entity Types

Entity type names are derived from the JSON path:

```rust
json!({
    "user": {              // → root_user
        "posts": [         // → root_user_posts
            {
                "comments": [  // → root_user_posts_comments
                    { ... }
                ]
            }
        ]
    }
})
```

## Understanding Foreign Keys

Foreign keys link child entities to parents:

```
Parent: {"id": 1, "name": "Alice"}
Child:  {"id": 10, "title": "Post", "posts_id": "1"}
                                     ^^^^^^^^^^^^^^^^
                                     Foreign key to parent
```

The foreign key name is: `{field_name}_id`
- Array field `posts` → foreign key `posts_id`
- Array field `comments` → foreign key `comments_id`

## Common Patterns

### Pattern: Process API pagination

```rust
async fn process_all_pages() -> anyhow::Result<()> {
    let melter = JsonMelter::new(MeltConfig::default());
    let mut writer = EntityWriter::new_file_writer("./api_data")?;

    let mut page = 1;
    loop {
        let response = fetch_page(page).await?;  // Your API call

        if response["items"].as_array().map_or(true, |a| a.is_empty()) {
            break;  // No more data
        }

        let entities = melter.melt(response)?;
        writer.write_entities(entities)?;

        page += 1;
    }

    writer.flush()?;
    Ok(())
}
```

### Pattern: Convert to CSV (using another tool)

After generating `.jsonl` files, convert to CSV:

```bash
# Using jq
cat root_posts.jsonl | jq -r '[.id, .title, .posts_id] | @csv' > posts.csv

# Using Miller
mlr --ijson --ocsv cat root_posts.jsonl > posts.csv
```

### Pattern: Load into SQLite

```bash
# Install sqlite-utils
pip install sqlite-utils

# Import each entity type
sqlite-utils insert data.db root root.jsonl --nl
sqlite-utils insert data.db posts root_posts.jsonl --nl
sqlite-utils insert data.db comments root_posts_comments.jsonl --nl

# Query with SQL
sqlite3 data.db "
  SELECT p.title, COUNT(c.id) as comment_count
  FROM posts p
  LEFT JOIN comments c ON c.posts_id = p.id
  GROUP BY p.id
"
```

## Tips and Best Practices

1. **Start with default config**: Use `MeltConfig::default()` first, customize later

2. **Inspect before committing**: Use the inspection pattern to see what entities are created

3. **Use meaningful output directories**:
   ```rust
   EntityWriter::new_file_writer("./output/2024-01-15")?
   ```

4. **Handle missing IDs**: The library generates IDs like `_gen_1`, `_gen_2` when objects lack an `id` field

5. **Scalar arrays**: Arrays of strings/numbers create entities with `value` and `_idx` fields
   ```json
   ["rust", "json"] → {"value": "rust", "_idx": 0}, {"value": "json", "_idx": 1}
   ```

6. **Large objects kept inline**: Small objects without IDs are kept in the parent entity

## Troubleshooting

### Problem: Too many entity types

**Symptom**: Deeply nested JSON creates many tables

**Solution**: Reduce `max_depth` in config
```rust
let config = MeltConfig {
    max_depth: 3,  // Only extract 3 levels
    ..Default::default()
};
```

### Problem: Want to keep some nested data together

**Symptom**: Don't want certain fields extracted

**Solution**: Add to `scalar_fields`
```rust
let config = MeltConfig {
    scalar_fields: vec!["metadata".to_string(), "config".to_string()],
    ..Default::default()
};
```

### Problem: Foreign key names conflict

**Symptom**: Your data already has `*_id` fields

**Solution**: Change the `id_prefix`
```rust
let config = MeltConfig {
    id_prefix: "_fkey".to_string(),  // posts_fkey instead of posts_id
    ..Default::default()
};
```

## Next Steps

1. Run the example: `cargo run --example api_example`
2. Try with your own JSON data
3. Experiment with configuration options
4. Load the output into your database or analysis tool

## Getting Help

- Check the [README.md](README.md) for API reference
- See [tidy-data-principles.md](tidy-data-principles.md) for background theory
- Look at [examples/api_example.rs](examples/api_example.rs) for complete code
