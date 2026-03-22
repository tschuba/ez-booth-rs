use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::shared::{BoothId, VendorId};
use crate::error::DomainError;

/// Represents a bazaar booth/event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Booth {
    pub id: BoothId,

    #[validate(length(min = 1, max = 200))]
    pub description: String,

    pub date: NaiveDate,

    #[validate(nested)]
    pub fees: FeeConfig,

    pub status: BoothStatus,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fee configuration for booth charges
///
/// Note: The validator crate doesn't support range validation for rust_decimal::Decimal.
/// Range validation is performed via the `validate_ranges()` method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct FeeConfig {
    /// Fixed participation fee per vendor
    pub participation_fee: Decimal,

    /// Sales commission percentage (0-100)
    pub sales_fee_percent: Decimal,

    /// Rounding step for fee calculations (e.g., 0.50 for half-dollar rounding)
    pub rounding_step: Decimal,
}

impl FeeConfig {
    /// Validate that all fee values are within acceptable ranges
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if:
    /// - Any fee value is negative
    /// - Sales fee percent is greater than 100
    pub fn validate_ranges(&self) -> Result<(), DomainError> {
        if self.participation_fee.is_sign_negative() {
            return Err(DomainError::Validation(
                "Participation fee cannot be negative".to_string(),
            ));
        }

        if self.sales_fee_percent.is_sign_negative() {
            return Err(DomainError::Validation(
                "Sales fee percent cannot be negative".to_string(),
            ));
        }

        if self.sales_fee_percent > Decimal::new(100, 0) {
            return Err(DomainError::Validation(
                "Sales fee percent cannot exceed 100%".to_string(),
            ));
        }

        if self.rounding_step.is_sign_negative() {
            return Err(DomainError::Validation(
                "Rounding step cannot be negative".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum BoothStatus {
    Open,
    Closed { closed_at: DateTime<Utc> },
}

impl Booth {
    /// Create a new booth with the given configuration
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the fee configuration is invalid
    pub fn new(description: String, date: NaiveDate, fees: FeeConfig) -> Result<Self, DomainError> {
        // Validate fee configuration
        fees.validate_ranges()?;

        let now = Utc::now();
        Ok(Self {
            id: BoothId::new(),
            description,
            date,
            fees,
            status: BoothStatus::Open,
            created_at: now,
            updated_at: now,
        })
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
