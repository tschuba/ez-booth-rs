mod error;
mod schema_mapper;
mod sqlite_parser;
mod types;
mod validator;

use std::collections::HashSet;
use std::sync::Arc;

use domain::{Booth, BoothRepository, Purchase, PurchaseRepository, Vendor, VendorRepository};
pub use error::MigrationError;
pub use schema_mapper::{map_booths, map_purchases, map_vendors};
pub use sqlite_parser::SqliteParser;
pub use types::{LegacyBooth, LegacyPurchase, LegacyPurchaseItem, LegacyVendor};
pub use validator::{validate_dataset, MigrationValidationSummary, ValidationIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationIssueStrategy {
    Cancel,
    SkipInvalid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationParseSummary {
    pub booths: Vec<Booth>,
    pub vendors: Vec<Vendor>,
    pub purchases: Vec<Purchase>,
    pub validation: MigrationValidationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationResult {
    pub booths_migrated: usize,
    pub vendors_migrated: usize,
    pub purchases_migrated: usize,
}

#[derive(Clone)]
pub struct MigrationService {
    booth_repository: Arc<dyn BoothRepository>,
    vendor_repository: Arc<dyn VendorRepository>,
    purchase_repository: Arc<dyn PurchaseRepository>,
}

impl MigrationService {
    pub fn new(
        booth_repository: Arc<dyn BoothRepository>,
        vendor_repository: Arc<dyn VendorRepository>,
        purchase_repository: Arc<dyn PurchaseRepository>,
    ) -> Self {
        Self {
            booth_repository,
            vendor_repository,
            purchase_repository,
        }
    }

    pub fn parse_and_validate(
        &self,
        bytes: Vec<u8>,
    ) -> Result<MigrationParseSummary, MigrationError> {
        let parser = SqliteParser::open_database(bytes)?;
        let legacy_booths = parser.parse_booths()?;
        let legacy_vendors = parser.parse_vendors()?;
        let legacy_purchases = parser.parse_purchases()?;

        let booths = schema_mapper::map_booths(&legacy_booths)?;
        let vendors = schema_mapper::map_vendors(&legacy_vendors)?;
        let purchases = schema_mapper::map_purchases(&legacy_purchases)?;
        let validation =
            validator::validate_dataset(&booths, &vendors, &purchases, &legacy_purchases);

        Ok(MigrationParseSummary {
            booths,
            vendors,
            purchases,
            validation,
        })
    }

    pub fn prepare_import(
        &self,
        summary: MigrationParseSummary,
        strategy: MigrationIssueStrategy,
    ) -> Result<MigrationParseSummary, MigrationError> {
        if summary.validation.issues.is_empty() {
            return Ok(summary);
        }

        match strategy {
            MigrationIssueStrategy::Cancel => Err(MigrationError::ReplaceFailed(
                "validation issues require operator confirmation before import".to_string(),
            )),
            MigrationIssueStrategy::SkipInvalid => Ok(filter_invalid_records(summary)),
        }
    }

    pub async fn replace_all(
        &self,
        booths: Vec<Booth>,
        vendors: Vec<Vendor>,
        purchases: Vec<Purchase>,
    ) -> Result<MigrationResult, MigrationError> {
        self.delete_existing_data().await?;

        for booth in &booths {
            self.booth_repository.save(booth).await?;
        }

        for vendor in &vendors {
            self.vendor_repository.save(vendor).await?;
        }

        for purchase in &purchases {
            self.purchase_repository.save(purchase).await?;
        }

        Ok(MigrationResult {
            booths_migrated: booths.len(),
            vendors_migrated: vendors.len(),
            purchases_migrated: purchases.len(),
        })
    }

    async fn delete_existing_data(&self) -> Result<(), MigrationError> {
        let purchases = self.purchase_repository.find_all().await?;
        for purchase in purchases {
            self.purchase_repository
                .delete_from_booth(&purchase.booth_id, &purchase.id)
                .await?;
        }

        let vendors = self.vendor_repository.find_all().await?;
        for vendor in vendors {
            self.vendor_repository
                .delete_from_booth(&vendor.booth_id, &vendor.vendor_id)
                .await?;
        }

        let booths = self.booth_repository.find_all().await?;
        let mut deleted = HashSet::new();
        for booth in booths {
            if deleted.insert(booth.id) {
                self.booth_repository.delete(&booth.id).await?;
            }
        }

        Ok(())
    }
}

fn filter_invalid_records(summary: MigrationParseSummary) -> MigrationParseSummary {
    let mut invalid_booth_ids = HashSet::new();
    let mut invalid_purchase_ids = HashSet::new();
    let mut invalid_vendor_keys = HashSet::new();
    let mut invalid_item_vendor_keys = HashSet::new();

    for issue in &summary.validation.issues {
        match issue {
            ValidationIssue::VendorMissingBooth {
                booth_id,
                vendor_id,
            } => {
                invalid_booth_ids.insert(booth_id.clone());
                invalid_vendor_keys.insert((booth_id.clone(), vendor_id.clone()));
            }
            ValidationIssue::PurchaseMissingBooth {
                booth_id,
                purchase_id,
            } => {
                invalid_booth_ids.insert(booth_id.clone());
                invalid_purchase_ids.insert(purchase_id.clone());
            }
            ValidationIssue::PurchaseWithoutItems { purchase_id }
            | ValidationIssue::PurchaseTotalMismatch { purchase_id, .. } => {
                invalid_purchase_ids.insert(purchase_id.clone());
            }
            ValidationIssue::PurchaseItemMissingVendor {
                booth_id,
                purchase_id,
                vendor_id,
                ..
            } => {
                invalid_item_vendor_keys.insert((
                    booth_id.clone(),
                    purchase_id.clone(),
                    vendor_id.clone(),
                ));
            }
        }
    }

    let booths = summary
        .booths
        .into_iter()
        .filter(|booth| !invalid_booth_ids.contains(&booth.id.to_string()))
        .collect::<Vec<_>>();

    let vendors = summary
        .vendors
        .into_iter()
        .filter(|vendor| {
            !invalid_booth_ids.contains(&vendor.booth_id.to_string())
                && !invalid_vendor_keys
                    .contains(&(vendor.booth_id.to_string(), vendor.vendor_id.to_string()))
        })
        .collect::<Vec<_>>();

    let purchases = summary
        .purchases
        .into_iter()
        .filter(|purchase| {
            !invalid_booth_ids.contains(&purchase.booth_id.to_string())
                && !invalid_purchase_ids.contains(&purchase.id.to_string())
        })
        .filter_map(|mut purchase| {
            purchase.items.retain(|item| {
                !invalid_item_vendor_keys.contains(&(
                    purchase.booth_id.to_string(),
                    purchase.id.to_string(),
                    item.vendor_id.to_string(),
                ))
            });

            if purchase.items.is_empty() {
                None
            } else {
                Some(purchase)
            }
        })
        .collect::<Vec<_>>();

    let validation = validate_dataset(&booths, &vendors, &purchases, &[]);

    MigrationParseSummary {
        booths,
        vendors,
        purchases,
        validation,
    }
}
