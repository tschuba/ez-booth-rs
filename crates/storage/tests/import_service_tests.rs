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

fn booth_backup_with_records(
    booth: Booth,
    vendors: Vec<Vendor>,
    purchases: Vec<Purchase>,
) -> BoothBackupData {
    BoothBackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "test-version".to_string(),
        checksum: None,
        device_info: None,
        booth,
        vendors,
        purchases,
    }
}

fn full_backup_with_records(
    booths: Vec<Booth>,
    vendors: Vec<Vendor>,
    purchases: Vec<Purchase>,
) -> BackupData {
    BackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "test-version".to_string(),
        checksum: None,
        device_info: None,
        booths,
        vendors,
        purchases,
        metadata: Default::default(),
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
async fn import_merge_keeps_existing_vendor_name_when_incoming_is_shorter() {
    let (booth_repo, vendor_repo, _purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Vendor Merge 2026");
    let existing_vendor = create_test_vendor(&booth, "12", Some("Ada Lovelace Crafts"));

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&existing_vendor).await.unwrap();

    let incoming_vendor = create_test_vendor(&booth, "12", Some("Ada"));

    let summary = service
        .import_booth_backup(
            BoothBackupData {
                version: BACKUP_FORMAT_VERSION,
                created_at: Utc::now(),
                app_version: "test-version".to_string(),
                checksum: None,
                device_info: None,
                booth: booth.clone(),
                vendors: vec![incoming_vendor],
                purchases: vec![],
            },
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.conflicts_resolved, 2);

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &existing_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Ada Lovelace Crafts"));
}

#[wasm_bindgen_test]
async fn import_merge_uses_stable_vendor_name_when_both_lengths_match() {
    let (booth_repo, vendor_repo, _purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Vendor Tie 2026");
    let existing_vendor = create_test_vendor(&booth, "12", Some("Zed Shop"));

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&existing_vendor).await.unwrap();

    let incoming_vendor = create_test_vendor(&booth, "12", Some("Ada Shop"));

    service
        .import_booth_backup(
            BoothBackupData {
                version: BACKUP_FORMAT_VERSION,
                created_at: Utc::now(),
                app_version: "test-version".to_string(),
                checksum: None,
                device_info: None,
                booth: booth.clone(),
                vendors: vec![incoming_vendor],
                purchases: vec![],
            },
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &existing_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Ada Shop"));
}

#[wasm_bindgen_test]
async fn import_merge_with_equal_booth_timestamps_keeps_existing_record() {
    let (booth_repo, _vendor_repo, _purchase_repo, service) = build_service().await;
    let mut booth = create_test_booth("Equal Timestamp Booth");
    let shared_timestamp = Utc::now();
    booth.updated_at = shared_timestamp;

    booth_repo.save(&booth).await.unwrap();

    let mut incoming_booth = booth.clone();
    incoming_booth.description = "A deterministic description".to_string();
    incoming_booth.updated_at = shared_timestamp;

    service
        .import_booth_backup(
            BoothBackupData {
                version: BACKUP_FORMAT_VERSION,
                created_at: Utc::now(),
                app_version: "test-version".to_string(),
                checksum: None,
                device_info: None,
                booth: incoming_booth,
                vendors: vec![],
                purchases: vec![],
            },
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_booth = booth_repo.find_by_id(&booth.id).await.unwrap().unwrap();
    assert_eq!(saved_booth.description, "Equal Timestamp Booth");
}

#[wasm_bindgen_test]
async fn import_merge_with_equal_purchase_timestamps_keeps_existing_record() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Equal Timestamp Purchase");
    let vendor = create_test_vendor(&booth, "12", Some("Ada Vendor"));
    let mut existing_purchase = create_test_purchase(&booth, &vendor.vendor_id);
    let shared_timestamp = Utc::now();
    existing_purchase.timestamp = shared_timestamp;

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&existing_purchase).await.unwrap();

    let mut incoming_purchase = existing_purchase.clone();
    incoming_purchase.note = Some("Imported deterministic note".to_string());
    incoming_purchase.timestamp = shared_timestamp;

    service
        .import_booth_backup(
            BoothBackupData {
                version: BACKUP_FORMAT_VERSION,
                created_at: Utc::now(),
                app_version: "test-version".to_string(),
                checksum: None,
                device_info: None,
                booth: booth.clone(),
                vendors: vec![vendor.clone()],
                purchases: vec![incoming_purchase],
            },
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_purchase = purchase_repo
        .find_by_id(&existing_purchase.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_purchase.note.as_deref(), None);
}

#[wasm_bindgen_test]
async fn repeated_multi_device_import_of_shared_history_does_not_duplicate_records() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Shared History 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Shared Vendor"));
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    let device_a_backup = BoothBackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "device-a".to_string(),
        checksum: None,
        device_info: None,
        booth: booth.clone(),
        vendors: vec![vendor.clone()],
        purchases: vec![purchase.clone()],
    };

    let mut device_b_purchase = create_test_purchase(&booth, &vendor.vendor_id);
    device_b_purchase.note = Some("Device B checkout".to_string());
    let device_b_backup = BoothBackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: "device-b".to_string(),
        checksum: None,
        device_info: None,
        booth: booth.clone(),
        vendors: vec![vendor.clone()],
        purchases: vec![purchase.clone(), device_b_purchase.clone()],
    };

    let summary_a = service
        .import_booth_backup(device_a_backup, ConflictStrategy::Merge)
        .await
        .unwrap();
    let summary_b = service
        .import_booth_backup(device_b_backup, ConflictStrategy::Merge)
        .await
        .unwrap();

    assert_eq!(summary_a.booths_imported, 1);
    assert_eq!(summary_a.vendors_imported, 1);
    assert_eq!(summary_a.purchases_imported, 1);
    assert_eq!(summary_b.purchases_imported, 2);

    let saved_booth = booth_repo.find_by_id(&booth.id).await.unwrap();
    assert!(saved_booth.is_some());

    let vendors = vendor_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(vendors.len(), 1);

    let purchases = purchase_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(purchases.len(), 2);
    assert!(purchases
        .iter()
        .any(|candidate| candidate.id == purchase.id));
    assert!(purchases
        .iter()
        .any(|candidate| candidate.id == device_b_purchase.id));
}

#[wasm_bindgen_test]
async fn parallel_three_device_booth_merge_preserves_all_unique_purchases() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Parallel Merge 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Shared Vendor"));

    let mut purchase_a = create_test_purchase(&booth, &vendor.vendor_id);
    purchase_a.note = Some("device-a".to_string());
    purchase_a.timestamp = Utc::now() - Duration::minutes(3);

    let mut purchase_b = create_test_purchase(&booth, &vendor.vendor_id);
    purchase_b.note = Some("device-b".to_string());
    purchase_b.timestamp = Utc::now() - Duration::minutes(2);

    let mut purchase_c = create_test_purchase(&booth, &vendor.vendor_id);
    purchase_c.note = Some("device-c".to_string());
    purchase_c.timestamp = Utc::now() - Duration::minutes(1);

    service
        .import_booth_backup(
            booth_backup_with_records(
                booth.clone(),
                vec![vendor.clone()],
                vec![purchase_a.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(
                booth.clone(),
                vec![vendor.clone()],
                vec![purchase_a.clone(), purchase_b.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(
                booth.clone(),
                vec![vendor.clone()],
                vec![purchase_a.clone(), purchase_c.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    assert!(booth_repo.find_by_id(&booth.id).await.unwrap().is_some());

    let vendors = vendor_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(vendors.len(), 1);

    let purchases = purchase_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(purchases.len(), 3);
    assert!(purchases
        .iter()
        .any(|purchase| purchase.id == purchase_a.id));
    assert!(purchases
        .iter()
        .any(|purchase| purchase.id == purchase_b.id));
    assert!(purchases
        .iter()
        .any(|purchase| purchase.id == purchase_c.id));
}

#[wasm_bindgen_test]
async fn round_trip_import_merges_new_records_without_readding_shared_history() {
    let (_booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Round Trip 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Round Trip Vendor"));

    let mut original_purchase = create_test_purchase(&booth, &vendor.vendor_id);
    original_purchase.note = Some("original".to_string());

    let device_a_backup = booth_backup_with_records(
        booth.clone(),
        vec![vendor.clone()],
        vec![original_purchase.clone()],
    );

    service
        .import_booth_backup(device_a_backup, ConflictStrategy::Merge)
        .await
        .unwrap();

    let mut new_purchase = create_test_purchase(&booth, &vendor.vendor_id);
    new_purchase.note = Some("device-b-new".to_string());

    let device_b_backup = booth_backup_with_records(
        booth.clone(),
        vec![vendor.clone()],
        vec![original_purchase.clone(), new_purchase.clone()],
    );

    service
        .import_booth_backup(device_b_backup, ConflictStrategy::Merge)
        .await
        .unwrap();

    let vendors = vendor_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(vendors.len(), 1);

    let purchases = purchase_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(purchases.len(), 2);
    assert!(purchases
        .iter()
        .any(|purchase| purchase.id == original_purchase.id));
    assert!(purchases
        .iter()
        .any(|purchase| purchase.id == new_purchase.id));
}

#[wasm_bindgen_test]
async fn multi_device_booth_merge_prefers_latest_booth_update_even_when_imported_out_of_order() {
    let (booth_repo, _vendor_repo, _purchase_repo, service) = build_service().await;
    let mut base_booth = create_test_booth("Config Merge 2026");
    let t1 = Utc::now() - Duration::hours(3);
    let t2 = Utc::now() - Duration::hours(2);
    let t3 = Utc::now() - Duration::hours(1);

    base_booth.updated_at = t1;

    let mut booth_a = base_booth.clone();
    booth_a.description = "Config from A".to_string();
    booth_a.updated_at = t1;

    let mut booth_b = base_booth.clone();
    booth_b.description = "Config from B".to_string();
    booth_b.updated_at = t2;
    booth_b.fees.participation_fee = dec!(15.00);

    let mut booth_c = base_booth.clone();
    booth_c.description = "Config from C".to_string();
    booth_c.updated_at = t3;
    booth_c.fees.participation_fee = dec!(20.00);

    service
        .import_booth_backup(
            booth_backup_with_records(booth_b.clone(), vec![], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();
    service
        .import_booth_backup(
            booth_backup_with_records(booth_a.clone(), vec![], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();
    service
        .import_booth_backup(
            booth_backup_with_records(booth_c.clone(), vec![], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_booth = booth_repo
        .find_by_id(&base_booth.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_booth.description, "Config from C");
    assert_eq!(saved_booth.fees.participation_fee, dec!(20.00));
}

#[wasm_bindgen_test]
async fn import_all_from_multiple_devices_preserves_other_booths_while_merging_shared_booth() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let shared_booth = create_test_booth("Shared Booth 2026");
    let other_booth = create_test_booth("Other Booth 2026");

    let shared_vendor = create_test_vendor(&shared_booth, "12", Some("Shared Vendor"));
    let other_vendor = create_test_vendor(&other_booth, "55", Some("Other Vendor"));

    let shared_purchase = create_test_purchase(&shared_booth, &shared_vendor.vendor_id);
    let other_purchase = create_test_purchase(&other_booth, &other_vendor.vendor_id);

    service
        .import_all(
            full_backup_with_records(
                vec![shared_booth.clone()],
                vec![shared_vendor.clone()],
                vec![shared_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_all(
            full_backup_with_records(
                vec![shared_booth.clone(), other_booth.clone()],
                vec![shared_vendor.clone(), other_vendor.clone()],
                vec![shared_purchase.clone(), other_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let booths = booth_repo.find_all().await.unwrap();
    assert_eq!(booths.len(), 2);

    let shared_vendors = vendor_repo.find_by_booth(&shared_booth.id).await.unwrap();
    let other_vendors = vendor_repo.find_by_booth(&other_booth.id).await.unwrap();
    assert_eq!(shared_vendors.len(), 1);
    assert_eq!(other_vendors.len(), 1);

    let shared_purchases = purchase_repo.find_by_booth(&shared_booth.id).await.unwrap();
    let other_purchases = purchase_repo.find_by_booth(&other_booth.id).await.unwrap();
    assert_eq!(shared_purchases.len(), 1);
    assert_eq!(other_purchases.len(), 1);
}

#[wasm_bindgen_test]
async fn multi_device_vendor_name_merge_prefers_richer_name_across_exports() {
    let (_booth_repo, vendor_repo, _purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Vendor Conflict 2026");

    let unnamed_vendor = create_test_vendor(&booth, "12", None);
    let short_name_vendor = create_test_vendor(&booth, "12", Some("Ada"));
    let rich_name_vendor = create_test_vendor(&booth, "12", Some("Ada Lovelace Crafts"));

    service
        .import_booth_backup(
            booth_backup_with_records(booth.clone(), vec![unnamed_vendor], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(booth.clone(), vec![short_name_vendor], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(booth.clone(), vec![rich_name_vendor], vec![]),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &VendorId::new("12".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Ada Lovelace Crafts"));
}

#[wasm_bindgen_test]
async fn same_purchase_id_from_multiple_devices_keeps_newer_purchase_data() {
    let (_booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Purchase Conflict 2026");
    let vendor = create_test_vendor(&booth, "12", Some("Conflict Vendor"));

    let mut older_purchase = create_test_purchase(&booth, &vendor.vendor_id);
    older_purchase.note = Some("older note".to_string());
    older_purchase.timestamp = Utc::now() - Duration::minutes(5);

    let mut newer_purchase = older_purchase.clone();
    newer_purchase.note = Some("newer note".to_string());
    newer_purchase.timestamp = Utc::now();

    service
        .import_booth_backup(
            booth_backup_with_records(
                booth.clone(),
                vec![vendor.clone()],
                vec![older_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(
                booth.clone(),
                vec![vendor.clone()],
                vec![newer_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let purchases = purchase_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(purchases.len(), 1);
    assert_eq!(purchases[0].note.as_deref(), Some("newer note"));

    let vendors = vendor_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(vendors.len(), 1);
}

#[wasm_bindgen_test]
async fn mixed_full_and_booth_imports_preserve_shared_history_without_duplication() {
    let (booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let shared_booth = create_test_booth("Mixed Shared Booth 2026");
    let extra_booth = create_test_booth("Mixed Extra Booth 2026");

    let shared_vendor = create_test_vendor(&shared_booth, "12", Some("Shared Vendor"));
    let extra_vendor = create_test_vendor(&extra_booth, "55", Some("Extra Vendor"));

    let shared_purchase = create_test_purchase(&shared_booth, &shared_vendor.vendor_id);
    let mut second_shared_purchase = create_test_purchase(&shared_booth, &shared_vendor.vendor_id);
    second_shared_purchase.note = Some("second shared".to_string());
    let extra_purchase = create_test_purchase(&extra_booth, &extra_vendor.vendor_id);

    service
        .import_all(
            full_backup_with_records(
                vec![shared_booth.clone(), extra_booth.clone()],
                vec![shared_vendor.clone(), extra_vendor.clone()],
                vec![shared_purchase.clone(), extra_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    service
        .import_booth_backup(
            booth_backup_with_records(
                shared_booth.clone(),
                vec![shared_vendor.clone()],
                vec![shared_purchase.clone(), second_shared_purchase.clone()],
            ),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let booths = booth_repo.find_all().await.unwrap();
    assert_eq!(booths.len(), 2);

    let shared_purchases = purchase_repo.find_by_booth(&shared_booth.id).await.unwrap();
    let extra_purchases = purchase_repo.find_by_booth(&extra_booth.id).await.unwrap();
    assert_eq!(shared_purchases.len(), 2);
    assert_eq!(extra_purchases.len(), 1);
    assert!(shared_purchases
        .iter()
        .any(|purchase| purchase.id == shared_purchase.id));
    assert!(shared_purchases
        .iter()
        .any(|purchase| purchase.id == second_shared_purchase.id));

    let shared_vendors = vendor_repo.find_by_booth(&shared_booth.id).await.unwrap();
    let extra_vendors = vendor_repo.find_by_booth(&extra_booth.id).await.unwrap();
    assert_eq!(shared_vendors.len(), 1);
    assert_eq!(extra_vendors.len(), 1);
}

#[wasm_bindgen_test]
async fn large_multi_device_merge_keeps_expected_record_counts() {
    let (_booth_repo, vendor_repo, purchase_repo, service) = build_service().await;
    let booth = create_test_booth("Stress Merge 2026");

    let shared_vendor = create_test_vendor(&booth, "1", Some("Vendor 1"));
    let mut vendors = vec![shared_vendor.clone()];
    for vendor_number in 2..=6 {
        vendors.push(create_test_vendor(
            &booth,
            &vendor_number.to_string(),
            Some(&format!("Vendor {vendor_number}")),
        ));
    }

    let mut device_a_purchases = Vec::new();
    let mut device_b_purchases = Vec::new();
    let mut device_c_purchases = Vec::new();

    for (index, vendor) in vendors.iter().enumerate() {
        let mut purchase_a = create_test_purchase(&booth, &vendor.vendor_id);
        purchase_a.note = Some(format!("device-a-{index}"));
        device_a_purchases.push(purchase_a);

        let mut purchase_b = create_test_purchase(&booth, &vendor.vendor_id);
        purchase_b.note = Some(format!("device-b-{index}"));
        device_b_purchases.push(purchase_b);

        let mut purchase_c = create_test_purchase(&booth, &vendor.vendor_id);
        purchase_c.note = Some(format!("device-c-{index}"));
        device_c_purchases.push(purchase_c);
    }

    let mut device_b_vendors = vendors.clone();
    device_b_vendors[0] = create_test_vendor(&booth, "1", Some("Vendor 1 Extended Name"));

    let mut device_c_booth = booth.clone();
    device_c_booth.updated_at = Utc::now() + Duration::minutes(1);
    device_c_booth.description = "Stress Merge Final".to_string();

    service
        .import_booth_backup(
            booth_backup_with_records(booth.clone(), vendors.clone(), device_a_purchases),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();
    service
        .import_booth_backup(
            booth_backup_with_records(booth.clone(), device_b_vendors, device_b_purchases),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();
    service
        .import_booth_backup(
            booth_backup_with_records(device_c_booth.clone(), vendors.clone(), device_c_purchases),
            ConflictStrategy::Merge,
        )
        .await
        .unwrap();

    let saved_vendor = vendor_repo
        .find_by_id(&booth.id, &VendorId::new("1".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Vendor 1 Extended Name"));

    let purchases = purchase_repo.find_by_booth(&booth.id).await.unwrap();
    assert_eq!(purchases.len(), 18);
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
