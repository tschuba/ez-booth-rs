use domain::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use domain::services::VendorService;
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
    pub vendor_repository: Arc<dyn VendorRepository>,
    pub purchase_repository: Arc<dyn PurchaseRepository>,
    pub vendor_service: Arc<VendorService<IndexedDbVendorRepository>>,
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
        let purchase_repository: Arc<dyn PurchaseRepository> =
            Arc::new(IndexedDbPurchaseRepository::new(db.clone()));

        // Create services (use separate instance for service layer)
        let vendor_service = Arc::new(VendorService::new(IndexedDbVendorRepository::new(db.clone())));

        Ok(Self {
            booth_repository,
            vendor_repository,
            purchase_repository,
            vendor_service,
        })
    }
}

/// Provide app state to the component tree
pub fn provide_app_state() -> Resource<(), Result<AppState, String>> {
    create_local_resource(|| (), |_| async { AppState::new().await })
}

/// Use app state from context
pub fn use_app_state() -> Resource<(), Result<AppState, String>> {
    use_context::<Resource<(), Result<AppState, String>>>()
        .expect("AppState context not found. Make sure provide_app_state() is called in a parent component.")
}
