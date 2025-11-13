//! Schema-guided extraction plans for optimized JSON melting
//!
//! This module provides pre-computed extraction plans based on inferred schemas,
//! eliminating runtime decision-making for homogeneous data streams.

use crate::melt::types::MeltConfig;
use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Type of array elements
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayType {
    /// Array of objects that should be extracted as entities
    Objects,
    /// Array of scalar values
    Scalars,
    /// Empty array (type unknown)
    Empty,
}

/// Extraction rule for a specific field
#[derive(Debug, Clone)]
pub enum FieldRule {
    /// Keep as scalar in parent entity
    Scalar,
    /// Extract as nested entity with given type name
    NestedEntity { entity_type: String },
    /// Extract array elements as entities with given type name
    ArrayEntity { entity_type: String, element_type: ArrayType },
}

/// Pre-computed extraction plan for an entity type
#[derive(Debug, Clone)]
pub struct EntityPlan {
    /// Entity type name (e.g., "root", "root_posts")
    pub entity_type: String,

    /// Fields that should be kept as scalars
    pub scalar_fields: HashSet<String>,

    /// Fields that should be extracted as nested entities
    pub nested_fields: HashMap<String, FieldRule>,

    /// Whether this entity type has an "id" field
    pub has_id_field: bool,
}

/// Complete melting plan derived from schema analysis
#[derive(Debug, Clone)]
pub struct MeltPlan {
    /// Plans for each entity type we'll encounter
    pub entity_plans: HashMap<String, EntityPlan>,

    /// Configuration for the melting process
    pub config: MeltConfig,
}

impl MeltPlan {
    /// Generate a melt plan by analyzing example data with schema inference
    ///
    /// # Arguments
    /// * `examples` - Sample JSON values to analyze (typically 10-100 records)
    /// * `config` - Melting configuration
    ///
    /// # Returns
    /// A pre-computed plan for efficient extraction
    pub fn from_examples(examples: &[Value], config: MeltConfig) -> Result<Self> {
        // Use schema inference to understand the structure
        let schema = crate::schema::infer_schema_streaming(examples);
        Self::from_schema(&schema, config)
    }

    /// Generate a melt plan from a JSON Schema
    pub fn from_schema(schema: &Value, config: MeltConfig) -> Result<Self> {
        let mut entity_plans = HashMap::new();

        // Analyze the root schema
        Self::analyze_schema(
            schema,
            "root",
            &config,
            &mut entity_plans,
            0,
        )?;

        Ok(MeltPlan {
            entity_plans,
            config,
        })
    }

    /// Recursively analyze a schema to build extraction plans
    fn analyze_schema(
        schema: &Value,
        entity_type: &str,
        config: &MeltConfig,
        plans: &mut HashMap<String, EntityPlan>,
        depth: usize,
    ) -> Result<()> {
        if depth > config.max_depth {
            return Ok(());
        }

        // Get the type of this schema node
        let schema_type = schema.get("type").and_then(|t| t.as_str());

        match schema_type {
            Some("object") => {
                Self::analyze_object_schema(schema, entity_type, config, plans, depth)?;
            }
            Some("array") => {
                Self::analyze_array_schema(schema, entity_type, config, plans, depth)?;
            }
            _ => {
                // Scalar type - no further extraction needed
            }
        }

        Ok(())
    }

    /// Analyze an object schema
    fn analyze_object_schema(
        schema: &Value,
        entity_type: &str,
        config: &MeltConfig,
        plans: &mut HashMap<String, EntityPlan>,
        depth: usize,
    ) -> Result<()> {
        let mut scalar_fields = HashSet::new();
        let mut nested_fields = HashMap::new();
        let mut has_id_field = false;

        // Get properties from schema
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (field_name, field_schema) in properties.iter() {
                // Check if this field should always be treated as scalar
                if config.scalar_fields.contains(&field_name.to_string()) {
                    scalar_fields.insert(field_name.clone());
                    continue;
                }

                // Check if this is the ID field
                if field_name == "id" {
                    has_id_field = true;
                    scalar_fields.insert(field_name.clone());
                    continue;
                }

                // Determine field type from schema
                let field_type = field_schema.get("type");

                match field_type {
                    Some(Value::String(t)) if t == "array" => {
                        // Array field - should be extracted
                        let nested_type = format!("{}{}{}", entity_type, config.separator, field_name);

                        // Determine array element type
                        let element_type = if let Some(items) = field_schema.get("items") {
                            if let Some(item_type) = items.get("type").and_then(|t| t.as_str()) {
                                if item_type == "object" {
                                    ArrayType::Objects
                                } else {
                                    ArrayType::Scalars
                                }
                            } else {
                                ArrayType::Objects // Default to objects
                            }
                        } else {
                            ArrayType::Empty
                        };

                        nested_fields.insert(
                            field_name.clone(),
                            FieldRule::ArrayEntity {
                                entity_type: nested_type.clone(),
                                element_type: element_type.clone(),
                            },
                        );

                        // Recursively analyze the array's item schema
                        if let Some(items) = field_schema.get("items") {
                            Self::analyze_schema(items, &nested_type, config, plans, depth + 1)?;
                        }
                    }
                    Some(Value::String(t)) if t == "object" => {
                        // Nested object - check if it should be extracted
                        let should_extract = Self::should_extract_object_from_schema(field_schema);

                        if should_extract {
                            let nested_type = format!("{}{}{}", entity_type, config.separator, field_name);
                            nested_fields.insert(
                                field_name.clone(),
                                FieldRule::NestedEntity {
                                    entity_type: nested_type.clone(),
                                },
                            );

                            // Recursively analyze
                            Self::analyze_schema(field_schema, &nested_type, config, plans, depth + 1)?;
                        } else {
                            scalar_fields.insert(field_name.clone());
                        }
                    }
                    _ => {
                        // Scalar field
                        scalar_fields.insert(field_name.clone());
                    }
                }
            }
        }

        // Store the plan for this entity type
        plans.insert(
            entity_type.to_string(),
            EntityPlan {
                entity_type: entity_type.to_string(),
                scalar_fields,
                nested_fields,
                has_id_field,
            },
        );

        Ok(())
    }

    /// Analyze an array schema
    fn analyze_array_schema(
        schema: &Value,
        entity_type: &str,
        config: &MeltConfig,
        plans: &mut HashMap<String, EntityPlan>,
        depth: usize,
    ) -> Result<()> {
        // For array schemas, analyze the items
        if let Some(items) = schema.get("items") {
            Self::analyze_schema(items, entity_type, config, plans, depth)?;
        }

        Ok(())
    }

    /// Determine if an object schema represents something that should be extracted
    fn should_extract_object_from_schema(schema: &Value) -> bool {
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            // Extract if it has an ID or multiple fields
            properties.contains_key("id") || properties.len() > 2
        } else {
            false
        }
    }

    /// Get the extraction plan for a given entity type
    pub fn get_plan(&self, entity_type: &str) -> Option<&EntityPlan> {
        self.entity_plans.get(entity_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_plan_generation() {
        let examples = vec![
            json!({"id": 1, "name": "Alice", "age": 30}),
            json!({"id": 2, "name": "Bob", "age": 25}),
        ];

        let config = MeltConfig::default();
        let plan = MeltPlan::from_examples(&examples, config).unwrap();

        // Should have a root plan
        let root_plan = plan.get_plan("root").unwrap();
        assert_eq!(root_plan.entity_type, "root");
        assert!(root_plan.has_id_field);
        assert!(root_plan.scalar_fields.contains("name"));
        assert!(root_plan.scalar_fields.contains("age"));
    }

    #[test]
    fn test_nested_array_plan() {
        let examples = vec![
            json!({
                "id": 1,
                "name": "Alice",
                "posts": [
                    {"id": 10, "title": "Post 1"},
                    {"id": 11, "title": "Post 2"}
                ]
            }),
        ];

        let config = MeltConfig::default();
        let plan = MeltPlan::from_examples(&examples, config).unwrap();

        // Should have root and posts plans
        let root_plan = plan.get_plan("root").unwrap();
        assert!(root_plan.nested_fields.contains_key("posts"));

        if let FieldRule::ArrayEntity { entity_type, element_type } =
            &root_plan.nested_fields["posts"]
        {
            assert_eq!(entity_type, "root_posts");
            assert_eq!(*element_type, ArrayType::Objects);
        } else {
            panic!("Expected ArrayEntity rule");
        }

        // Should have a plan for posts
        let posts_plan = plan.get_plan("root_posts").unwrap();
        assert!(posts_plan.has_id_field);
        assert!(posts_plan.scalar_fields.contains("title"));
    }
}
