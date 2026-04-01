wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use chrono::{Duration, NaiveDate, Utc};
use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{
    BackupData, BoothBackupData, ConflictStrategy, ImportService, ImportSummary,
    BACKUP_FORMAT_VERSION,
};
use ez_booth_storage::indexeddb::Database;
use ez_booth_storage::repositories::{
    IndexedDbBoothRepository, IndexedDbPurchaseRepository, IndexedDbVendorRepository,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use wasm_bindgen_test::*;

async fn create_test_db() -> Database {
    let db_name = format!("test_import_service_db_{}", js_sys::Math::random());
    Database::new_with_name(&db_name)
        .await
        .expect("Failed to create test database")
}

fn create_test_booth(description: &str) -> Booth {
    Booth::new(
        description.to_string(),
        NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
        FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(15.00),
            rounding_step: dec!(0.50),
        },
    )
    .unwrap()
}

fn create_test_vendor(booth: &Booth, vendor_id: &str, name: Option<&str>) -> Vendor {
    let vendor = Vendor::new(VendorId::new(vendor_id.to_string()), booth.id);
    match name {
        Some(name) => vendor.with_name(name.to_string()),
        None => vendor,
    }
}

fn create_test_purchase(booth: &Booth, vendor_id: &VendorId) -> Purchase {
    Purchase::new(
        booth.id,
        vec![PurchaseItem::new(dec!(42.00), vendor_id.clone()).unwrap()],
    )
    .unwrap()
}

fn full_backup(booth: Booth, vendor: Vendor, purchase: Purchase) -> BackupData {
    BackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "test-version".to_string(),
        checksum: None,
        device_info: None,
        booths: vec![booth],
        vendors: vec![vendor],
        purchases: vec![purchase],
        metadata: Default::default(),
    }
}

fn booth_backup(booth: Booth, vendor: Vendor, purchase: Purchase) -> BoothBackupData {
    BoothBackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "test-version".to_string(),
        checksum: None,
        device_info: None,
        booth,
        vendors: vec![vendor],
        purchases: vec![purchase],
    }
}

async fn build_service() -> (
    Arc<dyn BoothRepository>,
    Arc<dyn VendorRepository>,
    Arc<dyn PurchaseRepository>,
    ImportService,
) {
    let db = Arc::new(create_test_db().await);
    let booth_repo: Arc<dyn BoothRepository> = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo: Arc<dyn VendorRepository> =
        Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo: Arc<dyn PurchaseRepository> =
        Arc::new(IndexedDbPurchaseRepository::new(db.clone()));
    let service = ImportService::new(
        booth_repo.clone(),
        vendor_repo.clone(),
        purchase_repo.clone(),
    );
    (booth_repo, vendor_repo, purchase_repo, service)
}

#[wasm_bindgen_test]
async fn import_all_saves_new_records() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Spring Market 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Ada Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    let summary = service
        .import_all(
            full_backup(booth.clone(), vendor.clone(), purchase.clone()),
            ConflictStrategy::Skip,
        )
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.conflicts_resolved, 0);
    assert!(summary.skipped_records.is_empty());

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn import_skip_leaves_existing_records_untouched() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Spring Market 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Existing Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let mut incoming_booth = booth.clone();
    incoming_booth.description = "Updated Description".to_string();
    let incoming_vendor = create_test_vendor(&booth, "12", Some("Imported Vendor"));
    let mut incoming_purchase = purchase.clone();
    incoming_purchase.note = Some("Imported note".to_string());

    let summary = service
        .import_all(
            full_backup(incoming_booth, incoming_vendor, incoming_purchase),
            ConflictStrategy::Skip,
        )
        .await
        .unwrap();

    assert_eq!(summary.skipped_records.len(), 3);
    assert_eq!(summary.conflicts_resolved, 0);

    let saved_booth = booth_repo.find_by_id(&booth.id).await.unwrap().unwrap();
    assert_eq!(saved_booth.description, "Spring Market 2026");
}

#[wasm_bindgen_test]
async fn import_replace_overwrites_existing_records() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Spring Market 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Existing Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let mut incoming_booth = booth.clone();
    incoming_booth.description = "Updated Description".to_string();
    let incoming_vendor = create_test_vendor(&incoming_booth, "12", Some("Imported Vendor"));
    let mut incoming_purchase = purchase.clone();
    incoming_purchase.note = Some("Imported note".to_string());

    let summary = service
        .import_all(
            full_backup(
                incoming_booth.clone(),
                incoming_vendor.clone(),
                incoming_purchase.clone(),
            ),
            ConflictStrategy::Replace,
        )
        .await
        .unwrap();

    assert_eq!(summary.conflicts_resolved, 3);
    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);

    let saved_booth = booth_repo.find_by_id(&booth.id).await.unwrap().unwrap();
    assert_eq!(saved_booth.description, "Updated Description");

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &incoming_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Imported Vendor"));

    let saved_purchase = purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_purchase.note.as_deref(), Some("Imported note"));
}

#[wasm_bindgen_test]
async fn import_merge_prefers_newer_booth_and_purchase_data() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let mut booth = create_test_booth("Spring Market 2026");
    let vendor = create_test_vendor(&booth, "12", None);
    let mut purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth.updated_at = Utc::now() - Duration::days(1);
    purchase.timestamp = Utc::now() - Duration::days(1);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let mut incoming_booth = booth.clone();
    incoming_booth.description = "Merged Description".to_string();
    incoming_booth.updated_at = Utc::now();

    let incoming_vendor = create_test_vendor(&incoming_booth, "12", Some("Imported Vendor"));

    let mut incoming_purchase = purchase.clone();
    incoming_purchase.note = Some("Merged note".to_string());
    incoming_purchase.timestamp = Utc::now();

    let summary = service
        .import_all(
            full_backup(
                incoming_booth.clone(),
                incoming_vendor.clone(),
                incoming_purchase.clone(),
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    assert_eq!(summary.conflicts_resolved, 3);

    let saved_booth = booth_repo.find_by_id(&booth.id).await.unwrap().unwrap();
    assert_eq!(saved_booth.description, "Merged Description");

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &incoming_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Imported Vendor"));

    let saved_purchase = purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_purchase.note.as_deref(), Some("Merged note"));
}

#[wasm_bindgen_test]
async fn import_booth_backup_saves_only_that_scope() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Scoped Event 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Scoped Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    let summary = service
        .import_booth_backup(
            booth_backup(booth.clone(), vendor.clone(), purchase.clone()),
            ConflictStrategy::Skip,
        )
        .await
        .unwrap();

    assert_eq!(
        summary,
        ImportSummary {
            booths_imported: 1,
            vendors_imported: 1,
            purchases_imported: 1,
            conflicts_resolved: 0,
            skipped_records: vec![],
        }
    );

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn import_after_delete_recreates_booth_with_skip_strategy() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Spring Market 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Test Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let backup = booth_backup(booth.clone(), vendor.clone(), purchase.clone());

    purchase_repo
        .delete_from_booth(&booth.id, &purchase.id)
        .await
        .unwrap();
    vendor_repo
        .delete(&booth.id, &vendor.vendor_id)
        .await
        .unwrap();
    booth_repo.delete(&booth.id).await.unwrap();

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_none());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_none());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_none());

    let summary = service
        .import_booth_backup(backup, ConflictStrategy::Skip)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.skipped_records.len(), 0);
    assert_eq!(summary.conflicts_resolved, 0);

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn import_after_delete_recreates_booth_with_replace_strategy() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Summer Festival 2026");
    let vendor = create_test_vendor(&booth, "42", Some("Replaced Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let backup = booth_backup(booth.clone(), vendor.clone(), purchase.clone());

    purchase_repo
        .delete_from_booth(&booth.id, &purchase.id)
        .await
        .unwrap();
    vendor_repo
        .delete(&booth.id, &vendor.vendor_id)
        .await
        .unwrap();
    booth_repo.delete(&booth.id).await.unwrap();

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_none());

    let summary = service
        .import_booth_backup(backup, ConflictStrategy::Replace)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.conflicts_resolved, 0);

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn import_after_delete_recreates_booth_with_merge_strategy() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Autumn Market 2026");
    let vendor = create_test_vendor(&booth, "99", Some("Merged Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let backup = booth_backup(booth.clone(), vendor.clone(), purchase.clone());

    purchase_repo
        .delete_from_booth(&booth.id, &purchase.id)
        .await
        .unwrap();
    vendor_repo
        .delete(&booth.id, &vendor.vendor_id)
        .await
        .unwrap();
    booth_repo.delete(&booth.id).await.unwrap();

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_none());

    let summary = service
        .import_booth_backup(backup, ConflictStrategy::Merge)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.conflicts_resolved, 0);

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn import_full_backup_after_delete_recreates_records() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Winter Festival 2026");
    let vendor = create_test_vendor(&booth, "77", Some("Full Backup Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let backup = full_backup(booth.clone(), vendor.clone(), purchase.clone());

    purchase_repo
        .delete_from_booth(&booth.id, &purchase.id)
        .await
        .unwrap();
    vendor_repo
        .delete(&booth.id, &vendor.vendor_id)
        .await
        .unwrap();
    booth_repo.delete(&booth.id).await.unwrap();

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_none());

    let summary = service
        .import_all(backup, ConflictStrategy::Skip)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.skipped_records.len(), 0);

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(vendor_repo
        .find_by_id(&booth.id, &vendor.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(purchase_repo
        .find_by_id(&purchase.id)
        .await
        .unwrap()
        .is_some());
}
