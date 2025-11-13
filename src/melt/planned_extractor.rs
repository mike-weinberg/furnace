//! Schema-guided extraction using pre-computed plans
//!
//! This module provides PlannedMelter, which uses a MeltPlan to extract
//! entities without runtime decision-making.

use crate::melt::plan::{ArrayType, FieldRule, MeltPlan};
use crate::melt::types::{Entity, EntityId, ParentRef};
use anyhow::Result;
use serde_json::{Map, Value};

/// A JSON melter that uses a pre-computed plan for optimized extraction
pub struct PlannedMelter {
    plan: MeltPlan,
    id_counter: std::cell::RefCell<u64>,
}

impl PlannedMelter {
    /// Create a new planned melter with a pre-computed plan
    pub fn new(plan: MeltPlan) -> Self {
        PlannedMelter {
            plan,
            id_counter: std::cell::RefCell::new(0),
        }
    }

    /// Create a planned melter by analyzing sample data
    ///
    /// # Arguments
    /// * `examples` - Sample JSON values to analyze (typically 10-100 records)
    ///
    /// # Example
    /// ```rust
    /// use furnace::melt::{PlannedMelter, MeltConfig};
    /// use serde_json::json;
    ///
    /// let samples = vec![
    ///     json!({"id": 1, "name": "Alice", "posts": [{"id": 10, "title": "Post"}]}),
    ///     json!({"id": 2, "name": "Bob", "posts": [{"id": 20, "title": "Post"}]}),
    /// ];
    ///
    /// let melter = PlannedMelter::from_examples(&samples, MeltConfig::default()).unwrap();
    ///
    /// // Now use melter.melt() on thousands of similar records with optimized performance
    /// ```
    pub fn from_examples(examples: &[Value], config: crate::melt::types::MeltConfig) -> Result<Self> {
        let plan = MeltPlan::from_examples(examples, config)?;
        Ok(Self::new(plan))
    }

    /// Melt a JSON value using the pre-computed plan
    pub fn melt(&self, value: Value) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        self.extract_with_plan(value, "root", None, &mut entities)?;
        Ok(entities)
    }

    /// Extract entities using the pre-computed plan
    fn extract_with_plan(
        &self,
        value: Value,
        entity_type: &str,
        parent: Option<ParentRef>,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        // Get the plan for this entity type
        let Some(entity_plan) = self.plan.get_plan(entity_type) else {
            // No plan - fall back to treating as scalar
            return Ok(());
        };

        match value {
            Value::Object(obj) => {
                self.extract_object_with_plan(obj, entity_plan, entity_type, parent, entities)?;
            }
            Value::Array(arr) => {
                self.extract_array_with_plan(arr, entity_plan, entity_type, parent, entities)?;
            }
            _ => {
                // Scalar at root - ignore
            }
        }

        Ok(())
    }

    /// Extract an object using the plan (no runtime decisions!)
    fn extract_object_with_plan(
        &self,
        obj: Map<String, Value>,
        entity_plan: &crate::melt::plan::EntityPlan,
        entity_type: &str,
        parent: Option<ParentRef>,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        let mut entity_data = Map::new();
        let mut nested_extractions: Vec<(String, Value, FieldRule)> = Vec::new();

        // Separate fields according to the plan (pre-computed, no conditionals!)
        for (key, value) in obj.into_iter() {
            if entity_plan.scalar_fields.contains(&key) {
                // Plan says: keep as scalar
                entity_data.insert(key, value);
            } else if let Some(rule) = entity_plan.nested_fields.get(&key) {
                // Plan says: extract as nested entity
                nested_extractions.push((key, value, rule.clone()));
            } else {
                // Not in plan - default to scalar
                entity_data.insert(key, value);
            }
        }

        // Create the entity
        let mut entity = Entity::new(entity_type.to_string(), entity_data);

        if let Some(p) = parent {
            entity = entity.with_parent(p);
        }

        // Get or generate ID
        let entity_id = entity.get_or_generate_id(&mut self.id_counter.borrow_mut());

        // Add foreign key if needed
        if let Some(ref parent_ref) = entity.parent {
            if self.plan.config.include_parent_ids {
                let fk_name = format!("{}{}", parent_ref.field_name, self.plan.config.id_prefix);
                entity.data.insert(
                    fk_name,
                    Value::String(parent_ref.id.0.clone()),
                );
            }
        }

        entities.push(entity);

        // Process nested entities according to plan
        for (field_name, nested_value, rule) in nested_extractions {
            match rule {
                FieldRule::ArrayEntity { entity_type: nested_type, element_type } => {
                    let parent_ref = ParentRef {
                        entity_type: entity_type.to_string(),
                        id: entity_id.clone(),
                        field_name: field_name.clone(),
                    };

                    self.extract_array_elements(
                        nested_value,
                        &nested_type,
                        element_type,
                        Some(parent_ref),
                        entities,
                    )?;
                }
                FieldRule::NestedEntity { entity_type: nested_type } => {
                    let parent_ref = ParentRef {
                        entity_type: entity_type.to_string(),
                        id: entity_id.clone(),
                        field_name: field_name.clone(),
                    };

                    self.extract_with_plan(nested_value, &nested_type, Some(parent_ref), entities)?;
                }
                FieldRule::Scalar => {
                    // Shouldn't reach here, but handle gracefully
                }
            }
        }

        Ok(())
    }

    /// Extract an array using the plan
    fn extract_array_with_plan(
        &self,
        arr: Vec<Value>,
        _entity_plan: &crate::melt::plan::EntityPlan,
        entity_type: &str,
        parent: Option<ParentRef>,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        // When called at root level with array, extract elements directly
        for item in arr.into_iter() {
            self.extract_with_plan(item, entity_type, parent.clone(), entities)?;
        }

        Ok(())
    }

    /// Extract array elements (objects or scalars)
    fn extract_array_elements(
        &self,
        value: Value,
        entity_type: &str,
        element_type: ArrayType,
        parent: Option<ParentRef>,
        entities: &mut Vec<Entity>,
    ) -> Result<()> {
        let Value::Array(arr) = value else {
            return Ok(());
        };

        match element_type {
            ArrayType::Objects => {
                // Extract each object as an entity
                for item in arr.into_iter() {
                    self.extract_with_plan(item, entity_type, parent.clone(), entities)?;
                }
            }
            ArrayType::Scalars => {
                // Create entities for scalar values with index
                for (idx, item) in arr.into_iter().enumerate() {
                    let mut data = Map::new();
                    data.insert("value".to_string(), item);
                    data.insert("_idx".to_string(), Value::Number(idx.into()));

                    let mut entity = Entity::new(entity_type.to_string(), data);

                    if let Some(ref p) = parent {
                        entity = entity.with_parent(p.clone());
                        if self.plan.config.include_parent_ids {
                            let fk_name = format!("{}{}", p.field_name, self.plan.config.id_prefix);
                            entity.data.insert(
                                fk_name,
                                Value::String(p.id.0.clone()),
                            );
                        }
                    }

                    entities.push(entity);
                }
            }
            ArrayType::Empty => {
                // Empty array - nothing to extract
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::melt::types::MeltConfig;
    use serde_json::json;

    #[test]
    fn test_planned_simple_melt() {
        let samples = vec![
            json!({"id": 1, "name": "Alice"}),
            json!({"id": 2, "name": "Bob"}),
        ];

        let melter = PlannedMelter::from_examples(&samples, MeltConfig::default()).unwrap();

        let data = json!({"id": 3, "name": "Charlie"});
        let entities = melter.melt(data).unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, "root");
        assert_eq!(entities[0].data.get("name").unwrap(), "Charlie");
    }

    #[test]
    fn test_planned_nested_array() {
        let samples = vec![
            json!({
                "id": 1,
                "name": "Alice",
                "posts": [
                    {"id": 10, "title": "Post 1"},
                    {"id": 11, "title": "Post 2"}
                ]
            }),
        ];

        let melter = PlannedMelter::from_examples(&samples, MeltConfig::default()).unwrap();

        let data = json!({
            "id": 2,
            "name": "Bob",
            "posts": [
                {"id": 20, "title": "Bob's Post 1"},
                {"id": 21, "title": "Bob's Post 2"}
            ]
        });

        let entities = melter.melt(data).unwrap();

        // Should have root + 2 posts
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].entity_type, "root");
        assert_eq!(entities[1].entity_type, "root_posts");
        assert_eq!(entities[2].entity_type, "root_posts");

        // Check foreign keys
        assert!(entities[1].data.contains_key("posts_id"));
        assert_eq!(entities[1].data.get("posts_id").unwrap(), "2");
    }

    #[test]
    fn test_planned_scalar_array() {
        let samples = vec![
            json!({
                "id": 1,
                "tags": ["rust", "json"]
            }),
        ];

        let melter = PlannedMelter::from_examples(&samples, MeltConfig::default()).unwrap();

        let data = json!({
            "id": 2,
            "tags": ["performance", "optimization"]
        });

        let entities = melter.melt(data).unwrap();

        // Should have root + 2 tags
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[1].entity_type, "root_tags");
        assert_eq!(entities[1].data.get("value").unwrap(), "performance");
        assert_eq!(entities[1].data.get("_idx").unwrap(), 0);
    }
}
