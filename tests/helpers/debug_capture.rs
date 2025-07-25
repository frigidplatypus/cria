use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Debug message capture system for testing debug logging functionality
pub struct DebugCapture {
    messages: Arc<Mutex<VecDeque<String>>>,
}

impl DebugCapture {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Capture debug messages during test execution
    pub fn capture<F, R>(&self, test_fn: F) -> (R, Vec<String>)
    where
        F: FnOnce() -> R,
    {
        // Clear any existing messages
        self.messages.lock().unwrap().clear();
        
        // Run the test function
        let result = test_fn();
        
        // Return results and captured messages
        let messages: Vec<String> = self.messages.lock().unwrap().iter().cloned().collect();
        (result, messages)
    }

    /// Add a debug message (used by mock debug_log function)
    pub fn add_message(&self, message: String) {
        self.messages.lock().unwrap().push_back(message);
    }

    /// Get all captured messages without clearing
    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }

    /// Clear all captured messages
    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

impl Default for DebugCapture {
    fn default() -> Self {
        Self::new()
    }
}

// Global debug capture instance for testing
lazy_static::lazy_static! {
    static ref GLOBAL_DEBUG_CAPTURE: DebugCapture = DebugCapture::new();
}

/// Capture debug logs during test execution
/// 
/// # Example
/// ```rust
/// let (result, logs) = capture_debug_logs(|| {
///     // Code that calls debug_log
///     extract_urls_from_task(&task)
/// });
/// 
/// assert!(logs.iter().any(|log| log.contains("URL deduplication")));
/// ```
pub fn capture_debug_logs<F, R>(test_fn: F) -> (R, Vec<String>)
where
    F: FnOnce() -> R,
{
    GLOBAL_DEBUG_CAPTURE.capture(test_fn)
}

/// Mock version of debug_log for testing
/// In tests, replace calls to crate::debug::debug_log with this function
#[cfg(test)]
pub fn mock_debug_log(message: &str) {
    GLOBAL_DEBUG_CAPTURE.add_message(message.to_string());
}

/// Test utilities for asserting debug log contents
pub struct DebugAssertions;

impl DebugAssertions {
    /// Assert that logs contain a specific message
    pub fn assert_contains(logs: &[String], expected: &str) {
        assert!(
            logs.iter().any(|log| log.contains(expected)),
            "Expected to find '{}' in debug logs. Actual logs: {:?}",
            expected,
            logs
        );
    }

    /// Assert that logs contain messages in a specific order
    pub fn assert_order(logs: &[String], expected_sequence: &[&str]) {
        let mut search_start = 0;
        
        for expected_msg in expected_sequence {
            let found_index = logs[search_start..]
                .iter()
                .position(|log| log.contains(expected_msg));
                
            assert!(
                found_index.is_some(),
                "Expected to find '{}' after position {} in logs: {:?}",
                expected_msg,
                search_start,
                logs
            );
            
            search_start += found_index.unwrap() + 1;
        }
    }

    /// Assert logs contain exactly N messages matching a pattern
    pub fn assert_count(logs: &[String], pattern: &str, expected_count: usize) {
        let actual_count = logs.iter().filter(|log| log.contains(pattern)).count();
        assert_eq!(
            actual_count, expected_count,
            "Expected {} messages containing '{}', found {}. Logs: {:?}",
            expected_count, pattern, actual_count, logs
        );
    }

    /// Assert no logs contain a specific pattern
    pub fn assert_not_contains(logs: &[String], pattern: &str) {
        assert!(
            !logs.iter().any(|log| log.contains(pattern)),
            "Expected NOT to find '{}' in debug logs. Actual logs: {:?}",
            pattern,
            logs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_capture_basic() {
        let (result, logs) = capture_debug_logs(|| {
            mock_debug_log("test message 1");
            mock_debug_log("test message 2");
            42
        });
        
        assert_eq!(result, 42);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "test message 1");
        assert_eq!(logs[1], "test message 2");
    }

    #[test]
    fn test_capture_debug_logs_convenience() {
        let (result, logs) = capture_debug_logs(|| {
            mock_debug_log("convenience test");
            "test_result"
        });
        
        assert_eq!(result, "test_result");
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], "convenience test");
    }

    #[test]
    fn test_debug_assertions() {
        let logs = vec![
            "Starting process".to_string(),
            "Found 3 items".to_string(),
            "Processing complete".to_string(),
        ];
        
        DebugAssertions::assert_contains(&logs, "Found 3 items");
        DebugAssertions::assert_order(&logs, &["Starting", "Found", "complete"]);
        DebugAssertions::assert_count(&logs, "ing", 2); // "Starting" and "Processing"
        DebugAssertions::assert_not_contains(&logs, "error");
    }

    #[test]
    #[should_panic]
    fn test_debug_assertions_failure() {
        let logs = vec!["test message".to_string()];
        DebugAssertions::assert_contains(&logs, "missing message");
    }
}