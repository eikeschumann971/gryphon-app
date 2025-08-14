//! Core domain types and protobuf interop for Gryphon.
//!
//! Modules:
//! - agent: in-memory simulation agent model
//! - path: path representation & helpers
//! - batch: grouping of agents (e.g. for streaming frames)
//! - error: CoreError
//! - proto: generated protobuf (do not edit)

pub mod error;
pub mod agent;
pub mod path;
pub mod batch;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/gryphon.rs"));
}

pub mod prelude {
    pub use crate::agent::{Agent, AgentUpdate};
    pub use crate::path::{Path, PathPoint};
    pub use crate::batch::AgentBatch;
    pub use crate::error::CoreError;
}