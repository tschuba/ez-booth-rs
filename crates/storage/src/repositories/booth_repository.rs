use async_trait::async_trait;
use domain::{Booth, BoothId, BoothRepository, DomainResult};
use rexie::TransactionMode;
use serde_wasm_bindgen::{from_value, to_value};
use std::sync::Arc;
use wasm_bindgen::JsValue;

use crate::error::StorageError;
use crate::indexeddb::Database;

pub struct IndexedDbBoothRepository {
    db: Arc<Database>,
}

impl IndexedDbBoothRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait(?Send)]
impl BoothRepository for IndexedDbBoothRepository {
    async fn save(&self, booth: &Booth) -> DomainResult<()> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        let value = to_value(booth)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        store
            .put(&value, None)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        Ok(())
    }
    
    async fn find_by_id(&self, id: &BoothId) -> DomainResult<Option<Booth>> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        let key = JsValue::from_str(&id.as_str());
        
        let result = store
            .get(key)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        match result {
            Some(value) => {
                let booth: Booth = from_value(value)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(booth))
            }
            None => Ok(None),
        }
    }
    
    async fn find_all(&self) -> DomainResult<Vec<Booth>> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        let values = store
            .get_all(None, None)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        let booths: Vec<Booth> = values
            .into_iter()
            .filter_map(|value| from_value(value).ok())
            .collect();
        
        Ok(booths)
    }
    
    async fn delete(&self, id: &BoothId) -> DomainResult<()> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        let key = JsValue::from_str(&id.as_str());
        
        store
            .delete(key)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;
        
        Ok(())
    }
}
