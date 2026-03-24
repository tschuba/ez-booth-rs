use crate::models::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// An item in a vendor report with its associated transaction ID
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorReportItem {
    pub transaction_id: PurchaseId,
    pub item: PurchaseItem,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutItem {
    pub vendor: VendorId,
    pub price: Decimal,
    pub purchased_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkout {
    pub booth: BoothId,
    pub items: Vec<CheckoutItem>,
    pub print_receipt: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargedFees {
    pub participation_fee: Decimal,
    pub sales_fee: Decimal,
}

impl ChargedFees {
    pub fn total(&self) -> Decimal {
        self.participation_fee + self.sales_fee
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargingConfig {
    pub participation_fee: Decimal,
    pub sales_fee: Decimal,
    pub rounding_step: Decimal,
}

impl ChargingConfig {
    pub fn from_booth(booth: &Booth) -> Self {
        Self {
            participation_fee: booth.fees.participation_fee,
            sales_fee: booth.fees.sales_fee_percent,
            rounding_step: booth.fees.rounding_step,
        }
    }

    /// Round a value to the nearest multiple of the rounding step
    fn round_to_step(&self, value: Decimal) -> Decimal {
        if self.rounding_step == Decimal::ZERO {
            // If rounding step is 0, just round to 2 decimal places
            return value
                .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        }

        // Round to nearest multiple of rounding_step
        // Formula: round(value / step) * step
        let divided = value / self.rounding_step;
        let rounded =
            divided.round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        rounded * self.rounding_step
    }

    pub fn calculate_fees(&self, value: Decimal) -> ChargedFees {
        let sales_fee_raw = self.sales_fee * value / dec!(100);
        let sales_fee = self.round_to_step(sales_fee_raw);

        ChargedFees {
            participation_fee: self.participation_fee,
            sales_fee,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInput {
    pub total_sales_amount: Decimal,
    pub charging_config: ChargingConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceOutput {
    pub total_revenue: Decimal,
    pub charged_fees: ChargedFees,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorReportInput {
    pub vendors: Vec<VendorId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorReportData {
    pub vendor: Vendor,
    pub booth: Booth,
    pub items: Vec<VendorReportItem>,
    pub sales_sum: Decimal,
    pub participation_fee: Decimal,
    pub sales_fee: Decimal,
    pub total_revenue: Decimal,
}

impl PartialOrd for VendorReportData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for VendorReportData {}

impl Ord for VendorReportData {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use VendorId's built-in smart sorting (handles numeric vs alphanumeric)
        self.vendor.vendor_id.cmp(&other.vendor.vendor_id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeData {
    pub booth: Booth,
    pub vendors: Vec<Vendor>,
    pub purchases: Vec<Purchase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeReceiver {
    pub name: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeSubscription {
    pub id: String,
    pub booth: BoothId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounding_step_half_euro() {
        let config = ChargingConfig {
            participation_fee: dec!(1.00),
            sales_fee: dec!(15.0), // 15%
            rounding_step: dec!(0.50),
        };

        // 15% of 23.50 = 3.525 -> should round to 3.50
        let fees = config.calculate_fees(dec!(23.50));
        assert_eq!(fees.sales_fee, dec!(3.50));

        // 15% of 24.00 = 3.60 -> should round to 3.50
        let fees = config.calculate_fees(dec!(24.00));
        assert_eq!(fees.sales_fee, dec!(3.50));

        // 15% of 25.00 = 3.75 -> should round to 4.00
        let fees = config.calculate_fees(dec!(25.00));
        assert_eq!(fees.sales_fee, dec!(4.00));

        // 15% of 27.00 = 4.05 -> should round to 4.00
        let fees = config.calculate_fees(dec!(27.00));
        assert_eq!(fees.sales_fee, dec!(4.00));
    }

    #[test]
    fn test_rounding_step_quarter_euro() {
        let config = ChargingConfig {
            participation_fee: dec!(1.00),
            sales_fee: dec!(10.0), // 10%
            rounding_step: dec!(0.25),
        };

        // 10% of 23.00 = 2.30 -> should round to 2.25
        let fees = config.calculate_fees(dec!(23.00));
        assert_eq!(fees.sales_fee, dec!(2.25));

        // 10% of 24.00 = 2.40 -> should round to 2.50
        let fees = config.calculate_fees(dec!(24.00));
        assert_eq!(fees.sales_fee, dec!(2.50));

        // 10% of 26.75 = 2.675 -> should round to 2.75
        let fees = config.calculate_fees(dec!(26.75));
        assert_eq!(fees.sales_fee, dec!(2.75));
    }

    #[test]
    fn test_rounding_step_one_euro() {
        let config = ChargingConfig {
            participation_fee: dec!(5.00),
            sales_fee: dec!(15.0), // 15%
            rounding_step: dec!(1.00),
        };

        // 15% of 10.00 = 1.50 -> should round to 2.00
        let fees = config.calculate_fees(dec!(10.00));
        assert_eq!(fees.sales_fee, dec!(2.00));

        // 15% of 20.00 = 3.00 -> should stay 3.00
        let fees = config.calculate_fees(dec!(20.00));
        assert_eq!(fees.sales_fee, dec!(3.00));

        // 15% of 23.00 = 3.45 -> should round to 3.00
        let fees = config.calculate_fees(dec!(23.00));
        assert_eq!(fees.sales_fee, dec!(3.00));

        // 15% of 27.00 = 4.05 -> should round to 4.00
        let fees = config.calculate_fees(dec!(27.00));
        assert_eq!(fees.sales_fee, dec!(4.00));
    }

    #[test]
    fn test_rounding_step_zero_uses_cent_precision() {
        let config = ChargingConfig {
            participation_fee: dec!(1.00),
            sales_fee: dec!(15.0), // 15%
            rounding_step: dec!(0.00),
        };

        // 15% of 23.33 = 3.4995 -> should round to 3.50 (2 decimal places)
        let fees = config.calculate_fees(dec!(23.33));
        assert_eq!(fees.sales_fee, dec!(3.50));

        // 15% of 23.34 = 3.501 -> should round to 3.50 (2 decimal places)
        let fees = config.calculate_fees(dec!(23.34));
        assert_eq!(fees.sales_fee, dec!(3.50));
    }

    #[test]
    fn test_participation_fee_not_rounded() {
        let config = ChargingConfig {
            participation_fee: dec!(1.50),
            sales_fee: dec!(15.0),
            rounding_step: dec!(0.50),
        };

        let fees = config.calculate_fees(dec!(100.00));

        // Participation fee should always be the configured amount
        assert_eq!(fees.participation_fee, dec!(1.50));
        // Sales fee: 15% of 100 = 15.00 (already a multiple of 0.50)
        assert_eq!(fees.sales_fee, dec!(15.00));
    }
}
