use std::sync::Arc;

use domain::{BoothId, BoothRepository, PurchaseRepository, VendorRepository};

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
        data.booths = self.booth_repository.find_all().await?;
        data.vendors = self.vendor_repository.find_all().await?;
        data.purchases = self.purchase_repository.find_all().await?;
        Ok(data)
    }

    pub async fn export_booth(&self, booth_id: &BoothId) -> Result<BoothBackupData, ExportError> {
        let booth = self
            .booth_repository
            .find_by_id(booth_id)
            .await?
            .ok_or(ExportError::BoothNotFound(*booth_id))?;

        let mut data = BoothBackupData::new(booth.clone(), self.app_version.clone());
        data.vendors = self.vendor_repository.find_by_booth(booth_id).await?;
        data.purchases = self.purchase_repository.find_by_booth(booth_id).await?;
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
}
