use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    VendorIdEmpty,
    VendorIdTooLong,
    VendorIdDigitsOnly,
    VendorIdPatternMismatch { value: String },
    VendorIdInvalidRegex,
    VendorIdOmitted { value: String },
    VendorOmissionPatternTooLong,
    VendorOmissionRangeInvalid,
    VendorOmissionStepInvalid,
    VendorOmissionRulesTooMany,
    BoothNameEmpty,
    BoothNameTooLong,
    BoothDuplicateNameAndDate { description: String, date: String },
    ParticipationFeeNegative,
    SalesFeePercentNegative,
    SalesFeePercentTooLarge,
    RoundingStepNegative,
    PurchaseAmountNegative,
    PurchaseAmountTooLarge,
    PurchaseEmpty,
    PurchaseTotalTooLarge,
    ItemAmountNotPositive,
    ItemAmountTooManyDecimals,
    ItemAmountTooLarge,
    RegexPatternEmpty,
    RegexPatternTooLong,
    RegexPatternInvalid,
    DateInvalid,
    ParticipationFeeInvalid,
    SalesFeePercentInvalid,
    RoundingStepInvalid,
    QuickAmountInvalid { value: String },
    QuickAmountsEmpty,
    QuickAmountsNonPositive,
}

impl ValidationError {
    pub fn key(&self) -> &'static str {
        match self {
            Self::VendorIdEmpty => "validation.vendor_id_empty",
            Self::VendorIdTooLong => "validation.vendor_id_too_long",
            Self::VendorIdDigitsOnly => "validation.vendor_id_digits_only",
            Self::VendorIdPatternMismatch { .. } => "validation.vendor_id_pattern_mismatch",
            Self::VendorIdInvalidRegex => "validation.vendor_id_invalid_regex",
            Self::VendorIdOmitted { .. } => "validation.vendor_id_omitted",
            Self::VendorOmissionPatternTooLong => "validation.vendor_omission_pattern_too_long",
            Self::VendorOmissionRangeInvalid => "validation.vendor_omission_range_invalid",
            Self::VendorOmissionStepInvalid => "validation.vendor_omission_step_invalid",
            Self::VendorOmissionRulesTooMany => "validation.vendor_omission_rules_too_many",
            Self::BoothNameEmpty => "validation.booth_name_empty",
            Self::BoothNameTooLong => "validation.booth_name_too_long",
            Self::BoothDuplicateNameAndDate { .. } => "validation.booth_duplicate_name_and_date",
            Self::ParticipationFeeNegative => "validation.participation_fee_negative",
            Self::SalesFeePercentNegative => "validation.sales_fee_percent_negative",
            Self::SalesFeePercentTooLarge => "validation.sales_fee_percent_too_large",
            Self::RoundingStepNegative => "validation.rounding_step_negative",
            Self::PurchaseAmountNegative => "validation.purchase_amount_negative",
            Self::PurchaseAmountTooLarge => "validation.purchase_amount_too_large",
            Self::PurchaseEmpty => "validation.purchase_empty",
            Self::PurchaseTotalTooLarge => "validation.purchase_total_too_large",
            Self::ItemAmountNotPositive => "validation.item_amount_not_positive",
            Self::ItemAmountTooManyDecimals => "validation.item_amount_too_many_decimals",
            Self::ItemAmountTooLarge => "validation.item_amount_too_large",
            Self::RegexPatternEmpty => "validation.regex_pattern_empty",
            Self::RegexPatternTooLong => "validation.regex_pattern_too_long",
            Self::RegexPatternInvalid => "validation.regex_pattern_invalid",
            Self::DateInvalid => "validation.date_invalid",
            Self::ParticipationFeeInvalid => "validation.participation_fee_invalid",
            Self::SalesFeePercentInvalid => "validation.sales_fee_percent_invalid",
            Self::RoundingStepInvalid => "validation.rounding_step_invalid",
            Self::QuickAmountInvalid { .. } => "validation.quick_amount_invalid",
            Self::QuickAmountsEmpty => "validation.quick_amounts_empty",
            Self::QuickAmountsNonPositive => "validation.quick_amounts_non_positive",
        }
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::VendorIdPatternMismatch { value } => vec![("value", value.clone())],
            Self::VendorIdOmitted { value } => vec![("value", value.clone())],
            Self::BoothDuplicateNameAndDate { description, date } => {
                vec![("description", description.clone()), ("date", date.clone())]
            }
            Self::QuickAmountInvalid { value } => vec![("value", value.clone())],
            _ => Vec::new(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}
