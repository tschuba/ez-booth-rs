use async_trait::async_trait;
use ez_booth_core::entities::booth::Booth;
use ez_booth_core::entities::ids::BoothId;
use ez_booth_core::error::CoreError;
use ez_booth_core::services::booth_service::BoothRepository;
use rexie::TransactionMode;
use serde_json::Value;
use std::sync::Arc;

use crate::indexeddb::Database;
use crate::error::StorageError;

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
    async fn save(&self, booth: &Booth) -> Result<(), CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        let value = serde_json::to_value(booth)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        store
            .put(&value, None)
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        Ok(())
    }
    
    async fn find_by_id(&self, id: BoothId) -> Result<Option<Booth>, CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        let key = serde_json::to_value(id.as_str())
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        let result = store
            .get(&key)
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        match result {
            Some(value) => {
                let booth: Booth = serde_json::from_value(value)
                    .map_err(|e| CoreError::StorageError(e.to_string()))?;
                Ok(Some(booth))
            }
            None => Ok(None),
        }
    }
    
    async fn find_all(&self) -> Result<Vec<Booth>, CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        let values = store
            .get_all(None, None, None, None)
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        let booths: Vec<Booth> = values
            .into_iter()
            .filter_map(|(_, value)| serde_json::from_value(value).ok())
            .collect();
        
        Ok(booths)
    }
    
    async fn delete(&self, id: BoothId) -> Result<(), CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        let key = serde_json::to_value(id.as_str())
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        store
            .delete(&key)
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| CoreError::StorageError(format!("{:?}", e)))?;
        
        Ok(())
    }
}
