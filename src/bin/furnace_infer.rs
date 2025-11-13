//! furnace-infer: Infer JSON Schemas from examples with format detection
//!
//! Analogous to genson-cli but with superior format detection and type inference.
//!
//! Features:
//! - Detects formats: date, time, email, UUID, IPv4, IPv6 (not in genson-cli!)
//! - Proper required field tracking
//! - Smart type unification
//!
//! Usage:
//!   # Read from file, output to stdout
//!   furnace-infer data.json
//!
//!   # Read from stdin, output to stdout
//!   echo '{"id": 1, "email": "alice@example.com"}' | furnace-infer
//!
//!   # Process NDJSON with compact output
//!   furnace-infer --ndjson events.jsonl --compact

// Use MiMalloc allocator for better performance (recommended by simd-json)
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use clap::Parser;
use furnace::schema::SchemaBuilder;
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Read};

#[derive(Parser, Debug)]
#[command(name = "furnace-infer")]
#[command(about = "Infer JSON Schemas with format detection", long_about = None)]
struct Args {
    /// Input file (use stdin if omitted)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Process newline-delimited JSON (one JSON object per line)
    #[arg(long)]
    ndjson: bool,

    /// Compact output (no pretty-printing)
    #[arg(long)]
    compact: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read entire input for SIMD parsing
    let mut content = Vec::new();
    let reader: Box<dyn Read> = if let Some(file_path) = &args.input {
        Box::new(BufReader::new(File::open(file_path)?))
    } else {
        Box::new(stdin())
    };

    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_to_end(&mut content)?;

    // Build schema by processing examples with SIMD-accelerated parsing
    let mut builder = SchemaBuilder::new();
    let mut count = 0;

    // Try SIMD parsing first for maximum performance
    match simd_json::to_owned_value(&mut content) {
        Ok(simd_json::OwnedValue::Array(arr)) => {
            // JSON array - process all elements
            for elem in arr.iter() {
                let json_str = simd_json::to_string(elem)?;
                let value: Value = serde_json::from_str(&json_str)?;
                builder.add_value(&value);
                count += 1;
            }
        }
        Ok(elem) => {
            // Single JSON object
            let json_str = simd_json::to_string(&elem)?;
            let value: Value = serde_json::from_str(&json_str)?;
            builder.add_value(&value);
            count += 1;
        }
        Err(_) => {
            // Fallback to serde_json for NDJSON or malformed input
            let content_str = String::from_utf8_lossy(&content);
            for line in content_str.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let value: Value = serde_json::from_str(line)?;
                builder.add_value(&value);
                count += 1;

                if !args.ndjson {
                    break; // Only process first value in non-NDJSON mode
                }
            }
        }
    }

    if count == 0 {
        eprintln!("Warning: No JSON objects found in input");
    }

    // Get schema and output
    let schema = builder.build();

    let output = if args.compact {
        serde_json::to_string(&schema)?
    } else {
        serde_json::to_string_pretty(&schema)?
    };

    println!("{}", output);

    Ok(())
}
