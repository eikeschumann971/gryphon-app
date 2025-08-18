use crate::common::{EventStore, EventEnvelope, EventMetadata, DomainEvent};
use super::events::LogicalAgentEvent;
use super::projections::{LogicalAgentOverview, ObjectiveProjection, KnowledgeBaseAnalytics};
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Actor responsible for handling logical agent events
pub struct LogicalAgentEventActor {
    event_store: Arc<dyn EventStore + Send + Sync>,
    projection_store: Arc<RwLock<LogicalAgentProjectionStore>>,
    event_receiver: mpsc::Receiver<LogicalAgentEvent>,
}

impl LogicalAgentEventActor {
    pub fn new(
        event_store: Arc<dyn EventStore + Send + Sync>,
        event_receiver: mpsc::Receiver<LogicalAgentEvent>,
    ) -> Self {
        Self {
            event_store,
            projection_store: Arc::new(RwLock::new(LogicalAgentProjectionStore::new())),
            event_receiver,
        }
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.event_receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                tracing::error!("Failed to handle logical agent event: {}", e);
            }
        }
    }

    async fn handle_event(&self, event: LogicalAgentEvent) -> Result<(), String> {
        // Create event envelope
        let metadata = EventMetadata {
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            user_id: None,
            source: "LogicalAgentEventActor".to_string(),
        };

        let envelope = EventEnvelope::new(&event, "LogicalAgent", metadata)
            .map_err(|e| format!("Failed to create event envelope: {}", e))?;

        // Store event
        self.event_store
            .append_events(event.aggregate_id(), 0, vec![envelope])
            .await?;

        // Update projections
        self.update_projections(&event).await;

        tracing::info!("Handled logical agent event: {:?}", event.event_type());
        Ok(())
    }

    async fn update_projections(&self, event: &LogicalAgentEvent) {
        let mut store = self.projection_store.write().await;
        store.apply_event(event);
    }

    pub async fn get_agent_overview(&self, agent_id: &str) -> Option<LogicalAgentOverview> {
        let store = self.projection_store.read().await;
        store.agent_overviews.get(agent_id).cloned()
    }

    pub async fn get_objectives(&self, agent_id: &str) -> Vec<ObjectiveProjection> {
        let store = self.projection_store.read().await;
        store.objectives
            .values()
            .filter(|obj| obj.agent_id == agent_id)
            .cloned()
            .collect()
    }

    pub async fn get_knowledge_analytics(&self, agent_id: &str) -> Option<KnowledgeBaseAnalytics> {
        let store = self.projection_store.read().await;
        store.knowledge_analytics.get(agent_id).cloned()
    }
}

/// In-memory projection store for logical agent projections
#[derive(Debug)]
pub struct LogicalAgentProjectionStore {
    pub agent_overviews: HashMap<String, LogicalAgentOverview>,
    pub objectives: HashMap<Uuid, ObjectiveProjection>,
    pub knowledge_analytics: HashMap<String, KnowledgeBaseAnalytics>,
}

impl LogicalAgentProjectionStore {
    pub fn new() -> Self {
        Self {
            agent_overviews: HashMap::new(),
            objectives: HashMap::new(),
            knowledge_analytics: HashMap::new(),
        }
    }

}

impl Default for LogicalAgentProjectionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicalAgentProjectionStore {
    pub fn apply_event(&mut self, event: &LogicalAgentEvent) {
        match event {
            LogicalAgentEvent::AgentCreated { agent_id, name, timestamp, .. } => {
                let overview = LogicalAgentOverview::new(agent_id.clone(), name.clone(), *timestamp);
                self.agent_overviews.insert(agent_id.clone(), overview);
                
                let analytics = KnowledgeBaseAnalytics::new(agent_id.clone());
                self.knowledge_analytics.insert(agent_id.clone(), analytics);
            }
            LogicalAgentEvent::ObjectiveAdded { 
                agent_id, 
                objective_id, 
                description, 
                priority, 
                constraints, 
                timestamp, 
                .. 
            } => {
                let objective = ObjectiveProjection::new(
                    *objective_id,
                    agent_id.clone(),
                    description.clone(),
                    *priority,
                    constraints.clone(),
                    *timestamp,
                );
                self.objectives.insert(*objective_id, objective);
            }
            _ => {}
        }

        // Update existing projections
        if let Some(overview) = self.agent_overviews.get_mut(event.aggregate_id()) {
            overview.apply_event(event);
        }

        if let Some(analytics) = self.knowledge_analytics.get_mut(event.aggregate_id()) {
            analytics.apply_event(event);
        }

        // Update objectives if relevant
        match event {
            LogicalAgentEvent::ObjectiveCompleted { objective_id, .. } |
            LogicalAgentEvent::ObjectiveFailed { objective_id, .. } => {
                if let Some(objective) = self.objectives.get_mut(objective_id) {
                    objective.apply_event(event);
                }
            }
            _ => {}
        }
    }
}

/// Actor responsible for processing logical agent commands
pub struct LogicalAgentCommandActor {
    event_sender: mpsc::Sender<LogicalAgentEvent>,
}

impl LogicalAgentCommandActor {
    pub fn new(event_sender: mpsc::Sender<LogicalAgentEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_agent(&self, agent_id: String, name: String) -> Result<(), String> {
        let event = LogicalAgentEvent::AgentCreated {
            agent_id,
            name,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }

    pub async fn add_objective(
        &self,
        agent_id: String,
        description: String,
        priority: u8,
        constraints: Vec<String>,
    ) -> Result<Uuid, String> {
        let objective_id = Uuid::new_v4();
        let event = LogicalAgentEvent::ObjectiveAdded {
            agent_id,
            objective_id,
            description,
            priority,
            constraints,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))?;

        Ok(objective_id)
    }

    pub async fn complete_objective(&self, agent_id: String, objective_id: Uuid) -> Result<(), String> {
        let event = LogicalAgentEvent::ObjectiveCompleted {
            agent_id,
            objective_id,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
