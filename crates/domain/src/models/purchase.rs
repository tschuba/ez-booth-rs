use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::shared::{BoothId, ItemId, PurchaseId, VendorId};

/// Represents a purchase transaction with multiple items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    pub vendor_id: VendorId,
    pub items: Vec<PurchaseItem>,
    pub timestamp: DateTime<Utc>,
    pub note: Option<String>,
}

/// Individual item within a purchase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub id: ItemId,
    pub amount: Decimal,
}

impl Purchase {
    pub fn new(booth_id: BoothId, vendor_id: VendorId, items: Vec<PurchaseItem>) -> Self {
        Self {
            id: PurchaseId::new(),
            booth_id,
            vendor_id,
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
}

impl PurchaseItem {
    pub fn new(amount: Decimal) -> Self {
        Self {
            id: ItemId::new(),
            amount,
        }
    }
}
