use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::shared::{BoothId, ItemId, PurchaseId, VendorId};

/// Represents a purchase transaction with multiple items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    pub items: Vec<PurchaseItem>,
    pub timestamp: DateTime<Utc>,
    pub note: Option<String>,
}

/// Individual item within a purchase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub id: ItemId,
    pub amount: Decimal,
    pub vendor_id: VendorId,
}

impl Purchase {
    pub fn new(booth_id: BoothId, items: Vec<PurchaseItem>) -> Self {
        Self {
            id: PurchaseId::new(),
            booth_id,
            items,
            timestamp: Utc::now(),
            note: None,
        }
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    /// Calculate total amount of all items in the purchase
    pub fn total_amount(&self) -> Decimal {
        self.items.iter().map(|item| item.amount).sum()
    }

    /// Get unique vendor IDs from all items in this purchase
    pub fn get_vendor_ids(&self) -> Vec<VendorId> {
        use std::collections::HashSet;
        let mut vendors: Vec<VendorId> = self
            .items
            .iter()
            .map(|item| item.vendor_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        vendors.sort();
        vendors
    }

    /// Get the primary vendor (most common vendor in items, or first if tied)
    pub fn primary_vendor_id(&self) -> Option<VendorId> {
        use std::collections::HashMap;
        let mut counts: HashMap<VendorId, usize> = HashMap::new();
        for item in &self.items {
            *counts.entry(item.vendor_id.clone()).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(vendor_id, _)| vendor_id)
    }
}

impl PurchaseItem {
    pub fn new(amount: Decimal, vendor_id: VendorId) -> Self {
        Self {
            id: ItemId::new(),
            amount,
            vendor_id,
        }
    }
}
