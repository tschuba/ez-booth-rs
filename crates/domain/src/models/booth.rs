use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::shared::{BoothId, VendorId};

/// Represents a bazaar booth/event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Booth {
    pub id: BoothId,
    
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    
    pub date: NaiveDate,
    
    #[validate]
    pub fees: FeeConfig,
    
    pub status: BoothStatus,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FeeConfig {
    #[validate(range(min = 0.0))]
    pub participation_fee: Decimal,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub sales_fee_percent: Decimal,
    
    #[validate(range(min = 0.0))]
    pub rounding_step: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum BoothStatus {
    Open,
    Closed { closed_at: DateTime<Utc> },
}

impl Booth {
    pub fn new(description: String, date: NaiveDate, fees: FeeConfig) -> Self {
        let now = Utc::now();
        Self {
            id: BoothId::new(),
            description,
            date,
            fees,
            status: BoothStatus::Open,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn close(&mut self) {
        self.status = BoothStatus::Closed {
            closed_at: Utc::now(),
        };
        self.updated_at = Utc::now();
    }

    pub fn is_open(&self) -> bool {
        matches!(self.status, BoothStatus::Open)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.status, BoothStatus::Closed { .. })
    }

    pub fn update_description(&mut self, description: String) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn update_fees(&mut self, fees: FeeConfig) {
        self.fees = fees;
        self.updated_at = Utc::now();
    }
}

/// Summary statistics for a booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothSummary {
    pub booth_id: BoothId,
    pub total_revenue: Decimal,
    pub total_purchases: usize,
    pub unique_vendors: usize,
    pub vendor_summaries: Vec<VendorBoothSummary>,
}

/// Per-vendor statistics within a booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorBoothSummary {
    pub vendor_id: VendorId,
    pub gross_sales: Decimal,
    pub fees_due: Decimal,
    pub net_payout: Decimal,
    pub purchase_count: usize,
}
