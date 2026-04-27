use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LogStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_lines: usize,
    pub log_level_counts: HashMap<String, usize>,
}

impl LogStats {
    pub fn new() -> Self {
        LogStats {
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            total_lines: 0,
            log_level_counts: HashMap::new(),
        }
    }

    pub fn process_line(&mut self, line: &str) {
        self.total_lines += 1;

        let level = line.split_whitespace().next();

        match level {
            Some("ERROR") => {
                self.error_count += 1;
                *self
                    .log_level_counts
                    .entry("ERROR".to_string())
                    .or_insert(0) += 1;
            }
            Some("WARN") | Some("WARNING") => {
                self.warning_count += 1;
                *self
                    .log_level_counts
                    .entry("WARNING".to_string())
                    .or_insert(0) += 1;
            }
            Some("INFO") => {
                self.info_count += 1;
                *self.log_level_counts.entry("INFO".to_string()).or_insert(0) += 1;
            }
            _ => {}
        }
    }

    pub fn merge(&mut self, other: &LogStats) {
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.info_count += other.info_count;
        self.total_lines += other.total_lines;

        for (level, count) in &other.log_level_counts {
            *self.log_level_counts.entry(level.clone()).or_insert(0) += count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_for(stats: &LogStats, level: &str) -> usize {
        stats.log_level_counts.get(level).copied().unwrap_or(0)
    }

    #[test]
    fn new_starts_with_zero_counts() {
        let stats = LogStats::new();

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 0);
        assert!(stats.log_level_counts.is_empty());
    }

    #[test]
    fn process_line_counts_error_logs() {
        let mut stats = LogStats::new();

        stats.process_line("ERROR database connection failed");

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 1);
        assert_eq!(count_for(&stats, "ERROR"), 1);
    }

    #[test]
    fn process_line_counts_warn_logs_as_warning() {
        let mut stats = LogStats::new();

        stats.process_line("WARN disk usage high");

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 1);
        assert_eq!(count_for(&stats, "WARNING"), 1);
    }

    #[test]
    fn process_line_counts_warning_logs() {
        let mut stats = LogStats::new();

        stats.process_line("WARNING retry limit almost reached");

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 1);
        assert_eq!(count_for(&stats, "WARNING"), 1);
    }

    #[test]
    fn process_line_counts_info_logs() {
        let mut stats = LogStats::new();

        stats.process_line("INFO server started");

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.total_lines, 1);
        assert_eq!(count_for(&stats, "INFO"), 1);
    }

    #[test]
    fn process_line_counts_total_lines_even_for_unknown_logs() {
        let mut stats = LogStats::new();

        stats.process_line("[2026-04-26] DEBUG cache warmed");

        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 1);
        assert!(stats.log_level_counts.is_empty());
    }

    #[test]
    fn process_line_prioritizes_error_when_line_contains_multiple_levels() {
        let mut stats = LogStats::new();

        stats.process_line("ERROR while handling INFO message");

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.info_count, 0);
        assert_eq!(stats.total_lines, 1);
        assert_eq!(count_for(&stats, "ERROR"), 1);
        assert_eq!(count_for(&stats, "INFO"), 0);
    }

    #[test]
    fn merge_combines_all_counts_and_levels() {
        let mut first = LogStats::new();
        first.process_line("ERROR first failure");
        first.process_line("INFO first startup");

        let mut second = LogStats::new();
        second.process_line("WARNING second warning");
        second.process_line("INFO second startup");
        second.process_line("DEBUG second debug");

        first.merge(&second);

        assert_eq!(first.error_count, 1);
        assert_eq!(first.warning_count, 1);
        assert_eq!(first.info_count, 2);
        assert_eq!(first.total_lines, 5);
        assert_eq!(count_for(&first, "ERROR"), 1);
        assert_eq!(count_for(&first, "WARNING"), 1);
        assert_eq!(count_for(&first, "INFO"), 2);
    }
}
