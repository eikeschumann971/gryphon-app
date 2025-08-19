use crate::domains::logger::DomainLogger;
use std::sync::Arc;

struct NoOp;

impl DomainLogger for NoOp {
    fn info(&self, _msg: &str) {}
    fn warn(&self, _msg: &str) {}
    fn error(&self, _msg: &str) {}
}

/// No-op logger useful as default in unit tests
pub fn init_noop_logger() -> Arc<dyn DomainLogger> {
    Arc::new(NoOp {})
}
