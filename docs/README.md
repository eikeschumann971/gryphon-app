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
- Kafka is used as the event store for scalability and durability
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

1. Ensure Kafka and PostgreSQL are running
2. Update `config.toml` with your database and Kafka settings
3. Run the application: `cargo run`
4. Run tests: `cargo test`

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
