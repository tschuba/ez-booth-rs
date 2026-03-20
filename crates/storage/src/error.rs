use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("JS error: {0}")]
    JsError(String),
}

impl From<JsValue> for StorageError {
    fn from(err: JsValue) -> Self {
        StorageError::JsError(format!("{:?}", err))
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError(err.to_string())
    }
}

impl From<StorageError> for ez_booth_core::error::CoreError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound(msg) => ez_booth_core::error::CoreError::NotFound(msg),
            other => ez_booth_core::error::CoreError::StorageError(other.to_string()),
        }
    }
}
