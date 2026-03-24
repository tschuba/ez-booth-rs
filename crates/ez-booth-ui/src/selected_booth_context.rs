use domain::models::booth::Booth;
use domain::models::shared::BoothId;
use leptos::*;
use web_sys::window;
use uuid::Uuid;
use crate::state::use_app_state;

const SELECTED_BOOTH_STORAGE_KEY: &str = "ez-booth-selected-booth-id";

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedBoothContext(pub RwSignal<Option<Booth>>);

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
    booth_signal
}

pub fn use_selected_booth() -> RwSignal<Option<Booth>> {
    use_context::<SelectedBoothContext>()
        .expect("SelectedBoothContext not found. Did you call provide_selected_booth_context() at the root?")
        .0
}

#[component]
pub fn SelectedBoothProvider(children: Children) -> impl IntoView {
    let booth_signal = provide_selected_booth_context();
    let app_state = use_app_state();

    // Restore selected booth from localStorage on mount
    create_effect(move |_| {
        if let Some(stored_booth_id) = load_selected_booth_id() {
            // Wait for app_state to be ready
            if let Some(Ok(state)) = app_state.get() {
                let booth_repository = state.booth_repository.clone();
                spawn_local(async move {
                    // Validate that the booth still exists
                    match booth_repository.find_by_id(&stored_booth_id).await {
                        Ok(Some(booth)) => {
                            booth_signal.set(Some(booth));
                        }
                        Ok(None) => {
                            // Booth was deleted, clear the stored selection
                            save_selected_booth_id(None);
                        }
                        Err(_) => {
                            // Error loading booth, clear the stored selection
                            save_selected_booth_id(None);
                        }
                    }
                });
            }
        }
    });

    // Save selected booth to localStorage whenever it changes
    create_effect(move |_| {
        let booth = booth_signal.get();
        let booth_id_str = booth.as_ref().map(|b| b.id.as_str());
        save_selected_booth_id(booth_id_str.as_deref());
    });

    children()
}
