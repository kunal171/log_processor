use std::fs;
use std::time::Instant;
use num_cpus;

mod log_stats;
use log_stats::LogStats;

fn main(){
    println!("Thread count: {}", num_cpus::get());
    println!("Current thread: {:?}", std::thread::current().id());
    // Start measuring the time taken for the log processing
    let start = Instant::now();

    // Read the log file content into a string
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");

    // Create a new instance of LogStats to hold the analysis results
    let mut stats = LogStats::new();

    // Process each line of the log file and update the counts in LogStats
    for line in logs.lines() {
        stats.process_line(line);
    }

    // End timing
    let elapsed = start.elapsed();

    // Print the analysis results
    println!("Log Analysis:");
    println!("ERROR count: {}", stats.error_count);
    println!("WARNING count: {}", stats.warning_count);
    println!("INFO count: {}", stats.info_count);
    println!("Log level counts: {:?}", stats.log_level_counts);
    println!("\nExecution time: {:#?}", elapsed);  // Formatted output
}