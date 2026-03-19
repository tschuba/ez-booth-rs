use crate::error::{DomainError, DomainResult};
use crate::models::*;

/// Validation rules for booth entities
pub struct BoothValidator;

impl BoothValidator {
    pub fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("Booth name cannot be empty".to_string()));
        }
        if name.len() > 200 {
            return Err(DomainError::Validation("Booth name too long (max 200 characters)".to_string()));
        }
        Ok(())
    }
}

/// Validation rules for vendor entities
pub struct VendorValidator;

impl VendorValidator {
    pub fn validate_vendor_id(vendor_id: &VendorId) -> DomainResult<()> {
        if vendor_id.as_str().trim().is_empty() {
            return Err(DomainError::Validation("Vendor ID cannot be empty".to_string()));
        }
        if vendor_id.as_str().len() > 50 {
            return Err(DomainError::Validation("Vendor ID too long (max 50 characters)".to_string()));
        }
        Ok(())
    }
}

/// Validation rules for purchase entities
pub struct PurchaseValidator;

impl PurchaseValidator {
    pub fn validate_amount(amount: &Money) -> DomainResult<()> {
        if amount.cents() < 0 {
            return Err(DomainError::Validation("Purchase amount cannot be negative".to_string()));
        }
        if amount.cents() > 1_000_000_00 {
            return Err(DomainError::Validation("Purchase amount too large (max €1,000,000)".to_string()));
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
        assert!(PurchaseValidator::validate_amount(&Money::from_euros(10.0)).is_ok());
        assert!(PurchaseValidator::validate_amount(&Money::from_cents(-1)).is_err());
    }
}
