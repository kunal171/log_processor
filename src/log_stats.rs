use std::fs;

use std::collections::HashMap;

// Struct to represent the analysis results of a log file
pub struct LogStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub log_level_counts: HashMap<String, usize>,
}

// Implementation of LogStats struct to provide methods for processing log lines and updating counts
impl LogStats {
    // Constructor to create a new instance of LogStats with initial counts set to zero and an empty HashMap for log level counts
    pub fn new() -> Self {
        LogStats {
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            log_level_counts: HashMap::new(),
        }
    }

    // Method to process a single line of log and update the counts based on the log level (ERROR, WARNING, INFO)
    pub fn process_line(&mut self, line: &str) {
        if line.contains("ERROR") {
            self.error_count += 1;
            *self.log_level_counts.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if line.contains("WARN")|| line.contains("WARNING"){
            self.warning_count += 1;
            *self.log_level_counts.entry("WARNING".to_string()).or_insert(0) += 1;
        } else if line.contains("INFO") {
            self.info_count += 1;
            *self.log_level_counts.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }
}