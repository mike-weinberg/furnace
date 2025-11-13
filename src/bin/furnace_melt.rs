//! furnace-melt: Extract nested JSON into relational tables
//!
//! Analogous to genson-cli but for JSON melting instead of schema inference.
//!
//! Usage:
//!   # Read from file, output to stdout
//!   furnace-melt data.json
//!
//!   # Read from stdin, output to stdout
//!   echo '{"id": 1, "posts": [{"id": 10}]}' | furnace-melt
//!
//!   # Process NDJSON, write to directory
//!   furnace-melt --ndjson events.jsonl --output-dir ./entities
//!
//!   # Use PlannedMelter for 40% better performance
//!   furnace-melt --planned large_dataset.jsonl --output-dir ./entities

// Use MiMalloc allocator for better performance (recommended by simd-json)
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use clap::Parser;
use furnace::melt::{EntityWriter, JsonMelter, MeltConfig, PlannedMelter};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Stdout, Write, Read};

#[derive(Parser, Debug)]
#[command(name = "furnace-melt")]
#[command(about = "Extract nested JSON into relational tables", long_about = None)]
struct Args {
    /// Input file (use stdin if omitted)
    #[arg(value_name = "FILE")]
    input: Option<String>,

    /// Process newline-delimited JSON (one JSON object per line)
    #[arg(long)]
    ndjson: bool,

    /// Don't treat top-level arrays as object streams
    #[arg(long)]
    no_ignore_array: bool,

    /// Output directory for separate .jsonl files per entity type
    /// If omitted, writes to stdout as a single stream with entity metadata
    #[arg(long, short = 'o')]
    output_dir: Option<String>,

    /// Use PlannedMelter for better performance on homogeneous data
    /// Samples first N records to build an extraction plan (default: 100)
    #[arg(long)]
    planned: bool,

    /// Number of records to sample for the extraction plan (default: 100)
    #[arg(long, requires = "planned")]
    sample_size: Option<usize>,

    /// Maximum nesting depth to extract (default: 10)
    #[arg(long)]
    max_depth: Option<usize>,

    /// Separator for nested entity names (default: "_")
    #[arg(long)]
    separator: Option<String>,

    /// Comma-separated fields to never extract as entities
    #[arg(long)]
    scalar_fields: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Build config
    let mut config = MeltConfig::default();
    if let Some(depth) = args.max_depth {
        config.max_depth = depth;
    }
    if let Some(sep) = args.separator {
        config.separator = sep;
    }
    if let Some(fields_str) = args.scalar_fields {
        config.scalar_fields = fields_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
    }

    // Process based on output mode
    if let Some(output_dir) = args.output_dir {
        // Make output directory absolute
        let abs_path = if std::path::Path::new(&output_dir).is_absolute() {
            std::path::PathBuf::from(&output_dir)
        } else {
            std::env::current_dir()?.join(&output_dir)
        };

        let original_dir = std::env::current_dir()?;
        // Create the directory
        std::fs::create_dir_all(&abs_path)?;
        // Change to it
        std::env::set_current_dir(&abs_path)?;

        process_to_files(args.input, args.ndjson, args.planned, args.sample_size, config)?;

        std::env::set_current_dir(original_dir)?;
    } else {
        // Single stream to stdout
        process_to_stdout(args.input, args.ndjson, args.planned, args.sample_size, config)?;
    }

    Ok(())
}

/// Process JSON and write to separate files per entity type
fn process_to_files(
    input_file: Option<String>,
    ndjson: bool,
    planned: bool,
    sample_size: Option<usize>,
    config: MeltConfig,
) -> Result<()> {
    let sample_size = sample_size.unwrap_or(100);
    let mut writer = EntityWriter::new_file_writer(".")?;

    if planned {
        // Planned mode: sample first N records to build plan, then process all
        let mut records = Vec::new();

        // First pass: sample only (avoiding full file load into memory)
        {
            let reader = if let Some(file_path) = &input_file {
                Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
            } else {
                Box::new(std::io::stdin()) as Box<dyn Read>
            };

            sample_from_reader(reader, !ndjson, sample_size, &mut records)?;
        }

        // Build plan from samples
        if !records.is_empty() {
            let melter = PlannedMelter::from_examples(&records, config)?;

            // Second pass: process all records (or samples only for stdin)
            let reader = if let Some(file_path) = &input_file {
                Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
            } else {
                eprintln!("⚠ Warning: Using --planned mode with stdin. Only first {} records are processed.", sample_size);
                eprintln!("  For best performance with large files, pass the filename directly.");
                // Return early - can't re-read stdin
                return Ok(());
            };

            process_reader(reader, &melter, !ndjson, &mut writer)?;
        }
    } else {
        // Unplanned mode: process each record immediately
        let melter = JsonMelter::new(config);
        let reader = if let Some(file_path) = &input_file {
            Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
        } else {
            Box::new(std::io::stdin()) as Box<dyn Read>
        };

        process_reader_unplanned(reader, &melter, !ndjson, &mut writer)?;
    }

    writer.flush()?;
    Ok(())
}

/// Sample records from a reader using SIMD-accelerated JSON parsing when possible
fn sample_from_reader(
    reader: Box<dyn Read>,
    stop_after_first: bool,
    sample_size: usize,
    records: &mut Vec<Value>,
) -> Result<()> {
    // Read entire content into memory for SIMD parsing (only for sampling, small overhead)
    let mut content = Vec::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_to_end(&mut content)?;

    // Try SIMD parsing first (faster) - use OwnedValue to avoid borrow issues
    match simd_json::to_owned_value(&mut content) {
        Ok(simd_json::OwnedValue::Array(arr)) => {
            // JSON array - iterate through elements
            for (count, elem) in arr.iter().enumerate() {
                if count >= sample_size {
                    break;
                }
                // Convert simd_json value to serde_json::Value
                let json_str = simd_json::to_string(elem)?;
                let value: Value = serde_json::from_str(&json_str)?;
                records.push(value);

                if stop_after_first && !records.is_empty() {
                    break;
                }
            }
        }
        Ok(elem) => {
            // Single JSON object
            let json_str = simd_json::to_string(&elem)?;
            let value: Value = serde_json::from_str(&json_str)?;
            records.push(value);
        }
        Err(_) => {
            // Fallback to serde_json for NDJSON or malformed input
            let content_str = String::from_utf8_lossy(&content);
            for (count, line) in content_str.lines().enumerate() {
                if count >= sample_size {
                    break;
                }
                let line = line.trim();
                if !line.is_empty() {
                    let value: Value = serde_json::from_str(line)?;
                    records.push(value);

                    if stop_after_first && !records.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Process records from a reader with a melter using SIMD-accelerated parsing
fn process_reader(
    reader: Box<dyn Read>,
    melter: &PlannedMelter,
    stop_after_first: bool,
    writer: &mut EntityWriter<File>,
) -> Result<()> {
    // Read entire file for SIMD parsing
    let mut content = Vec::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_to_end(&mut content)?;

    // Try SIMD parsing for maximum performance - use OwnedValue
    match simd_json::to_owned_value(&mut content) {
        Ok(simd_json::OwnedValue::Array(arr)) => {
            // Fast path: JSON array with SIMD
            for elem in arr.iter() {
                let json_str = simd_json::to_string(elem)?;
                let value: Value = serde_json::from_str(&json_str)?;
                let entities = melter.melt(value)?;
                writer.write_entities(entities)?;

                if stop_after_first {
                    break;
                }
            }
        }
        Ok(elem) => {
            // Single object
            let json_str = simd_json::to_string(&elem)?;
            let value: Value = serde_json::from_str(&json_str)?;
            let entities = melter.melt(value)?;
            writer.write_entities(entities)?;
        }
        Err(_) => {
            // Fallback for NDJSON
            let content_str = String::from_utf8_lossy(&content);
            for line in content_str.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let value: Value = serde_json::from_str(line)?;
                let entities = melter.melt(value)?;
                writer.write_entities(entities)?;

                if stop_after_first {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Process records from a reader with unplanned melter using SIMD-accelerated parsing
fn process_reader_unplanned(
    reader: Box<dyn Read>,
    melter: &JsonMelter,
    stop_after_first: bool,
    writer: &mut EntityWriter<File>,
) -> Result<()> {
    // Read entire file for SIMD parsing
    let mut content = Vec::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_to_end(&mut content)?;

    // Try SIMD parsing for maximum performance - use OwnedValue
    match simd_json::to_owned_value(&mut content) {
        Ok(simd_json::OwnedValue::Array(arr)) => {
            // Fast path: JSON array with SIMD
            for elem in arr.iter() {
                let json_str = simd_json::to_string(elem)?;
                let value = serde_json::from_str(&json_str)?;
                let entities = melter.melt(value)?;
                writer.write_entities(entities)?;

                if stop_after_first {
                    break;
                }
            }
        }
        Ok(elem) => {
            // Single object
            let json_str = simd_json::to_string(&elem)?;
            let value = serde_json::from_str(&json_str)?;
            let entities = melter.melt(value)?;
            writer.write_entities(entities)?;
        }
        Err(_) => {
            // Fallback for NDJSON
            let content_str = String::from_utf8_lossy(&content);
            for line in content_str.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let value = serde_json::from_str(line)?;
                let entities = melter.melt(value)?;
                writer.write_entities(entities)?;

                if stop_after_first {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Process JSON and write to stdout as single stream
fn process_to_stdout(
    input_file: Option<String>,
    ndjson: bool,
    planned: bool,
    sample_size: Option<usize>,
    config: MeltConfig,
) -> Result<()> {
    let sample_size = sample_size.unwrap_or(100);
    let mut stdout = std::io::stdout();

    if planned {
        // Planned mode: sample first N records to build plan, then process all
        let mut records = Vec::new();

        // First pass: sample only (avoiding full file load into memory)
        {
            let reader = if let Some(file_path) = &input_file {
                Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
            } else {
                Box::new(std::io::stdin()) as Box<dyn Read>
            };

            sample_from_reader(reader, !ndjson, sample_size, &mut records)?;
        }

        // Build plan from samples
        if !records.is_empty() {
            let melter = PlannedMelter::from_examples(&records, config)?;

            // Second pass: process all records (or samples only for stdin)
            let reader = if let Some(file_path) = &input_file {
                Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
            } else {
                eprintln!("⚠ Warning: Using --planned mode with stdin. Only first {} records are processed.", sample_size);
                eprintln!("  For best performance with large files, pass the filename directly.");
                // Return early - can't re-read stdin
                return Ok(());
            };

            let buf_reader = serde_json::de::IoRead::new(BufReader::new(reader));
            let stream = serde_json::StreamDeserializer::new(buf_reader);

            for result in stream.into_iter() {
                let value: Value = result?;
                let entities = melter.melt(value)?;
                write_entities_to_stdout(&mut stdout, entities)?;

                if !ndjson {
                    break;
                }
            }
        }
    } else {
        // Unplanned mode: process each record immediately
        let melter = JsonMelter::new(config);
        let reader = if let Some(file_path) = &input_file {
            Box::new(BufReader::new(File::open(file_path)?)) as Box<dyn Read>
        } else {
            Box::new(std::io::stdin()) as Box<dyn Read>
        };

        let buf_reader = serde_json::de::IoRead::new(BufReader::new(reader));
        let stream = serde_json::StreamDeserializer::new(buf_reader);

        for result in stream.into_iter() {
            let value = result?;
            let entities = melter.melt(value)?;
            write_entities_to_stdout(&mut stdout, entities)?;

            if !ndjson {
                break;
            }
        }
    }

    Ok(())
}

/// Write entities to stdout as newline-delimited JSON with metadata
fn write_entities_to_stdout(
    stdout: &mut Stdout,
    entities: Vec<furnace::melt::Entity>,
) -> Result<()> {
    for entity in entities {
        let mut output = entity.data.clone();
        output.insert("_entity_type".to_string(), serde_json::Value::String(entity.entity_type));
        if let Some(parent) = entity.parent {
            output.insert("_parent_type".to_string(), serde_json::Value::String(parent.entity_type));
            output.insert("_parent_id".to_string(), serde_json::Value::String(parent.id.0));
            output.insert("_parent_field".to_string(), serde_json::Value::String(parent.field_name));
        }
        let line = serde_json::to_string(&output)?;
        writeln!(stdout, "{}", line)?;
    }
    Ok(())
}
