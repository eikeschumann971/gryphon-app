use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use super::events::LogicalAgentEvent;
use super::aggregate::{AgentStatus, ObjectiveStatus};

/// Projection for logical agent overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalAgentOverview {
    pub agent_id: String,
    pub name: String,
    pub status: AgentStatus,
    pub objectives_count: usize,
    pub completed_objectives: usize,
    pub facts_count: usize,
    pub rules_count: usize,
    pub last_activity: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LogicalAgentOverview {
    pub fn new(agent_id: String, name: String, created_at: DateTime<Utc>) -> Self {
        Self {
            agent_id,
            name,
            status: AgentStatus::Idle,
            objectives_count: 0,
            completed_objectives: 0,
            facts_count: 0,
            rules_count: 0,
            last_activity: created_at,
            created_at,
        }
    }

    pub fn apply_event(&mut self, event: &LogicalAgentEvent) {
        match event {
            LogicalAgentEvent::AgentCreated { name, timestamp, .. } => {
                self.name = name.clone();
                self.created_at = *timestamp;
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::ObjectiveAdded { timestamp, .. } => {
                self.objectives_count += 1;
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::ObjectiveCompleted { timestamp, .. } => {
                self.completed_objectives += 1;
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::StatusChanged { new_status, timestamp, .. } => {
                self.status = new_status.clone();
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::FactAdded { timestamp, .. } => {
                self.facts_count += 1;
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::RuleAdded { timestamp, .. } => {
                self.rules_count += 1;
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::DecisionMade { timestamp, .. } => {
                self.last_activity = *timestamp;
            }
            LogicalAgentEvent::KnowledgeBaseUpdated { timestamp, .. } => {
                self.last_activity = *timestamp;
            }
            _ => {}
        }
    }
}

/// Projection for objectives tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProjection {
    pub objective_id: Uuid,
    pub agent_id: String,
    pub description: String,
    pub priority: u8,
    pub status: ObjectiveStatus,
    pub constraints: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ObjectiveProjection {
    pub fn new(
        objective_id: Uuid,
        agent_id: String,
        description: String,
        priority: u8,
        constraints: Vec<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            objective_id,
            agent_id,
            description,
            priority,
            status: ObjectiveStatus::Pending,
            constraints,
            created_at,
            completed_at: None,
        }
    }

    pub fn apply_event(&mut self, event: &LogicalAgentEvent) {
        match event {
            LogicalAgentEvent::ObjectiveCompleted { objective_id, timestamp, .. } => {
                if *objective_id == self.objective_id {
                    self.status = ObjectiveStatus::Completed;
                    self.completed_at = Some(*timestamp);
                }
            }
            LogicalAgentEvent::ObjectiveFailed { objective_id, reason, timestamp, .. } => {
                if *objective_id == self.objective_id {
                    self.status = ObjectiveStatus::Failed(reason.clone());
                    self.completed_at = Some(*timestamp);
                }
            }
            _ => {}
        }
    }
}

/// Projection for knowledge base analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseAnalytics {
    pub agent_id: String,
    pub total_facts: usize,
    pub total_rules: usize,
    pub facts_by_source: HashMap<String, usize>,
    pub average_confidence: f64,
    pub last_updated: DateTime<Utc>,
}

impl KnowledgeBaseAnalytics {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            total_facts: 0,
            total_rules: 0,
            facts_by_source: HashMap::new(),
            average_confidence: 0.0,
            last_updated: Utc::now(),
        }
    }

    pub fn apply_event(&mut self, event: &LogicalAgentEvent) {
        match event {
            LogicalAgentEvent::FactAdded { source, confidence, timestamp, .. } => {
                self.total_facts += 1;
                *self.facts_by_source.entry(source.clone()).or_insert(0) += 1;
                
                // Simple running average calculation
                self.average_confidence = 
                    (self.average_confidence * (self.total_facts - 1) as f64 + confidence) / self.total_facts as f64;
                
                self.last_updated = *timestamp;
            }
            LogicalAgentEvent::RuleAdded { timestamp, .. } => {
                self.total_rules += 1;
                self.last_updated = *timestamp;
            }
            _ => {}
        }
    }
}
