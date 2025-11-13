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

use anyhow::Result;
use clap::Parser;
use furnace::schema::SchemaBuilder;
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

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

    // Create reader based on input source
    let reader: Box<dyn BufRead> = if let Some(file_path) = &args.input {
        Box::new(BufReader::new(File::open(file_path)?))
    } else {
        Box::new(BufReader::new(stdin()))
    };

    // Build schema by processing examples
    let mut builder = SchemaBuilder::new();
    let mut count = 0;

    if args.ndjson {
        // Process NDJSON
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line)?;
            builder.add_value(&value);
            count += 1;
        }
    } else {
        // Process single JSON value
        for line in reader.lines() {
            let line = line?;
            let value: Value = serde_json::from_str(&line)?;
            builder.add_value(&value);
            count += 1;
            break; // Only process first value in non-NDJSON mode
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
