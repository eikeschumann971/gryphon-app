# Tests — Gryphon App

This document explains the test layout, how to run individual test groups, environment needs, and the testing philosophy used in this repository.

## Test categories

- Unit tests
  - Fast, isolated tests for domain logic, helpers, and pure functions.
  - Examples: aggregate unit tests, planning algorithm tests, position/orientation tests.
  - Run: `cargo test` (runs unit + non-gated integration tests)

- Adapter / component tests
  - Tests that exercise adapters with lightweight real I/O (filesystem, in-memory stores).
  - Example: `tests/adapter_tests.rs` verifies `FilesystemDataSource` save/load behavior using tempdirs.
  - Run a single adapter test: `cargo test --test adapter_tests` or use the test name filter.

- Domain & application tests
  - Focused on aggregates, events, and application services.
  - Examples: `tests/domain_tests.rs`, `tests/application_tests.rs`.
  - Run: `cargo test --lib` or `cargo test --test domain_tests`.

- Graph builder tests
  - Validate GeoJSON -> Petgraph graph building, binary serialization roundtrips, and migration from legacy formats.
  - File: `tests/graph_builder_tests.rs`.
  - Run: `cargo test --test graph_builder_tests`.

- Integration tests (external systems)
  - Tests that require external infrastructure such as PostgreSQL and (optionally) Docker-managed services.
  - The Postgres integration test is gated behind the Cargo feature `pg_integration` to avoid forcing Docker in all local runs or CI.
  - File: `tests/pg_integration_tests.rs`.

## How to run the Postgres integration test

This test validates `PostgresGraphStore` end-to-end. Two approaches are supported:

1. Manual Docker container (recommended for local runs):

   Start a Postgres container first (example uses host port `5433`):

   ```bash
   docker run --rm -e POSTGRES_PASSWORD=postgres -p 5433:5432 -d postgres:15
   export PG_TEST_PORT=5433    # optional: test uses 5433 by default
   cargo test --features pg_integration -- tests::pg_integration_tests -- --nocapture
   ```

   The test will connect to `127.0.0.1:${PG_TEST_PORT}` and perform an insert/read roundtrip. Tear down the container after the test.

2. Self-starting container using `testcontainers` (older approach)

   The repo has `testcontainers` in `dev-dependencies`. If you prefer the test to start containers automatically, the test can be adapted to use `testcontainers` directly; note this requires matching the `testcontainers` API and images available on your host.

## Execution tips

- Run a specific test function by passing its name as the last argument to `cargo test`:

  ```bash
  cargo test --test path_planning_unit_tests some_specific_test_name
  ```

- To run only integration tests gated by `pg_integration`:

  ```bash
  cargo test --features pg_integration
  ```

- Use `-- --nocapture` to see test stdout/stderr during runs.

## CI guidance

- Default CI should run `cargo test` (fast unit + component tests).
- Run the `pg_integration` tests in a separate CI job that provisions Docker (or uses a cloud Postgres instance) and enables the `pg_integration` feature.
- Keep integration tests stable and short; prefer provisioning ephemeral resources per job.

## Testing philosophy and strategy

- Prefer fast, deterministic unit tests for core domain logic. Keep these independent of I/O and external services.
- Adapter tests validate the contract between domain and infrastructure with minimal external dependencies (tempfiles, in-memory stores).
- Gate expensive or flaky tests (Docker, network) behind feature flags and run them selectively in CI or locally when debugging infra-related issues.
- Where possible, prefer small end-to-end integration tests that exercise serialization, DB schema migrations, and upsert logic rather than full system end-to-end runs.

## Troubleshooting

- If the Postgres integration test fails to connect, ensure Docker is running and the container port is mapped correctly.
- If you see permission or auth errors, verify the `POSTGRES_PASSWORD` and credentials used by the test.

---

If you want, I can also:
- Add a small Makefile or test script to simplify starting/stopping the Postgres container and running the integration test.
- Re-adapt the integration test to auto-start containers via `testcontainers` instead of relying on an externally started container.
