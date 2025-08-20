# Gryphon App - Event-Sourced Multi-Agent System

## Overview

The Gryphon App is a sophisticated multi-agent system built using Domain-Driven Design (DDD) principles with event sourcing and CQRS patterns. The system manages various types of agents for autonomous operations.

## Architecture

### Domain-Driven Design
- **Aggregates**: Each domain has its own aggregate root that maintains consistency
- **Events**: All state changes are captured as domain events
- **Commands**: Business operations are modeled as commands
- **Projections**: Read models are built from events for querying

### Event Sourcing

- All state changes are stored as immutable events
- Kafka is used as the event store for scalability and durability (KRaft mode - no Zookeeper required)
- PostgreSQL stores snapshots for performance optimization

### Domains

#### LogicalAgent
Manages high-level reasoning and decision-making agents with:
- Objectives and goal management
- Knowledge base with facts and rules
- Decision trees for automated reasoning

#### TechnicalAgent
Handles physical and technical aspects of agents:
- Hardware specifications (sensors, actuators)
- Software modules and capabilities
- Component status monitoring

#### KinematicAgent
Manages motion and positioning:
- 3D position, velocity, and acceleration
- Kinematic constraints and models
- Trajectory tracking

#### PathPlanning
Handles route planning and obstacle avoidance:
- Multiple planning algorithms (A*, RRT, etc.)
- Workspace modeling with obstacles
- Path optimization

#### Dynamics
Simulates physical dynamics:
- Force and torque calculations
- Physics models (Newtonian, etc.)
- Real-time simulation

#### GUI
Provides user interface capabilities:
- Window and component management
- User session tracking
- Real-time visualization

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Event Store**: Apache Kafka 4.0+ (with KRaft)
- **Snapshot Store**: PostgreSQL
- **Async Runtime**: Tokio
- **Serialization**: Serde + JSON
- **Configuration**: TOML

## Getting Started

1. **Start Infrastructure (Kafka with KRaft + PostgreSQL):**

   ```bash
   docker-compose up -d
   ```

2. **Create Kafka Topics:**

   ```bash
   # Wait for Kafka to be ready, then create topics. The script will retry up to 10 times
   # (2s wait between attempts) and will try `docker exec gryphon-kafka` as a fallback
   # if the kafka CLI isn't available on your host.
   ./scripts/kafka-setup.sh
   ```

3. **Update configuration if needed:**

   ```bash
   # Edit config.toml with your specific settings
   ```

4. **Run the application:**

   ```bash
   cargo run
   ```

5. **Run tests:**

   ```bash
   cargo test
   ```

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

### Running Postgres integration test

Unit and lightweight integration tests run with `cargo test`.

The Postgres integration test is gated behind the `pg_integration` Cargo feature because it requires Docker/testcontainers and a running container environment. To run it locally:

1. Ensure Docker is running on your machine.
2. Run the tests with the feature enabled:

```bash
cargo test --features pg_integration -- --nocapture
```

The integration test will start a Postgres container using the `testcontainers` crate and perform read/write roundtrips to verify the Postgres adapter.

If you prefer not to run the integration test, the default `cargo test` will skip it.

## Infrastructure Components

- PostgreSQL database for snapshot storage
- Kafka broker settings and topics

## Configuration

See `config.toml` for configuration options including:

- Kafka broker settings and topics
- PostgreSQL connection parameters
- Event store configuration

## Development Guidelines

- Follow DDD principles for domain modeling
- All business logic changes must emit domain events
- Use projections for read models
- Maintain clean separation between domains
- Write comprehensive tests for aggregates and services
