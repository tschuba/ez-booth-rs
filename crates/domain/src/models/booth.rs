use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::shared::{Id, Money};

/// Represents a bazaar booth/event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booth {
    pub id: Id,
    pub name: String,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
}

impl Booth {
    pub fn new(name: String, date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            date,
            location: None,
            description: None,
            created_at: now,
            updated_at: now,
            is_archived: false,
        }
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn archive(&mut self) {
        self.is_archived = true;
        self.updated_at = Utc::now();
    }

    pub fn unarchive(&mut self) {
        self.is_archived = false;
        self.updated_at = Utc::now();
    }

    pub fn update(&mut self, name: String, date: DateTime<Utc>, location: Option<String>, description: Option<String>) {
        self.name = name;
        self.date = date;
        self.location = location;
        self.description = description;
        self.updated_at = Utc::now();
    }
}

/// Summary statistics for a booth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothSummary {
    pub booth_id: Id,
    pub total_revenue: Money,
    pub total_purchases: usize,
    pub unique_vendors: usize,
}
