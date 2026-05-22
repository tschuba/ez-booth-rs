use leptos::*;

use crate::t;

const STORAGE_WARNING_EXPANDED_KEY: &str = "ez-booth-storage-warning-expanded";

#[component]
pub fn StorageWarningInfo(
    #[prop(optional)] class: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let expanded = create_rw_signal(true);

    // Initial load from localStorage
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(value)) = storage.get_item(STORAGE_WARNING_EXPANDED_KEY) {
                    expanded.set(value == "true");
                }
            }
        }
    });

    let toggle_expanded = move |_| {
        expanded.update(|e| {
            *e = !*e;
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(STORAGE_WARNING_EXPANDED_KEY, &e.to_string());
                }
            }
        });
    };

    let class = if class.is_empty() {
        "".to_string()
    } else {
        format!(" {class}")
    };

    let actions = store_value(children.map(|children| children().into_view()));

    view! {
        <div class=format!(
            "rounded-xl border border-amber-200 bg-gradient-to-r from-amber-50 via-orange-50 to-amber-100 overflow-hidden shadow-sm{}",
            class
        )>
            <Show
                when=move || expanded.get()
                fallback=move || {
                    view! {
                        <button
                            on:click=toggle_expanded
                            class="flex w-full items-center justify-between px-4 py-2 text-left transition-colors hover:bg-amber-100/30"
                        >
                            <div class="flex items-center gap-3">
                                <span class="relative flex h-2 w-2 items-center justify-center">
                                    <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-amber-400 opacity-75"></span>
                                    <span class="relative inline-flex h-2 w-2 rounded-full bg-amber-500"></span>
                                </span>
                                <p class="text-[13px] font-bold uppercase tracking-wider text-amber-800">
                                    {t!("backup.storage_warning_label")}
                                </p>
                            </div>
                            <div class="flex items-center gap-2 text-[11px] font-bold uppercase tracking-widest text-amber-700">
                                <span>{t!("backup.storage_warning_expand")}</span>
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    class="h-3.5 w-3.5"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2.5"
                                        d="M19 9l-7 7-7-7"
                                    ></path>
                                </svg>
                            </div>
                        </button>
                    }
                }
            >
                <div class="relative p-5 transition-all duration-300 ease-in-out">
                    <div class="space-y-4">
                        <div class="space-y-2 text-amber-950">
                            <p class="text-sm font-bold uppercase tracking-wider text-amber-800">
                                {t!("backup.storage_warning_label")}
                            </p>
                            <p class="text-[15px] leading-relaxed">
                                {t!("backup.storage_warning_message")}
                            </p>
                            <p class="text-sm text-amber-900/80 italic">
                                {t!("backup.storage_warning_recommendation")}
                            </p>
                        </div>

                        <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                            <Show when=move || actions.with_value(|actions| actions.is_some())>
                                <div class="flex flex-wrap items-center gap-3">
                                    {actions.with_value(|actions| actions.clone())}
                                </div>
                            </Show>

                            <button
                                on:click=toggle_expanded
                                class="inline-flex items-center gap-2 self-end rounded-lg bg-amber-200/50 px-4 py-2 text-xs font-bold uppercase tracking-widest text-amber-800 transition-colors hover:bg-amber-200/80"
                            >
                                <span>{t!("backup.storage_warning_collapse")}</span>
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    class="h-3.5 w-3.5"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="3"
                                        d="M5 15l7-7 7 7"
                                    ></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn StorageIndicator() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-2 text-center sm:flex-row sm:justify-center sm:text-left">
            <div class="inline-flex items-center gap-2 rounded-full border border-sky-200 bg-sky-50 px-3 py-1 text-xs font-medium uppercase tracking-wide text-sky-800">
                <span class="h-2 w-2 rounded-full bg-sky-500"></span>
                <span>{t!("backup.storage_indicator_label")}</span>
            </div>
            <p class="text-sm text-gray-600">
                {t!("backup.storage_indicator_message")}
                <a href=format!("{}/booths", crate::base_path()) class="ml-1 font-medium text-blue-600 hover:text-blue-700">
                    {t!("backup.storage_indicator_link")}
                </a>
            </p>
        </div>
    }
}
