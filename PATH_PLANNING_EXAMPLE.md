# Path Planning Example Application

This directory contains three binary processes that demonstrate the path planning domain in action:

## 🏗️ Binaries Overview

### 1. Path Planning Planner (`pathplan_planner`)

**Purpose**: Central coordination service that manages PathPlanner aggregates
**Responsibilities**:
- Creates and manages PathPlanner instances
- Receives path planning requests from clients
- Manages worker registrations and assignments
- Coordinates the flow between clients and workers
- Maintains aggregate state and processes domain events

### 2. Path Planning Client (`pathplan_client`)
**Purpose**: Simulates real-world clients making path planning requests
**Features**:
- Predefined realistic scenarios (office, warehouse, hospital, etc.)
- Random request generation
- Detailed request logging and simulation
- Multiple operation modes (demo, random, continuous)

### 3. A* Path Planning Worker (`pathplan_worker`)
**Purpose**: Implements the A* algorithm and processes path planning assignments
**Capabilities**:
- Registers with the planner service
- Receives and accepts work assignments
- Executes A* pathfinding algorithm
- Reports completion or failure back to planner

## 🚀 Running the Example

### Prerequisites

Make sure you have Rust installed and the project builds successfully:

```bash
cargo build
```

### Start the Processes

### Terminal 1: Start the Planner Service

```bash
cargo run --bin pathplan_planner
```

This starts the central coordination service. You should see:

- Service startup messages
- PathPlanner creation with A* algorithm
- Status reports every 30 seconds

### Terminal 2: Start the A* Worker

```bash
cargo run --bin pathplan_worker
```

This starts the A* worker. You should see:

- Worker registration with the planner
- A* algorithm capability advertisement
- Assignment waiting and processing

Note: the worker binary `pathplan_worker` uses the implementation in `src/bin/pathplan_worker/worker.rs` and receives a domain `DynLogger` injected at startup. A historical `worker_new.rs` file was removed during refactor — ignore references to it.

### Terminal 3: Start the Client (Optional)

```bash
cargo run --bin pathplan_client
```

This starts sending path planning requests. You should see:

- Demo scenarios being sent
- Request details and simulated responses
- Random request generation

## 📋 Example Flow

1. Worker Registration: The A* worker registers with the planner, advertising its A* capabilities
2. Client Request: The client sends a path planning request (e.g., office robot navigation)
3. Request Processing: The planner validates the request and creates a PathPlan
4. Work Assignment: The planner assigns the plan to the available A* worker
5. Path Planning: The worker executes the A* algorithm and generates waypoints
6. Result Reporting: The worker reports success/failure back to the planner
7. Client Response: The planner notifies the client of completion

## 🔍 What You'll See

### Planner Output

```text
🗺️  Starting Path Planning Planner Service
✅ Created main PathPlanner with A* algorithm
🚀 Path Planning Planner Service is running
📡 Listening for path plan requests and worker events...
```

### Worker Output

```text
🤖 Starting A* Path Planning Worker
📝 Phase 1: Registration
📝 Registering worker astar-worker-[uuid] with planner
✅ Registration phase completed
🔄 Phase 2: Work Loop
```

### Client Output

```text
🚀 Starting Path Planning Client
🎬 Starting demo mode - sending predefined scenarios
📋 Scenario 1 of 5: Office Navigation
📝 Robot navigating from office entrance to meeting room
```

## 🛠️ Architecture Notes

### Communication Patterns

- **Planner ↔ Worker**: Event-driven communication using domain events
- **Client → Planner**: Request/response pattern for path planning requests
- **Worker → Planner**: Status updates and result reporting

### Domain Events

The system uses these key events:

- `PlannerCreated`: Planner initialization
- `PathPlanRequested`: New path planning request
- `WorkerRegistered`: Worker joins the system
- `WorkerReady`: Worker available for assignments
- `PlanAssigned`: Work assigned to worker
- `PlanCompleted`: Successful path generation
- `PlanFailed`: Path planning failure

### State Management

- **PathPlanner Aggregate**: Maintains workspace, active plans, workers, assignments
- **Event Sourcing**: All state changes are captured as domain events
- **CQRS**: Separate read/write models for optimal performance

## 🎯 Key Features Demonstrated

1. **Domain-Driven Design**: Clear separation of concerns and rich domain model
2. **Event Sourcing**: All changes captured as immutable events
3. **CQRS**: Command/query responsibility segregation
4. **Microservices**: Independent, communicating processes
5. **Worker Pool Management**: Dynamic worker registration and assignment
6. **Algorithm Abstraction**: Support for multiple planning algorithms
7. **Realistic Scenarios**: Real-world path planning use cases

## 🔧 Customization

### Adding New Algorithms

1. Extend `PlanningAlgorithm` enum in `types.rs`
2. Create a new worker binary implementing the algorithm
3. Register the worker with appropriate capabilities

### Adding New Scenarios

Modify `pathplan_client.rs` to add new `PlanningScenario` instances with different:

- Agent types (robots, AGVs, drones)
- Environment layouts
- Mission objectives

### Integration Testing

Run all three processes simultaneously to see the full end-to-end flow of the path planning domain.

## 📚 Learning Objectives

This example demonstrates:
- How to structure a DDD-based microservices application
- Event sourcing and CQRS patterns in practice
- Rust async programming with Tokio
- Domain modeling for robotics and autonomous systems
- Coordination patterns between distributed services
