use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::shared::{Id, Money, VendorId};

/// Represents a vendor at the booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: Id,
    pub vendor_id: VendorId,
    pub booth_id: Id,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Vendor {
    pub fn new(vendor_id: VendorId, booth_id: Id) -> Self {
        Self {
            id: Id::new(),
            vendor_id,
            booth_id,
            name: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

/// Summary statistics for a vendor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSummary {
    pub vendor_id: VendorId,
    pub vendor_name: Option<String>,
    pub total_revenue: Money,
    pub purchase_count: usize,
}

impl VendorSummary {
    pub fn new(vendor_id: VendorId, vendor_name: Option<String>) -> Self {
        Self {
            vendor_id,
            vendor_name,
            total_revenue: Money::default(),
            purchase_count: 0,
        }
    }

    pub fn add_purchase(&mut self, amount: Money) {
        self.total_revenue = self.total_revenue.add(&amount);
        self.purchase_count += 1;
    }
}
