wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use chrono::{Duration, NaiveDate, Utc};
use base64::Engine;
use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{
    detect_payload_format, hash_bytes, BinaryQrChunk, ConflictStrategy, ExportError, ExportScope,
    QrExportService, QrImportService, QrPayloadFormat, MAX_QR_CODES,
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

fn create_boundary_purchase(booth: &Booth, vendor: &Vendor, index: usize) -> Purchase {
    let mut purchase = create_test_purchase(booth, vendor, (index % 6) as i64, (index + 1) as i64);
    purchase.note = Some(format!(
        "Boundary purchase {index:04}: {}",
        "x".repeat(96)
    ));
    purchase
}

async fn seed_booth_data(
    booth_repo: &Arc<dyn BoothRepository>,
    vendor_repo: &Arc<dyn VendorRepository>,
    purchase_repo: &Arc<dyn PurchaseRepository>,
    booth: &Booth,
    vendors: &[Vendor],
    purchases: &[Purchase],
) {
    booth_repo.save(booth).await.unwrap();
    for vendor in vendors {
        vendor_repo.save(vendor).await.unwrap();
    }
    for purchase in purchases {
        purchase_repo.save(purchase).await.unwrap();
    }
}

async fn export_chunk_count_for_fixture(
    booth: &Booth,
    vendors: &[Vendor],
    purchases: &[Purchase],
) -> Result<usize, ExportError> {
    let (booth_repo, vendor_repo, purchase_repo, export_service, _) = build_services().await;
    seed_booth_data(
        &booth_repo,
        &vendor_repo,
        &purchase_repo,
        booth,
        vendors,
        purchases,
    )
    .await;

    export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .map(|export| export.chunks.len())
}

fn build_boundary_fixture(total_purchases: usize) -> (Booth, Vec<Vendor>, Vec<Purchase>) {
    let booth = create_test_booth("QR Boundary Booth");
    let vendors = (0..24)
        .map(|index| create_test_vendor(&booth, index + 1))
        .collect::<Vec<_>>();
    let purchases = (0..total_purchases)
        .map(|index| {
            let vendor = &vendors[index % vendors.len()];
            create_boundary_purchase(&booth, vendor, index)
        })
        .collect::<Vec<_>>();

    (booth, vendors, purchases)
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
    let (source_booth_repo, source_vendor_repo, source_purchase_repo, export_service, _) =
        build_services().await;
    let (target_booth_repo, target_vendor_repo, target_purchase_repo, _, import_service) =
        build_services().await;
    let booth = create_test_booth("QR Roundtrip Booth");
    let vendor_a = create_test_vendor(&booth, 1);
    let vendor_b = create_test_vendor(&booth, 2);
    let purchase_a = create_test_purchase(&booth, &vendor_a, 0, 42);
    let purchase_b = create_test_purchase(&booth, &vendor_b, 2, 21);

    seed_booth_data(
        &source_booth_repo,
        &source_vendor_repo,
        &source_purchase_repo,
        &booth,
        &[vendor_a.clone(), vendor_b.clone()],
        &[purchase_a.clone(), purchase_b.clone()],
    )
    .await;

    let export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .unwrap();

    assert!(!export.chunks.is_empty());
    assert!(export.chunks.len() <= MAX_QR_CODES);
    assert_eq!(export.backup.vendors.len(), 2);
    assert_eq!(export.backup.purchases.len(), 2);

    let payload = ez_booth_storage::export::serialize_chunk_payload(&export.chunks[0]).unwrap();
    assert_eq!(detect_payload_format(&payload).unwrap(), QrPayloadFormat::BinaryV2);

    let packet = payload.chars().map(|ch| ch as u8).collect::<Vec<_>>();
    let decoded_packet = BinaryQrChunk::decode_from_bytes(&packet).unwrap();
    assert_eq!(usize::from(decoded_packet.index), export.chunks[0].i);
    assert_eq!(decoded_packet.data, export.chunks[0].d);

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

    assert!(target_booth_repo.find_by_id(&booth.id).await.unwrap().is_some());
    assert!(target_vendor_repo
        .find_by_id(&booth.id, &vendor_a.vendor_id)
        .await
        .unwrap()
        .is_some());
    assert!(target_purchase_repo
        .find_by_id(&purchase_a.id)
        .await
        .unwrap()
        .is_some());
}

#[wasm_bindgen_test]
async fn qr_import_accepts_legacy_json_chunks() {
    let (_, _, _, _, import_service) = build_services().await;
    let compressed = ez_booth_storage::export::serialize_and_compress_backup(&{
        let booth = create_test_booth("Legacy QR Booth");
        let vendor = create_test_vendor(&booth, 1);
        let purchase = create_test_purchase(&booth, &vendor, 0, 42);
        let mut backup = ez_booth_storage::export::BoothBackupData::new(booth, "legacy-test");
        backup.vendors = vec![vendor];
        backup.purchases = vec![purchase];
        backup
    })
    .unwrap();

    let hash_hex = hash_bytes(&compressed)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let legacy_payload = format!(
        r#"{{"v":1,"i":0,"t":1,"h":"{hash_hex}","d":"{}"}}"#,
        base64::engine::general_purpose::STANDARD.encode(&compressed)
    );

    assert_eq!(
        detect_payload_format(&legacy_payload).unwrap(),
        QrPayloadFormat::JsonV1
    );

    let chunk = import_service.parse_chunk_payload(&legacy_payload).unwrap();
    let backup = import_service.collect_backup(vec![chunk]).unwrap();
    assert_eq!(backup.booth.description, "Legacy QR Booth");
    assert_eq!(backup.vendors.len(), 1);
    assert_eq!(backup.purchases.len(), 1);
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

#[wasm_bindgen_test]
async fn qr_import_skip_reports_conflicts_for_existing_records() {
    let (booth_repo, vendor_repo, purchase_repo, export_service, import_service) =
        build_services().await;
    let booth = create_test_booth("Existing Records Booth");
    let vendor_a = create_test_vendor(&booth, 1);
    let vendor_b = create_test_vendor(&booth, 2);
    let purchase_a = create_test_purchase(&booth, &vendor_a, 0, 42);
    let purchase_b = create_test_purchase(&booth, &vendor_b, 2, 21);

    seed_booth_data(
        &booth_repo,
        &vendor_repo,
        &purchase_repo,
        &booth,
        &[vendor_a.clone(), vendor_b.clone()],
        &[purchase_a.clone(), purchase_b.clone()],
    )
    .await;

    let export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .unwrap();

    let summary = import_service
        .import_chunks(export.chunks, ConflictStrategy::Skip)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 0);
    assert_eq!(summary.vendors_imported, 0);
    assert_eq!(summary.purchases_imported, 0);
    assert_eq!(summary.conflicts_resolved, 0);
    assert_eq!(summary.skipped_records.len(), 5);
}

#[wasm_bindgen_test]
async fn qr_import_collects_multichunk_payload_out_of_order_with_duplicates() {
    let (booth_repo, vendor_repo, purchase_repo, export_service, import_service) =
        build_services().await;
    let booth = create_test_booth("Large QR Booth");
    let vendors = (0..40)
        .map(|index| create_test_vendor(&booth, index + 1))
        .collect::<Vec<_>>();
    let purchases = (0..220)
        .map(|index| {
            let vendor = &vendors[index % vendors.len()];
            create_test_purchase(&booth, vendor, (index % 6) as i64, (index + 1) as i64)
        })
        .collect::<Vec<_>>();

    seed_booth_data(
        &booth_repo,
        &vendor_repo,
        &purchase_repo,
        &booth,
        &vendors,
        &purchases,
    )
    .await;

    let export = export_service
        .export_booth_as_qr(&booth.id, ExportScope::Full)
        .await
        .unwrap();

    assert!(export.chunks.len() > 1);

    let mut import_chunks = export.chunks.clone();
    import_chunks.reverse();
    import_chunks.push(export.chunks[0].clone());

    let backup = import_service.collect_backup(import_chunks).unwrap();
    assert_eq!(backup.booth.id, booth.id);
    assert_eq!(backup.vendors.len(), vendors.len());
    assert_eq!(backup.purchases.len(), purchases.len());
}

#[wasm_bindgen_test]
async fn qr_export_accepts_boundary_size_before_exceeding_max_qr_codes() {
    let (booth, vendors, purchases) = build_boundary_fixture(4_096);
    let mut low = 1_usize;
    let mut high = 32_usize;
    let mut max_success_chunk_count =
        export_chunk_count_for_fixture(&booth, &vendors, &purchases[..low]).await.unwrap();

    loop {
        match export_chunk_count_for_fixture(&booth, &vendors, &purchases[..high]).await {
            Ok(chunk_count) => {
                low = high;
                max_success_chunk_count = chunk_count;
                high *= 2;
                assert!(high <= 4_096, "expected to exceed max QR codes before 4096 purchases");
            }
            Err(ExportError::TooManyQrCodes { .. }) => break,
            Err(other) => panic!("unexpected export failure: {other:?}"),
        }
    }

    let mut max_success = low;
    let mut left = low + 1;
    let mut right = high;

    while left < right {
        let mid = left + (right - left) / 2;
        match export_chunk_count_for_fixture(&booth, &vendors, &purchases[..mid]).await {
            Ok(chunk_count) => {
                max_success = mid;
                max_success_chunk_count = chunk_count;
                left = mid + 1;
            }
            Err(ExportError::TooManyQrCodes { .. }) => {
                right = mid;
            }
            Err(other) => panic!("unexpected export failure: {other:?}"),
        }
    }

    let success_chunks = export_chunk_count_for_fixture(&booth, &vendors, &purchases[..max_success])
        .await
        .unwrap();
    assert_eq!(success_chunks, max_success_chunk_count);
    assert!(success_chunks <= MAX_QR_CODES);

    let failure = export_chunk_count_for_fixture(&booth, &vendors, &purchases[..right])
        .await
        .unwrap_err();
    match failure {
        ExportError::TooManyQrCodes { required, maximum } => {
            assert_eq!(required, MAX_QR_CODES + 1);
            assert_eq!(maximum, MAX_QR_CODES);
        }
        other => panic!("expected TooManyQrCodes, got {other:?}"),
    }
}

#[wasm_bindgen_test]
async fn qr_import_replace_overwrites_existing_records_from_chunks() {
    let (source_booth_repo, source_vendor_repo, source_purchase_repo, export_service, _) =
        build_services().await;
    let (target_booth_repo, target_vendor_repo, target_purchase_repo, _, import_service) =
        build_services().await;

    let source_booth = create_test_booth("Replacement Booth");
    let source_vendor = create_test_vendor(&source_booth, 12).with_name("Imported Vendor".to_string());
    let mut source_purchase = create_test_purchase(&source_booth, &source_vendor, 0, 42);
    source_purchase.note = Some("Imported note".to_string());

    seed_booth_data(
        &source_booth_repo,
        &source_vendor_repo,
        &source_purchase_repo,
        &source_booth,
        std::slice::from_ref(&source_vendor),
        std::slice::from_ref(&source_purchase),
    )
    .await;

    let export = export_service
        .export_booth_as_qr(&source_booth.id, ExportScope::Full)
        .await
        .unwrap();

    let mut existing_booth = source_booth.clone();
    existing_booth.description = "Existing Description".to_string();
    existing_booth.updated_at = Utc::now() - Duration::days(2);

    let existing_vendor = Vendor::new(source_vendor.vendor_id.clone(), source_booth.id)
        .with_name("Existing Vendor".to_string());

    let mut existing_purchase = source_purchase.clone();
    existing_purchase.note = Some("Existing note".to_string());
    existing_purchase.timestamp = Utc::now() - Duration::days(2);

    seed_booth_data(
        &target_booth_repo,
        &target_vendor_repo,
        &target_purchase_repo,
        &existing_booth,
        std::slice::from_ref(&existing_vendor),
        std::slice::from_ref(&existing_purchase),
    )
    .await;

    let summary = import_service
        .import_chunks(export.chunks, ConflictStrategy::Replace)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.conflicts_resolved, 3);

    let saved_booth = target_booth_repo
        .find_by_id(&source_booth.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_booth.description, source_booth.description);

    let saved_vendor = target_vendor_repo
        .find_by_id(&source_booth.id, &source_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Imported Vendor"));

    let saved_purchase = target_purchase_repo
        .find_by_id(&source_purchase.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_purchase.note.as_deref(), Some("Imported note"));
}

#[wasm_bindgen_test]
async fn qr_import_merge_prefers_newer_records_from_chunks() {
    let (source_booth_repo, source_vendor_repo, source_purchase_repo, export_service, _) =
        build_services().await;
    let (target_booth_repo, target_vendor_repo, target_purchase_repo, _, import_service) =
        build_services().await;

    let mut source_booth = create_test_booth("Merge Booth");
    source_booth.description = "Merged Description".to_string();
    source_booth.updated_at = Utc::now();

    let source_vendor = Vendor::new(VendorId::new("77".to_string()), source_booth.id)
        .with_name("Imported Vendor".to_string());

    let mut source_purchase = create_test_purchase(&source_booth, &source_vendor, 0, 64);
    source_purchase.note = Some("Merged note".to_string());
    source_purchase.timestamp = Utc::now();

    seed_booth_data(
        &source_booth_repo,
        &source_vendor_repo,
        &source_purchase_repo,
        &source_booth,
        std::slice::from_ref(&source_vendor),
        std::slice::from_ref(&source_purchase),
    )
    .await;

    let export = export_service
        .export_booth_as_qr(&source_booth.id, ExportScope::Full)
        .await
        .unwrap();

    let mut existing_booth = source_booth.clone();
    existing_booth.description = "Existing Description".to_string();
    existing_booth.updated_at = Utc::now() - Duration::days(1);

    let existing_vendor = Vendor::new(source_vendor.vendor_id.clone(), source_booth.id);

    let mut existing_purchase = source_purchase.clone();
    existing_purchase.note = Some("Existing note".to_string());
    existing_purchase.timestamp = Utc::now() - Duration::days(1);

    seed_booth_data(
        &target_booth_repo,
        &target_vendor_repo,
        &target_purchase_repo,
        &existing_booth,
        std::slice::from_ref(&existing_vendor),
        std::slice::from_ref(&existing_purchase),
    )
    .await;

    let summary = import_service
        .import_chunks(export.chunks, ConflictStrategy::Merge)
        .await
        .unwrap();

    assert_eq!(summary.booths_imported, 1);
    assert_eq!(summary.vendors_imported, 1);
    assert_eq!(summary.purchases_imported, 1);
    assert_eq!(summary.conflicts_resolved, 3);

    let saved_booth = target_booth_repo
        .find_by_id(&source_booth.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_booth.description, "Merged Description");

    let saved_vendor = target_vendor_repo
        .find_by_id(&source_booth.id, &source_vendor.vendor_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_vendor.name.as_deref(), Some("Imported Vendor"));

    let saved_purchase = target_purchase_repo
        .find_by_id(&source_purchase.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_purchase.note.as_deref(), Some("Merged note"));
}
