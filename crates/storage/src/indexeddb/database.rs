use rexie::{Rexie, TransactionMode, ObjectStore, Index};
use crate::error::StorageError;

const DB_NAME: &str = "ez_booth_v1";
const DB_VERSION: u32 = 1;

pub struct Database {
    db: Rexie,
}

impl Database {
    pub async fn new() -> Result<Self, StorageError> {
        let rexie = Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(
                ObjectStore::new("booths")
                    .key_path("id")
                    .add_index(Index::new("date", "date"))
                    .add_index(Index::new("status", "status.type"))
            )
            .add_object_store(
                ObjectStore::new("vendors")
                    .key_path_array(["booth_id", "id"])
                    .add_index(Index::new("booth_id", "booth_id"))
            )
            .add_object_store(
                ObjectStore::new("purchases")
                    .key_path_array(["booth_id", "id"])
                    .add_index(Index::new("booth_id", "booth_id"))
                    .add_index(Index::new("vendor_id", "vendor_id"))
                    .add_index(Index::new("purchased_at", "purchased_at"))
            )
            .add_object_store(
                ObjectStore::new("metadata")
                    .key_path("key")
            )
            .build()
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;
        
        Ok(Self { db: rexie })
    }
    
    pub fn db(&self) -> &Rexie {
        &self.db
    }
    
    /// Get a transaction for the specified stores
    pub fn transaction(
        &self,
        store_names: &[&str],
        mode: TransactionMode,
    ) -> Result<rexie::Transaction, StorageError> {
        self.db
            .transaction(store_names, mode)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))
    }
}
