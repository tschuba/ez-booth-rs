use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
