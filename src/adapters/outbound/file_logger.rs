use crate::domains::logger::{FileLogger, set_global_logger};
use std::sync::Arc;

/// Convenience function the application can call at startup to install a file logger
/// path: path to append logs to (e.g. "/var/log/gryphon/domain.log" or "./domain.log")
pub fn init_file_logger(path: &str) -> Result<(), String> {
    let logger = FileLogger::new(path).map_err(|e| format!("Failed to open log file: {}", e))?;
    set_global_logger(Arc::new(logger));
    Ok(())
}
