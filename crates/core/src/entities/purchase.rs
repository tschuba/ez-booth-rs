use crate::entities::ids::{BoothId, PurchaseId, VendorId};
use crate::error::{CoreError, Result};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents an individual item in a purchase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub vendor_id: VendorId,
    pub price: Decimal,
}

impl PurchaseItem {
    pub fn new(vendor_id: VendorId, price: Decimal) -> Result<Self> {
        Self::validate_price(price)?;
        Ok(Self { vendor_id, price })
    }

    fn validate_price(price: Decimal) -> Result<()> {
        if price < Decimal::ZERO {
            return Err(CoreError::InvalidPrice(
                "Price cannot be negative".to_string(),
            ));
        }
        Ok(())
    }
}

/// Represents a purchase transaction containing one or more items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    pub items: Vec<PurchaseItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Purchase {
    /// Create a new purchase for the given booth
    pub fn new(booth_id: BoothId) -> Self {
        let now = Utc::now();
        Self {
            id: PurchaseId::new(),
            booth_id,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an item to the purchase
    pub fn add_item(&mut self, item: PurchaseItem) -> Result<()> {
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove an item at the given index
    pub fn remove_item(&mut self, index: usize) -> Result<()> {
        if index >= self.items.len() {
            return Err(CoreError::Internal("Invalid item index".to_string()));
        }
        self.items.remove(index);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate the total price of all items
    pub fn total(&self) -> Decimal {
        self.items.iter().map(|item| item.price).sum()
    }

    /// Validate that the purchase has at least one item
    pub fn validate(&self) -> Result<()> {
        if self.items.is_empty() {
            return Err(CoreError::EmptyPurchase);
        }
        Ok(())
    }

    /// Get items grouped by vendor
    pub fn items_by_vendor(&self) -> std::collections::HashMap<VendorId, Vec<&PurchaseItem>> {
        let mut map = std::collections::HashMap::new();
        for item in &self.items {
            map.entry(item.vendor_id.clone())
                .or_insert_with(Vec::new)
                .push(item);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_creation() {
        let booth_id = BoothId::new();
        let purchase = Purchase::new(booth_id);
        
        assert_eq!(purchase.booth_id, booth_id);
        assert!(purchase.items.is_empty());
        assert_eq!(purchase.total(), Decimal::ZERO);
    }

    #[test]
    fn test_purchase_add_item() {
        let booth_id = BoothId::new();
        let mut purchase = Purchase::new(booth_id);
        
        let vendor_id = VendorId::new("1").unwrap();
        let item = PurchaseItem::new(vendor_id, Decimal::from(10)).unwrap();
        
        purchase.add_item(item).unwrap();
        assert_eq!(purchase.items.len(), 1);
        assert_eq!(purchase.total(), Decimal::from(10));
    }

    #[test]
    fn test_purchase_validation() {
        let booth_id = BoothId::new();
        let purchase = Purchase::new(booth_id);
        
        assert!(purchase.validate().is_err());
        
        let mut purchase_with_items = purchase.clone();
        let vendor_id = VendorId::new("1").unwrap();
        let item = PurchaseItem::new(vendor_id, Decimal::from(10)).unwrap();
        purchase_with_items.add_item(item).unwrap();
        
        assert!(purchase_with_items.validate().is_ok());
    }

    #[test]
    fn test_purchase_items_by_vendor() {
        let booth_id = BoothId::new();
        let mut purchase = Purchase::new(booth_id);
        
        let vendor1 = VendorId::new("1").unwrap();
        let vendor2 = VendorId::new("2").unwrap();
        
        purchase.add_item(PurchaseItem::new(vendor1.clone(), Decimal::from(10)).unwrap()).unwrap();
        purchase.add_item(PurchaseItem::new(vendor1.clone(), Decimal::from(20)).unwrap()).unwrap();
        purchase.add_item(PurchaseItem::new(vendor2.clone(), Decimal::from(15)).unwrap()).unwrap();
        
        let grouped = purchase.items_by_vendor();
        assert_eq!(grouped.get(&vendor1).unwrap().len(), 2);
        assert_eq!(grouped.get(&vendor2).unwrap().len(), 1);
    }
}
