use crate::components::toast::use_toast;
use crate::selected_booth_context;
use crate::state::use_app_state;
use crate::t;
use domain::models::booth::Booth;
use leptos::*;

#[component]
pub fn BoothSelector() -> impl IntoView {
    let selected_booth = selected_booth_context::use_selected_booth();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let app_state = use_app_state();
    let toast = use_toast();

    // Load available booths
    create_effect(move |_| {
        let state_result = app_state.get();
        if let Some(Ok(state)) = state_result {
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(loaded_booths) => {
                        set_booths.set(loaded_booths);
                    }
                    Err(e) => {
                        let error_msg = t!("booth.errors.load_failed")();
                        toast.error(&error_msg);
                        web_sys::console::error_1(&format!("Failed to load booths: {:?}", e).into());
                    }
                }
            });
        }
    });

    view! {
        <div class="ml-6 flex items-center text-blue-800">
            <label class="font-medium mr-2" for="header-booth-select">
                {t!("booth.selected_label")}
            </label>
            <select
                id="header-booth-select"
                class="min-w-[180px] px-3 py-1 border border-gray-300 rounded-lg focus:outline-none focus:ring focus:ring-blue-300"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if value.is_empty() {
                        selected_booth.set(None);
                    } else {
                        let booth = booths.get().into_iter().find(|b| b.id.as_str() == value);
                        selected_booth.set(booth);
                    }
                }
                prop:value={move || {
                    selected_booth
                        .get()
                        .as_ref()
                        .map(|b| b.id.as_str().to_string())
                        .unwrap_or_default()
                }}
            >
                <option value="">{t!("vendor.no_booth_selected")}</option>
                {move || {
                    booths
                        .get()
                        .into_iter()
                        .map(|booth| {
                            let booth_id = booth.id.as_str().to_string();
                            let is_selected = selected_booth
                                .get()
                                .as_ref()
                                .map(|sel| sel.id.as_str() == booth_id)
                                .unwrap_or(false);
                            view! {
                                <option value={booth_id} selected={is_selected}>
                                    {booth.description}
                                </option>
                            }
                        })
                        .collect_view()
                }}

            </select>
        </div>
    }
}
