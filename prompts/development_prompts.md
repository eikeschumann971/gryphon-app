# Development Prompts

## Domain Modeling Prompts

### When adding a new domain:
```
Create a new domain following the DDD pattern with:
1. Aggregate root with business logic
2. Domain events for all state changes  
3. Command and event actors for CQRS
4. Projections for read models
5. Application service for orchestration
```

### When extending existing domains:
```
Add new functionality to [domain] that:
1. Maintains aggregate consistency
2. Emits appropriate domain events
3. Updates relevant projections
4. Follows existing patterns
```

## Event Sourcing Prompts

### When designing events:
```
Design events that are:
1. Immutable and serializable
2. Contain all necessary data for replay
3. Have clear business meaning
4. Include proper metadata (timestamp, correlation_id)
```

### When adding projections:
```
Create a projection that:
1. Rebuilds from events
2. Optimizes for read queries
3. Handles event ordering
4. Provides eventual consistency
```

## Architecture Prompts

### When adding infrastructure:
```
Implement [component] that:
1. Follows ports and adapters pattern
2. Has proper error handling
3. Includes comprehensive tests
4. Maintains clean interfaces
```

## Testing Prompts

### When writing tests:
```
Create tests that cover:
1. Aggregate business rules
2. Event sourcing replay
3. Projection consistency
4. Integration scenarios
5. Error conditions
```
