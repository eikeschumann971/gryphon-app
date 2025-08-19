use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Aggregate not found: {id}")]
    AggregateNotFound { id: String },

    #[error("Invalid command: {reason}")]
    InvalidCommand { reason: String },

    #[error("Concurrency conflict: expected version {expected}, got {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
}

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),

    #[error("Configuration error: {0}")]
    Configuration(#[from] anyhow::Error),
}

pub type DomainResult<T> = Result<T, DomainError>;
pub type ApplicationResult<T> = Result<T, ApplicationError>;
