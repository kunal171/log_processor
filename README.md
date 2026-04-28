# Multi-threaded Log Processor

A Rust learning project for practicing practical concurrency, log processing, and
benchmarking trade-offs.

The project compares different ways to process a log file:

- single-threaded processing over already-loaded lines
- multi-threaded processing with scoped worker threads
- streaming file processing with `BufReader`

The goal is not only to make the fastest version, but to understand when each
approach is useful.

## What It Does

The processor reads `logs.txt` and counts log levels:

- `ERROR`
- `WARN` and `WARNING`
- `INFO`
- total lines processed

`WARN` and `WARNING` are normalized into the same warning count.

The parser checks only the first whitespace-separated token on each line.

Example:

```text
ERROR database failed      -> counts as ERROR
WARN disk almost full      -> counts as WARNING
WARNING retry threshold    -> counts as WARNING
INFO server started        -> counts as INFO
DEBUG previous ERROR seen  -> counts as unknown, but total_lines still increments
```

This avoids false positives from searching for `ERROR`, `WARN`, or `INFO`
anywhere in the line.

## Project Structure

```text
multi_threaded_log_processor/
├── Cargo.toml
├── logs.txt
├── README.md
└── src/
    ├── main.rs
    ├── log_stats.rs
    └── processor.rs
```

### `src/main.rs`

Coordinates the program:

- reads `logs.txt`
- prepares lines for the existing processors
- measures execution time with `Instant`
- prints stats for each processing approach

### `src/log_stats.rs`

Defines the core stats type:

```rust
pub struct LogStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_lines: usize,
    pub log_level_counts: HashMap<String, usize>,
}
```

It also contains:

- `LogStats::new()`
- `LogStats::process_line()`
- `LogStats::merge()`
- unit tests for parsing and merging

### `src/processor.rs`

Contains the processing strategies:

- single-threaded processing
- multi-threaded chunk processing
- streaming processing with `BufReader`

The multi-threaded version uses:

- `std::thread::scope`
- borrowed `&[String]` chunks
- `std::sync::mpsc`
- local worker stats
- final merge on the main thread

## Current Architecture

High-level flow:

```text
logs.txt
    |
fs::read_to_string()
    |
logs.lines().map(String::from).collect::<Vec<String>>()
    |
single-threaded baseline over &lines
    |
multi-threaded processing over &lines
    |
lines.chunks(chunk_size)
    |
scoped worker threads borrow &[String] chunks
    |
workers send LogStats through mpsc::Sender
    |
main receives from mpsc::Receiver
    |
final_stats.merge(local_stats)
```

Streaming flow:

```text
logs.txt
    |
File::open()
    |
BufReader::new(file)
    |
read one line at a time
    |
process line immediately
    |
discard line and continue
```

## Running The Project

From this directory:

```bash
cargo run
```

For optimized benchmarking:

```bash
cargo run --release
```

Run tests:

```bash
cargo test
```

Check formatting:

```bash
cargo fmt --check
```

Run Clippy:

```bash
cargo clippy
```

## Example Output

Example benchmark output:

```text
Single-threaded:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Execution time: 91.802817ms

Processing 184512 log lines with 12 workers
Multi Threaded Log Analysis:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"ERROR": 46128, "INFO": 92256, "WARNING": 46128}
Execution time: 21.837884ms

Streaming Log Analysis:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"ERROR": 46128, "INFO": 92256, "WARNING": 46128}
Execution time: 149.899483ms
```

## Benchmarking Lesson

The streaming version may look slower at first.

That does not mean `BufReader` is bad.

The important detail is what each timer includes.

If the single-threaded timer starts after:

```rust
let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");
let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();
```

then it only measures processing already-loaded memory.

But the streaming version measures:

```text
file I/O + line reading + processing
```

So the fair comparison is:

```text
read_to_string + collect Vec<String> + process
```

against:

```text
BufReader stream + process
```

Streaming is mainly useful because it keeps memory usage low for large files.
It is not guaranteed to beat processing data that is already loaded in memory.

## Rust Concepts Practiced

- structs and methods with `impl`
- `HashMap<String, usize>` counting
- matching on parsed log levels
- `Instant` benchmarking
- single-threaded baseline measurement
- chunking slices with `.chunks()`
- `usize::div_ceil()`
- scoped threads with `std::thread::scope`
- message passing with `std::sync::mpsc`
- local worker state and final merging
- avoiding unnecessary `String` cloning
- streaming input with `File`, `BufReader`, and `BufRead`

## Tests

The test suite covers:

- empty stats initialization
- counting `ERROR`
- counting `WARN` as `WARNING`
- counting `WARNING`
- counting `INFO`
- unknown log lines still incrementing total lines
- first-token parsing behavior
- merging two `LogStats` values
- multi-threaded empty input
- multi-threaded normal input
- more threads than lines
- one-thread processing
- zero-thread fallback to one worker

Run:

```bash
cargo test
```

## Next Steps

Recommended next learning steps:

1. Compare `BufReader + lines()` with `BufReader + read_line()` using a reusable buffer.
2. Make benchmarks fair by timing `read_to_string + collect + process` together.
3. Add CLI arguments for log file path and worker count.
4. Implement an `Arc<Mutex<LogStats>>` version to learn shared mutable state.
5. Compare channel-based local stats with lock-based shared stats.
6. Build a small thread pool so workers are reused instead of spawned every run.

## Learning Note

Multi-threading is not automatically faster.

It adds overhead:

- spawning threads
- splitting work into chunks
- sending messages
- receiving messages
- merging results

For small workloads, single-threaded code can win. For larger workloads,
multi-threading can help when the work is heavy enough to justify the overhead.
