use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Type-safe booth identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoothId(Uuid);

impl BoothId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for BoothId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BoothId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe vendor identifier (supports both numeric and alphanumeric IDs)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorId(String);

impl VendorId {
    pub fn new(id: impl Into<String>) -> Result<Self, crate::error::CoreError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(crate::error::CoreError::InvalidVendorId(
                "Vendor ID cannot be empty".to_string(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the vendor ID is purely numeric
    pub fn is_numeric(&self) -> bool {
        self.0.chars().all(|c| c.is_ascii_digit())
    }

    /// Try to parse as a number for numeric sorting
    pub fn as_number(&self) -> Option<u64> {
        self.0.parse().ok()
    }
}

impl fmt::Display for VendorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Implements natural sorting for vendor IDs:
/// - Numeric IDs are sorted numerically (1, 2, 10, 100)
/// - Alphanumeric IDs are sorted lexicographically
/// - Numeric IDs come before alphanumeric IDs
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

/// Type-safe purchase identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseId(Uuid);

impl PurchaseId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for PurchaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PurchaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id_numeric_sorting() {
        let ids = vec![
            VendorId::new("100").unwrap(),
            VendorId::new("2").unwrap(),
            VendorId::new("10").unwrap(),
            VendorId::new("1").unwrap(),
        ];

        let mut sorted = ids.clone();
        sorted.sort();

        assert_eq!(sorted[0].as_str(), "1");
        assert_eq!(sorted[1].as_str(), "2");
        assert_eq!(sorted[2].as_str(), "10");
        assert_eq!(sorted[3].as_str(), "100");
    }

    #[test]
    fn test_vendor_id_mixed_sorting() {
        let ids = vec![
            VendorId::new("ABC").unwrap(),
            VendorId::new("10").unwrap(),
            VendorId::new("2").unwrap(),
            VendorId::new("XYZ").unwrap(),
        ];

        let mut sorted = ids.clone();
        sorted.sort();

        // Numeric IDs come first, then alphanumeric
        assert_eq!(sorted[0].as_str(), "2");
        assert_eq!(sorted[1].as_str(), "10");
        assert_eq!(sorted[2].as_str(), "ABC");
        assert_eq!(sorted[3].as_str(), "XYZ");
    }
}
