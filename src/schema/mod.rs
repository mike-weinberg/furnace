//! JSON Schema inference
//!
//! This module provides fast, production-ready JSON Schema inference
//! with format detection and proper type unification.

pub mod builder;
pub mod inference;

pub use builder::{SchemaBuilder, infer_schema_streaming};
pub use inference::infer_schema;
