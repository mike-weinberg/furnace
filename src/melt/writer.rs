use crate::melt::types::Entity;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// Writes entities to multiple JSON Lines files, one per entity type
pub struct EntityWriter<W: Write> {
    writers: HashMap<String, W>,
}

impl EntityWriter<std::fs::File> {
    /// Create a new EntityWriter that writes to files in a directory
    pub fn new_file_writer<P: AsRef<Path>>(output_dir: P) -> Result<Self> {
        std::fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;

        Ok(EntityWriter {
            writers: HashMap::new(),
        })
    }

    /// Write entities to their respective files
    pub fn write_entities(&mut self, entities: Vec<Entity>) -> Result<()> {
        for entity in entities {
            // Ensure writer exists
            let entity_type = entity.entity_type.clone();
            if !self.writers.contains_key(&entity_type) {
                let filename = format!("{}.jsonl", entity_type);
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&filename)
                    .context(format!("Failed to open file: {}", filename))?;
                self.writers.insert(entity_type.clone(), file);
            }

            // Write the entity
            let writer = self.writers.get_mut(&entity_type).unwrap();
            let json = serde_json::to_string(&entity.data)
                .context("Failed to serialize entity")?;
            writeln!(writer, "{}", json)
                .context("Failed to write entity")?;
        }
        Ok(())
    }

    /// Flush all writers
    pub fn flush(&mut self) -> Result<()> {
        for writer in self.writers.values_mut() {
            writer.flush().context("Failed to flush writer")?;
        }
        Ok(())
    }
}

/// A simpler writer that writes all entities to a single output
pub struct SingleWriter<W: Write> {
    writer: W,
}

impl<W: Write> SingleWriter<W> {
    pub fn new(writer: W) -> Self {
        SingleWriter { writer }
    }

    pub fn write_entities(&mut self, entities: Vec<Entity>) -> Result<()> {
        for entity in entities {
            let mut data = entity.data.clone();

            // Add metadata
            data.insert(
                "_entity_type".to_string(),
                serde_json::Value::String(entity.entity_type.clone()),
            );

            if let Some(id) = entity.id {
                data.insert(
                    "_entity_id".to_string(),
                    serde_json::Value::String(id.0),
                );
            }

            if let Some(parent) = entity.parent {
                data.insert(
                    "_parent_type".to_string(),
                    serde_json::Value::String(parent.entity_type),
                );
                data.insert(
                    "_parent_id".to_string(),
                    serde_json::Value::String(parent.id.0),
                );
            }

            let json = serde_json::to_string(&data)
                .context("Failed to serialize entity")?;
            writeln!(self.writer, "{}", json)
                .context("Failed to write entity")?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().context("Failed to flush writer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_single_writer() {
        let mut buffer = Vec::new();
        let mut writer = SingleWriter::new(&mut buffer);

        let entity = Entity::new(
            "test".to_string(),
            serde_json::from_value(json!({"name": "Alice"})).unwrap(),
        );

        writer.write_entities(vec![entity]).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Alice"));
        assert!(output.contains("_entity_type"));
    }
}
