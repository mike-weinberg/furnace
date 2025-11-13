/// Comprehensive benchmark comparing all three schema inference implementations:
/// 1. Old implementation (infer_schema) - merge-based approach
/// 2. New streaming implementation (SchemaBuilder) - accumulator pattern
/// 3. genson-rs - third-party library using simd_json
///
/// This benchmark loads the same test data and runs all three implementations,
/// providing detailed timing comparisons and improvement ratios.

use std::fs;
use std::path::Path;
use std::time::Instant;
use furnace::{infer_schema, infer_schema_streaming};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    category: String,
    old_time_ms: f64,
    streaming_time_ms: f64,
    genson_time_ms: f64,
}

impl BenchmarkResult {
    fn streaming_vs_old_ratio(&self) -> f64 {
        self.old_time_ms / self.streaming_time_ms
    }
}

fn main() -> anyhow::Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║        Schema Inference Performance Comparison Benchmark                 ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Comparing three implementations:");
    println!("  1. Old (infer_schema)       - Merge-based approach with intermediate schemas");
    println!("  2. Streaming (SchemaBuilder) - Accumulator pattern with single-pass build");
    println!("  3. Genson-rs                - Third-party library using simd_json");
    println!();

    let examples_dir = Path::new("tests/schema_examples");
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

    let mut all_results = Vec::new();

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                      DETAILED BENCHMARK RESULTS                           ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    for (category, entries) in categories.iter() {
        println!("┌─────────────────────────────────────────────────────────────────────────┐");
        println!("│ Category: {:<63} │", category);
        println!("└─────────────────────────────────────────────────────────────────────────┘");
        println!();
        println!("{:<42} {:>10} {:>10} {:>10}", "Schema", "Old (ms)", "Stream (ms)", "Genson (ms)");
        println!("{:-<75}", "");

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

            // Benchmark 1: Old implementation (infer_schema)
            let start = Instant::now();
            let _schema_old = infer_schema(examples_array);
            let old_time = start.elapsed();

            // Benchmark 2: New streaming implementation (SchemaBuilder)
            let start = Instant::now();
            let _schema_streaming = infer_schema_streaming(examples_array);
            let streaming_time = start.elapsed();

            // Benchmark 3: genson-rs
            // Prepare data for genson-rs (convert to BorrowedValue OUTSIDE timer)
            let examples_json = serde_json::to_string(&examples)?;
            let mut json_bytes = examples_json.into_bytes();
            let examples_borrowed = simd_json::to_borrowed_value(&mut json_bytes)?;

            let start = Instant::now();
            let mut builder = genson_rs::SchemaBuilder::new(Some("AUTO"));
            match examples_borrowed {
                simd_json::BorrowedValue::Array(ref arr) => {
                    for example in arr {
                        builder.add_object(example);
                    }
                }
                _ => {}
            }
            let _ = builder.to_schema();
            let genson_time = start.elapsed();

            // Convert to milliseconds
            let old_ms = old_time.as_secs_f64() * 1000.0;
            let streaming_ms = streaming_time.as_secs_f64() * 1000.0;
            let genson_ms = genson_time.as_secs_f64() * 1000.0;

            let result = BenchmarkResult {
                name: schema_name.to_string(),
                category: category.clone(),
                old_time_ms: old_ms,
                streaming_time_ms: streaming_ms,
                genson_time_ms: genson_ms,
            };

            println!(
                "{:<42} {:>10.2} {:>10.2} {:>10.2}",
                truncate_string(schema_name, 42),
                old_ms,
                streaming_ms,
                genson_ms
            );

            all_results.push(result);
        }

        println!();
    }

    // Calculate statistics
    if all_results.is_empty() {
        println!("No benchmark results collected.");
        return Ok(());
    }

    let avg_old = all_results.iter().map(|r| r.old_time_ms).sum::<f64>() / all_results.len() as f64;
    let avg_streaming = all_results.iter().map(|r| r.streaming_time_ms).sum::<f64>() / all_results.len() as f64;
    let avg_genson = all_results.iter().map(|r| r.genson_time_ms).sum::<f64>() / all_results.len() as f64;

    let min_old = all_results.iter().map(|r| r.old_time_ms).fold(f64::INFINITY, f64::min);
    let min_streaming = all_results.iter().map(|r| r.streaming_time_ms).fold(f64::INFINITY, f64::min);
    let min_genson = all_results.iter().map(|r| r.genson_time_ms).fold(f64::INFINITY, f64::min);

    let max_old = all_results.iter().map(|r| r.old_time_ms).fold(0.0, f64::max);
    let max_streaming = all_results.iter().map(|r| r.streaming_time_ms).fold(0.0, f64::max);
    let max_genson = all_results.iter().map(|r| r.genson_time_ms).fold(0.0, f64::max);

    // Calculate improvement ratios
    let streaming_vs_old = avg_old / avg_streaming;
    let streaming_vs_genson = avg_genson / avg_streaming;
    let old_vs_genson = avg_genson / avg_old;

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                          SUMMARY STATISTICS                               ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    println!("Total benchmarks run: {}", all_results.len());
    println!();

    // Create a nice summary table
    println!("┌─────────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Metric          │ Old (ms)     │ Stream (ms)  │ Genson (ms)  │");
    println!("├─────────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Average         │ {:>12.2} │ {:>12.2} │ {:>12.2} │", avg_old, avg_streaming, avg_genson);
    println!("│ Minimum         │ {:>12.2} │ {:>12.2} │ {:>12.2} │", min_old, min_streaming, min_genson);
    println!("│ Maximum         │ {:>12.2} │ {:>12.2} │ {:>12.2} │", max_old, max_streaming, max_genson);
    println!("└─────────────────┴──────────────┴──────────────┴──────────────┘");
    println!();

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                       PERFORMANCE COMPARISONS                             ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    // Streaming vs Old
    println!("┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ Streaming vs Old (Merge-based)                                          │");
    println!("└─────────────────────────────────────────────────────────────────────────┘");
    if streaming_vs_old > 1.0 {
        println!("  ✓ Streaming is {:.2}x FASTER than Old implementation", streaming_vs_old);
        println!("  ✓ Improvement: {:.1}% speed increase", (streaming_vs_old - 1.0) * 100.0);
    } else {
        println!("  ✗ Streaming is {:.2}x SLOWER than Old implementation", 1.0 / streaming_vs_old);
        println!("  ✗ Regression: {:.1}% speed decrease", (1.0 - streaming_vs_old) * 100.0);
    }
    println!();

    // Streaming vs Genson
    println!("┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ Streaming vs Genson-rs (Third-party)                                    │");
    println!("└─────────────────────────────────────────────────────────────────────────┘");
    if streaming_vs_genson > 1.0 {
        println!("  ✓ Streaming is {:.2}x FASTER than Genson-rs", streaming_vs_genson);
        println!("  ✓ Improvement: {:.1}% speed advantage", (streaming_vs_genson - 1.0) * 100.0);
    } else {
        println!("  ✗ Streaming is {:.2}x SLOWER than Genson-rs", 1.0 / streaming_vs_genson);
        println!("  ✗ Gap: {:.1}% behind Genson-rs", (1.0 - streaming_vs_genson) * 100.0);
    }
    println!();

    // Old vs Genson
    println!("┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ Old vs Genson-rs (Third-party)                                          │");
    println!("└─────────────────────────────────────────────────────────────────────────┘");
    if old_vs_genson > 1.0 {
        println!("  ✓ Old is {:.2}x FASTER than Genson-rs", old_vs_genson);
    } else {
        println!("  ✗ Old is {:.2}x SLOWER than Genson-rs", 1.0 / old_vs_genson);
    }
    println!();

    // Top 5 improvements from Old to Streaming
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                    TOP 5 STREAMING IMPROVEMENTS                           ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    let mut sorted_by_improvement = all_results.clone();
    sorted_by_improvement.sort_by(|a, b| {
        b.streaming_vs_old_ratio()
            .partial_cmp(&a.streaming_vs_old_ratio())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("{:<45} {:>12} {:>15}", "Schema", "Speedup", "Category");
    println!("{:-<75}", "");
    for result in sorted_by_improvement.iter().take(5) {
        println!(
            "{:<45} {:>11.2}x {:>15}",
            truncate_string(&result.name, 45),
            result.streaming_vs_old_ratio(),
            truncate_string(&result.category, 15)
        );
    }
    println!();

    // Performance by category
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                    PERFORMANCE BY CATEGORY                                ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    let mut category_stats: std::collections::BTreeMap<String, Vec<&BenchmarkResult>> =
        std::collections::BTreeMap::new();

    for result in &all_results {
        category_stats
            .entry(result.category.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    println!("{:<25} {:>10} {:>10} {:>10} {:>12}",
        "Category", "Old (avg)", "Stream", "Genson", "Stream/Old");
    println!("{:-<75}", "");

    for (category, results) in category_stats.iter() {
        let cat_avg_old = results.iter().map(|r| r.old_time_ms).sum::<f64>() / results.len() as f64;
        let cat_avg_streaming = results.iter().map(|r| r.streaming_time_ms).sum::<f64>() / results.len() as f64;
        let cat_avg_genson = results.iter().map(|r| r.genson_time_ms).sum::<f64>() / results.len() as f64;
        let cat_ratio = cat_avg_old / cat_avg_streaming;

        println!(
            "{:<25} {:>10.2} {:>10.2} {:>10.2} {:>11.2}x",
            truncate_string(category, 25),
            cat_avg_old,
            cat_avg_streaming,
            cat_avg_genson,
            cat_ratio
        );
    }
    println!();

    // Final verdict
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                           FINAL VERDICT                                   ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    if streaming_vs_old > 1.0 && streaming_vs_genson > 1.0 {
        println!("  ✓✓ The new Streaming implementation is the CLEAR WINNER!");
        println!("     - {:.2}x faster than the Old merge-based approach", streaming_vs_old);
        println!("     - {:.2}x faster than Genson-rs third-party library", streaming_vs_genson);
    } else if streaming_vs_old > 1.0 {
        println!("  ✓ The Streaming implementation is better than the Old implementation");
        println!("    - {:.2}x faster than Old", streaming_vs_old);
        println!("    - But {:.2}x slower than Genson-rs", 1.0 / streaming_vs_genson);
    } else if streaming_vs_genson > 1.0 {
        println!("  ~ The Streaming implementation beats Genson-rs but not Old");
        println!("    - {:.2}x slower than Old", 1.0 / streaming_vs_old);
        println!("    - {:.2}x faster than Genson-rs", streaming_vs_genson);
    } else {
        println!("  ✗ The Streaming implementation needs more optimization");
        println!("    - {:.2}x slower than Old", 1.0 / streaming_vs_old);
        println!("    - {:.2}x slower than Genson-rs", 1.0 / streaming_vs_genson);
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════════════");

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
