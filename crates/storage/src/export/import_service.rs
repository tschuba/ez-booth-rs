use std::sync::Arc;

use domain::{Booth, BoothRepository, Purchase, PurchaseRepository, Vendor, VendorRepository};

use super::backup_format::{BackupData, BoothBackupData};
use super::error::{ImportError, SkippedRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    Skip,
    Replace,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportSummary {
    pub booths_imported: usize,
    pub vendors_imported: usize,
    pub purchases_imported: usize,
    pub conflicts_resolved: usize,
    pub skipped_records: Vec<SkippedRecord>,
}

#[derive(Clone)]
pub struct ImportService {
    booth_repository: Arc<dyn BoothRepository>,
    vendor_repository: Arc<dyn VendorRepository>,
    purchase_repository: Arc<dyn PurchaseRepository>,
}

impl ImportService {
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

    pub async fn import_all(
        &self,
        data: BackupData,
        strategy: ConflictStrategy,
    ) -> Result<ImportSummary, ImportError> {
        let mut summary = ImportSummary::default();

        for booth in data.booths {
            self.import_booth_record(booth, strategy, &mut summary)
                .await?;
        }

        for vendor in data.vendors {
            self.import_vendor_record(vendor, strategy, &mut summary)
                .await?;
        }

        for purchase in data.purchases {
            self.import_purchase_record(purchase, strategy, &mut summary)
                .await?;
        }

        Ok(summary)
    }

    pub async fn import_booth_backup(
        &self,
        data: BoothBackupData,
        strategy: ConflictStrategy,
    ) -> Result<ImportSummary, ImportError> {
        let mut summary = ImportSummary::default();

        self.import_booth_record(data.booth, strategy, &mut summary)
            .await?;

        for vendor in data.vendors {
            self.import_vendor_record(vendor, strategy, &mut summary)
                .await?;
        }

        for purchase in data.purchases {
            self.import_purchase_record(purchase, strategy, &mut summary)
                .await?;
        }

        Ok(summary)
    }

    async fn import_booth_record(
        &self,
        incoming: Booth,
        strategy: ConflictStrategy,
        summary: &mut ImportSummary,
    ) -> Result<(), ImportError> {
        match self.booth_repository.find_by_id(&incoming.id).await? {
            None => {
                self.booth_repository.save(&incoming).await?;
                summary.booths_imported += 1;
            }
            Some(existing) => match strategy {
                ConflictStrategy::Skip => summary.skipped_records.push(SkippedRecord {
                    record_type: "booth".to_string(),
                    record_id: incoming.id.to_string(),
                    reason: "record already exists".to_string(),
                }),
                ConflictStrategy::Replace => {
                    self.booth_repository.save(&incoming).await?;
                    summary.booths_imported += 1;
                    summary.conflicts_resolved += 1;
                }
                ConflictStrategy::Merge => {
                    let merged = if incoming.updated_at > existing.updated_at {
                        incoming
                    } else {
                        existing
                    };
                    self.booth_repository.save(&merged).await?;
                    summary.booths_imported += 1;
                    summary.conflicts_resolved += 1;
                }
            },
        }

        Ok(())
    }

    async fn import_vendor_record(
        &self,
        incoming: Vendor,
        strategy: ConflictStrategy,
        summary: &mut ImportSummary,
    ) -> Result<(), ImportError> {
        match self
            .vendor_repository
            .find_by_id(&incoming.booth_id, &incoming.vendor_id)
            .await?
        {
            None => {
                self.vendor_repository.save(&incoming).await?;
                summary.vendors_imported += 1;
            }
            Some(existing) => match strategy {
                ConflictStrategy::Skip => summary.skipped_records.push(SkippedRecord {
                    record_type: "vendor".to_string(),
                    record_id: incoming.vendor_id.to_string(),
                    reason: format!("record already exists in booth {}", incoming.booth_id),
                }),
                ConflictStrategy::Replace => {
                    self.vendor_repository.save(&incoming).await?;
                    summary.vendors_imported += 1;
                    summary.conflicts_resolved += 1;
                }
                ConflictStrategy::Merge => {
                    let merged_name = merge_vendor_name(&existing, &incoming);
                    let merged = Vendor {
                        name: merged_name,
                        created_at: incoming.created_at.min(existing.created_at),
                        ..incoming
                    };
                    self.vendor_repository.save(&merged).await?;
                    summary.vendors_imported += 1;
                    summary.conflicts_resolved += 1;
                }
            },
        }

        Ok(())
    }

    async fn import_purchase_record(
        &self,
        incoming: Purchase,
        strategy: ConflictStrategy,
        summary: &mut ImportSummary,
    ) -> Result<(), ImportError> {
        match self.purchase_repository.find_by_id(&incoming.id).await? {
            None => {
                self.purchase_repository.save(&incoming).await?;
                summary.purchases_imported += 1;
            }
            Some(existing) => match strategy {
                ConflictStrategy::Skip => summary.skipped_records.push(SkippedRecord {
                    record_type: "purchase".to_string(),
                    record_id: incoming.id.to_string(),
                    reason: "record already exists".to_string(),
                }),
                ConflictStrategy::Replace => {
                    if existing.booth_id != incoming.booth_id {
                        self.purchase_repository
                            .delete_from_booth(&existing.booth_id, &existing.id)
                            .await?;
                    }
                    self.purchase_repository.save(&incoming).await?;
                    summary.purchases_imported += 1;
                    summary.conflicts_resolved += 1;
                }
                ConflictStrategy::Merge => {
                    let existing_booth_id = existing.booth_id;
                    let existing_id = existing.id;
                    let merged = if incoming.timestamp > existing.timestamp {
                        incoming
                    } else {
                        existing
                    };

                    if merged.id == existing_id && existing_booth_id != merged.booth_id {
                        self.purchase_repository
                            .delete_from_booth(&existing_booth_id, &existing_id)
                            .await?;
                    }

                    self.purchase_repository.save(&merged).await?;
                    summary.purchases_imported += 1;
                    summary.conflicts_resolved += 1;
                }
            },
        }

        Ok(())
    }
}

fn merge_vendor_name(existing: &Vendor, incoming: &Vendor) -> Option<String> {
    match (
        normalize_vendor_name(existing.name.as_ref()),
        normalize_vendor_name(incoming.name.as_ref()),
    ) {
        (Some(existing_name), Some(incoming_name)) => {
            if existing_name.len() > incoming_name.len() {
                Some(existing_name.to_string())
            } else if incoming_name.len() > existing_name.len() {
                Some(incoming_name.to_string())
            } else if existing_name <= incoming_name {
                Some(existing_name.to_string())
            } else {
                Some(incoming_name.to_string())
            }
        }
        (Some(existing_name), None) => Some(existing_name.to_string()),
        (None, Some(incoming_name)) => Some(incoming_name.to_string()),
        (None, None) => None,
    }
}

fn normalize_vendor_name(name: Option<&String>) -> Option<&str> {
    name.map(|value| value.trim())
        .filter(|value| !value.is_empty())
}
