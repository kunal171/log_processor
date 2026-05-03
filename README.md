# Multi-threaded Log Processor

A Rust log processor that compares five processing strategies and benchmarks their performance.

## What It Does

Reads `logs.txt` and counts log levels (`ERROR`, `WARN`/`WARNING`, `INFO`) using five different approaches, printing timing for each.

Log level is determined by the first whitespace-separated token on each line:

```text
ERROR database failed      -> ERROR
WARN disk almost full      -> WARNING
WARNING retry threshold    -> WARNING
INFO server started        -> INFO
DEBUG previous ERROR seen  -> unknown (total_lines still increments)
```

## Processing Strategies

1. **Single-threaded** — simple loop over preloaded lines
2. **Channel-based multi-threaded** — scoped workers send local `LogStats` through `mpsc` channel, main thread merges
3. **Shared mutex multi-threaded** — scoped workers build local stats, lock `Arc<Mutex<LogStats>>` once to merge
4. **Streaming (lines)** — `BufReader::lines()`, allocates a `String` per line
5. **Streaming (reusable buffer)** — `BufReader::read_line()` with one reused `String`

## Project Structure

```text
multi_threaded_log_processor/
├── Cargo.toml
├── logs.txt
└── src/
    ├── main.rs        — file reading, timing, output comparison
    ├── log_stats.rs   — LogStats struct, process_line, merge, unit tests
    └── processor.rs   — all five processing strategies, unit tests
```

## Architecture

### Preloaded strategies

```text
logs.txt
    |
fs::read_to_string() -> lines().collect::<Vec<String>>()
    |
single-threaded:    iterate &lines, build LogStats
channel-based:      chunk &lines -> scoped workers -> mpsc -> merge
shared mutex:       chunk &lines -> scoped workers -> lock once -> merge
```

### Streaming strategies

```text
logs.txt
    |
File::open() -> BufReader
    |
.lines():       new String per line -> process -> discard
read_line():    reuse one String -> process -> clear -> repeat
```

## Usage

```bash
# Run all strategies with timing
cargo run

# Optimized benchmarking
cargo run --release

# Run tests
cargo test

# Check formatting and lints
cargo fmt --check
cargo clippy
```

## Dependencies

- `num_cpus` — detect available CPU threads for worker count

## Example Output

```text
Read + Collect:
Log lines loaded: 184512
Execution time: 30.615203ms

Single-threaded Process Only:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Execution time: 85.790197ms

Processing 184512 log lines with 12 workers
Multi Threaded Log Analysis:
Process-only execution time: 17.627876ms

Shared Mutex Log Analysis:
Process-only execution time: 20.854808ms

Streaming Log Analysis With BufReader::lines:
Execution time: 124.010925ms

Streaming Log Analysis With Reusable Buffer:
Execution time: 119.533088ms
```

## Benchmarking Notes

Fair comparison requires measuring the same work. The single-threaded timer starts after loading lines into memory, but streaming timers include file I/O. The fair comparison is:

```text
read + collect + process   vs   BufReader stream + process
```

Streaming is mainly useful for keeping memory usage low on large files, not guaranteed to be faster than processing preloaded data.

## Tests

Tests in `log_stats.rs`:

- zero initialization
- ERROR, WARN, WARNING, INFO counting
- unknown lines increment total_lines only
- first-token parsing priority
- merge combines two LogStats

Tests in `processor.rs`:

- empty input
- normal multi-threaded processing
- more threads than lines
- single-thread processing
- zero-thread fallback
