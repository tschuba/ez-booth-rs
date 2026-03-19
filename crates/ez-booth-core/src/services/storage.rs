use crate::models::Event;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Event not found: {0}")]
    NotFound(Uuid),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

/// Storage abstraction for events
pub trait EventStorage: Send + Sync {
    fn save_event(&self, event: &Event) -> StorageResult<()>;
    fn load_event(&self, id: &Uuid) -> StorageResult<Event>;
    fn load_all_events(&self) -> StorageResult<Vec<Event>>;
    fn delete_event(&self, id: &Uuid) -> StorageResult<()>;
    fn export_data(&self) -> StorageResult<String>;
    fn import_data(&self, data: &str) -> StorageResult<Vec<Event>>;
}
