use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents a unique identifier for an entity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new(id: impl Into<String>) -> Self {
        EntityId(id.into())
    }
}

/// An entity extracted from JSON - represents one row in a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// The entity type (table name), e.g., "users", "users_posts"
    pub entity_type: String,

    /// The data for this entity
    pub data: Map<String, Value>,

    /// Optional ID of this entity
    pub id: Option<EntityId>,

    /// Parent entity information for foreign keys
    pub parent: Option<ParentRef>,
}

/// Reference to a parent entity (for foreign keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentRef {
    pub entity_type: String,
    pub id: EntityId,
    pub field_name: String,
}

impl Entity {
    pub fn new(entity_type: String, data: Map<String, Value>) -> Self {
        Entity {
            entity_type,
            data,
            id: None,
            parent: None,
        }
    }

    pub fn with_id(mut self, id: EntityId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_parent(mut self, parent: ParentRef) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Get or generate an ID for this entity
    pub fn get_or_generate_id(&mut self, counter: &mut u64) -> EntityId {
        if let Some(ref id) = self.id {
            return id.clone();
        }

        // Try to find an ID field in the data
        if let Some(Value::Number(n)) = self.data.get("id") {
            if let Some(id) = n.as_u64() {
                let entity_id = EntityId::new(id.to_string());
                self.id = Some(entity_id.clone());
                return entity_id;
            }
        }

        if let Some(Value::String(s)) = self.data.get("id") {
            let entity_id = EntityId::new(s.clone());
            self.id = Some(entity_id.clone());
            return entity_id;
        }

        // Generate a synthetic ID
        *counter += 1;
        let entity_id = EntityId::new(format!("_gen_{}", counter));
        self.id = Some(entity_id.clone());
        entity_id
    }
}

/// Configuration for the melting process
#[derive(Debug, Clone)]
pub struct MeltConfig {
    /// Maximum depth to extract entities (0 = only root)
    pub max_depth: usize,

    /// Prefix for generated foreign key columns
    pub fk_prefix: String,

    /// Prefix for generated IDs
    pub id_prefix: String,

    /// Separator for nested entity type names
    pub separator: String,

    /// Whether to include parent ID in child entities
    pub include_parent_ids: bool,

    /// Fields to always treat as scalar values (don't extract as entities)
    pub scalar_fields: Vec<String>,
}

impl Default for MeltConfig {
    fn default() -> Self {
        MeltConfig {
            max_depth: 10,
            fk_prefix: String::from(""),
            id_prefix: String::from("_id"),
            separator: String::from("_"),
            include_parent_ids: true,
            scalar_fields: vec![],
        }
    }
}
