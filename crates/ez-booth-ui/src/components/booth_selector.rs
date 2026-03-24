use crate::components::toast::use_toast;
use crate::i18n::{use_locale, Locale};
use crate::selected_booth_context;
use crate::state::use_app_state;
use crate::t;
use chrono::Datelike;
use domain::models::booth::{Booth, BoothStatus};
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn BoothSelector() -> impl IntoView {
    let selected_booth = selected_booth_context::use_selected_booth();
    let booth_list_version = selected_booth_context::use_booth_list_version();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let (is_open, set_is_open) = create_signal(false);
    let app_state = use_app_state();
    let toast = use_toast();
    let locale = use_locale();

    // Load available booths - reloads when booth_list_version changes
    create_effect(move |_| {
        // Track booth_list_version to make this effect reactive to booth changes
        let _ = booth_list_version.get();
        
        let state_result = app_state.get();
        if let Some(Ok(state)) = state_result {
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(loaded_booths) => {
                        web_sys::console::log_1(&format!("BoothSelector: Loaded {} booths", loaded_booths.len()).into());
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

    // Close dropdown when clicking outside
    let dropdown_ref = create_node_ref::<html::Div>();
    
    // Handle Escape key to close dropdown
    create_effect(move |_| {
        if is_open.get() {
            let handle_keydown = move |event: web_sys::KeyboardEvent| {
                if event.key() == "Escape" {
                    set_is_open.set(false);
                }
            };

            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(handle_keydown) as Box<dyn Fn(web_sys::KeyboardEvent)>
            );

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let _ = document.add_event_listener_with_callback(
                        "keydown",
                        closure.as_ref().unchecked_ref(),
                    );
                }
            }

            closure.forget();
        }
    });
    
    // Format date based on locale
    // German: "24. Mär" or "24. März"
    // English: "Mar 24" or "March 24"
    let format_date = move |date: chrono::NaiveDate| -> String {
        match locale.get() {
            Locale::De => {
                // German format: DD. MMM (e.g., "24. Mär")
                let day = date.day();
                let month = match date.month() {
                    1 => "Jan", 2 => "Feb", 3 => "Mär", 4 => "Apr",
                    5 => "Mai", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                    9 => "Sep", 10 => "Okt", 11 => "Nov", 12 => "Dez",
                    _ => "?",
                };
                format!("{}. {}", day, month)
            }
            Locale::En => {
                // English format: MMM DD (e.g., "Mar 24")
                date.format("%b %d").to_string()
            }
        }
    };

    // Get status indicator color
    let status_color = |status: &BoothStatus| -> &'static str {
        match status {
            BoothStatus::Open => "bg-green-500",
            BoothStatus::Closed { .. } => "bg-gray-400",
        }
    };

    view! {
        <div 
            class="relative ml-6"
            node_ref=dropdown_ref
        >
            // Badge Button
            <button
                class={move || {
                    let base = "flex items-center gap-2 px-4 py-2 rounded-full transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2";
                    let variant = if selected_booth.get().is_some() {
                        "bg-blue-50 hover:bg-blue-100 text-blue-900 border border-blue-200"
                    } else {
                        "bg-gray-100 hover:bg-gray-200 text-gray-700 border border-gray-300"
                    };
                    format!("{} {}", base, variant)
                }}
                on:click=move |_| set_is_open.update(|open| *open = !*open)
                aria-expanded=move || is_open.get()
                aria-label={t!("booth.selector_aria_label")()}
            >
                {move || {
                    if let Some(booth) = selected_booth.get() {
                        let date_str = format_date(booth.date);
                        let status_class = status_color(&booth.status);
                        view! {
                            <>
                                // Status indicator dot
                                <span class={format!("w-2 h-2 rounded-full {}", status_class)} aria-hidden="true"></span>
                                // Date
                                <span class="text-sm font-medium">{date_str}</span>
                                // Separator
                                <span class="text-gray-400" aria-hidden="true">"•"</span>
                                // Description
                                <span class="text-sm font-semibold max-w-[200px] truncate">{booth.description}</span>
                            </>
                        }.into_view()
                    } else {
                        view! {
                            <>
                                <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                                </svg>
                                <span class="text-sm font-medium">{t!("booth.select_booth_cta")()}</span>
                            </>
                        }.into_view()
                    }
                }}
                // Dropdown chevron icon
                <svg 
                    class={move || format!(
                        "w-4 h-4 transition-transform duration-200 {}",
                        if is_open.get() { "rotate-180" } else { "" }
                    )}
                    fill="none" 
                    stroke="currentColor" 
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </button>

            // Dropdown Menu with backdrop
            <Show when=move || is_open.get()>
                <>
                    // Invisible backdrop to capture outside clicks
                    <div 
                        class="fixed inset-0 z-40"
                        on:click=move |_| set_is_open.set(false)
                    ></div>
                    
                    // Dropdown menu
                    <div class="absolute right-0 mt-2 w-80 bg-white rounded-lg shadow-xl border border-gray-200 py-2 z-50 max-h-96 overflow-y-auto">
                    {move || {
                        let booth_list = booths.get();
                        if booth_list.is_empty() {
                            view! {
                                <div class="px-4 py-8 text-center text-gray-500">
                                    <svg class="w-12 h-12 mx-auto mb-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                                    </svg>
                                    <p class="text-sm font-medium">{t!("booth.no_booths_found")()}</p>
                                    <p class="text-xs mt-1">{t!("booth.create_first_booth")()}</p>
                                </div>
                            }.into_view()
                        } else {
                            booth_list.into_iter().map(|booth| {
                                let booth_clone = booth.clone();
                                let is_selected = selected_booth.get().as_ref().map(|b| b.id == booth.id).unwrap_or(false);
                                let date_str = format_date(booth.date);
                                let status_class = status_color(&booth.status);
                                let status_label = match booth.status {
                                    BoothStatus::Open => t!("booth.status_open")(),
                                    BoothStatus::Closed { .. } => t!("booth.status_closed")(),
                                };
                                
                                view! {
                                    <button
                                        class={move || {
                                            let base = "w-full px-4 py-3 text-left hover:bg-gray-50 transition-colors duration-150 flex items-center gap-3";
                                            if is_selected {
                                                format!("{} bg-blue-50 border-l-4 border-blue-500", base)
                                            } else {
                                                format!("{} border-l-4 border-transparent", base)
                                            }
                                        }}
                                        on:click=move |_| {
                                            selected_booth.set(Some(booth_clone.clone()));
                                            set_is_open.set(false);
                                        }
                                        aria-pressed=is_selected
                                    >
                                        <div class="flex-shrink-0">
                                            <span class={format!("w-3 h-3 rounded-full block {}", status_class)} title={status_label.clone()}></span>
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2 mb-1">
                                                <span class="text-sm font-semibold text-gray-900">{booth.description}</span>
                                                {is_selected.then(|| view! {
                                                    <svg class="w-4 h-4 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
                                                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                                    </svg>
                                                })}
                                            </div>
                                            <div class="flex items-center gap-2 text-xs text-gray-500">
                                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                </svg>
                                                <span>{date_str}</span>
                                                <span class="text-gray-400">"•"</span>
                                                <span class={format!(
                                                    "px-2 py-0.5 rounded-full text-xs font-medium {}",
                                                    match booth.status {
                                                        BoothStatus::Open => "bg-green-100 text-green-700",
                                                        BoothStatus::Closed { .. } => "bg-gray-100 text-gray-600",
                                                    }
                                                )}>{status_label}</span>
                                            </div>
                                        </div>
                                    </button>
                                }
                            }).collect_view()
                        }
                    }}
                    </div>
                </>
            </Show>
        </div>
    }
}
