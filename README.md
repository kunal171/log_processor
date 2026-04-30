# Multi-threaded Log Processor

A Rust learning project for practicing practical concurrency, log processing,
streaming input, shared state, and benchmarking trade-offs.

This is the Phase 1B project in the Rust foundations track.

Phase 1B core is complete. Remaining work is optional experimentation.

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

## Processing Strategies

The project compares several approaches:

- single-threaded processing over already-loaded lines
- channel-based multi-threaded processing with scoped worker threads
- shared-state multi-threaded processing with `Arc<Mutex<LogStats>>`
- streaming file processing with `BufReader::lines()`
- streaming file processing with `read_line(&mut line)` and a reusable buffer

The goal is not only to make the fastest version, but to understand when each
approach is useful.

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
- measures read/collect time separately from process-only time
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
- channel-based multi-threaded chunk processing
- shared-mutex multi-threaded chunk processing
- simple streaming with `BufReader::lines()`
- optimized streaming with a reusable `String` buffer

## Architecture

Read and collect flow:

```text
logs.txt
    |
fs::read_to_string()
    |
logs.lines().map(String::from).collect::<Vec<String>>()
    |
measure read + collect time
```

Single-threaded flow:

```text
Vec<String>
    |
process_logs_singlethreaded(&lines)
    |
final LogStats
```

Channel-based multi-threaded flow:

```text
Vec<String>
    |
split into chunks
    |
scoped worker threads borrow &[String] chunks
    |
each worker builds local LogStats
    |
worker sends LogStats through mpsc::Sender
    |
main receives from mpsc::Receiver
    |
main merges worker stats
```

Shared-mutex multi-threaded flow:

```text
Vec<String>
    |
split into chunks
    |
scoped worker threads borrow &[String] chunks
    |
each worker builds local LogStats
    |
worker locks Arc<Mutex<LogStats>> once
    |
worker merges local stats into shared stats
```

Streaming flow:

```text
logs.txt
    |
File::open()
    |
BufReader::new(file)
    |
read one line at a time with .lines()
or reuse one String with read_line(&mut line)
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
Read + Collect:
Log lines loaded: 184512
Execution time: 30.615203ms

Single-threaded Process Only:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Execution time: 85.790197ms

Read + Collect + Single-threaded Process:
Total lines processed: 184512
Execution time: 116.4054ms

Processing 184512 log lines with 12 workers
Multi Threaded Log Analysis:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"INFO": 92256, "ERROR": 46128, "WARNING": 46128}
Process-only execution time: 17.627876ms
Read + Collect + Multi-threaded Process: 48.243079ms

Shared Mutex Log Analysis:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"ERROR": 46128, "INFO": 92256, "WARNING": 46128}
Process-only execution time: 20.854808ms
Read + Collect + Shared Mutex Process: 51.470011ms

Streaming Log Analysis With BufReader::lines:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"INFO": 92256, "WARNING": 46128, "ERROR": 46128}
Execution time: 124.010925ms

Streaming Log Analysis With Reusable Buffer:
Total lines processed: 184512
ERROR count: 46128
WARNING count: 46128
INFO count: 92256
Log level counts: {"INFO": 92256, "WARNING": 46128, "ERROR": 46128}
Execution time: 119.533088ms
```

## Benchmarking Lessons

Benchmarking has to compare the same amount of work.

If the single-threaded timer starts after this work:

```rust
let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");
let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();
```

then it only measures processing already-loaded memory.

But streaming measures:

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

In this project, the fairer comparison showed:

```text
read + collect + channel multi-threaded  -> fastest current full path
read + collect + shared mutex            -> slightly slower, but close
reusable-buffer streaming                -> better than simple .lines() streaming
read + collect + single-threaded         -> slower than both threaded versions
```

Streaming is mainly useful because it keeps memory usage low for large files.
It is not guaranteed to beat processing data that is already loaded in memory.

## `Arc<Mutex<LogStats>>` Lesson

The shared-mutex version teaches safe shared mutable state across threads.

Definitions:

- `Arc<T>` means atomic reference counting.
- `Arc<T>` lets multiple threads share ownership of the same value.
- `Mutex<T>` means mutual exclusion.
- `Mutex<T>` allows only one thread at a time to mutate protected data.
- `Arc<Mutex<T>>` lets many threads own the same protected value safely.

This project uses:

```rust
Arc<Mutex<LogStats>>
```

The good version does not lock for every line.

Instead:

```text
worker processes chunk locally
    |
worker creates local LogStats
    |
worker locks shared stats once
    |
worker merges local stats
```

That is much better than:

```text
for every line:
    lock shared stats
    process line
    unlock shared stats
```

The general rule:

```text
do expensive work locally
lock shared state briefly
apply final update
unlock quickly
```

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
- shared ownership with `Arc<T>`
- shared mutation with `Mutex<T>`
- lock granularity
- avoiding unnecessary `String` cloning
- streaming input with `File`, `BufReader`, and `BufRead`
- fair benchmark design

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

## Phase 1B Status

Phase 1B core is complete.

Completed learning goals:

- single-threaded baseline
- channel-based multi-threading
- scoped threads
- borrowed chunk processing
- `mpsc` message passing
- `Arc<Mutex<LogStats>>` shared state
- fair benchmarking
- streaming with `BufReader`
- reusable-buffer streaming

Optional follow-ups:

1. Add an intentionally slow mutex-per-line version to show lock overhead.
2. Add tests for `process_logs_with_shared_mutex`.
3. Implement streaming multi-threaded batch processing.
4. Build a basic thread pool so workers are reused instead of spawned every run.
5. Add CLI arguments for log file path and worker count.

Recommended next project:

Move to Phase 1C: `web_crawler`.

## Learning Note

Multi-threading is not automatically faster.

It adds overhead:

- spawning threads
- splitting work into chunks
- sending messages
- receiving messages
- merging results
- locking shared state

For small workloads, single-threaded code can win. For larger workloads,
multi-threading can help when the work is heavy enough to justify the overhead.
