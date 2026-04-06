use crate::error::StorageError;
use rexie::{Index, ObjectStore, Rexie, TransactionMode};

const DB_NAME: &str = "ez_booth_v1";
pub const DB_VERSION: u32 = 5;

pub struct Database {
    db: Rexie,
}

impl Database {
    pub async fn new() -> Result<Self, StorageError> {
        Self::new_with_name(DB_NAME).await
    }

    /// Create a new database with a custom name (useful for testing)
    pub async fn new_with_name(db_name: &str) -> Result<Self, StorageError> {
        let rexie = Rexie::builder(db_name)
            .version(DB_VERSION)
            .add_object_store(
                ObjectStore::new("booths")
                    .key_path("id")
                    .add_index(Index::new("date", "date"))
                    .add_index(Index::new_array(
                        "description_date",
                        ["description", "date"],
                    )),
            )
            .add_object_store(
                ObjectStore::new("vendors")
                    .key_path_array(["booth_id", "id"])
                    .add_index(Index::new("booth_id", "booth_id")),
            )
            .add_object_store(
                ObjectStore::new("purchases")
                    .key_path_array(["booth_id", "id"])
                    .add_index(Index::new("booth_id", "booth_id"))
                    .add_index(Index::new("purchased_at", "purchased_at")),
            )
            .add_object_store(
                ObjectStore::new("error_logs")
                    .auto_increment(true)
                    .add_index(Index::new("timestamp", "timestamp")),
            )
            .add_object_store(ObjectStore::new("export_records").key_path("booth_id"))
            .add_object_store(
                ObjectStore::new("archive_audit_log")
                    .key_path("id")
                    .add_index(Index::new("timestamp", "timestamp"))
                    .add_index(Index::new("booth_id", "booth_id")),
            )
            .add_object_store(ObjectStore::new("metadata").key_path("key"))
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
