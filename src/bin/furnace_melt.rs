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

use anyhow::Result;
use clap::Parser;
use furnace::melt::{EntityWriter, JsonMelter, MeltConfig, PlannedMelter};
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Stdout, Write};

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

    // Create reader based on input source
    let reader: Box<dyn BufRead> = if let Some(file_path) = &args.input {
        Box::new(BufReader::new(File::open(file_path)?))
    } else {
        Box::new(BufReader::new(stdin()))
    };

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

        process_to_files(reader, ".".to_string(), args.ndjson, args.planned, args.sample_size, config)?;

        std::env::set_current_dir(original_dir)?;
    } else {
        // Single stream to stdout
        process_to_stdout(reader, args.ndjson, args.planned, args.sample_size, config)?;
    }

    Ok(())
}

/// Process JSON and write to separate files per entity type
fn process_to_files(
    reader: Box<dyn BufRead>,
    output_dir: String,
    ndjson: bool,
    planned: bool,
    sample_size: Option<usize>,
    config: MeltConfig,
) -> Result<()> {
    let sample_size = sample_size.unwrap_or(100);
    let mut writer = EntityWriter::new_file_writer(&output_dir)?;

    if planned {
        // Planned mode: sample first N records, build plan, then process all
        let mut records = Vec::new();
        let mut all_lines: Vec<String> = Vec::new();

        // Collect all lines first
        for line in reader.lines() {
            let line = line?;
            let line = line.trim().to_string();
            if !line.is_empty() {
                all_lines.push(line);
            }
            if !ndjson && !all_lines.is_empty() {
                break;
            }
        }

        // Build samples for plan
        for (i, line) in all_lines.iter().enumerate() {
            if i < sample_size {
                let value: Value = serde_json::from_str(line)?;
                records.push(value);
            }
        }

        // Build plan from samples
        if !records.is_empty() {
            let melter = PlannedMelter::from_examples(&records, config)?;

            // Process all records with the plan
            for line in all_lines {
                let value: Value = serde_json::from_str(&line)?;
                let entities = melter.melt(value)?;
                writer.write_entities(entities)?;
            }
        }
    } else {
        // Unplanned mode: process each record immediately
        let melter = JsonMelter::new(config);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line)?;
            let entities = melter.melt(value)?;
            writer.write_entities(entities)?;

            if !ndjson {
                break;
            }
        }
    }

    writer.flush()?;
    Ok(())
}

/// Process JSON and write to stdout as single stream
fn process_to_stdout(
    reader: Box<dyn BufRead>,
    ndjson: bool,
    planned: bool,
    sample_size: Option<usize>,
    config: MeltConfig,
) -> Result<()> {
    let sample_size = sample_size.unwrap_or(100);
    let mut stdout = std::io::stdout();

    if planned {
        // Planned mode: collect all lines, sample first N, build plan, process all
        let mut records = Vec::new();
        let mut all_lines: Vec<String> = Vec::new();

        // Collect all lines first
        for line in reader.lines() {
            let line = line?;
            let line = line.trim().to_string();
            if !line.is_empty() {
                all_lines.push(line);
            }
            if !ndjson && !all_lines.is_empty() {
                break;
            }
        }

        // Build samples for plan
        for (i, line) in all_lines.iter().enumerate() {
            if i < sample_size {
                let value: Value = serde_json::from_str(line)?;
                records.push(value);
            }
        }

        // Build plan from samples
        if !records.is_empty() {
            let melter = PlannedMelter::from_examples(&records, config)?;

            // Process all records with the plan
            for line in all_lines {
                let value: Value = serde_json::from_str(&line)?;
                let entities = melter.melt(value)?;
                write_entities_to_stdout(&mut stdout, entities)?;
            }
        }
    } else {
        // Unplanned mode: process each record immediately
        let melter = JsonMelter::new(config);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line)?;
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
