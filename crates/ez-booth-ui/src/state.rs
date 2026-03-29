use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::services::{BoothService, ReportService, VendorService};
use ez_booth_storage::export::{ExportService, ImportService};
use ez_booth_storage::indexeddb::Database;
use ez_booth_storage::repositories::{
    IndexedDbBoothRepository, IndexedDbPurchaseRepository, IndexedDbVendorRepository,
};
use leptos::*;
use std::sync::Arc;

/// Application state containing repositories and services
#[derive(Clone)]
pub struct AppState {
    pub booth_repository: Arc<dyn BoothRepository>,
    pub booth_service: Arc<BoothService<IndexedDbBoothRepository>>,
    pub vendor_repository: Arc<dyn VendorRepository>,
    pub purchase_repository: Arc<dyn PurchaseRepository>,
    pub indexed_purchase_repository: Arc<IndexedDbPurchaseRepository>,
    pub export_service: Arc<ExportService>,
    pub import_service: Arc<ImportService>,
    pub vendor_service: Arc<VendorService<IndexedDbVendorRepository, IndexedDbBoothRepository>>,
    pub report_service: Arc<
        ReportService<
            IndexedDbPurchaseRepository,
            IndexedDbBoothRepository,
            IndexedDbVendorRepository,
        >,
    >,
}

impl AppState {
    /// Initialize application state with database connection
    pub async fn new() -> Result<Self, String> {
        // Initialize IndexedDB
        let db = Database::new()
            .await
            .map_err(|e| format!("Failed to initialize database: {:?}", e))?;

        let db = Arc::new(db);

        // Create repositories
        let booth_repository: Arc<dyn BoothRepository> =
            Arc::new(IndexedDbBoothRepository::new(db.clone()));
        let vendor_repository: Arc<dyn VendorRepository> =
            Arc::new(IndexedDbVendorRepository::new(db.clone()));
        let indexed_purchase_repository = Arc::new(IndexedDbPurchaseRepository::new(db.clone()));
        let purchase_repository: Arc<dyn PurchaseRepository> = indexed_purchase_repository.clone();
        let export_service = Arc::new(ExportService::new(
            booth_repository.clone(),
            vendor_repository.clone(),
            purchase_repository.clone(),
        ));
        let import_service = Arc::new(ImportService::new(
            booth_repository.clone(),
            vendor_repository.clone(),
            purchase_repository.clone(),
        ));

        // Create services (use separate instances for service layer)
        let vendor_service = Arc::new(VendorService::new(
            IndexedDbVendorRepository::new(db.clone()),
            IndexedDbBoothRepository::new(db.clone()),
        ));
        let booth_service = Arc::new(BoothService::new(IndexedDbBoothRepository::new(db.clone())));
        let report_service = Arc::new(ReportService::new(
            IndexedDbPurchaseRepository::new(db.clone()),
            IndexedDbBoothRepository::new(db.clone()),
            IndexedDbVendorRepository::new(db.clone()),
        ));

        Ok(Self {
            booth_repository,
            booth_service,
            vendor_repository,
            purchase_repository,
            indexed_purchase_repository,
            export_service,
            import_service,
            vendor_service,
            report_service,
        })
    }
}

/// Provide app state to the component tree
pub fn provide_app_state() -> Resource<(), Result<AppState, String>> {
    create_local_resource(|| (), |_| async { AppState::new().await })
}

/// Use app state from context
pub fn use_app_state() -> Resource<(), Result<AppState, String>> {
    if let Some(app_state) = use_context::<Resource<(), Result<AppState, String>>>() {
        app_state
    } else {
        web_sys::console::warn_1(
            &"AppState context not found. Returning fallback error resource.".into(),
        );
        create_local_resource(
            || (),
            |_| async {
                Err(
                    "AppState context not found. Make sure provide_app_state() is called in a parent component.".to_string(),
                )
            },
        )
    }
}
