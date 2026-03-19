use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoothKey {
    pub booth_id: String,
}

impl PartialOrd for BoothKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BoothKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.booth_id.cmp(&other.booth_id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Booth {
    pub key: BoothKey,
    pub description: String,
    pub date: NaiveDate,
    pub participation_fee: Decimal,
    pub sales_fee: Decimal,
    pub fees_rounding_step: Decimal,
    pub closed: bool,
    pub closed_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseKey {
    pub booth: BoothKey,
    pub purchase_id: String,
}

impl PartialOrd for PurchaseKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PurchaseKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.booth
            .cmp(&other.booth)
            .then_with(|| self.purchase_id.cmp(&other.purchase_id))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub key: PurchaseKey,
    pub value: Decimal,
    pub purchased_on: DateTime<Utc>,
    pub items: Vec<PurchaseItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseItemKey {
    pub purchase: PurchaseKey,
    pub item_id: String,
}

impl PartialOrd for PurchaseItemKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PurchaseItemKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.purchase
            .cmp(&other.purchase)
            .then_with(|| self.item_id.cmp(&other.item_id))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub key: PurchaseItemKey,
    pub vendor: VendorKey,
    pub price: Decimal,
    pub purchased_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorKey {
    pub booth: BoothKey,
    pub vendor_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vendor {
    pub key: VendorKey,
}
