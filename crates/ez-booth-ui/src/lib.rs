use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod booth_ordering;
mod components;
mod error;
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
                    // Header (hidden during print)
                    <header class="bg-white shadow print:hidden">
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
                                    // Visual separator
                                    <span class="text-gray-300 mx-2">"|"</span>

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
                                        title={move || t!("settings.language")}
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

                    // Main content (remove padding during print)
                    <main class="py-8 print:py-0">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path="/booths" view=BoothListPage/>
                            <Route path="/vendors" view=VendorListPage/>
                            <Route path="/checkout" view=CheckoutPage/>
                        </Routes>
                    </main>

                    // Footer (hidden during print)
                    <footer class="bg-white border-t mt-auto print:hidden">
                        <Container>
                            <div class="py-4 text-center text-sm text-gray-600">
                                {t!("app.copyright")}
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
