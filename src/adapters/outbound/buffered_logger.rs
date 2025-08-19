use crate::domains::logger::DomainLogger;
use std::sync::Arc;
use tokio::sync::mpsc;

enum Level {
    Info,
    Warn,
    Error,
}

struct LogMessage {
    level: Level,
    msg: String,
}

/// Non-blocking buffered logger. Messages are forwarded to the provided `bridge`
/// from a background task. `capacity` controls the channel buffer size.
pub fn init_buffered_logger(bridge: Arc<dyn DomainLogger>, capacity: usize) -> Arc<dyn DomainLogger> {
    let (tx, mut rx) = mpsc::channel::<LogMessage>(capacity);

    // Spawn background task to drain the channel and forward to the bridge.
    let bridge_task = bridge.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg.level {
                Level::Info => bridge_task.info(&msg.msg),
                Level::Warn => bridge_task.warn(&msg.msg),
                Level::Error => bridge_task.error(&msg.msg),
            }
        }
    });

    struct BufferedLogger {
        sender: mpsc::Sender<LogMessage>,
    }

    impl DomainLogger for BufferedLogger {
        fn info(&self, msg: &str) {
            // Non-blocking: try_send, drop on full
            let _ = self.sender.try_send(LogMessage { level: Level::Info, msg: msg.to_string() });
        }

        fn warn(&self, msg: &str) {
            let _ = self.sender.try_send(LogMessage { level: Level::Warn, msg: msg.to_string() });
        }

        fn error(&self, msg: &str) {
            let _ = self.sender.try_send(LogMessage { level: Level::Error, msg: msg.to_string() });
        }
    }

    Arc::new(BufferedLogger { sender: tx })
}
