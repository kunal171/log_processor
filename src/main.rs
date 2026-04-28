use std::fs;
use std::time::Instant;

mod log_stats;
mod processor;

fn main() {
    let file_path = "logs.txt";
    let thread_count = num_cpus::get();

    let load_start = Instant::now();
    let logs = fs::read_to_string(file_path).expect("Failed to read log file");
    let lines: Vec<String> = logs.lines().map(|line| line.to_string()).collect();
    let load_elapsed = load_start.elapsed();

    let single_start = Instant::now();
    let single_stats = processor::process_logs_singlethreaded(&lines);
    let single_elapsed = single_start.elapsed();
    let single_total_elapsed = load_elapsed + single_elapsed;

    println!("Read + Collect:");
    println!("Log lines loaded: {}", lines.len());
    println!("Execution time: {load_elapsed:#?}");
    println!();

    println!("Single-threaded Process Only:");
    println!("Total lines processed: {}", single_stats.total_lines);
    println!("ERROR count: {}", single_stats.error_count);
    println!("WARNING count: {}", single_stats.warning_count);
    println!("INFO count: {}", single_stats.info_count);
    println!("Execution time: {single_elapsed:#?}");
    println!();

    println!("Read + Collect + Single-threaded Process:");
    println!("Total lines processed: {}", single_stats.total_lines);
    println!("Execution time: {single_total_elapsed:#?}");
    println!();

    let multi_start = Instant::now();
    let multi_stats = processor::process_logs_multithreaded(&lines, thread_count);
    let multi_elapsed = multi_start.elapsed();
    let multi_total_elapsed = load_elapsed + multi_elapsed;

    println!(
        "Processing {} log lines with {thread_count} workers",
        lines.len()
    );
    println!("Multi Threaded Log Analysis:");
    println!("Total lines processed: {}", multi_stats.total_lines);
    println!("ERROR count: {}", multi_stats.error_count);
    println!("WARNING count: {}", multi_stats.warning_count);
    println!("INFO count: {}", multi_stats.info_count);
    println!("Log level counts: {:?}", multi_stats.log_level_counts);
    println!("Process-only execution time: {multi_elapsed:#?}");
    println!("Read + Collect + Multi-threaded Process: {multi_total_elapsed:#?}");
    println!();

    let streaming_lines_start = Instant::now();
    let streaming_lines_stats = processor::process_log_file_streaming_lines(file_path)
        .expect("Failed to process log file with BufReader lines");
    let streaming_lines_elapsed = streaming_lines_start.elapsed();

    println!("Streaming Log Analysis With BufReader::lines:");
    println!(
        "Total lines processed: {}",
        streaming_lines_stats.total_lines
    );
    println!("ERROR count: {}", streaming_lines_stats.error_count);
    println!("WARNING count: {}", streaming_lines_stats.warning_count);
    println!("INFO count: {}", streaming_lines_stats.info_count);
    println!(
        "Log level counts: {:?}",
        streaming_lines_stats.log_level_counts
    );
    println!("Execution time: {streaming_lines_elapsed:#?}");
    println!();

    let streaming_reuse_start = Instant::now();
    let streaming_reuse_stats = processor::process_log_file_streaming_reuse_buffer(file_path)
        .expect("Failed to process log file with reusable streaming buffer");
    let streaming_reuse_elapsed = streaming_reuse_start.elapsed();

    println!("Streaming Log Analysis With Reusable Buffer:");
    println!(
        "Total lines processed: {}",
        streaming_reuse_stats.total_lines
    );
    println!("ERROR count: {}", streaming_reuse_stats.error_count);
    println!("WARNING count: {}", streaming_reuse_stats.warning_count);
    println!("INFO count: {}", streaming_reuse_stats.info_count);
    println!(
        "Log level counts: {:?}",
        streaming_reuse_stats.log_level_counts
    );
    println!("Execution time: {streaming_reuse_elapsed:#?}");
}
