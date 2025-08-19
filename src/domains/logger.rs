use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

/// Domain-level logging port (Hexagonal port).
/// Keep this API intentionally small and non-fallible from the domain perspective.
pub trait DomainLogger: Send + Sync + 'static {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

type DynLogger = Arc<dyn DomainLogger>;

/// A tiny global registry used for wiring a domain logger at application startup.
/// This keeps domain code free of direct filesystem dependencies while avoiding
/// invasive signature changes throughout the codebase during this refactor.
static GLOBAL_LOGGER: once_cell::sync::Lazy<Mutex<Option<DynLogger>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Install the domain logger. Should be called from application/bootstrap code.
pub fn set_global_logger(logger: DynLogger) {
    let mut guard = GLOBAL_LOGGER.lock().unwrap();
    *guard = Some(logger);
}

/// Get the installed logger (if any).
pub fn get_global_logger() -> Option<DynLogger> {
    GLOBAL_LOGGER.lock().unwrap().clone()
}

/// Convenience helpers used by domain code to log without taking a logger param.
pub fn info(msg: &str) {
    if let Some(logger) = get_global_logger() {
        logger.info(msg);
    }
}

pub fn warn(msg: &str) {
    if let Some(logger) = get_global_logger() {
        logger.warn(msg);
    }
}

pub fn error(msg: &str) {
    if let Some(logger) = get_global_logger() {
        logger.error(msg);
    }
}

/// A simple file-based adapter implementing the DomainLogger trait.
pub struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    /// Open or create the file and append.
    pub fn new(path: &str) -> std::io::Result<Self> {
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file: Mutex::new(f) })
    }
}

impl DomainLogger for FileLogger {
    fn info(&self, msg: &str) {
        let mut f = self.file.lock().unwrap();
        let _ = writeln!(f, "[{}] INFO  - {}", Utc::now().to_rfc3339(), msg);
    }

    fn warn(&self, msg: &str) {
        let mut f = self.file.lock().unwrap();
        let _ = writeln!(f, "[{}] WARN  - {}", Utc::now().to_rfc3339(), msg);
    }

    fn error(&self, msg: &str) {
        let mut f = self.file.lock().unwrap();
        let _ = writeln!(f, "[{}] ERROR - {}", Utc::now().to_rfc3339(), msg);
    }
}
