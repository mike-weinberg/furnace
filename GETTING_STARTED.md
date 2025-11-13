# Getting Started with JSON Melt

A 5-minute guide to start using json-melt.

## What is JSON Melt?

Converts nested JSON → flat relational tables (like a database).

**Before:**
```json
{"user": "Alice", "posts": [{"title": "Post 1"}, {"title": "Post 2"}]}
```

**After:**
- `users.jsonl`: `{"user": "Alice"}`
- `posts.jsonl`: `{"title": "Post 1", "user_id": "1"}`, `{"title": "Post 2", "user_id": "1"}`

## Quick Start (30 seconds)

### 1. Run the Example

```bash
cargo run --example quickstart
```

This will:
- Process sample JSON
- Create `.jsonl` files
- Show you the results

### 2. Look at the Output

```bash
cat root.jsonl           # Main user data
cat root_posts.jsonl     # Posts with foreign keys
cat root_posts_tags.jsonl # Tags linked to posts
```

### 3. Understand the Pattern

Notice how:
- Each table is a separate file
- Child tables have `*_id` foreign keys
- Arrays are split into their own tables

## Your First Code (3 minutes)

Create `src/main.rs`:

```rust
use json_melt::{JsonMelter, EntityWriter, MeltConfig};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // 1. Your JSON data
    let data = json!({
        "id": 1,
        "name": "Alice",
        "posts": [
            {"id": 10, "title": "Hello World"}
        ]
    });

    // 2. Create melter
    let melter = JsonMelter::new(MeltConfig::default());

    // 3. Melt into entities
    let entities = melter.melt(data)?;

    // 4. Write to files
    let mut writer = EntityWriter::new_file_writer(".")?;
    writer.write_entities(entities)?;
    writer.flush()?;

    println!("✓ Created root.jsonl and root_posts.jsonl");
    Ok(())
}
```

Run it:
```bash
cargo run
```

## Common Patterns

### Pattern 1: Process API Response

```rust
// Your API client returns JSON
let response = api_client.get("/users/1").await?;

// Melt it
let melter = JsonMelter::new(MeltConfig::default());
let entities = melter.melt(response)?;

// Save it
let mut writer = EntityWriter::new_file_writer("./data")?;
writer.write_entities(entities)?;
```

### Pattern 2: Process Multiple Pages

```rust
let melter = JsonMelter::new(MeltConfig::default());
let mut writer = EntityWriter::new_file_writer("./data")?;

// Process each page
for page in 1..=5 {
    let response = api_client.get(&format!("/users?page={}", page)).await?;
    let entities = melter.melt(response)?;
    writer.write_entities(entities)?;  // All pages accumulate in same files!
}

writer.flush()?;
```

### Pattern 3: Read JSON Lines File

```rust
use std::fs::File;
use std::io::BufReader;

let file = File::open("input.jsonl")?;
let reader = BufReader::new(file);
let mut writer = EntityWriter::new_file_writer("./output")?;

json_melt::melt_json(reader, &mut writer, MeltConfig::default())?;
```

## Understanding the Output

### File Names

Files are named by JSON path:

```
your_json.users              → root_users.jsonl
your_json.users.posts        → root_users_posts.jsonl
your_json.users.posts.tags   → root_users_posts_tags.jsonl
```

### Foreign Keys

Child tables link to parents:

```json
// root_users.jsonl
{"id": 1, "name": "Alice"}

// root_users_posts.jsonl
{"id": 10, "title": "Post", "posts_id": "1"}  ← Links to user 1
                             ^^^^^^^^^^^^^
```

### Arrays of Scalars

```json
{"tags": ["rust", "json"]}
```

Becomes:

```json
// root_tags.jsonl
{"value": "rust", "_idx": 0, "tags_id": "..."}
{"value": "json", "_idx": 1, "tags_id": "..."}
```

## Next Steps

### Learn More

1. **[HOW_IT_WORKS.md](HOW_IT_WORKS.md)** - Visual diagrams and detailed explanation
2. **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Complete reference with all patterns
3. **[README.md](README.md)** - API documentation

### Try Examples

```bash
# Simple example
cargo run --example quickstart

# Complex API responses
cargo run --example api_example

# Multiple pages
cargo run --example api_pagination
```

### Customize Behavior

```rust
let config = MeltConfig {
    max_depth: 5,                    // Only extract 5 levels deep
    id_prefix: "_fk".to_string(),    // Use _fk instead of _id
    separator: "__".to_string(),     // root__users instead of root_users
    scalar_fields: vec![              // Never extract these fields
        "metadata".to_string(),
    ],
    ..Default::default()
};

let melter = JsonMelter::new(config);
```

## Real-World Workflow

### 1. Process API Data

```bash
# Run your processor
cargo run --release

# Output: root.jsonl, root_users.jsonl, root_posts.jsonl
```

### 2. Query with jq

```bash
# Find users from USA
cat root_users.jsonl | jq 'select(.country == "USA")'

# Count posts per user
cat root_posts.jsonl | jq -r .posts_id | sort | uniq -c
```

### 3. Load into SQLite

```bash
# Install sqlite-utils
pip install sqlite-utils

# Import
sqlite-utils insert data.db users root_users.jsonl --nl
sqlite-utils insert data.db posts root_posts.jsonl --nl

# Query
sqlite3 data.db "
  SELECT u.name, COUNT(p.id) as post_count
  FROM users u
  LEFT JOIN posts p ON p.posts_id = u.id
  GROUP BY u.id
"
```

### 4. Analyze with Python

```python
import pandas as pd

users = pd.read_json('root_users.jsonl', lines=True)
posts = pd.read_json('root_posts.jsonl', lines=True)

merged = posts.merge(users, left_on='posts_id', right_on='id')
print(merged.groupby('name')['title'].count())
```

## Tips

1. **Start simple**: Use default config first
2. **Inspect output**: Run quickstart to see what files are created
3. **One concept at a time**: Process one JSON object, understand the output, then scale up
4. **Use jq**: Great for exploring .jsonl files: `cat file.jsonl | jq`
5. **Check foreign keys**: Look for `*_id` fields to understand relationships

## Common Questions

**Q: Where do files go?**
A: In the directory you specify: `EntityWriter::new_file_writer("./output")?`

**Q: Can I change file names?**
A: File names come from entity types. Change the `separator` in config to customize.

**Q: What if objects don't have IDs?**
A: The library generates them: `_gen_1`, `_gen_2`, etc.

**Q: How do I process huge files?**
A: Use the streaming API - it processes line-by-line without loading everything into memory.

**Q: Can I output to stdout?**
A: Yes! Use `SingleWriter` instead of `EntityWriter`.

## Getting Help

- **Examples**: See `examples/` directory
- **Full docs**: Read [USAGE_GUIDE.md](USAGE_GUIDE.md)
- **How it works**: See [HOW_IT_WORKS.md](HOW_IT_WORKS.md)
- **Background**: Read [tidy-data-principles.md](tidy-data-principles.md)

## You're Ready!

Try it with your own JSON:

```rust
let my_json = json!({
    // Your data here
});

let melter = JsonMelter::new(MeltConfig::default());
let entities = melter.melt(my_json)?;
let mut writer = EntityWriter::new_file_writer(".")?;
writer.write_entities(entities)?;
writer.flush()?;
```

Then explore the `.jsonl` files!
