use crate::proto::gryphon;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// State of a single agent within the simulation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    /// Heading in radians.
    pub heading: f64,
}

/// A minimal trait to generalize update logic (extensible later).
pub trait AgentUpdate {
    fn advance(&mut self, dt: f64);
}

impl AgentUpdate for Agent {
    fn advance(&mut self, dt: f64) {
        // Placeholder: simple forward movement along heading at 1 unit/sec.
        let speed = 1.0;
        self.x += self.heading.cos() * speed * dt;
        self.y += self.heading.sin() * speed * dt;
    }
}

impl From<&Agent> for gryphon::AgentState {
    fn from(a: &Agent) -> Self {
        gryphon::AgentState {
            id: a.id,
            x: a.x,
            y: a.y,
            heading: a.heading,
        }
    }
}

impl From<gryphon::AgentState> for Agent {
    fn from(p: gryphon::AgentState) -> Self {
        Agent {
            id: p.id,
            x: p.x,
            y: p.y,
            heading: p.heading,
        }
    }
}
}

impl Agent {
    pub fn new(id: u32, x: f64, y: f64, heading: f64) -> Self {
        Self { id, x, y, heading }
    }
}