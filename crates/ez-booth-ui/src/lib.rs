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
                <div class="min-h-screen bg-gray-50">
                    // Header
                    <header class="bg-white shadow">
                        <Container>
                            <div class="flex items-center justify-between py-4">
                                <a href="/" class="text-2xl font-bold text-blue-600">
                                    {t!("app.title")}
                                </a>
                                {
    let selected_booth = selected_booth_context::use_selected_booth();
    let (booths, set_booths) = create_signal(Vec::new());
    let app_state = use_app_state();
    let toast = use_toast();
    // Load available booths once at header level
    create_effect(move |_| {
        let state_result = app_state.get();
        if let Some(Ok(state)) = state_result {
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(loaded_booths) => {
                        set_booths.set(loaded_booths);
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to load booths: {:?}", e));
                    }
                }
            });
        }
    });
    view! {
        <div class="ml-6 flex items-center text-blue-800">
            <label class="font-medium mr-2" for="header-booth-select">{t!("booth.selected_label")}</label>
            <select
                id="header-booth-select"
                class="px-3 py-1 border border-gray-300 rounded-lg focus:outline-none focus:ring focus:ring-blue-300"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if value.is_empty() {
                        selected_booth.set(None);
                    } else {
                        let booth = booths.get().into_iter().find(|b| b.id.as_str() == value);
                        selected_booth.set(booth);
                    }
                }
                value={move || selected_booth.get().as_ref().map(|b| b.id.as_str().to_string()).unwrap_or_default()}
                style="min-width: 180px"
            >
                <option value="">{t!("vendor.no_booth_selected")}</option>
                {move || booths.get().into_iter().map(|booth| {
                    let booth_id = booth.id.as_str().to_string();
                    view! {
                        <option value={booth_id.clone()} selected={
                            match selected_booth.get().as_ref() {
                                Some(sel) => sel.id.as_str() == booth_id,
                                None => false
                            }
                        }>
                            {booth.description.clone()}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}

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

                    // Main content
                    <main class="py-8">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path="/booths" view=BoothListPage/>
                            <Route path="/vendors" view=VendorListPage/>
                            <Route path="/checkout" view=CheckoutPage/>
                            <Route path="/reports" view=ReportsPlaceholder/>
                        </Routes>
                    </main>

                    // Footer
                    <footer class="bg-white border-t mt-auto">
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

// Placeholder components for routes not yet implemented
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

#[component]
fn ReportsPlaceholder() -> impl IntoView {
    view! {
        <Container>
            <Card title_view={t!("report.title").into_view()}>
                <p class="text-gray-600">{t!("report.interface_coming_soon")}</p>
            </Card>
        </Container>
    }
}
