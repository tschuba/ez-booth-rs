wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use chrono::NaiveDate;
use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{ExportError, ExportService, BACKUP_FORMAT_VERSION};
use ez_booth_storage::indexeddb::Database;
use ez_booth_storage::repositories::{
    IndexedDbBoothRepository, IndexedDbPurchaseRepository, IndexedDbVendorRepository,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use wasm_bindgen_test::*;

async fn create_test_db() -> Database {
    let db_name = format!("test_export_db_{}", js_sys::Math::random());
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

fn create_test_purchase(booth: &Booth, vendor_id: &VendorId) -> Purchase {
    Purchase::new(
        booth.id,
        vec![PurchaseItem::new(dec!(42.00), vendor_id.clone()).unwrap()],
    )
    .unwrap()
}

#[wasm_bindgen_test]
async fn test_export_all_collects_all_records_and_serializes() {
    let db = Arc::new(create_test_db().await);
    let booth_repo: Arc<dyn BoothRepository> = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo: Arc<dyn VendorRepository> =
        Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo: Arc<dyn PurchaseRepository> =
        Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

    let booth = create_test_booth("Spring Market 2026");
    let vendor =
        Vendor::new(VendorId::new("12".to_string()), booth.id).with_name("Ada Vendor".to_string());
    let purchase = create_test_purchase(&booth, &vendor.vendor_id);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&purchase).await.unwrap();

    let service = ExportService::with_app_version(
        booth_repo.clone(),
        vendor_repo.clone(),
        purchase_repo.clone(),
        "test-version",
    );

    let backup = service.export_all().await.unwrap();
    assert_eq!(backup.version, BACKUP_FORMAT_VERSION);
    assert_eq!(backup.app_version, "test-version");
    assert_eq!(backup.booths.len(), 1);
    assert_eq!(backup.vendors.len(), 1);
    assert_eq!(backup.purchases.len(), 1);
    assert!(backup.metadata.is_empty());

    let serialized = service.serialize_full_backup(&backup).unwrap();
    assert_eq!(serialized.file_name, "ez-booth-backup-2026-03-29.json");
    assert!(serialized.json.contains("\n  \"booths\""));
    assert!(serialized.json.contains("Spring Market 2026"));
}

#[wasm_bindgen_test]
async fn test_export_booth_filters_to_requested_booth() {
    let db = Arc::new(create_test_db().await);
    let booth_repo: Arc<dyn BoothRepository> = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo: Arc<dyn VendorRepository> =
        Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo: Arc<dyn PurchaseRepository> =
        Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

    let booth_a = create_test_booth("Spring Market 2026");
    let booth_b = create_test_booth("Autumn Fair 2026");
    let vendor_a = Vendor::new(VendorId::new("12".to_string()), booth_a.id);
    let vendor_b = Vendor::new(VendorId::new("99".to_string()), booth_b.id);
    let purchase_a = create_test_purchase(&booth_a, &vendor_a.vendor_id);
    let purchase_b = create_test_purchase(&booth_b, &vendor_b.vendor_id);

    booth_repo.save(&booth_a).await.unwrap();
    booth_repo.save(&booth_b).await.unwrap();
    vendor_repo.save(&vendor_a).await.unwrap();
    vendor_repo.save(&vendor_b).await.unwrap();
    purchase_repo.save(&purchase_a).await.unwrap();
    purchase_repo.save(&purchase_b).await.unwrap();

    let service = ExportService::with_app_version(
        booth_repo.clone(),
        vendor_repo.clone(),
        purchase_repo.clone(),
        "test-version",
    );

    let backup = service.export_booth(&booth_a.id).await.unwrap();
    assert_eq!(backup.version, BACKUP_FORMAT_VERSION);
    assert_eq!(backup.booth.id, booth_a.id);
    assert_eq!(backup.vendors.len(), 1);
    assert_eq!(backup.vendors[0].booth_id, booth_a.id);
    assert_eq!(backup.purchases.len(), 1);
    assert_eq!(backup.purchases[0].booth_id, booth_a.id);

    let serialized = service.serialize_booth_backup(&backup).unwrap();
    assert_eq!(
        serialized.file_name,
        "ez-booth-spring-market-2026-2026-03-29.json"
    );
    assert!(serialized.json.contains("Spring Market 2026"));
    assert!(!serialized.json.contains("Autumn Fair 2026"));
}

#[wasm_bindgen_test]
async fn test_export_booth_returns_not_found_for_missing_booth() {
    let db = Arc::new(create_test_db().await);
    let booth_repo: Arc<dyn BoothRepository> = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo: Arc<dyn VendorRepository> =
        Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo: Arc<dyn PurchaseRepository> =
        Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

    let service =
        ExportService::with_app_version(booth_repo, vendor_repo, purchase_repo, "test-version");

    let missing_booth_id = booth_a_like_id();
    let error = service.export_booth(&missing_booth_id).await.unwrap_err();

    match error {
        ExportError::BoothNotFound(found_id) => assert_eq!(found_id, missing_booth_id),
        other => panic!("Expected BoothNotFound error, got {other:?}"),
    }
}

fn booth_a_like_id() -> domain::BoothId {
    domain::BoothId::new()
}
