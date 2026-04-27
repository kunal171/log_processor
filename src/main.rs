use std::fs;
use std::time::Instant;

mod log_stats;
mod processor;

fn main() {
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");
    let thread_count = num_cpus::get();
    let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();
    // Read log file and prepare lines for processing
    let single_start = Instant::now();
    let single_stats = processor::process_logs_singlethreaded(&lines);
    let single_elapsed = single_start.elapsed();

    println!("Single-threaded:");
    println!("Total lines processed: {}", single_stats.total_lines);
    println!("ERROR count: {}", single_stats.error_count);
    println!("WARNING count: {}", single_stats.warning_count);
    println!("INFO count: {}", single_stats.info_count);
    println!("Execution time: {single_elapsed:#?}");

    // Process logs using multiple threads
    let start = Instant::now();


    println!(
        "Processing {} log lines with {} workers",
        lines.len(),
        thread_count
    );

    let stats = processor::process_logs_multithreaded(&lines, thread_count);
    let elapsed = start.elapsed();

    println!("Multi Threaded Log Analysis:");
    println!("Total lines processed: {}", stats.total_lines);
    println!("ERROR count: {}", stats.error_count);
    println!("WARNING count: {}", stats.warning_count);
    println!("INFO count: {}", stats.info_count);
    println!("Log level counts: {:?}", stats.log_level_counts);
    println!("Execution time: {elapsed:#?}");
}
