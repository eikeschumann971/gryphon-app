pub mod adapters;
pub mod application;
pub mod common;
pub mod config;
pub mod domains;

pub use config::Config;

// Re-export common types
pub use common::*;

// Re-export domain modules
pub use domains::*;
