use std::fs;


mod log_stats;
use log_stats::LogStats;

fn main(){
    // Read the log file content into a string
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");

    // Create a new instance of LogStats to hold the analysis results
    let mut stats = LogStats::new();

    // Process each line of the log file and update the counts in LogStats
    for line in logs.lines() {
        stats.process_line(line);
    }

    // Print the analysis results
    println!("Log Analysis:");
    println!("ERROR count: {}", stats.error_count);
    println!("WARNING count: {}", stats.warning_count);
    println!("INFO count: {}", stats.info_count);
    println!("Log level counts: {:?}", stats.log_level_counts);
}