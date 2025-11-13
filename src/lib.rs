use anyhow::{Context, Result};
use serde_json::Value;
use std::io::BufRead;

pub mod types;
pub mod extractor;
pub mod writer;
pub mod schema_inference;
pub mod schema_builder;

pub use types::{Entity, EntityId, MeltConfig};
pub use extractor::JsonMelter;
pub use writer::{EntityWriter, SingleWriter};
pub use schema_inference::infer_schema;
pub use schema_builder::{SchemaBuilder, infer_schema_streaming};

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
