use async_trait::async_trait;
use domain::{
    BoothId, BoothRunningTotals, DomainResult, PaginatedPurchases, Purchase, PurchaseId,
    PurchaseRepository, VendorId,
};
use rexie::TransactionMode;
use rust_decimal::Decimal;
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

        let value =
            to_value(purchase).map_err(|e| StorageError::SerializationError(e.to_string()))?;

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
            .get_all(
                Some(rexie::KeyRange::only(&key).map_err(StorageError::from)?),
                None,
            )
            .await
            .map_err(|e| StorageError::DatabaseError(format!("{:?}", e)))?;

        let purchases: Vec<Purchase> = values
            .into_iter()
            .filter_map(|value| from_value(value).ok())
            .collect();

        Ok(purchases)
    }

    async fn find_by_booth_paginated(
        &self,
        booth_id: &BoothId,
        offset: usize,
        limit: usize,
    ) -> DomainResult<PaginatedPurchases> {
        // Get all purchases for the booth
        let mut all_purchases = self.find_by_booth(booth_id).await?;
        
        // Sort by timestamp descending (newest first)
        all_purchases.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        let total_count = all_purchases.len();
        
        // Apply pagination
        let items: Vec<Purchase> = all_purchases
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        Ok(PaginatedPurchases { items, total_count })
    }

    async fn get_running_totals(&self, booth_id: &BoothId) -> DomainResult<BoothRunningTotals> {
        let purchases = self.find_by_booth(booth_id).await?;
        
        let total_sales: Decimal = purchases.iter().map(|p| p.total_amount()).sum();
        let total_items: usize = purchases.iter().map(|p| p.items.len()).sum();
        let total_checkouts = purchases.len();
        
        Ok(BoothRunningTotals {
            total_sales,
            total_items,
            total_checkouts,
        })
    }

    async fn find_by_vendor(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
    ) -> DomainResult<Vec<Purchase>> {
        // Get all purchases for the booth, then filter by vendor
        // Note: vendor_id is now on PurchaseItem, not Purchase
        let all_purchases = self.find_by_booth(booth_id).await?;

        let filtered: Vec<Purchase> = all_purchases
            .into_iter()
            .filter(|p| p.items.iter().any(|item| &item.vendor_id == vendor_id))
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
