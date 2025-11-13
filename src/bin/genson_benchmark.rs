/// Benchmark genson-rs on the schema inference examples
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== Benchmarking genson-rs ===\n");

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

    let mut all_times = Vec::new();

    for (category, entries) in categories.iter() {
        println!("{}:", category);
        let mut times = Vec::new();

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

            // Benchmark genson-rs
            // Use raw JSON bytes directly to avoid conversion overhead
            let examples_json = serde_json::to_string(&examples)?;
            let start = Instant::now();
            let mut builder = genson_rs::SchemaBuilder::new(Some("AUTO"));
            let mut json_bytes = examples_json.into_bytes();
            let examples_array = simd_json::to_borrowed_value(&mut json_bytes)?;

            match examples_array {
                simd_json::BorrowedValue::Array(arr) => {
                    for example in arr {
                        builder.add_object(&example);
                    }
                }
                _ => {}
            }
            let _ = builder.to_schema();
            let elapsed = start.elapsed();

            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            times.push(elapsed_ms);
            all_times.push(elapsed_ms);

            println!("  {:<50} {:8.2}ms", schema_name, elapsed_ms);
        }

        if !times.is_empty() {
            let avg = times.iter().sum::<f64>() / times.len() as f64;
            println!("  Average: {:8.2}ms\n", avg);
        }
    }

    if !all_times.is_empty() {
        let overall_avg = all_times.iter().sum::<f64>() / all_times.len() as f64;
        println!("=== Overall Statistics ===");
        println!("Total benchmarks: {}", all_times.len());
        println!("Average time: {:.2}ms", overall_avg);
        println!("Min time: {:.2}ms", all_times.iter().cloned().fold(f64::INFINITY, f64::min));
        println!("Max time: {:.2}ms", all_times.iter().cloned().fold(0.0, f64::max));
    }

    Ok(())
}
