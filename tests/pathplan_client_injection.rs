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
async fn test_pathplanclient_logs_on_new() {
    let capture = Arc::new(BridgeCapture::new());
    let bridge = capture.clone() as Arc<dyn gryphon_app::domains::logger::DomainLogger>;

    // instantiate PathPlanClient with the capture bridge as logger
    let _client = gryphon_app::application::PathPlanClient::new(bridge).await.expect("should construct client");

    // give any async tasks a moment
    tokio::time::sleep(Duration::from_millis(20)).await;

    let msgs = capture.messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Using default configuration")));
}
