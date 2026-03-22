use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod error;
mod i18n;
mod pages;

use components::*;
use i18n::*;
use pages::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Provide i18n context
    provide_i18n();

    // Provide metadata context
    provide_meta_context();

    let locale = use_locale();

    view! {
        <Router>
            <div class="min-h-screen bg-gray-50">
                // Header
                <header class="bg-white shadow">
                    <Container>
                        <div class="flex items-center justify-between py-4">
                            <a href="/" class="text-2xl font-bold text-blue-600">
                                {t!("app.title")}
                            </a>
                            <nav class="flex items-center space-x-4">
                                <a href="/booths" class="text-gray-700 hover:text-blue-600">
                                    {t!("booth.list_title")}
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

                // Main content
                <main class="py-8">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/booths" view=BoothListPlaceholder/>
                        <Route path="/checkout" view=CheckoutPlaceholder/>
                        <Route path="/reports" view=ReportsPlaceholder/>
                    </Routes>
                </main>

                // Footer
                <footer class="bg-white border-t mt-auto">
                    <Container>
                        <div class="py-4 text-center text-sm text-gray-600">
                            "EZ Booth © 2026"
                        </div>
                    </Container>
                </footer>
            </div>
        </Router>
    }
}

// Placeholder components for routes not yet implemented
#[component]
fn BoothListPlaceholder() -> impl IntoView {
    view! {
        <Container>
            <Card title="Booths">
                <p class="text-gray-600">"Booth management coming soon..."</p>
            </Card>
        </Container>
    }
}

#[component]
fn CheckoutPlaceholder() -> impl IntoView {
    view! {
        <Container>
            <Card title="Checkout">
                <p class="text-gray-600">"Checkout interface coming soon..."</p>
            </Card>
        </Container>
    }
}

#[component]
fn ReportsPlaceholder() -> impl IntoView {
    view! {
        <Container>
            <Card title="Reports">
                <p class="text-gray-600">"Reports interface coming soon..."</p>
            </Card>
        </Container>
    }
}
