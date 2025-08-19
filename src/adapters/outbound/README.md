# Logging Adapters

This folder provides domain logging adapters (Hexagonal outbound adapters) which implement the `DomainLogger` trait from `src/domains/logger.rs`.

Available adapters

- `file_logger` - initializes `fast_log` and forwards domain logging to the file-backed logger via the `log` facade.
- `console_logger` - simple console logger (info/warn -> stdout, error -> stderr).
- `multi_logger` - forwards to a primary logger and an optional secondary (used to combine file + console).
- `buffered_logger` - non-blocking buffered adapter that forwards messages to an underlying bridge from a background task. Useful for high-throughput, non-blocking logging.
- `noop_logger` - a no-op implementation that drops all messages; intended as the default for unit tests.

Quick usage

- Initialize a combined logger (file + console fallback):

  let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");

- Initialize a buffered logger wrapping a bridge logger (e.g., file logger):

  let file_logger = gryphon_app::adapters::outbound::file_logger::init_file_logger("./domain.log").unwrap();
  let buffered = gryphon_app::adapters::outbound::init_buffered_logger(file_logger, 1024);

- Use `init_noop_logger()` in unit tests to avoid side effects.

Notes

- The `buffered_logger` uses a Tokio mpsc channel and spawns a background task; ensure your application runs a Tokio runtime.
- The `multi_logger` will attach the console logger as a secondary output when the file logger initializes successfully; if the file logger fails, it returns the console logger instead.
