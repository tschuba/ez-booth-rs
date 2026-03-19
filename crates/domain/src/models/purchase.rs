use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::shared::{Id, Money, VendorId};

/// Represents a purchase transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: Id,
    pub booth_id: Id,
    pub vendor_id: VendorId,
    pub amount: Money,
    pub timestamp: DateTime<Utc>,
    pub note: Option<String>,
}

impl Purchase {
    pub fn new(booth_id: Id, vendor_id: VendorId, amount: Money) -> Self {
        Self {
            id: Id::new(),
            booth_id,
            vendor_id,
            amount,
            timestamp: Utc::now(),
            note: None,
        }
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }
}
