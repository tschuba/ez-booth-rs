use crate::error::{DomainError, DomainResult};
use crate::error_code::ValidationError;
use crate::models::*;
use rust_decimal::Decimal;

/// Validation rules for booth entities
pub struct BoothValidator;

impl BoothValidator {
    pub fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation(ValidationError::BoothNameEmpty));
        }
        if name.len() > 200 {
            return Err(DomainError::Validation(ValidationError::BoothNameTooLong));
        }
        Ok(())
    }
}

/// Validation rules for vendor entities
pub struct VendorValidator;

impl VendorValidator {
    pub fn validate_vendor_id(vendor_id: &VendorId) -> DomainResult<()> {
        if vendor_id.as_str().trim().is_empty() {
            return Err(DomainError::Validation(ValidationError::VendorIdEmpty));
        }
        if vendor_id.as_str().len() > 50 {
            return Err(DomainError::Validation(ValidationError::VendorIdTooLong));
        }
        Ok(())
    }
}

/// Validation rules for purchase entities
pub struct PurchaseValidator;

impl PurchaseValidator {
    /// Validate that a purchase amount is within acceptable bounds
    ///
    /// # Errors
    ///
    /// Returns error if amount is negative or exceeds €1,000,000
    pub fn validate_amount(amount: &Decimal) -> DomainResult<()> {
        if amount.is_sign_negative() {
            return Err(DomainError::Validation(
                ValidationError::PurchaseAmountNegative,
            ));
        }

        // Maximum €1,000,000
        let max_amount = Decimal::new(1_000_000, 0);
        if *amount > max_amount {
            return Err(DomainError::Validation(
                ValidationError::PurchaseAmountTooLarge,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booth_name_validation() {
        assert!(BoothValidator::validate_name("Valid Name").is_ok());
        assert!(BoothValidator::validate_name("").is_err());
        assert!(BoothValidator::validate_name("   ").is_err());
        assert!(BoothValidator::validate_name(&"x".repeat(201)).is_err());
    }

    #[test]
    fn test_vendor_id_validation() {
        assert!(VendorValidator::validate_vendor_id(&VendorId::new("123".to_string())).is_ok());
        assert!(VendorValidator::validate_vendor_id(&VendorId::new("".to_string())).is_err());
    }

    #[test]
    fn test_purchase_amount_validation() {
        use rust_decimal_macros::dec;

        // Valid amounts
        assert!(PurchaseValidator::validate_amount(&dec!(10.0)).is_ok());
        assert!(PurchaseValidator::validate_amount(&Decimal::ZERO).is_ok());
        assert!(PurchaseValidator::validate_amount(&dec!(999999.99)).is_ok());

        // Invalid amounts
        assert!(PurchaseValidator::validate_amount(&dec!(-1.0)).is_err());
        assert!(PurchaseValidator::validate_amount(&dec!(-0.01)).is_err());
        assert!(PurchaseValidator::validate_amount(&Decimal::new(1_000_001, 0)).is_err());
    }
}
