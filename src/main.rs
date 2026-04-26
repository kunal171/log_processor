use std::fs;
use std::time::Instant;
use num_cpus;

mod log_stats;
mod processor;

fn main(){
    // Start measuring the time taken for the log processing
    let start = Instant::now();

    // Read the log file content into a string
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");

    // Get the number of CPU cores available to determine how many threads to spawn for processing
    let thread_count = num_cpus::get();
    println!("Processing logs with {} threads...", thread_count);

    // Split the log content into lines and collect them into a vector for processing
    let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();
    println!("Total lines to process: {}", lines.len());

    // Process each line of the log file and update the counts in LogStats
    let stats = processor::process_logs_multithreaded(lines, thread_count);

    // End timing
    let elapsed = start.elapsed();

    // Print the analysis results
    println!("Log Analysis:");
    println!("ERROR count: {}", stats.error_count);
    println!("WARNING count: {}", stats.warning_count);
    println!("INFO count: {}", stats.info_count);
    println!("Log level counts: {:?}", stats.log_level_counts);
    println!("Total lines processed: {}", stats.total_lines);
    println!("\nExecution time: {:#?}", elapsed);  // Formatted output
}