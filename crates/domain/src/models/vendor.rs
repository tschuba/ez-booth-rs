use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::shared::{BoothId, VendorId};

/// Represents a vendor at the booth
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vendor {
    pub vendor_id: VendorId,
    pub booth_id: BoothId,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Vendor {
    pub fn new(vendor_id: VendorId, booth_id: BoothId) -> Self {
        Self {
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

/// Summary statistics for a vendor across all booths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSummary {
    pub vendor_id: VendorId,
    pub vendor_name: Option<String>,
    pub total_revenue: Decimal,
    pub purchase_count: usize,
}

impl VendorSummary {
    pub fn new(vendor_id: VendorId, vendor_name: Option<String>) -> Self {
        Self {
            vendor_id,
            vendor_name,
            total_revenue: Decimal::ZERO,
            purchase_count: 0,
        }
    }

    pub fn add_purchase(&mut self, amount: Decimal) {
        self.total_revenue += amount;
        self.purchase_count += 1;
    }
}
