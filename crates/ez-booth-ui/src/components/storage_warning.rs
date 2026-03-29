use leptos::*;
use web_sys::window;

use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::t;

const STORAGE_WARNING_DISMISSED_KEY: &str = "ez-booth-storage-warning-dismissed";

fn read_warning_dismissed() -> bool {
    window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| {
            storage
                .get_item(STORAGE_WARNING_DISMISSED_KEY)
                .ok()
                .flatten()
        })
        .as_deref()
        == Some("true")
}

fn write_warning_dismissed(dismissed: bool) {
    if let Some(storage) = window().and_then(|window| window.local_storage().ok().flatten()) {
        let _ = if dismissed {
            storage.set_item(STORAGE_WARNING_DISMISSED_KEY, "true")
        } else {
            storage.remove_item(STORAGE_WARNING_DISMISSED_KEY)
        };
    }
}

#[component]
pub fn StorageWarningBanner() -> impl IntoView {
    let (dismissed, set_dismissed) = create_signal(read_warning_dismissed());

    let dismiss_banner = move || {
        write_warning_dismissed(true);
        set_dismissed.set(true);
    };

    view! {
        <Show when=move || !dismissed.get()>
            <div class="border-b border-amber-200 bg-gradient-to-r from-amber-50 via-orange-50 to-amber-100 print:hidden">
                <div class="mx-auto max-w-7xl px-4 py-3 sm:px-6 lg:px-8">
                    <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                        <div class="space-y-1 text-sm text-amber-950">
                            <p class="font-semibold uppercase tracking-wide text-amber-800">
                                {t!("backup.storage_warning_label")}
                            </p>
                            <p>{t!("backup.storage_warning_message")}</p>
                            <p class="text-amber-900/80">{t!("backup.storage_warning_recommendation")}</p>
                        </div>

                        <div class="flex flex-wrap items-center gap-2">
                            <a
                                href="/settings"
                                class="inline-flex items-center justify-center rounded-lg bg-amber-900 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-amber-950"
                            >
                                {t!("backup.storage_warning_cta")}
                            </a>
                            <Button
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Small
                                class="border border-amber-300 bg-white/70 text-amber-900 hover:bg-white".to_string()
                                on_click=Box::new(dismiss_banner)
                            >
                                {t!("backup.storage_warning_dismiss")}
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn StorageWarningInfo(
    #[prop(optional)] class: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = if class.is_empty() {
        "".to_string()
    } else {
        format!(" {class}")
    };

    let actions = store_value(children.map(|children| children().into_view()));

    view! {
        <div class=format!(
            "rounded-xl border border-amber-200 bg-gradient-to-r from-amber-50 via-orange-50 to-amber-100 px-4 py-4{}",
            class
        )>
            <div class="space-y-4">
                <div class="space-y-1 text-sm text-amber-950">
                    <p class="font-semibold uppercase tracking-wide text-amber-800">
                        {t!("backup.storage_warning_label")}
                    </p>
                    <p>{t!("backup.storage_warning_message")}</p>
                    <p class="text-amber-900/80">{t!("backup.storage_warning_recommendation")}</p>
                </div>

                <Show when=move || actions.with_value(|actions| actions.is_some())>
                    <div class="flex flex-wrap items-center gap-3">
                        {actions.with_value(|actions| actions.clone())}
                    </div>
                </Show>
            </div>
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
                <a href="/settings" class="ml-1 font-medium text-blue-600 hover:text-blue-700">
                    {t!("backup.storage_indicator_link")}
                </a>
            </p>
        </div>
    }
}
