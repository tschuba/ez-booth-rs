use crate::entities::ids::{BoothId, VendorId};
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a vendor selling items at a booth
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vendor {
    pub id: VendorId,
    pub booth_id: BoothId,
    pub created_at: DateTime<Utc>,
}

impl Vendor {
    /// Create a new vendor with the given ID and booth
    pub fn new(id: VendorId, booth_id: BoothId) -> Result<Self> {
        Ok(Self {
            id,
            booth_id,
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_creation() {
        let booth_id = BoothId::new();
        let vendor_id = VendorId::new("123").unwrap();
        let vendor = Vendor::new(vendor_id.clone(), booth_id).unwrap();
        
        assert_eq!(vendor.id, vendor_id);
        assert_eq!(vendor.booth_id, booth_id);
    }
}
