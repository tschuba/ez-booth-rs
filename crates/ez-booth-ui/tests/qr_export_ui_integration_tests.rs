wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use async_trait::async_trait;
use chrono::NaiveDate;
use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::{Booth, BoothId, FeeConfig, Purchase, PurchaseId, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{ExportService, ImportService, QrExportService};
use ez_booth_storage::indexeddb::Database;
use ez_booth_storage::repositories::{
    IndexedDbBoothRepository, IndexedDbPurchaseRepository, IndexedDbVendorRepository,
};
use leptos::*;
use rust_decimal_macros::dec;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

struct DelayedBoothRepository {
    inner: Arc<IndexedDbBoothRepository>,
    delay_ms: i32,
}

#[async_trait(?Send)]
impl BoothRepository for DelayedBoothRepository {
    async fn save(&self, booth: &Booth) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.save(booth).await
    }

    async fn find_by_id(&self, id: &BoothId) -> domain::error::DomainResult<Option<Booth>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_id(id).await
    }

    async fn find_all(&self) -> domain::error::DomainResult<Vec<Booth>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_all().await
    }

    async fn find_by_description_and_date(
        &self,
        description: &str,
        date: &NaiveDate,
    ) -> domain::error::DomainResult<Option<Booth>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_description_and_date(description, date).await
    }

    async fn delete(&self, id: &BoothId) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.delete(id).await
    }
}

struct DelayedVendorRepository {
    inner: Arc<IndexedDbVendorRepository>,
    delay_ms: i32,
}

#[async_trait(?Send)]
impl VendorRepository for DelayedVendorRepository {
    async fn save(&self, vendor: &Vendor) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.save(vendor).await
    }

    async fn find_by_id(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
    ) -> domain::error::DomainResult<Option<Vendor>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_id(booth_id, vendor_id).await
    }

    async fn find_by_booth(&self, booth_id: &BoothId) -> domain::error::DomainResult<Vec<Vendor>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_booth(booth_id).await
    }

    async fn find_all(&self) -> domain::error::DomainResult<Vec<Vendor>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_all().await
    }

    async fn delete(&self, booth_id: &BoothId, vendor_id: &VendorId) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.delete(booth_id, vendor_id).await
    }

    async fn delete_from_booth(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
    ) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.delete_from_booth(booth_id, vendor_id).await
    }
}

struct DelayedPurchaseRepository {
    inner: Arc<IndexedDbPurchaseRepository>,
    delay_ms: i32,
}

#[async_trait(?Send)]
impl PurchaseRepository for DelayedPurchaseRepository {
    async fn save(&self, purchase: &Purchase) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.save(purchase).await
    }

    async fn find_by_id(
        &self,
        id: &PurchaseId,
    ) -> domain::error::DomainResult<Option<Purchase>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_id(id).await
    }

    async fn find_by_booth(
        &self,
        booth_id: &BoothId,
    ) -> domain::error::DomainResult<Vec<Purchase>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_booth(booth_id).await
    }

    async fn find_by_booth_paginated(
        &self,
        booth_id: &BoothId,
        offset: usize,
        limit: usize,
    ) -> domain::error::DomainResult<domain::repositories::PaginatedPurchases> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_booth_paginated(booth_id, offset, limit).await
    }

    async fn get_running_totals(
        &self,
        booth_id: &BoothId,
    ) -> domain::error::DomainResult<domain::repositories::BoothRunningTotals> {
        sleep_ms(self.delay_ms).await;
        self.inner.get_running_totals(booth_id).await
    }

    async fn find_by_vendor(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
    ) -> domain::error::DomainResult<Vec<Purchase>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_by_vendor(booth_id, vendor_id).await
    }

    async fn find_all(&self) -> domain::error::DomainResult<Vec<Purchase>> {
        sleep_ms(self.delay_ms).await;
        self.inner.find_all().await
    }

    async fn delete(&self, id: &PurchaseId) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.delete(id).await
    }

    async fn delete_from_booth(
        &self,
        booth_id: &BoothId,
        id: &PurchaseId,
    ) -> domain::error::DomainResult<()> {
        sleep_ms(self.delay_ms).await;
        self.inner.delete_from_booth(booth_id, id).await
    }
}

async fn sleep_ms(timeout_ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window available");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            resolve.unchecked_ref(),
            timeout_ms,
        );
    });

    JsFuture::from(promise).await.expect("timeout should resolve");
}

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("window available")
        .document()
        .expect("document available")
}

fn click(element: &web_sys::Element) {
    element
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("element should be clickable")
        .click();
}

fn button_is_disabled(selector: &str) -> bool {
    document()
        .query_selector(selector)
        .expect("selector query should succeed")
        .expect("button should exist")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("element should be a button")
        .disabled()
}

async fn wait_for_selector(selector: &str) -> web_sys::Element {
    for _ in 0..80 {
        if let Some(element) = document()
            .query_selector(selector)
            .expect("selector query should succeed")
        {
            return element;
        }

        sleep_ms(50).await;
    }

    panic!("timed out waiting for selector: {selector}");
}

async fn wait_for_selector_to_disappear(selector: &str) {
    for _ in 0..80 {
        if document()
            .query_selector(selector)
            .expect("selector query should succeed")
            .is_none()
        {
            return;
        }

        sleep_ms(50).await;
    }

    panic!("selector still present after timeout: {selector}");
}

async fn wait_for_button_enabled(selector: &str) {
    for _ in 0..80 {
        if !button_is_disabled(selector) {
            return;
        }

        sleep_ms(50).await;
    }

    panic!("button did not become enabled: {selector}");
}

async fn create_test_app_state() -> (ez_booth_ui::AppState, BoothId) {
    let db_name = format!("test_qr_export_ui_{}", js_sys::Math::random());
    let db = Arc::new(
        Database::new_with_name(&db_name)
            .await
            .expect("test database should initialize"),
    );

    let indexed_booth_repository = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let indexed_vendor_repository = Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let indexed_purchase_repository = Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

    let booth = Booth::new(
        "QR Export Test Booth".to_string(),
        NaiveDate::from_ymd_opt(2026, 3, 30).expect("valid date"),
        FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(15.00),
            rounding_step: dec!(0.50),
        },
    )
    .expect("test booth should be valid");
    indexed_booth_repository
        .save(&booth)
        .await
        .expect("test booth should save");

    let vendor = Vendor::new(VendorId::new("101".to_string()), booth.id)
        .with_name("Test Vendor".to_string());
    indexed_vendor_repository
        .save(&vendor)
        .await
        .expect("test vendor should save");

    let purchase = Purchase::new(
        booth.id,
        vec![PurchaseItem::new(dec!(12.50), vendor.vendor_id.clone()).expect("valid purchase item")],
    )
    .expect("test purchase should be valid");
    indexed_purchase_repository
        .save(&purchase)
        .await
        .expect("test purchase should save");

    let booth_repository: Arc<dyn BoothRepository> = Arc::new(DelayedBoothRepository {
        inner: indexed_booth_repository.clone(),
        delay_ms: 200,
    });
    let vendor_repository: Arc<dyn VendorRepository> = Arc::new(DelayedVendorRepository {
        inner: indexed_vendor_repository.clone(),
        delay_ms: 200,
    });
    let purchase_repository: Arc<dyn PurchaseRepository> = Arc::new(DelayedPurchaseRepository {
        inner: indexed_purchase_repository.clone(),
        delay_ms: 200,
    });

    let app_state = ez_booth_ui::AppState {
        booth_repository: booth_repository.clone(),
        booth_service: Arc::new(domain::services::BoothService::new(
            IndexedDbBoothRepository::new(db.clone()),
        )),
        vendor_repository: vendor_repository.clone(),
        purchase_repository: purchase_repository.clone(),
        indexed_purchase_repository: indexed_purchase_repository.clone(),
        export_service: Arc::new(ExportService::new(
            booth_repository.clone(),
            vendor_repository.clone(),
            purchase_repository.clone(),
        )),
        qr_export_service: Arc::new(QrExportService::new(
            booth_repository.clone(),
            vendor_repository.clone(),
            purchase_repository.clone(),
        )),
        import_service: Arc::new(ImportService::new(
            booth_repository,
            vendor_repository,
            purchase_repository,
        )),
        vendor_service: Arc::new(domain::services::VendorService::new(
            IndexedDbVendorRepository::new(db.clone()),
            IndexedDbBoothRepository::new(db.clone()),
        )),
        report_service: Arc::new(domain::services::ReportService::new(
            IndexedDbPurchaseRepository::new(db.clone()),
            IndexedDbBoothRepository::new(db.clone()),
            IndexedDbVendorRepository::new(db.clone()),
        )),
    };

    (app_state, booth.id)
}

#[wasm_bindgen_test]
async fn booth_dropdown_qr_export_opens_and_closes_modal() {
    let document = document();
    let root = document
        .create_element("div")
        .expect("root creation should succeed")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("root should be an html element");
    root.set_class_name("js-qr-export-test-root");
    document
        .body()
        .expect("body available")
        .append_child(&root)
        .expect("should append root");

    let booth_id = BoothId::new();

    leptos::mount_to(root.clone(), move || {
        ez_booth_ui::provide_i18n();
        let app_state: Resource<(), Result<ez_booth_ui::AppState, String>> =
            create_local_resource(|| (), |_| async { Err("test app state".to_string()) });
        provide_context(app_state);

        view! {
            <ez_booth_ui::ToastProvider>
                <ez_booth_ui::DropdownMenu
                    trigger=view! {
                        <ez_booth_ui::Button
                            variant=ez_booth_ui::ButtonVariant::Secondary
                            class="js-open-actions".to_string()
                            title="Actions".to_string()
                            aria_label="Actions".to_string()
                        >
                            "Actions"
                        </ez_booth_ui::Button>
                    }
                >
                    <ez_booth_ui::ExportButton
                        scope=ez_booth_ui::ExportScope::Booth(booth_id)
                        menu_item=true
                    />
                </ez_booth_ui::DropdownMenu>
            </ez_booth_ui::ToastProvider>
        }
    });

    let action_button = wait_for_selector(".js-open-actions").await;
    click(&action_button);

    let qr_export_item = wait_for_selector(".js-qr-export-menu-item").await;
    click(&qr_export_item);

    let modal = wait_for_selector("[role=\"dialog\"]").await;
    let title = wait_for_selector("#modal-title").await;
    assert!(title.text_content().unwrap_or_default().len() > 0);
    assert!(modal.class_name().contains("fixed"));

    let close_button = wait_for_selector("button[aria-label=\"Close modal\"]").await;
    click(&close_button);
    wait_for_selector_to_disappear("[role=\"dialog\"]").await;

    root.remove();
}

#[wasm_bindgen_test]
async fn qr_export_generate_button_enables_after_backup_loads() {
    let document = document();
    let root = document
        .create_element("div")
        .expect("root creation should succeed")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("root should be an html element");
    root.set_class_name("js-qr-export-enabled-test-root");
    document
        .body()
        .expect("body available")
        .append_child(&root)
        .expect("should append root");

    let (app_state, booth_id) = create_test_app_state().await;

    leptos::mount_to(root.clone(), move || {
        ez_booth_ui::provide_i18n();
        let app_state = app_state.clone();
        let app_state: Resource<(), Result<ez_booth_ui::AppState, String>> =
            create_local_resource(|| (), move |_| {
                let app_state = app_state.clone();
                async move { Ok(app_state) }
            });
        provide_context(app_state);

        view! {
            <ez_booth_ui::ToastProvider>
                <ez_booth_ui::DropdownMenu
                    trigger=view! {
                        <ez_booth_ui::Button
                            variant=ez_booth_ui::ButtonVariant::Secondary
                            class="js-open-actions".to_string()
                            title="Actions".to_string()
                            aria_label="Actions".to_string()
                        >
                            "Actions"
                        </ez_booth_ui::Button>
                    }
                >
                    <ez_booth_ui::ExportButton
                        scope=ez_booth_ui::ExportScope::Booth(booth_id)
                        menu_item=true
                    />
                </ez_booth_ui::DropdownMenu>
            </ez_booth_ui::ToastProvider>
        }
    });

    let action_button = wait_for_selector(".js-open-actions").await;
    click(&action_button);

    let qr_export_item = wait_for_selector(".js-qr-export-menu-item").await;
    click(&qr_export_item);

    wait_for_selector(".js-qr-generate-button").await;
    assert!(button_is_disabled(".js-qr-generate-button"));

    wait_for_button_enabled(".js-qr-generate-button").await;
    assert!(!button_is_disabled(".js-qr-generate-button"));

    root.remove();
}
