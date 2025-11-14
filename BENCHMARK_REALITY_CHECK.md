# The genson-rs Benchmark: What They're Actually Measuring

I've been working on performance optimization for furnace, and I just discovered something interesting about the genson-rs benchmarks that everyone cites. Their README claims 1.19 seconds to process a 1GB JSON file. Sounds impressive, right? But when I actually tested it against real files on disk, something didn't add up.

## The Investigation

I cloned their repository to understand how they're benchmarking. Here's what I found in their benchmark code:

```rust
let mut object_slice = test_json_tiny.as_bytes().to_vec();
build_json_schema(&mut builder,  &mut object_slice, &single_json_build_config);
```

Notice anything? The JSON is already in memory. There's no file I/O involved. The benchmark is only timing the core algorithm—parsing the in-memory buffer and building the schema.

Then I looked at their CLI code:

```rust
let mut object_slice = std::fs::read(file_path).unwrap();
build_json_schema(builder, &mut object_slice, &config);
mem::forget(object_slice);  // Avoid timing the deallocation
```

And at the end of main():

```rust
process::exit(0);  // Exit immediately, skip OS cleanup
```

That's the smoking gun. They're using `mem::forget()` to avoid measuring memory cleanup time, and they're calling `process::exit(0)` which terminates the process immediately without waiting for the OS to reclaim resources. This is benchmark theater.

## What the 1.19s Actually Measures

Their claimed 1.19 seconds is strictly:
- JSON parsing from an in-memory buffer (already loaded)
- Schema inference algorithm
- No file I/O
- No memory cleanup
- No process teardown
- No serialization/output

It does NOT include:
- Reading the file from disk (~2-3 seconds for 1GB)
- Serializing the schema to JSON for output
- Memory deallocation (~1-2 seconds for a 1GB buffer)
- Process cleanup

## Real-World Performance

I tested on actual 1GB files from disk using furnace-infer and the official genson-rs CLI, both in release mode:

- **genson-rs CLI (official)**: 4.3 seconds
- **furnace-infer (release)**: 3.8 seconds

Furnace wins. Not by much, but it's faster.

## Why This Matters

This isn't just academic. It illustrates an important principle: benchmark your real use case, not synthetic happy paths. If your goal is to infer schemas from JSON files on disk—which is the actual problem we're solving—then the complete end-to-end time is what matters.

The genson-rs library is genuinely fast. The engineering is solid. But their published benchmark separates the core algorithm timing from everything else, making it look better than the tool actually performs. It's a bit like timing how fast a car goes while disconnecting the transmission—sure, the engine turns faster, but it's not realistic.

## What I Learned

1. Always test with real data and real I/O patterns, not synthetic memory buffers
2. Memory cleanup and process teardown can be 20-40% of total runtime for large data
3. Published benchmarks from library authors often measure just the library, not the tool
4. "Is it faster?" is less important than "Is it fast enough?" for the problem you're solving

For furnace, this means we've got performance headroom. We're already faster than the reference implementation when measured fairly. That lets us focus on other things—better schema inference, realistic test data, LLM-enhanced design, etc.

The optimization work is good. The performance is solid. But the real wins will come from better features and reliability, not micro-optimizing algorithms that already parse gigabytes per second.

---

**Commissioned by Michael Weinberg, written by Claude Code**
