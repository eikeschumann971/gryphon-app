use crate::common::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::aggregate::AgentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalAgentEvent {
    AgentCreated {
        agent_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },
    ObjectiveAdded {
        agent_id: String,
        objective_id: Uuid,
        description: String,
        priority: u8,
        constraints: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    ObjectiveCompleted {
        agent_id: String,
        objective_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    ObjectiveFailed {
        agent_id: String,
        objective_id: Uuid,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    StatusChanged {
        agent_id: String,
        new_status: AgentStatus,
        timestamp: DateTime<Utc>,
    },
    FactAdded {
        agent_id: String,
        fact_id: Uuid,
        statement: String,
        confidence: f64,
        source: String,
        timestamp: DateTime<Utc>,
    },
    RuleAdded {
        agent_id: String,
        rule_id: Uuid,
        condition: String,
        action: String,
        priority: u8,
        timestamp: DateTime<Utc>,
    },
    DecisionMade {
        agent_id: String,
        decision_id: Uuid,
        context: String,
        decision: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    KnowledgeBaseUpdated {
        agent_id: String,
        update_type: String,
        details: String,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for LogicalAgentEvent {
    fn event_type(&self) -> &'static str {
        match self {
            LogicalAgentEvent::AgentCreated { .. } => "LogicalAgentCreated",
            LogicalAgentEvent::ObjectiveAdded { .. } => "ObjectiveAdded",
            LogicalAgentEvent::ObjectiveCompleted { .. } => "ObjectiveCompleted",
            LogicalAgentEvent::ObjectiveFailed { .. } => "ObjectiveFailed",
            LogicalAgentEvent::StatusChanged { .. } => "StatusChanged",
            LogicalAgentEvent::FactAdded { .. } => "FactAdded",
            LogicalAgentEvent::RuleAdded { .. } => "RuleAdded",
            LogicalAgentEvent::DecisionMade { .. } => "DecisionMade",
            LogicalAgentEvent::KnowledgeBaseUpdated { .. } => "KnowledgeBaseUpdated",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            LogicalAgentEvent::AgentCreated { agent_id, .. } => agent_id,
            LogicalAgentEvent::ObjectiveAdded { agent_id, .. } => agent_id,
            LogicalAgentEvent::ObjectiveCompleted { agent_id, .. } => agent_id,
            LogicalAgentEvent::ObjectiveFailed { agent_id, .. } => agent_id,
            LogicalAgentEvent::StatusChanged { agent_id, .. } => agent_id,
            LogicalAgentEvent::FactAdded { agent_id, .. } => agent_id,
            LogicalAgentEvent::RuleAdded { agent_id, .. } => agent_id,
            LogicalAgentEvent::DecisionMade { agent_id, .. } => agent_id,
            LogicalAgentEvent::KnowledgeBaseUpdated { agent_id, .. } => agent_id,
        }
    }

    fn event_version(&self) -> u64 {
        1 // You might want to implement proper versioning
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            LogicalAgentEvent::AgentCreated { timestamp, .. } => *timestamp,
            LogicalAgentEvent::ObjectiveAdded { timestamp, .. } => *timestamp,
            LogicalAgentEvent::ObjectiveCompleted { timestamp, .. } => *timestamp,
            LogicalAgentEvent::ObjectiveFailed { timestamp, .. } => *timestamp,
            LogicalAgentEvent::StatusChanged { timestamp, .. } => *timestamp,
            LogicalAgentEvent::FactAdded { timestamp, .. } => *timestamp,
            LogicalAgentEvent::RuleAdded { timestamp, .. } => *timestamp,
            LogicalAgentEvent::DecisionMade { timestamp, .. } => *timestamp,
            LogicalAgentEvent::KnowledgeBaseUpdated { timestamp, .. } => *timestamp,
        }
    }
}
