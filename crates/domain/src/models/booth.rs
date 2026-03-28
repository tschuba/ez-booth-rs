use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::shared::{BoothId, VendorId};
use crate::error::DomainError;
use crate::error_code::ValidationError;

/// Rules for omitting specific vendor IDs during checkout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorIdOmissionRules {
    pub rules: Vec<OmissionRule>,
}

impl VendorIdOmissionRules {
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn is_omitted(&self, vendor_id: &str) -> bool {
        self.rules.iter().any(|rule| rule.matches(vendor_id))
    }
}

impl Default for VendorIdOmissionRules {
    fn default() -> Self {
        Self {
            rules: vec![OmissionRule::RangeWithStep {
                start: 56,
                end: 182,
                step: 6,
            }],
        }
    }
}

/// A single omission rule for blocking vendor IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OmissionRule {
    Exact(String),
    Wildcard(String),
    Regex(String),
    Range { start: u32, end: u32 },
    RangeWithStep { start: u32, end: u32, step: u32 },
}

impl OmissionRule {
    pub fn matches(&self, vendor_id: &str) -> bool {
        match self {
            Self::Exact(value) => vendor_id == value,
            Self::Wildcard(pattern) => wildcard_matches(pattern, vendor_id),
            Self::Regex(pattern) => regex::Regex::new(pattern)
                .ok()
                .map(|re| re.is_match(vendor_id))
                .unwrap_or(false),
            Self::Range { start, end } => vendor_id
                .parse::<u32>()
                .ok()
                .map(|value| value >= *start && value <= *end)
                .unwrap_or(false),
            Self::RangeWithStep { start, end, step } => {
                if *step == 0 {
                    return false;
                }

                vendor_id
                    .parse::<u32>()
                    .ok()
                    .map(|value| value >= *start && value <= *end && (value - *start) % *step == 0)
                    .unwrap_or(false)
            }
        }
    }
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let mut regex_pattern = String::with_capacity(pattern.len() + 2);
    regex_pattern.push('^');

    for ch in pattern.chars() {
        match ch {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            _ => regex_pattern.push_str(&regex::escape(&ch.to_string())),
        }
    }

    regex_pattern.push('$');

    regex::Regex::new(&regex_pattern)
        .ok()
        .map(|re| re.is_match(value))
        .unwrap_or(false)
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Booth {
    pub id: BoothId,

    pub description: String,

    pub date: NaiveDate,

    pub fees: FeeConfig,

    pub status: BoothStatus,

    #[serde(default)]
    pub vendor_id_validation: VendorIdValidation,

    #[serde(default)]
    pub vendor_id_omission_rules: VendorIdOmissionRules,

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                ValidationError::ParticipationFeeNegative,
            ));
        }

        if self.sales_fee_percent.is_sign_negative() {
            return Err(DomainError::Validation(
                ValidationError::SalesFeePercentNegative,
            ));
        }

        if self.sales_fee_percent > Decimal::new(100, 0) {
            return Err(DomainError::Validation(
                ValidationError::SalesFeePercentTooLarge,
            ));
        }

        if self.rounding_step.is_sign_negative() {
            return Err(DomainError::Validation(
                ValidationError::RoundingStepNegative,
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
        if description.trim().is_empty() {
            return Err(DomainError::Validation(ValidationError::BoothNameEmpty));
        }
        if description.len() > 200 {
            return Err(DomainError::Validation(ValidationError::BoothNameTooLong));
        }

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
            vendor_id_omission_rules: VendorIdOmissionRules::default(),
            keyboard_config: CheckoutKeyboardConfig::default(),
            created_at: now,
            updated_at: now,
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omission_rule_exact_matches() {
        assert!(OmissionRule::Exact("123".to_string()).matches("123"));
        assert!(!OmissionRule::Exact("123".to_string()).matches("124"));
    }

    #[test]
    fn omission_rule_wildcard_matches() {
        assert!(OmissionRule::Wildcard("12*".to_string()).matches("1234"));
        assert!(OmissionRule::Wildcard("*34".to_string()).matches("1234"));
        assert!(OmissionRule::Wildcard("1?34".to_string()).matches("1234"));
        assert!(!OmissionRule::Wildcard("12*".to_string()).matches("9912"));
    }

    #[test]
    fn omission_rule_regex_matches() {
        assert!(OmissionRule::Regex("^A\\d+$".to_string()).matches("A42"));
        assert!(!OmissionRule::Regex("^A\\d+$".to_string()).matches("B42"));
    }

    #[test]
    fn omission_rule_range_with_step_matches() {
        let rule = OmissionRule::RangeWithStep {
            start: 56,
            end: 182,
            step: 6,
        };

        assert!(rule.matches("56"));
        assert!(rule.matches("62"));
        assert!(rule.matches("182"));
        assert!(!rule.matches("60"));
        assert!(!rule.matches("183"));
    }

    #[test]
    fn default_omission_rules_match_expected_values() {
        let rules = VendorIdOmissionRules::default();

        assert!(rules.is_omitted("56"));
        assert!(rules.is_omitted("62"));
        assert!(rules.is_omitted("182"));
        assert!(!rules.is_omitted("55"));
        assert!(!rules.is_omitted("60"));
        assert!(!rules.is_omitted("183"));
    }
}
