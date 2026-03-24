use domain::models::booth::Booth;
use domain::models::shared::BoothId;
use leptos::*;
use web_sys::window;
use uuid::Uuid;
use crate::state::use_app_state;

const SELECTED_BOOTH_STORAGE_KEY: &str = "ez-booth-selected-booth-id";

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedBoothContext(pub RwSignal<Option<Booth>>);

/// Context for triggering booth list reloads
/// Increment this signal to notify all components that the booth list has changed
#[derive(Clone, Copy, Debug)]
pub struct BoothListVersionContext(pub RwSignal<u32>);

/// Get localStorage from the browser window
fn get_local_storage() -> Option<web_sys::Storage> {
    let window = window()?;
    window.local_storage().ok().flatten()
}

/// Load the selected booth ID from localStorage
fn load_selected_booth_id() -> Option<BoothId> {
    let storage = get_local_storage()?;
    let id_str = storage.get_item(SELECTED_BOOTH_STORAGE_KEY).ok()??;
    let uuid = Uuid::parse_str(&id_str).ok()?;
    Some(BoothId::from_uuid(uuid))
}

/// Save the selected booth ID to localStorage
fn save_selected_booth_id(booth_id: Option<&str>) {
    if let Some(storage) = get_local_storage() {
        match booth_id {
            Some(id) => {
                let _ = storage.set_item(SELECTED_BOOTH_STORAGE_KEY, id);
            }
            None => {
                let _ = storage.remove_item(SELECTED_BOOTH_STORAGE_KEY);
            }
        }
    }
}

pub fn provide_selected_booth_context() -> RwSignal<Option<Booth>> {
    let booth_signal = create_rw_signal(None::<Booth>);
    provide_context(SelectedBoothContext(booth_signal));
    
    // Provide booth list version signal for triggering reloads
    let booth_list_version = create_rw_signal(0u32);
    provide_context(BoothListVersionContext(booth_list_version));
    
    booth_signal
}

pub fn use_selected_booth() -> RwSignal<Option<Booth>> {
    use_context::<SelectedBoothContext>()
        .expect("SelectedBoothContext not found. Did you call provide_selected_booth_context() at the root?")
        .0
}

/// Get the booth list version signal to trigger or react to booth list changes
/// Increment this signal when booths are created, updated, or deleted
pub fn use_booth_list_version() -> RwSignal<u32> {
    use_context::<BoothListVersionContext>()
        .expect("BoothListVersionContext not found. Did you call provide_selected_booth_context() at the root?")
        .0
}

#[component]
pub fn SelectedBoothProvider(children: Children) -> impl IntoView {
    let booth_signal = provide_selected_booth_context();

    // Restore selected booth from localStorage on mount
    // Track if we've already attempted restoration
    let restored = create_rw_signal(false);
    
    // Get app_state resource - we need to track it reactively
    // We use a separate effect to wait for AppState context to be available
    create_effect(move |_| {
        // Only try to restore once
        if restored.get() {
            return;
        }
        
        // Try to get app_state from context
        let Some(app_state) = use_context::<Resource<(), Result<crate::state::AppState, String>>>() else {
            web_sys::console::log_1(&"AppState context not available yet...".into());
            return;
        };
        
        // Track app_state resource to make effect reactive
        let Some(Ok(state)) = app_state.get() else {
            web_sys::console::log_1(&"AppState not ready yet...".into());
            return;
        };
        
        web_sys::console::log_1(&"AppState is ready, checking for saved booth...".into());
        
        // Only proceed if we have a stored booth ID
        let Some(stored_booth_id) = load_selected_booth_id() else {
            web_sys::console::log_1(&"No booth ID in localStorage".into());
            restored.set(true); // Mark as done even if no booth to restore
            return;
        };
        
        web_sys::console::log_1(&format!("Attempting to restore booth ID: {}", stored_booth_id.as_str()).into());
        
        // Mark as restored to prevent duplicate attempts
        restored.set(true);
        
        let booth_repository = state.booth_repository.clone();
        spawn_local(async move {
            // Validate that the booth still exists
            match booth_repository.find_by_id(&stored_booth_id).await {
                Ok(Some(booth)) => {
                    web_sys::console::log_1(&format!("Booth restored: {}", booth.description).into());
                    booth_signal.set(Some(booth));
                }
                Ok(None) => {
                    web_sys::console::log_1(&"Booth not found, clearing storage".into());
                    // Booth was deleted, clear the stored selection
                    save_selected_booth_id(None);
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Error loading booth: {:?}, clearing storage", e).into());
                    // Error loading booth, clear the stored selection
                    save_selected_booth_id(None);
                }
            }
        });
    });

    // Save selected booth to localStorage whenever it changes
    // Track if this is the first run to avoid clearing localStorage before restoration
    let is_first_save = create_rw_signal(true);
    
    create_effect(move |_| {
        let booth = booth_signal.get();
        
        // Skip saving on the very first run to allow restoration to happen first
        if is_first_save.get() {
            web_sys::console::log_1(&"Save effect: Initial run, skipping...".into());
            is_first_save.set(false);
            return;
        }
        
        let booth_id_str = booth.as_ref().map(|b| b.id.as_str());
        
        if let Some(id) = booth_id_str.as_deref() {
            web_sys::console::log_1(&format!("Saving booth ID to localStorage: {}", id).into());
        } else {
            web_sys::console::log_1(&"Clearing booth ID from localStorage".into());
        }
        
        save_selected_booth_id(booth_id_str.as_deref());
    });

    children()
}
