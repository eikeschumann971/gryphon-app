use crate::domains::logger::{FileLogger, DomainLogger};
use std::sync::Arc;

struct BridgeLogger;

impl DomainLogger for BridgeLogger {
    fn info(&self, msg: &str) {
        log::info!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        log::warn!("{}", msg);
    }

    fn error(&self, msg: &str) {
        log::error!("{}", msg);
    }
}

/// Initialize the file logger and return a domain logger instance the application can inject.
pub fn init_file_logger(path: &str) -> Result<Arc<dyn DomainLogger>, String> {
    FileLogger::init(path).map_err(|e| format!("Failed to initialize fast_log: {}", e))?;
    Ok(Arc::new(BridgeLogger {}))
}
