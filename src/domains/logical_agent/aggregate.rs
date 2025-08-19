use super::events::LogicalAgentEvent;
use crate::common::{AggregateRoot, DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalAgent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub objectives: Vec<Objective>,
    pub knowledge_base: KnowledgeBase,
    pub decision_tree: DecisionTree,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<LogicalAgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Planning,
    Executing,
    Paused,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub status: ObjectiveStatus,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: Uuid,
    pub statement: String,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub condition: String,
    pub action: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    pub root_node: Option<DecisionNode>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub id: Uuid,
    pub condition: String,
    pub action: Option<String>,
    pub children: Vec<DecisionNode>,
}

impl LogicalAgent {
    pub fn new(id: String, name: String) -> Self {
        let mut agent = Self {
            id: id.clone(),
            name: name.clone(),
            status: AgentStatus::Idle,
            objectives: Vec::new(),
            knowledge_base: KnowledgeBase {
                facts: Vec::new(),
                rules: Vec::new(),
                last_updated: chrono::Utc::now(),
            },
            decision_tree: DecisionTree {
                root_node: None,
                version: 0,
            },
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = LogicalAgentEvent::AgentCreated {
            agent_id: id,
            name,
            timestamp: chrono::Utc::now(),
        };

        agent.add_event(event);
        agent
    }

    pub fn add_objective(
        &mut self,
        description: String,
        priority: u8,
        constraints: Vec<String>,
    ) -> DomainResult<Uuid> {
        if priority > 10 {
            return Err(DomainError::InvalidCommand {
                reason: "Priority must be between 0 and 10".to_string(),
            });
        }

        let objective_id = Uuid::new_v4();
        let objective = Objective {
            id: objective_id,
            description: description.clone(),
            priority,
            status: ObjectiveStatus::Pending,
            constraints: constraints.clone(),
        };

        self.objectives.push(objective);

        let event = LogicalAgentEvent::ObjectiveAdded {
            agent_id: self.id.clone(),
            objective_id,
            description,
            priority,
            constraints,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(objective_id)
    }

    pub fn update_status(&mut self, new_status: AgentStatus) -> DomainResult<()> {
        self.status = new_status.clone();

        let event = LogicalAgentEvent::StatusChanged {
            agent_id: self.id.clone(),
            new_status,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(())
    }

    pub fn add_fact(
        &mut self,
        statement: String,
        confidence: f64,
        source: String,
    ) -> DomainResult<Uuid> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(DomainError::InvalidCommand {
                reason: "Confidence must be between 0.0 and 1.0".to_string(),
            });
        }

        let fact_id = Uuid::new_v4();
        let fact = Fact {
            id: fact_id,
            statement: statement.clone(),
            confidence,
            source: source.clone(),
        };

        self.knowledge_base.facts.push(fact);
        self.knowledge_base.last_updated = chrono::Utc::now();

        let event = LogicalAgentEvent::FactAdded {
            agent_id: self.id.clone(),
            fact_id,
            statement,
            confidence,
            source,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(fact_id)
    }
}

impl AggregateRoot for LogicalAgent {
    type Event = LogicalAgentEvent;

    fn aggregate_id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) -> DomainResult<()> {
        match event {
            LogicalAgentEvent::AgentCreated { agent_id, name, .. } => {
                self.id = agent_id.clone();
                self.name = name.clone();
                self.status = AgentStatus::Idle;
            }
            LogicalAgentEvent::ObjectiveAdded {
                objective_id,
                description,
                priority,
                constraints,
                ..
            } => {
                let objective = Objective {
                    id: *objective_id,
                    description: description.clone(),
                    priority: *priority,
                    status: ObjectiveStatus::Pending,
                    constraints: constraints.clone(),
                };
                self.objectives.push(objective);
            }
            LogicalAgentEvent::StatusChanged { new_status, .. } => {
                self.status = new_status.clone();
            }
            LogicalAgentEvent::FactAdded {
                fact_id,
                statement,
                confidence,
                source,
                ..
            } => {
                let fact = Fact {
                    id: *fact_id,
                    statement: statement.clone(),
                    confidence: *confidence,
                    source: source.clone(),
                };
                self.knowledge_base.facts.push(fact);
                self.knowledge_base.last_updated = chrono::Utc::now();
            }
            _ => {
                // Handle other events as needed
            }
        }
        self.version += 1;
        Ok(())
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    fn add_event(&mut self, event: Self::Event) {
        self.uncommitted_events.push(event);
    }
}
