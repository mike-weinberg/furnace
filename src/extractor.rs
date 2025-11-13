use crate::types::{Entity, MeltConfig, ParentRef};
use anyhow::Result;
use serde_json::{Map, Value};

/// The core JSON melter that extracts relational entities from JSON
pub struct JsonMelter {
    config: MeltConfig,
    id_counter: std::cell::RefCell<u64>,
}

impl JsonMelter {
    pub fn new(config: MeltConfig) -> Self {
        JsonMelter {
            config,
            id_counter: std::cell::RefCell::new(0),
        }
    }

    /// Melt a JSON value into a collection of entities
    pub fn melt(&self, value: Value) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        self.extract_entity(value, "root", None, 0, &mut entities)?;
        Ok(entities)
    }

    /// Recursively extract entities from a JSON value
    fn extract_entity(
        &self,
        value: Value,
        entity_type: &str,
        parent: Option<ParentRef>,
        depth: usize,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        if depth > self.config.max_depth {
            return Ok(());
        }

        match value {
            Value::Object(obj) => {
                self.extract_from_object(obj, entity_type, parent, depth, entities)?;
            }
            Value::Array(arr) => {
                self.extract_from_array(arr, entity_type, parent, depth, entities)?;
            }
            _ => {
                // Scalar values at the root are just ignored or could be wrapped
            }
        }

        Ok(())
    }

    /// Extract entities from a JSON object
    fn extract_from_object(
        &self,
        obj: Map<String, Value>,
        entity_type: &str,
        parent: Option<ParentRef>,
        depth: usize,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        let mut entity_data = Map::new();
        let mut nested_entities: Vec<(String, Value)> = Vec::new();

        // Separate scalar fields from nested objects/arrays
        for (key, value) in obj.into_iter() {
            if self.is_scalar_field(&key) {
                // Always treat as scalar
                entity_data.insert(key, value);
            } else {
                match &value {
                    Value::Array(_) => {
                        // Always extract arrays as separate entities
                        nested_entities.push((key, value));
                    }
                    Value::Object(_) => {
                        // Check if object should be extracted
                        if Self::should_extract_object(&value) {
                            nested_entities.push((key, value));
                        } else {
                            // Small objects can be kept inline
                            entity_data.insert(key, value);
                        }
                    }
                    _ => {
                        entity_data.insert(key, value);
                    }
                }
            }
        }

        // Create the entity for this object
        let mut entity = Entity::new(entity_type.to_string(), entity_data);

        if let Some(p) = parent {
            entity = entity.with_parent(p);
        }

        // Get or generate an ID for this entity
        let entity_id = entity.get_or_generate_id(&mut self.id_counter.borrow_mut());

        // Add foreign key reference to the entity data if there's a parent
        if let Some(ref parent_ref) = entity.parent {
            if self.config.include_parent_ids {
                let fk_name = format!("{}{}", parent_ref.field_name, self.config.id_prefix);
                entity.data.insert(
                    fk_name,
                    Value::String(parent_ref.id.0.clone()),
                );
            }
        }

        entities.push(entity);

        // Process nested entities
        for (field_name, nested_value) in nested_entities {
            let nested_type = format!("{}{}{}", entity_type, self.config.separator, field_name);
            let parent_ref = ParentRef {
                entity_type: entity_type.to_string(),
                id: entity_id.clone(),
                field_name: field_name.clone(),
            };

            self.extract_entity(
                nested_value,
                &nested_type,
                Some(parent_ref),
                depth + 1,
                entities,
            )?;
        }

        Ok(())
    }

    /// Extract entities from a JSON array
    fn extract_from_array(
        &self,
        arr: Vec<Value>,
        entity_type: &str,
        parent: Option<ParentRef>,
        depth: usize,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        // Check if this is an array of objects (entity array)
        if Self::is_entity_array(&arr) {
            for item in arr.into_iter() {
                self.extract_entity(item, entity_type, parent.clone(), depth, entities)?;
            }
        } else {
            // Array of scalars - could create a separate entity type
            // For now, we'll create entities for each scalar with an index
            for (idx, item) in arr.into_iter().enumerate() {
                let mut data = Map::new();
                data.insert("value".to_string(), item);
                data.insert("_idx".to_string(), Value::Number(idx.into()));

                let mut entity = Entity::new(entity_type.to_string(), data);

                if let Some(ref p) = parent {
                    entity = entity.with_parent(p.clone());
                    if self.config.include_parent_ids {
                        let fk_name = format!("{}{}", p.field_name, self.config.id_prefix);
                        entity.data.insert(
                            fk_name,
                            Value::String(p.id.0.clone()),
                        );
                    }
                }

                entities.push(entity);
            }
        }

        Ok(())
    }

    /// Check if an array should be treated as an entity array
    fn is_entity_array(arr: &[Value]) -> bool {
        if arr.is_empty() {
            return false;
        }

        // If most elements are objects, treat as entity array
        let object_count = arr.iter().filter(|v| matches!(v, Value::Object(_))).count();
        object_count > arr.len() / 2
    }

    /// Check if an object should be extracted as a separate entity
    fn should_extract_object(value: &Value) -> bool {
        if let Value::Object(obj) = value {
            // Extract if it has an ID or has multiple fields
            obj.contains_key("id") || obj.len() > 2
        } else {
            false
        }
    }

    /// Check if a field should always be treated as scalar
    fn is_scalar_field(&self, field_name: &str) -> bool {
        self.config.scalar_fields.contains(&field_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_object() {
        let input = json!({
            "id": 1,
            "name": "Alice"
        });

        let melter = JsonMelter::new(MeltConfig::default());
        let entities = melter.melt(input).unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, "root");
        assert_eq!(entities[0].data.get("name").unwrap(), "Alice");
    }

    #[test]
    fn test_nested_array() {
        let input = json!({
            "id": 1,
            "name": "Alice",
            "posts": [
                {"id": 10, "title": "Post 1"},
                {"id": 11, "title": "Post 2"}
            ]
        });

        let melter = JsonMelter::new(MeltConfig::default());
        let entities = melter.melt(input).unwrap();

        // Should have root entity + 2 post entities
        assert_eq!(entities.len(), 3);

        // Check root entity
        assert_eq!(entities[0].entity_type, "root");
        assert_eq!(entities[0].data.get("name").unwrap(), "Alice");

        // Check post entities
        assert_eq!(entities[1].entity_type, "root_posts");
        assert_eq!(entities[2].entity_type, "root_posts");

        // Check foreign keys
        assert!(entities[1].data.contains_key("posts_id"));
    }

    #[test]
    fn test_scalar_array() {
        let input = json!({
            "id": 1,
            "tags": ["rust", "json", "data"]
        });

        let melter = JsonMelter::new(MeltConfig::default());
        let entities = melter.melt(input).unwrap();

        // Should have root entity + 3 tag entities
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[1].entity_type, "root_tags");
        assert_eq!(entities[1].data.get("value").unwrap(), "rust");
    }
}
