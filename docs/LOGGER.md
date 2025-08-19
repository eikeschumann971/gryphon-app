Domain logger adapters and usage

This project follows a hexagonal approach: the domain depends on a small `DomainLogger` port
(defined in `src/domains/logger.rs`). Concrete logging behavior is provided by adapters
in `src/adapters/outbound` and injected into application and runtime services.

Available adapters

- `file_logger` (uses `fast_log`): write structured timestamps to a rolling file.
- `console_logger`: simple stdout/stderr fallback adapter.
- `multi_logger`: combines file + console and picks console if file init fails.
- `buffered_logger`: async buffered forwarder useful when callers should not block.
- `noop_logger`: no-op adapter for unit tests.

Typical usage (binary launcher)

- Construct a combined logger and pass into application services via constructor injection:

  let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");
  let service = gryphon_app::application::PathPlanClient::new(logger.clone()).await?;

Testing notes

- Use `init_noop_logger()` or a small capture adapter in tests to assert domain logs are emitted
without writing to disk.

Design rules

- Keep `println!` calls only for *human-friendly* demo output. Prefer `logger.info/warn/error` for structured domain logs.
- Do not use any global logging registry in domain code; inject the `DynLogger` (type alias for Arc<dyn DomainLogger>) instead.
