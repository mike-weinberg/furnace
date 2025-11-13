//! # Furnace - JSON Processing Toolkit
//!
//! A unified library for JSON melting (extracting nested JSON into relational tables)
//! and JSON Schema inference with format detection.
//!
//! ## Modules
//!
//! - **melt**: Extract nested JSON into flat, relational entities
//! - **schema**: Infer JSON Schemas from examples with format detection
//!
//! ## Quick Start
//!
//! ### JSON Melting
//!
//! ```rust
//! use furnace::melt::{JsonMelter, EntityWriter, MeltConfig};
//! use serde_json::json;
//!
//! # fn main() -> anyhow::Result<()> {
//! let data = json!({
//!     "id": 1,
//!     "name": "Alice",
//!     "posts": [
//!         {"id": 10, "title": "First Post"},
//!         {"id": 11, "title": "Second Post"}
//!     ]
//! });
//!
//! let config = MeltConfig::default();
//! let melter = JsonMelter::new(config);
//! let entities = melter.melt(data)?;
//!
//! // entities[0] = root entity (id, name)
//! // entities[1-2] = posts entities with foreign keys
//! # Ok(())
//! # }
//! ```
//!
//! ### Schema Inference
//!
//! ```rust
//! use furnace::schema::infer_schema_streaming;
//! use serde_json::json;
//!
//! let examples = vec![
//!     json!({"name": "Alice", "age": 30}),
//!     json!({"name": "Bob", "age": 25}),
//! ];
//!
//! let schema = infer_schema_streaming(&examples);
//! // schema includes type information, required fields, and format detection
//! ```

use anyhow::{Context, Result};
use serde_json::Value;
use std::io::BufRead;

pub mod melt;
pub mod schema;

// Re-export commonly used types for convenience
pub use melt::{Entity, EntityId, EntityWriter, JsonMelter, MeltConfig, MeltPlan, PlannedMelter, SingleWriter};
pub use schema::{SchemaBuilder, infer_schema, infer_schema_streaming};

/// Main entry point: melt a JSON stream into relational entities
pub fn melt_json<R: BufRead>(
    reader: R,
    writer: &mut EntityWriter<std::fs::File>,
    config: MeltConfig,
) -> Result<()> {
    let melter = JsonMelter::new(config);

    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        let value: Value = serde_json::from_str(&line)
            .context("Failed to parse JSON")?;

        let entities = melter.melt(value)?;
        writer.write_entities(entities)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_melting() {
        let input = json!({
            "id": 1,
            "name": "Alice",
            "posts": [
                {"id": 10, "title": "Post 1"},
                {"id": 11, "title": "Post 2"}
            ]
        });

        let config = MeltConfig::default();
        let melter = JsonMelter::new(config);
        let entities = melter.melt(input).unwrap();

        // Should have root entity and posts entity
        assert!(entities.len() >= 2);
    }
}
