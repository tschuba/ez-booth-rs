use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod error;
mod i18n;
mod pages;
mod state;
mod selected_booth_context;
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
                                    <a href="/reports" class="text-gray-700 hover:text-blue-600">
                                        {t!("report.title")}
                                    </a>
                                    <button
                                        class="text-gray-700 hover:text-blue-600 text-sm"
                                        on:click=move |_| {
                                            let new_locale = match locale.get() {
                                                Locale::De => Locale::En,
                                                Locale::En => Locale::De,
                                            };
                                            locale.set(new_locale);
                                        }
                                    >
                                        {move || match locale.get() {
                                            Locale::De => "EN",
                                            Locale::En => "DE",
                                        }}
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
                            <Route path="/reports" view=ReportsPage/>
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
