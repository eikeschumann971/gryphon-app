use crate::common::{DomainEvent, DomainResult};
use serde::{Deserialize, Serialize};

pub trait AggregateRoot: Send + Sync + Clone {
    type Event: DomainEvent + Serialize + for<'de> Deserialize<'de>;
    
    fn aggregate_id(&self) -> &str;
    fn version(&self) -> u64;
    
    /// Apply an event to update the aggregate state
    fn apply(&mut self, event: &Self::Event) -> DomainResult<()>;
    
    /// Get uncommitted events
    fn uncommitted_events(&self) -> &[Self::Event];
    
    /// Mark events as committed
    fn mark_events_as_committed(&mut self);
    
    /// Add a new event to the uncommitted events list
    fn add_event(&mut self, event: Self::Event);
}

#[derive(Debug, Clone)]
pub struct AggregateStore<T: AggregateRoot> {
    pub aggregate: T,
    pub uncommitted_events: Vec<T::Event>,
    pub version: u64,
}

impl<T: AggregateRoot> AggregateStore<T> {
    pub fn new(aggregate: T) -> Self {
        Self {
            version: aggregate.version(),
            aggregate,
            uncommitted_events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: T::Event) {
        self.uncommitted_events.push(event.clone());
        self.aggregate.apply(&event).expect("Failed to apply event");
        self.version += 1;
    }

    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    pub fn load_from_history(aggregate: T, events: Vec<T::Event>) -> DomainResult<Self> {
        let mut store = Self::new(aggregate);
        
        for event in events {
            store.aggregate.apply(&event)?;
            store.version += 1;
        }
        
        Ok(store)
    }
}
