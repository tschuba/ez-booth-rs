wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use chrono::{Duration, NaiveDate, Utc};
use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{
    ConflictStrategy, ExportScope, QrExportService, QrImportService, MAX_QR_CODES,
};
use ez_booth_storage::indexeddb::Database;
use ez_booth_storage::repositories::{
    IndexedDbBoothRepository, IndexedDbPurchaseRepository, IndexedDbVendorRepository,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use wasm_bindgen_test::*;

async fn create_test_db() -> Database {
    let db_name = format!("test_qr_export_import_db_{}", js_sys::Math::random());
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

fn create_test_vendor(booth: &Booth, index: usize) -> Vendor {
    Vendor::new(VendorId::new(index.to_string()), booth.id).with_name(format!("Vendor {index}"))
}

fn create_test_purchase(booth: &Booth, vendor: &Vendor, days_ago: i64, amount: i64) -> Purchase {
    let mut purchase = Purchase::new(
        booth.id,
        vec![PurchaseItem::new(
            dec!(1.00) * rust_decimal::Decimal::from(amount),
            vendor.vendor_id.clone(),
        )
        .unwrap()],
    )
    .unwrap();
    purchase.timestamp = Utc::now() - Duration::days(days_ago);
    purchase
}

async fn build_services() -> (
    Arc<dyn BoothRepository>,
    Arc<dyn VendorRepository>,
    Arc<dyn PurchaseRepository>,
    QrExportService,
    QrImportService,
) {
    let db = Arc::new(create_test_db().await);
    let booth_repo: Arc<dyn BoothRepository> = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo: Arc<dyn VendorRepository> =
        Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo: Arc<dyn PurchaseRepository> =
        Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

    let export_service = QrExportService::new(
        booth_repo.clone(),
        vendor_repo.clone(),
        purchase_repo.clone(),
    );
    let import_service = QrImportService::new(
        booth_repo.clone(),
        vendor_repo.clone(),
        purchase_repo.clone(),
    );

    (
        booth_repo,
        vendor_repo,
        purchase_repo,
        export_service,
        import_service,
    )
}

#[wasm_bindgen_test]
async fn qr_export_import_roundtrip_for_booth_backup() {
    let (booth_repo, vendor_repo, purchase_repo, export_service, import_service) =
        build_services().await;
    let booth = create_test_booth("QR Roundtrip Booth");
    let vendor_a = create_test_vendor(&booth, 1);
    let vendor_b = create_test_vendor(&booth, 2);
    let purchase_a = create_test_purchase(&booth, &vendor_a, 0, 42);
    let purchase_b = create_test_purchase(&booth, &vendor_b, 2, 21);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor_a).await.unwrap();
    vendor_repo.save(&vendor_b).await.unwrap();
    purchase_repo.save(&purchase_a).await.unwrap();
    purchase_repo.save(&purchase_b).await.unwrap();

    let export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .unwrap();

    assert!(!export.chunks.is_empty());
    assert!(export.chunks.len() <= MAX_QR_CODES);
    assert_eq!(export.backup.vendors.len(), 2);
    assert_eq!(export.backup.purchases.len(), 2);

    let imported_backup = import_service
        .collect_backup(export.chunks.clone())
        .unwrap();
    assert_eq!(imported_backup.booth.id, booth.id);
    assert_eq!(imported_backup.vendors.len(), 2);
    assert_eq!(imported_backup.purchases.len(), 2);

    let summary = import_service
        .import_chunks(export.chunks, ConflictStrategy::Merge)
        .await
        .unwrap();
    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 2);
    assert_eq!(summary.purchases_imported, 2);
}

#[wasm_bindgen_test]
async fn qr_export_scope_filters_old_purchases() {
    let (booth_repo, vendor_repo, purchase_repo, export_service, _) = build_services().await;
    let booth = create_test_booth("Filtered Booth");
    let vendor = create_test_vendor(&booth, 1);
    let recent_purchase = create_test_purchase(&booth, &vendor, 3, 50);
    let old_purchase = create_test_purchase(&booth, &vendor, 14, 25);

    booth_repo.save(&booth).await.unwrap();
    vendor_repo.save(&vendor).await.unwrap();
    purchase_repo.save(&recent_purchase).await.unwrap();
    purchase_repo.save(&old_purchase).await.unwrap();

    let week_export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Week)
        .await
        .unwrap();
    let full_export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .unwrap();

    assert_eq!(week_export.backup.purchases.len(), 1);
    assert_eq!(full_export.backup.purchases.len(), 2);
    assert!(week_export.compressed_bytes.len() < full_export.compressed_bytes.len());
}
