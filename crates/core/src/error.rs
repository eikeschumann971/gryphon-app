use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("protobuf conversion error: {0}")]
    Proto(String),
}

pub type CoreResult<T> = Result<T, CoreError>;  
