use std::thread;    
use crate::log_stats::LogStats;

pub fn process_logs_multithreaded(lines: Vec<String>, thread_count: usize) -> LogStats {

    if lines.is_empty() {
        return LogStats::new();
    }

    let chunk_size = (lines.len() + thread_count - 1) / thread_count; // Calculate chunk size for each thread
    // println!("Chunk size for each thread: {}", chunk_size);
    // println!("Spawning threads to process log lines...");
    // println!("Starting log processing...");

    let mut handles = Vec::new();

    for (index, chunks) in lines.chunks(chunk_size).enumerate() {
        let thread_id = index + 1; // Thread IDs start from 1
        let chunk = chunks.to_vec();
        println!("Thread {} spawned", thread_id);

        // Spawn a thread to process the assigned chunk of log lines
        let handle = thread::spawn(move || {
            // println!("Thread {} processing {} lines", thread_id, chunk.len());
            // println!("Thread {} started", thread_id);
            // Create a local LogStats instance for this thread to accumulate results
            let mut local_stats = LogStats::new();
            // Process each line in the assigned chunk and update the local LogStats instance
            for line in chunk {
                local_stats.process_line(&line);
            }

            println!("Thread {} finished", thread_id);
            local_stats
        });
        handles.push(handle);
    }

    // Wait for all threads to finish and merge their results into a final LogStats instance
    let mut final_stats = LogStats::new();
    // Join each thread and merge its local LogStats into the final LogStats instance
    for handle in handles {
        let local_stats = handle.join().expect("Thread panicked");
        final_stats.merge(&local_stats);
    }
    final_stats
}