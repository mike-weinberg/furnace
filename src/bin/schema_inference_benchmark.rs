/// Benchmark our Rust schema inference implementation vs genson-rs
use std::fs;
use std::path::Path;
use std::time::Instant;
use json_melt::infer_schema;

fn main() -> anyhow::Result<()> {
    println!("=== Benchmarking Rust Schema Inference vs genson-rs ===\n");

    let examples_dir = Path::new("schema_inference/src/tests/examples");
    let manifest_file = examples_dir.join("manifest.json");

    // Load manifest
    let manifest_content = fs::read_to_string(&manifest_file)?;
    let manifest: Vec<serde_json::Value> = serde_json::from_str(&manifest_content)?;

    // Organize by category
    let mut categories: std::collections::BTreeMap<String, Vec<_>> = std::collections::BTreeMap::new();
    for entry in &manifest {
        let cat = entry["category"].as_str().unwrap_or("unknown").to_string();
        if !categories.contains_key(&cat) {
            categories.insert(cat.clone(), Vec::new());
        }
        categories.get_mut(&cat).unwrap().push(entry);
    }

    println!("=== Benchmarking by Complexity ===\n");

    let mut all_ours = Vec::new();
    let mut all_genson = Vec::new();

    for (category, entries) in categories.iter() {
        println!("{}:", category);
        let mut times_ours = Vec::new();
        let mut times_genson = Vec::new();

        // Sample up to 10 from each category
        for entry in entries.iter().take(10) {
            let schema_name = entry["name"].as_str().unwrap_or("unknown");
            let schema_file_path = entry["schema_file"].as_str().unwrap_or("");

            let output_file = if schema_file_path.starts_with("/") {
                Path::new(schema_file_path).parent().unwrap().join("schema_with_examples.json")
            } else {
                examples_dir.parent().unwrap().parent().unwrap()
                    .join(schema_file_path)
                    .parent().unwrap()
                    .join("schema_with_examples.json")
            };

            if !output_file.exists() {
                continue;
            }

            let data_content = fs::read_to_string(&output_file)?;
            let data: serde_json::Value = serde_json::from_str(&data_content)?;
            let examples = &data["examples"];

            if !examples.is_array() {
                continue;
            }

            let examples_array = examples.as_array().unwrap();

            // Benchmark our implementation
            let start = Instant::now();
            let _schema = infer_schema(examples_array);
            let elapsed_ours = start.elapsed();

            // Benchmark genson-rs
            let examples_json = serde_json::to_string(&examples)?;
            let start = Instant::now();
            let mut builder = genson_rs::SchemaBuilder::new(Some("AUTO"));
            let mut json_bytes = examples_json.into_bytes();
            let examples_array_genson = simd_json::to_borrowed_value(&mut json_bytes)?;

            match examples_array_genson {
                simd_json::BorrowedValue::Array(arr) => {
                    for example in arr {
                        builder.add_object(&example);
                    }
                }
                _ => {}
            }
            let _ = builder.to_schema();
            let elapsed_genson = start.elapsed();

            let time_ours_ms = elapsed_ours.as_secs_f64() * 1000.0;
            let time_genson_ms = elapsed_genson.as_secs_f64() * 1000.0;
            let ratio = time_genson_ms / time_ours_ms;

            times_ours.push(time_ours_ms);
            times_genson.push(time_genson_ms);
            all_ours.push(time_ours_ms);
            all_genson.push(time_genson_ms);

            println!(
                "  {:<40} Ours: {:7.2}ms  Genson: {:7.2}ms  Ratio: {:6.2}x",
                schema_name, time_ours_ms, time_genson_ms, ratio
            );
        }

        if !times_ours.is_empty() {
            let avg_ours = times_ours.iter().sum::<f64>() / times_ours.len() as f64;
            let avg_genson = times_genson.iter().sum::<f64>() / times_genson.len() as f64;
            let ratio = avg_genson / avg_ours;
            println!(
                "  Average:                             Ours: {:7.2}ms  Genson: {:7.2}ms  Ratio: {:6.2}x\n",
                avg_ours, avg_genson, ratio
            );
        }
    }

    if !all_ours.is_empty() {
        let overall_ours = all_ours.iter().sum::<f64>() / all_ours.len() as f64;
        let overall_genson = all_genson.iter().sum::<f64>() / all_genson.len() as f64;
        let overall_ratio = overall_genson / overall_ours;

        println!("=== Overall Statistics ===");
        println!("Total benchmarks: {}", all_ours.len());
        println!("Our implementation average: {:.2}ms", overall_ours);
        println!("Genson-rs average: {:.2}ms", overall_genson);
        println!("Speedup ratio (Genson/Ours): {:.2}x", overall_ratio);

        if overall_ratio < 1.0 {
            println!("\n✓ Our implementation is {:.2}x FASTER than genson-rs", 1.0 / overall_ratio);
        } else {
            println!("\n✗ Our implementation is {:.2}x SLOWER than genson-rs", overall_ratio);
        }
    }

    Ok(())
}
