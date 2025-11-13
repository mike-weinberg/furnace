/// Validates that inferred schemas correctly describe their source examples
///
/// This is the correct validation test for schema inference tools:
/// 1. Infer a schema from example data
/// 2. Validate all examples against the inferred schema
/// 3. Ensure 100% of examples validate successfully
///
/// This differs from schema_validation.rs which incorrectly compared
/// inferred schemas against hand-written prescriptive schemas.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== Schema Correctness Validation ===\n");
    println!("Testing that inferred schemas correctly describe their source examples\n");

    let examples_dir = Path::new("schema_inference/src/tests/examples");
    let manifest_file = examples_dir.join("manifest.json");

    // Load manifest
    let manifest_content = fs::read_to_string(&manifest_file)?;
    let manifest: Vec<Value> = serde_json::from_str(&manifest_content)?;

    let mut total_tests = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    println!("Testing all {} schemas...\n", manifest.len());

    for (idx, entry) in manifest.iter().enumerate() {
        let schema_name = entry["name"].as_str().unwrap_or("unknown");
        let schema_file_path = entry["schema_file"].as_str().unwrap_or("");

        let schema_file = if schema_file_path.starts_with("/") {
            Path::new(schema_file_path).to_path_buf()
        } else {
            examples_dir.parent().unwrap().parent().unwrap().join(schema_file_path)
        };

        // Load examples
        let examples_file = schema_file.parent().unwrap().join("schema_with_examples.json");
        if !examples_file.exists() {
            skipped += 1;
            continue;
        }

        let data_content = fs::read_to_string(&examples_file)?;
        let data: Value = serde_json::from_str(&data_content)?;
        let examples = data["examples"].as_array().unwrap();

        if examples.is_empty() {
            skipped += 1;
            continue;
        }

        // Infer schema using our streaming implementation
        let inferred_schema = furnace::infer_schema_streaming(examples);

        total_tests += 1;

        // Validate all examples against the inferred schema
        let mut validation_failures = 0;
        for example in examples.iter() {
            if !validates_against_schema(example, &inferred_schema) {
                validation_failures += 1;
            }
        }

        // Report result
        if validation_failures == 0 {
            passed += 1;
            if (idx + 1) % 10 == 0 {
                println!("✓ Processed {}/{} schemas ({} passed so far)", idx + 1, manifest.len(), passed);
            }
        } else {
            failed += 1;
            println!(
                "✗ {}: {}/{} examples failed validation",
                schema_name,
                validation_failures,
                examples.len()
            );
        }
    }

    println!("\n=== Integration Test Results ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Skipped: {}", skipped);
    println!("Total:  {}", total_tests);
    println!(
        "Success Rate: {:.1}%",
        if total_tests > 0 {
            100.0 * passed as f64 / total_tests as f64
        } else {
            0.0
        }
    );

    if failed > 0 {
        println!("\n⚠ Some examples failed validation!");
        std::process::exit(1);
    } else if passed > 0 {
        println!("\n✓ All examples validate correctly against their inferred schemas!");
    }

    Ok(())
}

/// Validate an example against an inferred schema
/// Implements the same validation logic as Python test_integration.py
fn validates_against_schema(example: &Value, schema: &Value) -> bool {
    let schema_type = schema.get("type");

    match schema_type {
        Some(Value::String(type_str)) => match type_str.as_str() {
            "null" => example.is_null(),
            "boolean" => example.is_boolean(),
            "integer" => example.is_i64(),
            "number" => example.is_number(),
            "string" => example.is_string(),
            "array" => {
                if !example.is_array() {
                    return false;
                }
                // If schema has items, validate each item
                if let Some(items_schema) = schema.get("items") {
                    let arr = example.as_array().unwrap();
                    arr.iter()
                        .all(|item| validates_against_schema(item, items_schema))
                } else {
                    true
                }
            }
            "object" => {
                if !example.is_object() {
                    return false;
                }

                let obj = example.as_object().unwrap();
                let properties = schema.get("properties").and_then(|v| v.as_object());
                let empty_vec = vec![];
                let required = schema
                    .get("required")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);

                // Check required fields
                for req_field in required {
                    if let Value::String(field_name) = req_field {
                        if !obj.contains_key(field_name) {
                            return false;
                        }
                    }
                }

                // Check present fields against property schemas
                if let Some(props) = properties {
                    for (key, value) in obj.iter() {
                        if let Some(prop_schema) = props.get(key) {
                            if !validates_against_schema(value, prop_schema) {
                                return false;
                            }
                        }
                    }
                }

                true
            }
            _ => true, // Unknown type - accept
        },
        Some(Value::Array(types)) => {
            // Multiple types (e.g., nullable) - validate against any type
            types.iter().any(|t| {
                let schema_copy = json!({ "type": t });
                validates_against_schema(example, &schema_copy)
            })
        }
        None => {
            // Check for anyOf
            if let Some(Value::Array(any_of_schemas)) = schema.get("anyOf") {
                any_of_schemas
                    .iter()
                    .any(|subschema| validates_against_schema(example, subschema))
            } else {
                // No type or anyOf - accept anything
                true
            }
        }
        _ => true, // Accept if we can't determine type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_validation() {
        let schema = json!({ "type": "null" });
        assert!(validates_against_schema(&json!(null), &schema));
        assert!(!validates_against_schema(&json!("string"), &schema));
    }

    #[test]
    fn test_string_validation() {
        let schema = json!({ "type": "string" });
        assert!(validates_against_schema(&json!("hello"), &schema));
        assert!(!validates_against_schema(&json!(42), &schema));
    }

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        });

        assert!(validates_against_schema(
            &json!({"name": "Alice", "age": 30}),
            &schema
        ));
        assert!(!validates_against_schema(&json!({"age": 30}), &schema)); // Missing required
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "items": { "type": "integer" }
        });

        assert!(validates_against_schema(&json!([1, 2, 3]), &schema));
        assert!(!validates_against_schema(&json!([1, "two", 3]), &schema));
    }

    #[test]
    fn test_nullable_validation() {
        let schema = json!({
            "type": ["string", "null"]
        });

        assert!(validates_against_schema(&json!("hello"), &schema));
        assert!(validates_against_schema(&json!(null), &schema));
        assert!(!validates_against_schema(&json!(42), &schema));
    }
}
