use std::sync::{Arc, Mutex};
use std::time::Duration;

struct BridgeCapture {
    messages: Arc<Mutex<Vec<String>>>,
}

impl BridgeCapture {
    fn new() -> Self { Self { messages: Arc::new(Mutex::new(Vec::new())) } }
}

impl gryphon_app::domains::logger::DomainLogger for BridgeCapture {
    fn info(&self, msg: &str) { self.messages.lock().unwrap().push(format!("INFO:{}", msg)); }
    fn warn(&self, msg: &str) { self.messages.lock().unwrap().push(format!("WARN:{}", msg)); }
    fn error(&self, msg: &str) { self.messages.lock().unwrap().push(format!("ERR:{}", msg)); }
}

#[tokio::test]
async fn test_buffered_and_noop_logger() {
    let capture = Arc::new(BridgeCapture::new());
    let bridge = capture.clone() as Arc<dyn gryphon_app::domains::logger::DomainLogger>;

    // Create a buffered logger that forwards to the bridge with small capacity
    let buffered = gryphon_app::adapters::outbound::init_buffered_logger(bridge.clone(), 8);

    // Send messages
    buffered.info("one");
    buffered.warn("two");
    buffered.error("three");

    // Give the background task a moment
    tokio::time::sleep(Duration::from_millis(50)).await;

    let msgs = capture.messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("INFO:one")));
    assert!(msgs.iter().any(|m| m.contains("WARN:two")));
    assert!(msgs.iter().any(|m| m.contains("ERR:three")));

    // No-op logger should accept calls and not panic; ensure it exists
    let noop = gryphon_app::adapters::outbound::init_noop_logger();
    noop.info("ignored");
    noop.error("ignored-err");
}
