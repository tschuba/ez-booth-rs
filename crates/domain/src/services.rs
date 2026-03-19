use crate::models::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutItem {
    pub vendor: VendorKey,
    pub price: Decimal,
    pub purchased_on: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkout {
    pub booth: BoothKey,
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
            participation_fee: booth.participation_fee,
            sales_fee: booth.sales_fee,
            rounding_step: booth.fees_rounding_step,
        }
    }

    pub fn calculate_fees(&self, value: Decimal) -> ChargedFees {
        let sales_fee = (self.sales_fee * value / dec!(100))
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        
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
    pub vendors: Vec<VendorKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorReportData {
    pub vendor: Vendor,
    pub booth: Booth,
    pub items: Vec<PurchaseItem>,
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
        let key = &self.vendor.key;
        let other_key = &other.vendor.key;

        // Try to parse vendor IDs as numbers for natural sorting
        let vendor_num = key.vendor_id.parse::<i64>().ok();
        let other_vendor_num = other_key.vendor_id.parse::<i64>().ok();

        match (vendor_num, other_vendor_num) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => key
                .booth
                .cmp(&other_key.booth)
                .then_with(|| key.vendor_id.cmp(&other_key.vendor_id)),
        }
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
    pub booth: BoothKey,
}
