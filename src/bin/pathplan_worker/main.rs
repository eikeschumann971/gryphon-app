mod worker;
mod planning;
mod mock;
mod communication;

use worker::run_worker;
use gryphon_app::adapters::outbound::file_logger::init_file_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting A* Path Planning Worker");
    // Initialize domain file logger and inject into worker
    let logger = match init_file_logger("./domain.log") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to initialize file logger: {}", e);
            // Fallback: create a no-op logger by creating a Bridge that logs to stdout via log
            // We'll use the init_file_logger's failure as fatal-ish but continue with a simple bridge.
            // Construct a simple bridge that forwards to the log macros (which currently go to tracing console)
            struct ConsoleLogger;
            impl gryphon_app::domains::logger::DomainLogger for ConsoleLogger {
                fn info(&self, msg: &str) { println!("{}", msg); }
                fn warn(&self, msg: &str) { println!("WARN: {}", msg); }
                fn error(&self, msg: &str) { eprintln!("ERROR: {}", msg); }
            }

            std::sync::Arc::new(ConsoleLogger {}) as gryphon_app::domains::DynLogger
        }
    };
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    run_worker(logger).await?;
    
    Ok(())
}
