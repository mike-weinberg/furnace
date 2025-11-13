# How JSON Melt Works

## The Problem

You have nested JSON from an API:

```json
{
  "id": 1,
  "name": "Alice",
  "posts": [
    {"id": 10, "title": "Post 1"},
    {"id": 11, "title": "Post 2"}
  ]
}
```

This is hard to query because:
- Posts are nested inside the user
- Can't filter/sort posts independently
- Can't join with other data easily
- Not suitable for databases or data analysis tools

## The Solution

JSON Melt **splits** nested structures into separate tables:

```
User Table (root.jsonl):
{"id": 1, "name": "Alice"}

Posts Table (root_posts.jsonl):
{"id": 10, "title": "Post 1", "posts_id": "1"}  ← Foreign key to user
{"id": 11, "title": "Post 2", "posts_id": "1"}  ← Foreign key to user
```

Now you can:
- Query posts independently
- Join posts back to users using `posts_id`
- Load into SQLite, PostgreSQL, etc.
- Analyze with pandas, R, or any tabular tool

## Visual Example

### Before (Nested JSON):

```
┌─────────────────────────────────┐
│ User                            │
│ ┌─────────────────────────────┐ │
│ │ id: 1                       │ │
│ │ name: "Alice"               │ │
│ │ posts: [                    │ │
│ │   ┌───────────────────────┐ │ │
│ │   │ id: 10               │ │ │
│ │   │ title: "Post 1"      │ │ │
│ │   └───────────────────────┘ │ │
│ │   ┌───────────────────────┐ │ │
│ │   │ id: 11               │ │ │
│ │   │ title: "Post 2"      │ │ │
│ │   └───────────────────────┘ │ │
│ │ ]                           │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
```

### After (Relational Tables):

```
┌─────────────────────┐         ┌──────────────────────────────┐
│ root.jsonl          │         │ root_posts.jsonl             │
├─────────────────────┤         ├──────────────────────────────┤
│ id: 1               │◄────────│ id: 10                       │
│ name: "Alice"       │         │ title: "Post 1"              │
└─────────────────────┘         │ posts_id: "1"  ← Foreign Key │
                                ├──────────────────────────────┤
                                │ id: 11                       │
                                │ title: "Post 2"              │
                                │ posts_id: "1"  ← Foreign Key │
                                └──────────────────────────────┘
```

## Step-by-Step Process

### 1. Input JSON

```json
{
  "user_id": 1,
  "username": "alice",
  "posts": [
    {
      "id": 100,
      "title": "Hello",
      "tags": ["intro", "welcome"]
    }
  ]
}
```

### 2. Entity Detection

The library identifies:
- **Root entity**: Main object
- **posts array**: Should be extracted (array of objects)
- **tags array**: Should be extracted (array of scalars)

### 3. Entity Extraction

```
Root Entity:
  Type: "root"
  Data: {"user_id": 1, "username": "alice"}
  ID: _gen_1

Posts Entity:
  Type: "root_posts"
  Data: {"id": 100, "title": "Hello"}
  ID: 100
  Parent: root (_gen_1)
  Foreign Key: posts_id = "_gen_1"

Tag Entity 1:
  Type: "root_posts_tags"
  Data: {"value": "intro", "_idx": 0}
  Parent: root_posts (100)
  Foreign Key: tags_id = "100"

Tag Entity 2:
  Type: "root_posts_tags"
  Data: {"value": "welcome", "_idx": 1}
  Parent: root_posts (100)
  Foreign Key: tags_id = "100"
```

### 4. Output Files

**root.jsonl:**
```json
{"user_id":1,"username":"alice"}
```

**root_posts.jsonl:**
```json
{"id":100,"title":"Hello","posts_id":"_gen_1"}
```

**root_posts_tags.jsonl:**
```json
{"value":"intro","_idx":0,"tags_id":"100"}
{"value":"welcome","_idx":1,"tags_id":"100"}
```

## Complex Example: GitHub Issues

### Input:

```json
{
  "repo": "rust-lang/rust",
  "issues": [
    {
      "number": 42,
      "title": "Bug report",
      "labels": [
        {"name": "bug", "color": "red"},
        {"name": "P-high", "color": "orange"}
      ],
      "comments": [
        {"id": 1, "text": "I can reproduce"},
        {"id": 2, "text": "Me too"}
      ]
    }
  ]
}
```

### Entity Hierarchy:

```
root
├── root_issues
│   ├── root_issues_labels
│   └── root_issues_comments
```

### Output Files:

**root.jsonl:**
```json
{"repo":"rust-lang/rust"}
```

**root_issues.jsonl:**
```json
{"number":42,"title":"Bug report","issues_id":"_gen_1"}
```

**root_issues_labels.jsonl:**
```json
{"name":"bug","color":"red","labels_id":"_gen_2"}
{"name":"P-high","color":"orange","labels_id":"_gen_2"}
```

**root_issues_comments.jsonl:**
```json
{"id":1,"text":"I can reproduce","comments_id":"_gen_2"}
{"id":2,"text":"Me too","comments_id":"_gen_2"}
```

### Relationship Diagram:

```
root (_gen_1)
    │
    │ issues_id = "_gen_1"
    ↓
root_issues (_gen_2)
    ├─ labels_id = "_gen_2"  →  root_issues_labels
    └─ comments_id = "_gen_2" →  root_issues_comments
```

## Key Concepts

### 1. Entity Types (Table Names)

Named by JSON path with underscores:

| JSON Path | Entity Type |
|-----------|-------------|
| Root | `root` |
| `users` | `root_users` |
| `users[].posts` | `root_users_posts` |
| `users[].posts[].tags` | `root_users_posts_tags` |

### 2. Foreign Keys

Pattern: `{field_name}_id`

- Parent field `posts` → child has `posts_id`
- Parent field `comments` → child has `comments_id`
- Parent field `items` → child has `items_id`

### 3. ID Generation

- Uses existing `id` field if present
- Generates `_gen_N` if missing (N = counter)
- Ensures every entity is identifiable

### 4. Array Handling

**Arrays of objects:**
```json
"posts": [{"id": 1, "title": "Post"}]
```
→ Each object becomes a row in `root_posts.jsonl`

**Arrays of scalars:**
```json
"tags": ["rust", "json"]
```
→ Each value becomes a row with `value` and `_idx` fields

### 5. Small Objects

Small objects (< 3 fields, no ID) stay inline:

```json
{
  "name": "Alice",
  "address": {"city": "NYC", "zip": "10001"}  ← Kept inline (small)
}
```

```json
{"name":"Alice","address":{"city":"NYC","zip":"10001"}}
```

Large objects (has ID or many fields) are extracted:

```json
{
  "name": "Alice",
  "address": {
    "id": 42,
    "street": "123 Main St",
    "city": "NYC",
    "state": "NY",
    "zip": "10001"
  }
}
```

→ Creates `root_address.jsonl`

## Use Cases

### 1. API Pagination

Process multiple API pages into a single dataset:

```rust
for page in 1..10 {
    let response = fetch_api_page(page).await?;
    let entities = melter.melt(response)?;
    writer.write_entities(entities)?;
}
```

All pages accumulate in the same files!

### 2. Log Analysis

Convert JSON logs to queryable tables:

```bash
cat application.log | cargo run --example stream_processor
# Creates: root.jsonl, root_errors.jsonl, root_requests.jsonl
```

### 3. Database Import

Load into SQLite/PostgreSQL:

```bash
# Generate .jsonl files
cargo run --example api_example

# Import to SQLite
sqlite-utils insert db.sqlite users root_users.jsonl --nl
sqlite-utils insert db.sqlite posts root_users_posts.jsonl --nl

# Query with SQL!
sqlite3 db.sqlite "SELECT * FROM users JOIN posts ON posts.posts_id = users.id"
```

### 4. Data Analysis

Process with pandas, R, etc:

```python
import pandas as pd

users = pd.read_json('root_users.jsonl', lines=True)
posts = pd.read_json('root_users_posts.jsonl', lines=True)

# Join them
result = posts.merge(users, left_on='posts_id', right_on='id')
```

## Why This Matters

Following **Tidy Data** principles:

1. ✓ Each variable is a column
2. ✓ Each observation is a row
3. ✓ Each type of observational unit is a table

This makes data:
- **Queryable**: Use SQL, grep, jq, etc.
- **Joinable**: Combine with other datasets
- **Analyzable**: Works with standard tools
- **Portable**: CSV/Parquet/database-ready

## Performance

- **Streaming**: Processes one JSON object at a time
- **Low memory**: Doesn't load entire dataset
- **Fast**: Rust performance with zero-copy where possible
- **Scalable**: Handle millions of records

## Next Steps

1. **Try it**: `cargo run --example quickstart`
2. **Read**: [USAGE_GUIDE.md](USAGE_GUIDE.md)
3. **Experiment**: Use your own JSON data
4. **Customize**: Adjust `MeltConfig` for your needs
