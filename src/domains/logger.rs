use chrono::Utc;
use log::{error as log_error, info as log_info, warn as log_warn};
use std::sync::Arc;

/// Domain-level logging port (Hexagonal port).
/// Keep this API intentionally small and non-fallible from the domain perspective.
pub trait DomainLogger: Send + Sync + 'static {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

pub type DynLogger = Arc<dyn DomainLogger>;

/// A file-based adapter using `fast_log` for file writing and rotation.
pub struct FileLogger;

impl FileLogger {
    /// Initialize the fast_log file logger.
    /// Path is the file path used by fast_log's Rolling file appender.
    pub fn init(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Configure fast_log with a rolling file appender
        fast_log::init(
            fast_log::config::Config::new()
                .console()
                .file(path)
                .level(log::LevelFilter::Info),
        )?;
        Ok(())
    }
}

impl DomainLogger for FileLogger {
    fn info(&self, msg: &str) {
        log_info!("{} - {}", Utc::now().to_rfc3339(), msg);
    }

    fn warn(&self, msg: &str) {
        log_warn!("{} - {}", Utc::now().to_rfc3339(), msg);
    }

    fn error(&self, msg: &str) {
        log_error!("{} - {}", Utc::now().to_rfc3339(), msg);
    }
}
