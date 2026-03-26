use crate::error::DomainError;
use crate::models::VendorIdValidation;

/// Validate a vendor ID against the booth's validation rules
///
/// # Arguments
///
/// * `value` - The vendor ID string to validate
/// * `rule` - The validation rule to apply
///
/// # Errors
///
/// Returns `DomainError::Validation` if the vendor ID doesn't match the rule
pub fn validate_vendor_id(value: &str, rule: &VendorIdValidation) -> Result<(), DomainError> {
    match rule {
        VendorIdValidation::Unrestricted => Ok(()),
        VendorIdValidation::DigitsOnly => {
            if value.is_empty() {
                return Err(DomainError::Validation(
                    "Vendor ID cannot be empty".to_string(),
                ));
            }
            if value.chars().all(|c| c.is_ascii_digit()) {
                Ok(())
            } else {
                Err(DomainError::Validation(
                    "Vendor ID must contain only digits (0-9)".to_string(),
                ))
            }
        }
        VendorIdValidation::Regex(pattern) => {
            if value.is_empty() {
                return Err(DomainError::Validation(
                    "Vendor ID cannot be empty".to_string(),
                ));
            }
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(value) {
                        Ok(())
                    } else {
                        Err(DomainError::Validation(format!(
                            "Vendor ID '{}' does not match the required pattern",
                            value
                        )))
                    }
                }
                Err(_) => Err(DomainError::Validation(
                    "Invalid regex pattern configured for vendor ID validation".to_string(),
                )),
            }
        }
    }
}

/// Validate a regex pattern for vendor ID validation
///
/// This should be called when saving a booth with a regex validation rule
/// to ensure the pattern is valid before persisting it.
///
/// # Errors
///
/// Returns `DomainError::Validation` if the pattern is invalid
pub fn validate_regex_pattern(pattern: &str) -> Result<(), DomainError> {
    if pattern.is_empty() {
        return Err(DomainError::Validation(
            "Regex pattern cannot be empty".to_string(),
        ));
    }

    if pattern.len() > 256 {
        return Err(DomainError::Validation(
            "Regex pattern is too long (max 256 characters)".to_string(),
        ));
    }

    regex::Regex::new(pattern)
        .map_err(|e| DomainError::Validation(format!("Invalid regex pattern: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unrestricted_allows_any() {
        let rule = VendorIdValidation::Unrestricted;
        assert!(validate_vendor_id("123", &rule).is_ok());
        assert!(validate_vendor_id("abc", &rule).is_ok());
        assert!(validate_vendor_id("V-123", &rule).is_ok());
        assert!(validate_vendor_id("", &rule).is_ok());
    }

    #[test]
    fn test_digits_only_accepts_digits() {
        let rule = VendorIdValidation::DigitsOnly;
        assert!(validate_vendor_id("123", &rule).is_ok());
        assert!(validate_vendor_id("0", &rule).is_ok());
        assert!(validate_vendor_id("999999", &rule).is_ok());
    }

    #[test]
    fn test_digits_only_rejects_non_digits() {
        let rule = VendorIdValidation::DigitsOnly;
        assert!(validate_vendor_id("12a", &rule).is_err());
        assert!(validate_vendor_id("V123", &rule).is_err());
        assert!(validate_vendor_id("12-3", &rule).is_err());
        assert!(validate_vendor_id("", &rule).is_err());
    }

    #[test]
    fn test_regex_pattern_matching() {
        let rule = VendorIdValidation::Regex("^V\\d+$".to_string());
        assert!(validate_vendor_id("V123", &rule).is_ok());
        assert!(validate_vendor_id("V1", &rule).is_ok());
        assert!(validate_vendor_id("V999", &rule).is_ok());
    }

    #[test]
    fn test_regex_pattern_rejection() {
        let rule = VendorIdValidation::Regex("^V\\d+$".to_string());
        assert!(validate_vendor_id("123", &rule).is_err());
        assert!(validate_vendor_id("X123", &rule).is_err());
        assert!(validate_vendor_id("V", &rule).is_err());
        assert!(validate_vendor_id("", &rule).is_err());
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let rule = VendorIdValidation::Regex("[invalid".to_string());
        let result = validate_vendor_id("test", &rule);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_regex_pattern_valid() {
        assert!(validate_regex_pattern("^V\\d+$").is_ok());
        assert!(validate_regex_pattern("^[A-Z]\\d{3}$").is_ok());
        assert!(validate_regex_pattern("\\d+").is_ok());
    }

    #[test]
    fn test_validate_regex_pattern_invalid() {
        assert!(validate_regex_pattern("[invalid").is_err());
        assert!(validate_regex_pattern("").is_err());
    }

    #[test]
    fn test_validate_regex_pattern_too_long() {
        let long_pattern = "a".repeat(257);
        assert!(validate_regex_pattern(&long_pattern).is_err());
    }
}
