//! Benchmark comparing unplanned vs planned melting performance
//!
//! This benchmark demonstrates the performance advantage of using PlannedMelter
//! for processing homogeneous JSON streams.

use furnace::{JsonMelter, PlannedMelter, MeltConfig};
use serde_json::json;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== Furnace Melt Performance Benchmark ===\n");
    println!("Comparing unplanned (JsonMelter) vs planned (PlannedMelter) extraction\n");

    // Generate test data
    let mut test_data = Vec::new();
    for i in 0..1000 {
        test_data.push(json!({
            "id": i,
            "username": format!("user{}", i),
            "email": format!("user{}@example.com", i),
            "profile": {
                "bio": format!("I am user {}", i),
                "location": "San Francisco",
                "joined": "2024-01-01"
            },
            "posts": [
                {
                    "id": i * 10,
                    "title": format!("Post {} - First", i),
                    "content": "This is my first post about Rust",
                    "tags": ["rust", "programming", "tech"],
                    "likes": i % 100
                },
                {
                    "id": i * 10 + 1,
                    "title": format!("Post {} - Second", i),
                    "content": "Another post about performance",
                    "tags": ["performance", "optimization"],
                    "likes": (i * 2) % 100
                }
            ],
            "friends": [
                {"id": i + 1, "name": format!("Friend {}", i + 1)},
                {"id": i + 2, "name": format!("Friend {}", i + 2)}
            ]
        }));
    }

    let config = MeltConfig::default();

    // Benchmark 1: Unplanned melting (JsonMelter)
    println!("=== Benchmark 1: Unplanned Melting (JsonMelter) ===");
    println!("Processing 1000 complex JSON records with runtime decisions...\n");

    let melter = JsonMelter::new(config.clone());
    let start = Instant::now();

    let mut total_entities = 0;
    for data in &test_data {
        let entities = melter.melt(data.clone())?;
        total_entities += entities.len();
    }

    let unplanned_duration = start.elapsed();

    println!("Time: {:?}", unplanned_duration);
    println!("Total entities extracted: {}", total_entities);
    println!("Average per record: {:.2}μs\n", unplanned_duration.as_micros() as f64 / 1000.0);

    // Benchmark 2: Planned melting (PlannedMelter)
    println!("=== Benchmark 2: Planned Melting (PlannedMelter) ===");
    println!("Step 1: Analyzing 10 sample records to build extraction plan...");

    let warmup_samples: Vec<_> = test_data.iter().take(10).cloned().collect();
    let plan_start = Instant::now();
    let planned_melter = PlannedMelter::from_examples(&warmup_samples, config.clone())?;
    let plan_duration = plan_start.elapsed();

    println!("Plan generation time: {:?}\n", plan_duration);

    println!("Step 2: Processing 1000 records with pre-computed plan...");

    let start = Instant::now();

    total_entities = 0;
    for data in &test_data {
        let entities = planned_melter.melt(data.clone())?;
        total_entities += entities.len();
    }

    let planned_duration = start.elapsed();

    println!("Time: {:?}", planned_duration);
    println!("Total entities extracted: {}", total_entities);
    println!("Average per record: {:.2}μs\n", planned_duration.as_micros() as f64 / 1000.0);

    // Analysis
    println!("=== Performance Analysis ===\n");

    let speedup = unplanned_duration.as_secs_f64() / planned_duration.as_secs_f64();
    let total_planned_time = plan_duration + planned_duration;

    println!("Unplanned total:     {:?}", unplanned_duration);
    println!("Planned extraction:  {:?}", planned_duration);
    println!("Plan generation:     {:?}", plan_duration);
    println!("Planned total:       {:?}", total_planned_time);
    println!();
    println!("Speedup (extraction only): {:.2}x faster", speedup);

    let amortized_speedup = unplanned_duration.as_secs_f64() / total_planned_time.as_secs_f64();
    println!("Speedup (amortized):       {:.2}x faster", amortized_speedup);

    println!("\n=== Key Insights ===\n");
    println!("✓ Planned melting eliminates per-record conditional logic");
    println!("✓ Pre-computed extraction rules are executed directly");
    println!("✓ Plan generation cost is amortized over large datasets");
    println!("✓ Ideal for: API pagination, log streams, database exports");

    if speedup >= 2.0 {
        println!("\n🚀 Significant performance improvement! ({:.1}x faster)", speedup);
    } else if speedup >= 1.2 {
        println!("\n✓ Measurable performance improvement ({:.1}x faster)", speedup);
    } else {
        println!("\n⚠ Performance improvement is modest ({:.1}x faster)", speedup);
        println!("   Note: Improvement varies with data complexity and field count");
    }

    Ok(())
}
