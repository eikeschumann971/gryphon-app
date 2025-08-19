use crate::domains::logger::DomainLogger;
use std::sync::Arc;

struct ConsoleBridge;

impl DomainLogger for ConsoleBridge {
    fn info(&self, msg: &str) { println!("{}", msg); }
    fn warn(&self, msg: &str) { println!("WARN: {}", msg); }
    fn error(&self, msg: &str) { eprintln!("ERROR: {}", msg); }
}

/// Initialize a simple console-backed DomainLogger (useful as a fallback)
pub fn init_console_logger() -> Arc<dyn DomainLogger> {
    Arc::new(ConsoleBridge {})
}
