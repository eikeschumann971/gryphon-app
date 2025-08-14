use crate::agent::Agent;
use crate::proto::gryphon;

#[derive(Debug, Clone)]
pub struct AgentBatch {
    pub sequence: u64,
    pub agents: Vec<Agent>,
}

impl AgentBatch {
    pub fn new(sequence: u64, agents: Vec<Agent>) -> Self {
        Self { sequence, agents }
    }

    pub fn to_proto(&self) -> gryphon::AgentBatch {
        gryphon::AgentBatch {
            sequence: self.sequence,
            agents: self.agents.iter().map(|a| a.into()).collect(),
        }
    }

    pub fn from_proto(p: gryphon::AgentBatch) -> Self {
        Self {
            sequence: p.sequence,
            agents: p.agents.into_iter().map(Into::into).collect(),
        }
    }
}  
