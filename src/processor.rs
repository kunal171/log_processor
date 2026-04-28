use crate::log_stats::LogStats;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

pub fn process_logs_multithreaded(lines: &[String], thread_count: usize) -> LogStats {
    if lines.is_empty() {
        return LogStats::new();
    }

    let thread_count = thread_count.max(1);
    let chunk_size = lines.len().div_ceil(thread_count);
    let (tx, rx) = mpsc::channel();

    // let mut handles = Vec::new();

    // Spawn worker threads to process chunks of log lines

    thread::scope(|scope| {
        for chunk in lines.chunks(chunk_size) {
            let tx = tx.clone();

            scope.spawn(move || {
                let mut local_stats = LogStats::new();

                for line in chunk {
                    local_stats.process_line(line);
                }

                tx.send(local_stats).expect("Failed to send stats");
            });
        }

        drop(tx);
    });

    let mut final_stats = LogStats::new();

    for local_stats in rx {
        final_stats.merge(&local_stats);
    }

    // // Wait for all worker threads to finish
    // for handle in handles {
    //     handle.join().expect("Worker thread panicked");
    // }

    final_stats
}

// A single-threaded version of the log processing for comparison and testing purposes
pub fn process_logs_singlethreaded(lines: &[String]) -> LogStats {
    let mut stats = LogStats::new();

    for line in lines {
        stats.process_line(line);
    }

    stats
}

// A simple streaming version that reads the file line by line.
// This is easy to read, but reader.lines() allocates a new String per line.
pub fn process_log_file_streaming_lines(file_path: &str) -> io::Result<LogStats> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut stats = LogStats::new();

    for line_result in reader.lines() {
        let line = line_result?;
        stats.process_line(&line);
    }

    Ok(stats)
}

// A streaming version that reuses one String buffer for all lines.
// This avoids allocating a fresh String for every line.
pub fn process_log_file_streaming_reuse_buffer(file_path: &str) -> io::Result<LogStats> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let mut stats = LogStats::new();
    let mut line = String::new();

    loop {
        line.clear();

        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }

        stats.process_line(line.trim_end());
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(values: &[&str]) -> Vec<String> {
        values.iter().map(|line| line.to_string()).collect()
    }

    fn count_for(stats: &LogStats, level: &str) -> usize {
        stats.log_level_counts.get(level).copied().unwrap_or(0)
    }

    #[test]
    fn returns_empty_stats_for_empty_input() {
        let stats = process_logs_multithreaded(&[], 4);

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 0);
        assert!(stats.log_level_counts.is_empty());
    }

    #[test]
    fn processes_logs_across_multiple_threads() {
        let input = lines(&[
            "INFO server started",
            "ERROR database unavailable",
            "WARN disk almost full",
            "DEBUG cache warmed",
            "WARNING retry threshold reached",
            "INFO request completed",
        ]);

        let stats = process_logs_multithreaded(&input, 3);

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 2);
        assert_eq!(stats.info_count, 2);
        assert_eq!(stats.total_lines, 6);
        assert_eq!(count_for(&stats, "ERROR"), 1);
        assert_eq!(count_for(&stats, "WARNING"), 2);
        assert_eq!(count_for(&stats, "INFO"), 2);
    }

    #[test]
    fn handles_more_threads_than_lines() {
        let input = lines(&["ERROR one", "INFO two"]);

        let stats = process_logs_multithreaded(&input, 8);

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.total_lines, 2);
        assert_eq!(count_for(&stats, "ERROR"), 1);
        assert_eq!(count_for(&stats, "INFO"), 1);
    }

    #[test]
    fn produces_same_result_with_one_thread() {
        let input = lines(&[
            "ERROR one",
            "ERROR two",
            "WARN three",
            "INFO four",
            "TRACE five",
        ]);

        let stats = process_logs_multithreaded(&input, 1);

        assert_eq!(stats.error_count, 2);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.total_lines, 5);
        assert_eq!(count_for(&stats, "ERROR"), 2);
        assert_eq!(count_for(&stats, "WARNING"), 1);
        assert_eq!(count_for(&stats, "INFO"), 1);
    }

    #[test]
    fn treats_zero_threads_as_one_thread() {
        let input = lines(&["ERROR one", "WARN two", "INFO three"]);

        let stats = process_logs_multithreaded(&input, 0);

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.total_lines, 3);
    }
}
