use crate::common::{EventStore, EventEnvelope, EventMetadata};
use super::events::TechnicalAgentEvent;
use super::projections::TechnicalAgentOverview;
use super::aggregate::AgentType;
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct TechnicalAgentEventActor {
    event_store: Arc<dyn EventStore + Send + Sync>,
    projection_store: Arc<RwLock<TechnicalAgentProjectionStore>>,
    event_receiver: mpsc::Receiver<TechnicalAgentEvent>,
}

impl TechnicalAgentEventActor {
    pub fn new(
        event_store: Arc<dyn EventStore + Send + Sync>,
        event_receiver: mpsc::Receiver<TechnicalAgentEvent>,
    ) -> Self {
        Self {
            event_store,
            projection_store: Arc::new(RwLock::new(TechnicalAgentProjectionStore::new())),
            event_receiver,
        }
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.event_receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                tracing::error!("Failed to handle technical agent event: {}", e);
            }
        }
    }

    async fn handle_event(&self, event: TechnicalAgentEvent) -> Result<(), String> {
        let metadata = EventMetadata {
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            user_id: None,
            source: "TechnicalAgentEventActor".to_string(),
        };

        let envelope = EventEnvelope::new(&event, "TechnicalAgent", metadata)
            .map_err(|e| format!("Failed to create event envelope: {}", e))?;

        self.event_store
            .append_events(event.aggregate_id(), 0, vec![envelope])
            .await?;

        self.update_projections(&event).await;

        tracing::info!("Handled technical agent event: {:?}", event.event_type());
        Ok(())
    }

    async fn update_projections(&self, event: &TechnicalAgentEvent) {
        let mut store = self.projection_store.write().await;
        store.apply_event(event);
    }

    pub async fn get_agent_overview(&self, agent_id: &str) -> Option<TechnicalAgentOverview> {
        let store = self.projection_store.read().await;
        store.agent_overviews.get(agent_id).cloned()
    }
}

#[derive(Debug)]
pub struct TechnicalAgentProjectionStore {
    pub agent_overviews: HashMap<String, TechnicalAgentOverview>,
}

impl TechnicalAgentProjectionStore {
    pub fn new() -> Self {
        Self {
            agent_overviews: HashMap::new(),
        }
    }

    pub fn apply_event(&mut self, event: &TechnicalAgentEvent) {
        match event {
            TechnicalAgentEvent::AgentCreated { agent_id, name, agent_type, timestamp, .. } => {
                let overview = TechnicalAgentOverview::new(
                    agent_id.clone(), 
                    name.clone(), 
                    agent_type.clone(), 
                    *timestamp
                );
                self.agent_overviews.insert(agent_id.clone(), overview);
            }
            _ => {}
        }

        if let Some(overview) = self.agent_overviews.get_mut(event.aggregate_id()) {
            overview.apply_event(event);
        }
    }
}

pub struct TechnicalAgentCommandActor {
    event_sender: mpsc::Sender<TechnicalAgentEvent>,
}

impl TechnicalAgentCommandActor {
    pub fn new(event_sender: mpsc::Sender<TechnicalAgentEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_agent(&self, agent_id: String, name: String, agent_type: AgentType) -> Result<(), String> {
        let event = TechnicalAgentEvent::AgentCreated {
            agent_id,
            name,
            agent_type,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
