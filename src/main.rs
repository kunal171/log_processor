use std::fs;
use std::time::Instant;

mod log_stats;
mod processor;

fn main() {
    let start = Instant::now();
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");
    let thread_count = num_cpus::get();
    let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();

    println!(
        "Processing {} log lines with {} workers",
        lines.len(),
        thread_count
    );

    let stats = processor::process_logs_multithreaded(lines, thread_count);
    let elapsed = start.elapsed();

    println!("Log Analysis:");
    println!("Total lines processed: {}", stats.total_lines);
    println!("ERROR count: {}", stats.error_count);
    println!("WARNING count: {}", stats.warning_count);
    println!("INFO count: {}", stats.info_count);
    println!("Log level counts: {:?}", stats.log_level_counts);
    println!("Execution time: {elapsed:#?}");
}
