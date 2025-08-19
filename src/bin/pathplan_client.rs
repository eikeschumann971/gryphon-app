// Minimal binary wrapper – configuration and Arc are used inside library

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Compose a domain logger (file with console fallback)
    let logger: gryphon_app::domains::DynLogger =
        match gryphon_app::adapters::outbound::file_logger::init_file_logger("./domain.log") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to initialize file logger: {}", e);
                gryphon_app::adapters::outbound::init_console_logger()
            }
        };

    logger.info("Starting Path Planning Client (Event-Driven)");

    // Delegate to the reusable library PathPlanClient
    let client = gryphon_app::application::PathPlanClient::new(logger.clone()).await?;
    // If a run method exists, call it. Keep this commented until implemented in library.
    // client.run().await?;

    // We purposely don't keep `client` unused here in the minimal wrapper; drop at end.
    drop(client);

    Ok(())
}
