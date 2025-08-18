# RouteRequest Command Example

This document demonstrates how to use the new `RouteRequest` command in the path planning aggregate.

## Overview

The `RouteRequest` command allows agents to request path planning services by providing:
- Agent identification
- Start and destination positions (2D)
- Start and destination orientations 
- Request metadata

## Data Structures

### RouteRequest
```rust
pub struct RouteRequest {
    pub request_id: String,        // Unique identifier for this request
    pub agent_id: String,          // ID of the requesting agent
    pub start_position: Position2D,     // Starting 2D position
    pub destination_position: Position2D, // Target 2D position
    pub start_orientation: Orientation2D,   // Starting orientation
    pub destination_orientation: Orientation2D, // Target orientation
    pub created_at: DateTime<Utc>,      // When the request was created
}
```

### Position2D
```rust
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}
```

### Orientation2D
```rust
pub struct Orientation2D {
    pub angle: f64, // Angle in radians
}
```

## Usage Example

```rust
use gryphon_app::domains::path_planning::*;
use chrono::Utc;

// Create a path planner
let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

// Create a route request
let route_request = RouteRequest {
    request_id: "req-123".to_string(),
    agent_id: "agent-1".to_string(),
    start_position: Position2D { x: 10.0, y: 20.0 },
    destination_position: Position2D { x: 50.0, y: 80.0 },
    start_orientation: Orientation2D { angle: 0.0 },        // Facing east
    destination_orientation: Orientation2D { angle: 1.57 }, // Facing north (90 degrees)
    created_at: Utc::now(),
};

// Submit the route request
match planner.handle_route_request(route_request) {
    Ok(()) => {
        println!("Route request accepted and processing started");
        
        // Check the emitted events
        for event in planner.uncommitted_events() {
            match event {
                PathPlanningEvent::RouteRequested { plan_id, agent_id, .. } => {
                    println!("Plan {} created for agent {}", plan_id, agent_id);
                }
                _ => {}
            }
        }
        
        // Check active plans
        for plan in &planner.active_plans {
            println!("Active plan: {} for agent {}", plan.id, plan.agent_id);
            println!("Status: {:?}", plan.status);
        }
    }
    Err(error) => {
        println!("Route request failed: {}", error);
    }
}
```

## Event Flow

When a `RouteRequest` is successfully handled:

1. **Validation**: The planner validates that start and destination positions are within workspace bounds
2. **Plan Creation**: A new plan ID is generated and a `PathPlan` is prepared
3. **Event Emission**: A `RouteRequested` event is emitted with all request details
4. **State Update**: The event is applied to add the plan to the active plans list

## Error Handling

The command validates:
- Start position is within workspace bounds
- Destination position is within workspace bounds

If validation fails, a `DomainError::InvalidCommand` is returned with a descriptive reason.

## Integration with Event Sourcing

The `RouteRequested` event contains:
- All original request data (for audit trail)
- Generated plan ID
- Timestamp of processing
- Planner ID

This event can be consumed by:
- Path planning projections (for analytics)
- External systems monitoring route requests
- Planning algorithms (to begin actual path computation)
- Agent management systems (for coordination)

## Next Steps

After a route request is accepted, typically:
1. A path planning algorithm processes the request
2. A `PlanCompleted` or `PlanFailed` event is emitted
3. The resulting path is made available to the requesting agent
