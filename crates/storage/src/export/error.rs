use domain::{BoothId, DomainError};
use thiserror::Error;

use crate::StorageError;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Booth not found: {0}")]
    BoothNotFound(BoothId),

    #[error("Failed to serialize backup: {0}")]
    Serialization(String),
}
