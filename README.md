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

## License

This project is licensed under the MIT License.
