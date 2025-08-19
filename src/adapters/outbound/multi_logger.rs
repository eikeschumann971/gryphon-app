use crate::domains::logger::DomainLogger;
use std::sync::Arc;

/// A simple multi-forwarding logger that forwards to two DomainLogger instances.
/// This allows optional file + console outputs without changing the DomainLogger trait.
pub struct MultiLogger {
    primary: Arc<dyn DomainLogger>,
    secondary: Option<Arc<dyn DomainLogger>>,
}

impl MultiLogger {
    pub fn new(primary: Arc<dyn DomainLogger>, secondary: Option<Arc<dyn DomainLogger>>) -> Self {
        Self { primary, secondary }
    }
}

impl DomainLogger for MultiLogger {
    fn info(&self, msg: &str) {
        self.primary.info(msg);
        if let Some(sec) = &self.secondary {
            sec.info(msg);
        }
    }

    fn warn(&self, msg: &str) {
        self.primary.warn(msg);
        if let Some(sec) = &self.secondary {
            sec.warn(msg);
        }
    }

    fn error(&self, msg: &str) {
        self.primary.error(msg);
        if let Some(sec) = &self.secondary {
            sec.error(msg);
        }
    }
}

/// Initialize a combined logger: try to initialize file logger and attach console as secondary.
pub fn init_combined_logger(path: &str) -> std::sync::Arc<dyn DomainLogger> {
    let console = crate::adapters::outbound::init_console_logger();
    match crate::adapters::outbound::file_logger::init_file_logger(path) {
        Ok(file_logger) => std::sync::Arc::new(MultiLogger::new(file_logger, Some(console)))
            as std::sync::Arc<dyn DomainLogger>,
        Err(_) => console as std::sync::Arc<dyn DomainLogger>,
    }
}
