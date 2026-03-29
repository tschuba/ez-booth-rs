use std::collections::HashSet;
use std::sync::Arc;

use domain::{
    BoothId, BoothRepository, Purchase, PurchaseRepository, Vendor, VendorRepository,
};
use log::info;

use super::backup_format::{
    generate_booth_backup_filename, generate_full_backup_filename, BackupData, BoothBackupData,
};
use super::error::ExportError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedBackup {
    pub file_name: String,
    pub json: String,
}

#[derive(Clone)]
pub struct ExportService {
    booth_repository: Arc<dyn BoothRepository>,
    vendor_repository: Arc<dyn VendorRepository>,
    purchase_repository: Arc<dyn PurchaseRepository>,
    app_version: String,
}

impl ExportService {
    pub fn new(
        booth_repository: Arc<dyn BoothRepository>,
        vendor_repository: Arc<dyn VendorRepository>,
        purchase_repository: Arc<dyn PurchaseRepository>,
    ) -> Self {
        Self::with_app_version(
            booth_repository,
            vendor_repository,
            purchase_repository,
            env!("CARGO_PKG_VERSION"),
        )
    }

    pub fn with_app_version(
        booth_repository: Arc<dyn BoothRepository>,
        vendor_repository: Arc<dyn VendorRepository>,
        purchase_repository: Arc<dyn PurchaseRepository>,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            booth_repository,
            vendor_repository,
            purchase_repository,
            app_version: app_version.into(),
        }
    }

    pub async fn export_all(&self) -> Result<BackupData, ExportError> {
        let mut data = BackupData::new(self.app_version.clone());

        let booths = self.booth_repository.find_all().await?;
        let vendors = self.vendor_repository.find_all().await?;
        let purchases = self.purchase_repository.find_all().await?;

        let (booths, vendors, purchases) = Self::filter_orphaned_records(booths, vendors, purchases);

        data.booths = booths;
        data.vendors = vendors;
        data.purchases = purchases;

        Ok(data)
    }

    pub async fn export_booth(&self, booth_id: &BoothId) -> Result<BoothBackupData, ExportError> {
        let booth = self
            .booth_repository
            .find_by_id(booth_id)
            .await?
            .ok_or(ExportError::BoothNotFound(*booth_id))?;

        let mut data = BoothBackupData::new(booth.clone(), self.app_version.clone());

        let vendors = self.vendor_repository.find_by_booth(booth_id).await?;
        let purchases = self.purchase_repository.find_by_booth(booth_id).await?;

        let (vendors, purchases) = Self::filter_booth_orphaned_purchases(*booth_id, vendors, purchases);

        data.vendors = vendors;
        data.purchases = purchases;

        Ok(data)
    }

    pub fn serialize_full_backup(
        &self,
        data: &BackupData,
    ) -> Result<SerializedBackup, ExportError> {
        Ok(SerializedBackup {
            file_name: generate_full_backup_filename(data.created_at),
            json: serde_json::to_string_pretty(data)
                .map_err(|err| ExportError::Serialization(err.to_string()))?,
        })
    }

    pub fn serialize_booth_backup(
        &self,
        data: &BoothBackupData,
    ) -> Result<SerializedBackup, ExportError> {
        Ok(SerializedBackup {
            file_name: generate_booth_backup_filename(
                &data.booth.id,
                &data.booth.description,
                data.created_at,
            ),
            json: serde_json::to_string_pretty(data)
                .map_err(|err| ExportError::Serialization(err.to_string()))?,
        })
    }

    fn filter_orphaned_records(
        booths: Vec<domain::Booth>,
        vendors: Vec<Vendor>,
        purchases: Vec<Purchase>,
    ) -> (Vec<domain::Booth>, Vec<Vendor>, Vec<Purchase>) {
        let booth_ids: HashSet<BoothId> = booths.iter().map(|booth| booth.id).collect();

        let mut skipped_vendor_count = 0;
        let valid_vendors: Vec<Vendor> = vendors
            .into_iter()
            .filter(|vendor| {
                if booth_ids.contains(&vendor.booth_id) {
                    true
                } else {
                    skipped_vendor_count += 1;
                    info!(
                        "Skipping vendor {} during export because booth {} is missing",
                        vendor.vendor_id, vendor.booth_id
                    );
                    false
                }
            })
            .collect();

        let vendor_pairs: HashSet<(BoothId, String)> = valid_vendors
            .iter()
            .map(|vendor| (vendor.booth_id, vendor.vendor_id.as_str().to_string()))
            .collect();

        let mut skipped_missing_booth_purchase_count = 0;
        let mut skipped_missing_vendor_purchase_count = 0;
        let valid_purchases: Vec<Purchase> = purchases
            .into_iter()
            .filter(|purchase| {
                if !booth_ids.contains(&purchase.booth_id) {
                    skipped_missing_booth_purchase_count += 1;
                    info!(
                        "Skipping purchase {} during export because booth {} is missing",
                        purchase.id, purchase.booth_id
                    );
                    return false;
                }

                if let Some(item) = purchase.items.iter().find(|item| {
                    !vendor_pairs
                        .contains(&(purchase.booth_id, item.vendor_id.as_str().to_string()))
                }) {
                    skipped_missing_vendor_purchase_count += 1;
                    info!(
                        "Skipping purchase {} during export because item {} references missing vendor {} for booth {}",
                        purchase.id, item.id, item.vendor_id, purchase.booth_id
                    );
                    false
                } else {
                    true
                }
            })
            .collect();

        let skipped_total = skipped_vendor_count
            + skipped_missing_booth_purchase_count
            + skipped_missing_vendor_purchase_count;

        if skipped_total > 0 {
            info!(
                "Export skipped {} orphaned records ({} vendors, {} purchases with missing booths, {} purchases with missing vendors)",
                skipped_total,
                skipped_vendor_count,
                skipped_missing_booth_purchase_count,
                skipped_missing_vendor_purchase_count
            );
        }

        (booths, valid_vendors, valid_purchases)
    }

    fn filter_booth_orphaned_purchases(
        booth_id: BoothId,
        vendors: Vec<Vendor>,
        purchases: Vec<Purchase>,
    ) -> (Vec<Vendor>, Vec<Purchase>) {
        let vendor_ids: HashSet<String> = vendors
            .iter()
            .map(|vendor| vendor.vendor_id.as_str().to_string())
            .collect();

        let mut skipped_purchase_count = 0;
        let valid_purchases: Vec<Purchase> = purchases
            .into_iter()
            .filter(|purchase| {
                if let Some(item) = purchase
                    .items
                    .iter()
                    .find(|item| !vendor_ids.contains(item.vendor_id.as_str()))
                {
                    skipped_purchase_count += 1;
                    info!(
                        "Skipping purchase {} during booth export because item {} references missing vendor {} for booth {}",
                        purchase.id, item.id, item.vendor_id, booth_id
                    );
                    false
                } else {
                    true
                }
            })
            .collect();

        if skipped_purchase_count > 0 {
            info!(
                "Booth export for {} skipped {} purchases with missing vendor references",
                booth_id, skipped_purchase_count
            );
        }

        (vendors, valid_purchases)
    }
}
