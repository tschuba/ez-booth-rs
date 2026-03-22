use async_trait::async_trait;
use domain::{BoothId, DomainResult, Purchase, PurchaseId, PurchaseRepository, VendorId};
use rexie::TransactionMode;
use serde_wasm_bindgen::{from_value, to_value};
use std::sync::Arc;
use wasm_bindgen::JsValue;

use crate::error::StorageError;
use crate::indexeddb::Database;

pub struct IndexedDbPurchaseRepository {
    db: Arc<Database>,
}

impl IndexedDbPurchaseRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait(?Send)]
impl PurchaseRepository for IndexedDbPurchaseRepository {
    async fn save(&self, purchase: &Purchase) -> DomainResult<()> {
        let transaction = self
            .db
            .transaction(&["purchases"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;

        let store = transaction
            .store("purchases")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let value = to_value(purchase)
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

    async fn find_by_id(&self, id: &PurchaseId) -> DomainResult<Option<Purchase>> {
        let transaction = self
            .db
            .transaction(&["purchases"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;

        let store = transaction
            .store("purchases")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let key = JsValue::from_str(&id.as_str());

        let result = store
            .get(key)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        match result {
            Some(value) => {
                let purchase: Purchase = from_value(value)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(purchase))
            }
            None => Ok(None),
        }
    }

    async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Purchase>> {
        let transaction = self
            .db
            .transaction(&["purchases"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;

        let store = transaction
            .store("purchases")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let index = store
            .index("booth_id")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let key = JsValue::from_str(&booth_id.as_str());

        let values = index
            .get_all(Some(rexie::KeyRange::only(&key).map_err(StorageError::from)?), None)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let purchases: Vec<Purchase> = values
            .into_iter()
            .filter_map(|value| from_value(value).ok())
            .collect();

        Ok(purchases)
    }

    async fn find_by_vendor(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
    ) -> DomainResult<Vec<Purchase>> {
        // Get all purchases for the booth, then filter by vendor
        // Note: vendor_id is on Purchase, not on PurchaseItem in the current model
        let all_purchases = self.find_by_booth(booth_id).await?;

        let filtered: Vec<Purchase> = all_purchases
            .into_iter()
            .filter(|p| &p.vendor_id == vendor_id)
            .collect();

        Ok(filtered)
    }

    async fn find_all(&self) -> DomainResult<Vec<Purchase>> {
        let transaction = self
            .db
            .transaction(&["purchases"], TransactionMode::ReadOnly)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;

        let store = transaction
            .store("purchases")
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let values = store
            .get_all(None, None)
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let purchases: Vec<Purchase> = values
            .into_iter()
            .filter_map(|value| from_value(value).ok())
            .collect();

        Ok(purchases)
    }

    async fn delete(&self, id: &PurchaseId) -> DomainResult<()> {
        let transaction = self
            .db
            .transaction(&["purchases"], TransactionMode::ReadWrite)
            .map_err(|e| StorageError::TransactionError(format!("{:?}", e)))?;

        let store = transaction
            .store("purchases")
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
