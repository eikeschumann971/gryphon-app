# Gryphon App

A sophisticated multi-agent system built with Domain-Driven Design (DDD), Event Sourcing, and CQRS patterns in Rust.

## Features

- **Event Sourcing**: All state changes captured as immutable events using Kafka
- **DDD Architecture**: Clean domain separation with aggregates, events, and services  
- **CQRS**: Command and Query Responsibility Segregation with projections
- **Multi-Agent Support**: Six specialized agent domains for autonomous operations
- **Real-time Processing**: Async event processing with Tokio
- **Persistent Snapshots**: PostgreSQL for optimized state reconstruction

## Domains

- **LogicalAgent**: High-level reasoning and decision-making
- **TechnicalAgent**: Hardware/software management  
- **KinematicAgent**: Motion and positioning control
- **PathPlanning**: Route planning and obstacle avoidance
- **Dynamics**: Physics simulation and dynamics
- **GUI**: User interface and visualization

## Quick Start

1. **Start Infrastructure**:

   ```bash
   docker-compose up -d
   # Follow Kafka logs (live):
   docker compose logs -f kafka --tail=200
   # Check container status:
   docker ps --filter "name=gryphon-kafka"
   # top and remove containers + volumes (useful to force 
   # a fresh Kafka storage format):
   docker compose down -v
   ```

   ```bash
   # will run /tmp/kafka-setup.sh (the file is mounted into the container by docker-compose)
   docker compose exec kafka bash -lc "/tmp/kafka-setup.sh"
   # or
   docker exec -it gryphon-kafka bash -lc "/tmp/kafka-setup.sh"
   # list topics (host)
   docker compose exec kafka bash -lc "kafka-topics --bootstrap-server localhost:9092 --list"
   ```

   ```bash
   docker exec -i gryphon-postgres psql -U postgres -d gryphon_app -f /docker-entrypoint-initdb.d/init-db.sql
   ```

   ```bash
   bash scripts/run_pg_integration_test.sh
   ```

2. **Build and Run**:

   ```bash
   cargo build
   cargo run
   ```

3. **Run Tests**:

   ```bash
   cargo test
   ```

4. **Run Production**:

   ```bash
   cargo run --bin pathplan_planner_kafka
   cargo run --release --bin pathplan_planner_kafka

   cargo run --bin pathplan_worker_kafka
   cargo run --release --bin pathplan_worker_kafka

   cargo run --bin pathplan_client_kafka
   cargo run --release --bin pathplan_client_kafka
   ```

   ```bash
   pkill -f pathplan
   ```

## Quick tips & notes

- Your compose uses KRaft (no Zookeeper) and includes a storage format step; on first run it will format storage. If you change the `CLUSTER_ID` or want to reinitialize, run `docker compose down -v` to remove `kafka_data` so the container can re-format.
- Internal vs external listeners:
  - Other containers in the same compose can talk to Kafka at `kafka:29092`.
  - From your host (macOS) use `localhost:9092` (mapped in the compose file).
- If the Kafka container fails to start, check `docker compose logs kafka` for errors about storage (meta.properties), permissions, or cluster id.
- The compose mounts `kafka-setup.sh` — ensure that file is executable if you edit it locally.

## Configuration

Edit `config.toml` to configure:
- Kafka brokers and topics
- PostgreSQL connection
- Event store settings

## Architecture

```
├── src/
│   ├── domains/           # Domain logic (aggregates, events, projections)
│   ├── application/       # Application services
│   ├── adapters/          # Infrastructure adapters
│   │   ├── inbound/       # Event and snapshot stores
│   │   └── outbound/      # Kafka and PostgreSQL adapters
│   ├── common/            # Shared types and traits
│   └── config.rs          # Configuration management
├── tests/                 # Integration and unit tests
├── docs/                  # Documentation
└── scripts/               # Database and setup scripts
```

## Development

See `docs/README.md` for detailed architecture documentation and `prompts/development_prompts.md` for development guidelines.

## Path planning resources and graph persistence

Runtime map and graph assets are stored under `resources/path_planning` by default. The repository contains sample geojson files and an example graphs directory used by the filesystem adapter.

- `resources/path_planning/geojson/` — source GeoJSON map files used to build the runtime graph.
- `resources/path_planning/graphs/` — binary serialized graphs written by the Filesystem adapter.

The runtime data directory can also be set with the environment variable `PATH_PLANNING_DATA_DIR`. If unset, the application falls back to `resources/path_planning` in the project tree or `/usr/share/gryphon-app/path_planning`.

### Graph file binary format

Graphs are persisted with a small header followed by a bincode-serialized Petgraph payload. Header layout (little-endian):

- 4 bytes: ASCII magic `PGPH`
- 1 byte: format version (u8). Current version is `1`.
- 4 bytes: header JSON length (u32 little-endian)
- N bytes: header JSON (UTF-8)
- remaining bytes: bincode(payload) — Petgraph Graph serialized with serde/bincode

The JSON header contains optional metadata like `source_file`, `created_by`, and may include a `version` field for compatibility. The Filesystem adapter supports migrating legacy v0 files (pure bincode Petgraph without header) to the v1 header format on load.

### Postgres graph storage

The Postgres adapter stores graph bytes directly in a `graphs` table (created on demand). Table schema (created by the adapter):

CREATE TABLE IF NOT EXISTS graphs (
   name TEXT PRIMARY KEY,
   data BYTEA NOT NULL,
   version INT,
   header JSONB,
   created_at TIMESTAMPTZ DEFAULT now(),
   updated_at TIMESTAMPTZ DEFAULT now()
);

The adapter inserts or upserts graph bytes and writes parsed header metadata when present.

### Configuration example (config.toml)

Add or update the `postgres` section in `config.toml` with your DB connection details. Example:

```toml
[postgres]
host = "127.0.0.1"
port = 5432
database = "gryphon"
username = "gryphon"
password = "secret"
# optional: desired max pool connections (adapter will use deadpool defaults if not set)
max_connections = 10
```

### Running tests including the Postgres integration test

Unit and lightweight integration tests run with `cargo test`.

The Postgres integration test is gated behind the `pg_integration` Cargo feature because it requires Docker/testcontainers and a running container environment. To run it locally:

1. Ensure Docker is running on your machine.
2. Run the tests with the feature enabled:

```bash
cargo test --features pg_integration -- --nocapture
```

The integration test will start a Postgres container using the `testcontainers` crate and perform read/write roundtrips to verify the Postgres adapter.

If you prefer not to run the integration test, the default `cargo test` will skip it.

## License

This project is licensed under the MIT License.
