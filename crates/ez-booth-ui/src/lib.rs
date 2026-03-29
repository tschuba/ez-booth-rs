use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod audio;
mod booth_ordering;
mod components;
mod error;
mod error_translator;
mod formatting;
mod i18n;
mod pages;
mod selected_booth_context;
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
            <Router>
                <div class="min-h-screen bg-gray-50 print:bg-white">
                    <div class="fixed left-0 right-0 top-0 z-40 bg-white print:hidden">
                        // Header (hidden during print)
                        <header>
                            <StorageWarningBanner />
                            <Container>
                                <div class="flex items-center justify-between py-4">
                                    <a href="/" class="text-2xl font-bold text-blue-600">
                                        {t!("app.title")}
                                    </a>
                                    <BoothSelector />
                                    <nav class="flex items-center space-x-4">
                                        <a href="/booths" class="text-gray-700 hover:text-blue-600">
                                            {t!("booth.list_title")}
                                        </a>
                                        <a href="/vendors" class="text-gray-700 hover:text-blue-600">
                                            {t!("vendor.list_title")}
                                        </a>
                                         <a href="/checkout" class="text-gray-700 hover:text-blue-600">
                                             {t!("checkout.title")}
                                         </a>
                                         // Language switcher with globe icon
                                         <button
                                             class="flex items-center gap-1.5 text-gray-700 hover:text-blue-600 text-sm"
                                             on:click=move |_| {
                                                 let new_locale = match locale.get() {
                                                     Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => Locale::En,
                                                     Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => Locale::De,
                                                 };
                                                 locale.set(new_locale);
                                             }
                                         >
                                            // Globe SVG icon
                                            <svg
                                                class="w-4 h-4"
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                                xmlns="http://www.w3.org/2000/svg"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
                                                />
                                            </svg>

                                            // Language code
                                            <span>
                                                {move || match locale.get() {
                                                    Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "EN",
                                                    Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "DE",
                                                }}
                                            </span>
                                        </button>
                                    </nav>
                                </div>
                            </Container>
                        </header>

                        <AppViewHeader />
                    </div>

                    // Main content (remove padding during print)
                    <main class="pb-28 pt-36 print:py-0">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path="/booths" view=BoothListPage/>
                            <Route path="/vendors" view=VendorListPage/>
                            <Route path="/checkout" view=CheckoutPage/>
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
