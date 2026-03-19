use crate::entities::ids::BoothId;
use crate::error::{CoreError, Result};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a flea market booth with commission configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Booth {
    pub id: BoothId,
    pub name: String,
    pub commission_rate: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Booth {
    /// Create a new booth with the given name and commission rate
    pub fn new(name: impl Into<String>, commission_rate: Decimal) -> Result<Self> {
        let name = name.into();
        Self::validate_name(&name)?;
        Self::validate_commission_rate(commission_rate)?;

        let now = Utc::now();
        Ok(Self {
            id: BoothId::new(),
            name,
            commission_rate,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the booth name
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        Self::validate_name(&name)?;
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the commission rate
    pub fn set_commission_rate(&mut self, rate: Decimal) -> Result<()> {
        Self::validate_commission_rate(rate)?;
        self.commission_rate = rate;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validate booth name
    fn validate_name(name: &str) -> Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidBoothName(
                "Booth name cannot be empty".to_string(),
            ));
        }
        if trimmed.len() > 100 {
            return Err(CoreError::InvalidBoothName(
                "Booth name cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate commission rate (must be between 0 and 100)
    fn validate_commission_rate(rate: Decimal) -> Result<()> {
        if rate < Decimal::ZERO || rate > Decimal::from(100) {
            return Err(CoreError::InvalidCommissionRate(
                rate.to_string().parse().unwrap_or(0.0),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booth_creation() {
        let booth = Booth::new("Test Booth", Decimal::from(15)).unwrap();
        assert_eq!(booth.name, "Test Booth");
        assert_eq!(booth.commission_rate, Decimal::from(15));
    }

    #[test]
    fn test_booth_invalid_name() {
        assert!(Booth::new("", Decimal::from(15)).is_err());
        assert!(Booth::new("   ", Decimal::from(15)).is_err());
    }

    #[test]
    fn test_booth_invalid_commission() {
        assert!(Booth::new("Test", Decimal::from(-1)).is_err());
        assert!(Booth::new("Test", Decimal::from(101)).is_err());
    }

    #[test]
    fn test_booth_update_name() {
        let mut booth = Booth::new("Original", Decimal::from(15)).unwrap();
        booth.set_name("Updated").unwrap();
        assert_eq!(booth.name, "Updated");
    }

    #[test]
    fn test_booth_update_commission() {
        let mut booth = Booth::new("Test", Decimal::from(15)).unwrap();
        booth.set_commission_rate(Decimal::from(20)).unwrap();
        assert_eq!(booth.commission_rate, Decimal::from(20));
    }
}
