use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::shared::{BoothId, VendorId};
use crate::error::DomainError;

/// Vendor ID validation rules for a booth
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", content = "pattern")]
pub enum VendorIdValidation {
    /// No restrictions on vendor ID format
    Unrestricted,
    /// Only ASCII digits (0-9) are allowed
    #[default]
    DigitsOnly,
    /// Custom regular expression pattern
    Regex(String),
}

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

    #[serde(default)]
    pub vendor_id_validation: VendorIdValidation,

    #[serde(default)]
    pub keyboard_config: CheckoutKeyboardConfig,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutKeyboardConfig {
    pub quick_amounts: Vec<Decimal>,
}

impl Default for CheckoutKeyboardConfig {
    fn default() -> Self {
        Self {
            quick_amounts: vec![
                Decimal::new(5, 1),
                Decimal::new(1, 0),
                Decimal::new(5, 0),
                Decimal::new(10, 0),
                Decimal::new(15, 0),
            ],
        }
    }
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
    /// Returns `DomainError::Validation` if:
    /// - Description is empty or longer than 200 characters
    /// - Fee configuration is invalid
    pub fn new(description: String, date: NaiveDate, fees: FeeConfig) -> Result<Self, DomainError> {
        // Validate fee configuration
        fees.validate_ranges()?;

        let now = Utc::now();
        let booth = Self {
            id: BoothId::new(),
            description,
            date,
            fees,
            status: BoothStatus::Open,
            vendor_id_validation: VendorIdValidation::default(),
            keyboard_config: CheckoutKeyboardConfig::default(),
            created_at: now,
            updated_at: now,
        };

        // Validate the booth (includes description length validation)
        booth
            .validate()
            .map_err(|e| DomainError::Validation(format!("Invalid booth: {}", e)))?;

        Ok(booth)
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

    pub fn update_keyboard_config(&mut self, keyboard_config: CheckoutKeyboardConfig) {
        self.keyboard_config = keyboard_config;
        self.updated_at = Utc::now();
    }
}

/// Summary statistics for a booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothSummary {
    pub booth_id: BoothId,
    pub total_revenue: Decimal,
    pub total_purchases: usize,
    pub total_items: usize,
    pub unique_vendors: usize,
    pub vendor_summaries: Vec<VendorBoothSummary>,
    /// Configured participation fee per vendor
    pub participation_fee: Decimal,
    /// Configured revenue share percentage
    pub sales_fee_percent: Decimal,
    /// Total participation fees collected from all vendors
    pub total_participation_fees: Decimal,
    /// Total revenue share collected from all vendors
    pub total_sales_fees: Decimal,
    /// Total booth revenue (participation fees + revenue share)
    pub total_booth_revenue: Decimal,
}

/// Per-vendor statistics within a booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorBoothSummary {
    pub vendor_id: VendorId,
    pub gross_sales: Decimal,
    pub fees_due: Decimal,
    pub net_payout: Decimal,
    pub item_count: usize,
}
