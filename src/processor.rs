use crate::log_stats::LogStats;
use std::thread;
use std::sync::mpsc;

pub fn process_logs_multithreaded(lines: Vec<String>, thread_count: usize) -> LogStats {
    if lines.is_empty() {
        return LogStats::new();
    }

    let chunk_size = (lines.len() + thread_count - 1) / thread_count; // Calculate chunk size for each thread

    let (tx, rx) = mpsc::channel();

    for (index, chunks) in lines.chunks(chunk_size).enumerate() {
        let thread_id = index + 1; // Thread IDs start from 1
        let chunk = chunks.to_vec();
        println!("Thread {} spawned", thread_id);
    

        // Clone the transmitter for this thread to use
        let tx = tx.clone();
        // Spawn a thread to process the assigned chunk of log lines
        thread::spawn(move || {
            // println!("Thread {} processing {} lines", thread_id, chunk.len());
            // println!("Thread {} started", thread_id);
            // Create a local LogStats instance for this thread to accumulate results
            let mut local_stats = LogStats::new();
            // Process each line in the assigned chunk and update the local LogStats instance
            for line in chunk {
                local_stats.process_line(&line);
            }

            println!("Thread {} finished", thread_id);
            tx.send(local_stats).expect("Failed to send stats");
                            
  
        });
        // handles.push(handle);
      
    }

          drop(tx);
    // Wait for all threads to finish and merge their results into a final LogStats instance
    let mut final_stats = LogStats::new();
    // Join each thread and merge its local LogStats into the final LogStats instance
    for local_stats in rx {
        final_stats.merge(&local_stats);
    }
    final_stats
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
        let stats = process_logs_multithreaded(Vec::new(), 4);

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

        let stats = process_logs_multithreaded(input, 3);

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

        let stats = process_logs_multithreaded(input, 8);

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

        let stats = process_logs_multithreaded(input, 1);

        assert_eq!(stats.error_count, 2);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.total_lines, 5);
        assert_eq!(count_for(&stats, "ERROR"), 2);
        assert_eq!(count_for(&stats, "WARNING"), 1);
        assert_eq!(count_for(&stats, "INFO"), 1);
    }
}
