# Event-Driven Path Planning Example

This document describes the refactored path planning example that demonstrates the proper use of the event-driven architecture with Kafka/EventStore integration.

## Architecture Overview

The sample applications now use a proper event-driven architecture instead of in-memory channels:

```
┌─────────────────┐    Events     ┌─────────────────┐    Events     ┌─────────────────┐
│  PathPlan       │ ============> │  Event Store    │ ============> │  PathPlanner    │
│  Client         │               │  (Kafka/Memory) │               │  Service        │
└─────────────────┘               └─────────────────┘               └─────────────────┘
                                           │
                                           │ Events
                                           ▼
                                  ┌─────────────────┐
                                  │  PathPlan       │
                                  │  Worker         │
                                  └─────────────────┘
```

## Components

### 1. PathPlan Client (`pathplan_client`)

**Purpose**: Simulates real-world clients making path planning requests by publishing events to the event store.

**Key Changes**:
- Uses `KafkaEventStore` or falls back to `InMemoryEventStore`
- Publishes `PathPlanRequested` events instead of direct method calls
- Demonstrates proper event envelope creation with metadata
- Shows realistic path planning scenarios with intuitive degree-based angles

**Events Published**:
- `PathPlanRequested`: Contains start/destination positions, orientations, agent ID, and request metadata

**Example Usage**:
```bash
cargo run --bin pathplan_client
```

### 2. PathPlanner Service (`pathplan_planner`)

**Purpose**: Manages PathPlanner aggregates using event sourcing and coordinates work assignments through events.

**Key Changes**:
- Uses event store to persist and restore PathPlanner state
- Polls for new `PathPlanRequested` events (2-second intervals)
- Manages worker registrations and assignments through events
- Publishes `PlanAssigned` events to workers
- Demonstrates proper event sourcing patterns

**Events Consumed**:
- `PathPlanRequested`: Triggers plan assignment logic
- `WorkerRegistered`: Adds workers to available pool
- `PlanCompleted`/`PlanFailed`: Updates worker status

**Events Published**:
- `PlannerCreated`: When a new planner is created
- `PlanAssigned`: When a plan is assigned to a worker

**Example Usage**:
```bash
cargo run --bin pathplan_planner
```

### 3. PathPlan Worker (`pathplan_worker`)

**Purpose**: Consumes `PlanAssigned` events and publishes completion events.

**Note**: The current worker implementation lives in `src/bin/pathplan_worker/worker.rs` and accepts an injected domain logger (`DynLogger`) — there is no longer a `worker_new.rs` file. The worker consumes `PlanAssigned` events from the event store and publishes `PlanCompleted`/`PlanFailed` events.

## Event Flow

1. **Client Request**:
   ```
   Client → PathPlanRequested Event → Event Store
   ```

2. **Planner Processing**:
   ```
   Event Store → PathPlanRequested Event → Planner Service
   Planner Service → PlanAssigned Event → Event Store
   ```

3. **Worker Processing** (to be implemented):
   ```
   Event Store → PlanAssigned Event → Worker
   Worker → PlanCompleted Event → Event Store
   ```

4. **Response Handling**:
   ```
   Event Store → PlanCompleted Event → Planner Service
   Planner Service → (Notify Client)
   ```

## Configuration

The applications automatically try to use Kafka if available, falling back to in-memory event store:

```toml
# config.toml
[kafka]
brokers = ["localhost:9092"]
client_id = "gryphon-app"
group_id = "gryphon-app-group"

[kafka.topics]
path_planning_events = "path-planning-events"
# ... other topics
```

## Event Store Integration

### Kafka EventStore
- **Production-ready**: Uses rdkafka for reliable event publishing
- **Scalable**: Supports multiple consumers and partitioning
- **Persistent**: Events are stored durably in Kafka topics

### In-Memory EventStore
- **Development**: Fast local testing without external dependencies
- **Fallback**: Automatically used when Kafka is unavailable
- **Simple**: Perfect for unit tests and development

## Key Benefits

1. **Decoupling**: Components communicate only through events
2. **Scalability**: Easy to add more workers or planners
3. **Resilience**: Events are persisted and can be replayed
4. **Auditability**: Complete event history for debugging
5. **Testability**: Easy to test individual components in isolation

## Running the Example

1. **Start Kafka** (optional, will fall back to in-memory if not available):
   ```bash
   docker-compose up -d kafka
   ```

2. **Start the Planner Service**:
   ```bash
   cargo run --bin pathplan_planner
   ```

3. **Start the Client** (in another terminal):
   ```bash
   cargo run --bin pathplan_client
   ```

4. **Observe the Event Flow**:
   - Client publishes `PathPlanRequested` events
   - Planner polls and processes events
   - Planner publishes `PlanAssigned` events
   - Status updates show event processing

## Sample Output

### Client Output:
```
🚀 Starting Path Planning Client (Event-Driven)
📋 Loaded configuration from config.toml
✅ Connected to Kafka event store
📤 Publishing path plan request event:
   🆔 Request ID: req-550e8400-e29b-41d4-a716-446655440000
   🤖 Agent: office-robot-001
   📍 Start: (-50.0, -30.0) @ 0.00rad
   🎯 Goal:  (40.0, 25.0) @ 1.57rad
   📏 Distance: 102.2 units
   📡 Publishing event to event store...
   ✅ Event published successfully!
   🎯 Plan ID: plan-440e8400-e29b-41d4-a716-446655440001
```

### Planner Output:
```
🗺️  Starting Path Planning Planner Service (Event-Driven)
📋 Loaded configuration from config.toml
✅ Connected to Kafka event store
✅ Created new PathPlanner with A* algorithm and persisted creation event
🚀 Path Planning Planner Service is running (Event-Driven)
📡 Polling event store for new events...
📥 Found 1 new events for planner main-path-planner
🎯 Processing PathPlanRequested event:
   Request ID: req-550e8400-e29b-41d4-a716-446655440000
   Plan ID: plan-440e8400-e29b-41d4-a716-446655440001
   Agent: office-robot-001
   From: (-50.0, -30.0) -> To: (40.0, 25.0)
⚠️  No available workers for plan plan-440e8400-e29b-41d4-a716-446655440001. Request queued.
```

## Next Steps

1. **Update PathPlan Worker**: Refactor the worker to consume `PlanAssigned` events
2. **Add Worker Registration**: Implement worker registration through events
3. **Error Handling**: Add comprehensive error handling and retry logic
4. **Monitoring**: Add metrics and observability for event processing
5. **Performance**: Optimize event polling and processing
