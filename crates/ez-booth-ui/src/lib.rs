use leptos::*;
use leptos_meta::*;
use leptos_router::*;

const BASE_PATH: &str = match option_env!("ROUTER_BASE") {
    Some(b) => b,
    None => "",
};

mod audio;
mod booth_ordering;
mod components;
pub mod config;
mod error;
mod error_logging;
mod error_translator;
mod formatting;
mod i18n;
mod pages;
mod selected_booth_context;
mod settings_support;
mod state;
mod utils;
pub use selected_booth_context::SelectedBoothProvider;

use components::*;
use i18n::*;
use pages::*;
use state::*;

#[component]
fn AppViewHeader() -> impl IntoView {
    let location = use_location();

    let title = Signal::derive(move || match location.pathname.get().as_str() {
        "/booths" => Some(t!("booth.list_title")()),
        "/vendors" => Some(t!("vendor.list_title")()),
        "/checkout" => Some(t!("checkout.title")()),
        "/settings" => Some(t!("settings.title")()),
        _ => None,
    });

    view! {
        <Show when=move || title.get().is_some()>
            <div class="bg-white shadow-sm">
                <Container>
                    <div class="flex min-h-16 items-center py-3">
                        <h1 class="text-2xl font-bold text-slate-900">{move || title.get().unwrap_or_default()}</h1>
                    </div>
                </Container>
            </div>
        </Show>
    }
}

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Provide i18n context
    provide_i18n();

    // Provide metadata context
    provide_meta_context();

    // Provide app state (repositories, services)
    let app_state = provide_app_state();
    provide_context(app_state);

    let locale = use_locale();
    let selected_booth = selected_booth_context::use_selected_booth();

    // Update document title when locale changes
    {
        let locale = locale.clone();
        create_effect(move |_| {
            let _ = locale.get(); // Track locale changes
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let title = t!("app.page_title")();
                    document.set_title(&title);
                }
            }
        });
    }

    view! {
        <ToastProvider>
            <Router base=BASE_PATH>
                <div class="min-h-screen bg-gray-50 print:bg-white">
                    <div class="fixed left-0 right-0 top-0 z-40 bg-white print:hidden">
                        // Header (hidden during print)
                        <header>
                            <Container>
                                <div class="flex flex-wrap items-center justify-between gap-3 py-4 md:flex-nowrap">
                                    <A href="/" class="shrink-0 text-2xl font-bold text-blue-600">
                                        {t!("app.title")}
                                    </A>
                                    <div class="hidden md:block">
                                        <BoothSelector />
                                    </div>
                                    <div class="md:hidden">
                                        <Show when=move || selected_booth.get().is_some()>
                                            <BoothSelector />
                                        </Show>
                                    </div>
                                    <nav class="flex flex-wrap items-center gap-x-4 gap-y-2">
                                        <div class="flex items-center space-x-4">
                                            <A href="/booths" class="text-gray-700 transition-colors hover:text-blue-600">
                                                {t!("booth.list_title")}
                                            </A>
                                            <A href="/vendors" class="text-gray-700 transition-colors hover:text-blue-600">
                                                {t!("vendor.list_title")}
                                            </A>
                                            <A href="/checkout" class="text-gray-700 transition-colors hover:text-blue-600">
                                                {t!("checkout.title")}
                                            </A>
                                        </div>
                                        <div class="hidden h-6 w-px bg-gray-300 md:block"></div>
                                        <div class="flex items-center space-x-4">
                                            <button
                                                class="flex items-center gap-2 text-sm font-medium text-gray-700 transition-colors hover:text-blue-600"
                                                on:click=move |_| {
                                                    let new_locale = match locale.get() {
                                                        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => Locale::En,
                                                        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => Locale::De,
                                                    };
                                                    locale.set(new_locale);
                                                }
                                                aria-label=move || {
                                                    match locale.get() {
                                                        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => {
                                                            t!("app.language_toggle_to_english")()
                                                        }
                                                        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => {
                                                            t!("app.language_toggle_to_german")()
                                                        }
                                                    }
                                                }
                                            >
                                                <span class="text-2xl leading-none">
                                                    {move || match locale.get() {
                                                        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "🇬🇧",
                                                        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "🇩🇪",
                                                    }}
                                                </span>
                                                <span class="text-sm">
                                                    {move || match locale.get() {
                                                        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "EN",
                                                        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "DE",
                                                    }}
                                                </span>
                                            </button>
                                            <A
                                                href="/settings"
                                                class="flex items-center gap-2 text-sm font-medium text-gray-700 transition-colors hover:text-blue-600"
                                            >
                                                <Icon icon=LuSettings class="h-4 w-4" />
                                                <span>{t!("settings.title")}</span>
                                            </A>
                                        </div>
                                    </nav>
                                 </div>
                              </Container>
                         </header>

                         <Show when=move || selected_booth.get().is_none()>
                             <div class="border-b border-amber-200 bg-amber-50 py-3 md:hidden">
                                 <Container>
                                     <BoothSelector />
                                 </Container>
                             </div>
                         </Show>

                         <AppViewHeader />
                     </div>

                    // Main content (remove padding during print)
                    <main class="pb-28 pt-36 print:py-0">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path="/booths" view=BoothListPage/>
                            <Route path="/vendors" view=VendorListPage/>
                            <Route path="/checkout" view=CheckoutPage/>
                            <Route path="/settings" view=SettingsPage/>
                        </Routes>
                    </main>

                    // Footer (hidden during print)
                    <footer class="fixed bottom-0 left-0 right-0 z-20 border-t bg-white/95 backdrop-blur print:hidden">
                        <Container>
                            <div class="flex flex-col gap-2 py-3 text-center text-sm text-gray-600">
                                <StorageIndicator />
                                <div>{t!("app.copyright")}</div>
                            </div>
                        </Container>
                    </footer>
                </div>
            </Router>
        </ToastProvider>
    }
}

// Placeholder component for checkout (legacy - can be removed if not needed)
#[component]
fn CheckoutPlaceholder() -> impl IntoView {
    view! {
        <Container>
            <Card title_view={t!("checkout.title").into_view()}>
                <p class="text-gray-600">{t!("checkout.interface_coming_soon")}</p>
            </Card>
        </Container>
    }
}
