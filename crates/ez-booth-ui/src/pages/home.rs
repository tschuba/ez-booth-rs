use crate::components::*;
use crate::t;
use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Container>
            <div class="py-12">
                <Card title="EZ Booth">
                    <p class="text-gray-600 mb-4">
                        {t!("app.welcome")}
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
                        <a href="/booths" class="block">
                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                <h3 class="text-lg font-semibold mb-2">{t!("booth.list_title")}</h3>
                                <p class="text-gray-600 text-sm">"Manage your booths"</p>
                            </div>
                        </a>
                        <a href="/checkout" class="block">
                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                <h3 class="text-lg font-semibold mb-2">{t!("checkout.title")}</h3>
                                <p class="text-gray-600 text-sm">"Process sales"</p>
                            </div>
                        </a>
                    </div>
                </Card>
            </div>
        </Container>
    }
}
