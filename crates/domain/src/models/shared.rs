use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for entities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Money value in cents to avoid floating point issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    pub fn from_cents(cents: i64) -> Self {
        Self(cents)
    }

    pub fn from_euros(euros: f64) -> Self {
        Self((euros * 100.0).round() as i64)
    }

    pub fn cents(&self) -> i64 {
        self.0
    }

    pub fn euros(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    pub fn add(&self, other: &Money) -> Money {
        Money(self.0 + other.0)
    }

    pub fn subtract(&self, other: &Money) -> Money {
        Money(self.0 - other.0)
    }
}

impl Default for Money {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "€{:.2}", self.euros())
    }
}

/// Vendor identifier - supports both numeric and text IDs with smart sorting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorId(String);

impl VendorId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the vendor ID is numeric-only
    pub fn is_numeric(&self) -> bool {
        self.0.chars().all(|c| c.is_ascii_digit())
    }

    /// Get numeric value if ID is numeric
    pub fn as_number(&self) -> Option<u64> {
        self.0.parse().ok()
    }
}

impl fmt::Display for VendorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for VendorId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for VendorId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Smart ordering for vendor IDs: numeric IDs sort numerically, text IDs alphabetically
impl Ord for VendorId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => self.0.cmp(&other.0),
        }
    }
}

impl PartialOrd for VendorId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id_numeric_sorting() {
        let mut ids = vec![
            VendorId::new("100".to_string()),
            VendorId::new("2".to_string()),
            VendorId::new("30".to_string()),
        ];
        ids.sort();
        assert_eq!(ids[0].as_str(), "2");
        assert_eq!(ids[1].as_str(), "30");
        assert_eq!(ids[2].as_str(), "100");
    }

    #[test]
    fn test_vendor_id_mixed_sorting() {
        let mut ids = vec![
            VendorId::new("A10".to_string()),
            VendorId::new("5".to_string()),
            VendorId::new("B2".to_string()),
            VendorId::new("15".to_string()),
        ];
        ids.sort();
        // Numeric IDs come first, then alphabetic
        assert_eq!(ids[0].as_str(), "5");
        assert_eq!(ids[1].as_str(), "15");
        assert_eq!(ids[2].as_str(), "A10");
        assert_eq!(ids[3].as_str(), "B2");
    }

    #[test]
    fn test_money_operations() {
        let m1 = Money::from_euros(10.50);
        let m2 = Money::from_euros(5.25);
        let sum = m1.add(&m2);
        assert_eq!(sum.euros(), 15.75);
    }
}
