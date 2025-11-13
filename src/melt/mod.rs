//! JSON melting - extract nested JSON into relational tables
//!
//! This module handles the extraction of nested JSON structures into
//! flat, relational entities suitable for database storage or analysis.
//!
//! ## Performance Optimization
//!
//! For processing streams of homogeneous JSON (like API responses),
//! use `PlannedMelter` for significantly better performance by pre-computing
//! extraction decisions.

pub mod types;
pub mod extractor;
pub mod writer;
pub mod plan;
pub mod planned_extractor;

pub use types::{Entity, EntityId, MeltConfig, ParentRef};
pub use extractor::JsonMelter;
pub use writer::{EntityWriter, SingleWriter};
pub use plan::{MeltPlan, EntityPlan, FieldRule, ArrayType};
pub use planned_extractor::PlannedMelter;
