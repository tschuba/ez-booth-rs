use crate::components::{Card, Container};
use crate::t;
use leptos::*;

#[component]
pub fn ReportsPage() -> impl IntoView {
    view! {
        <Container>
            <Card title_view={t!("report.title").into_view()}>
                <div class="space-y-6">
                    <p class="text-gray-600">{t!("report.interface_coming_soon")}</p>
                    <div class="p-4 bg-blue-50 rounded">
                        <p class="text-sm text-gray-700">
                            "Phase 3.4 - Reporting & Printing feature is in development."
                        </p>
                        <ul class="mt-2 text-sm text-gray-600 list-disc list-inside">
                            <li>"Booth summary reports with vendor breakdowns"</li>
                            <li>"Individual vendor reports with itemized sales"</li>
                            <li>"Print-friendly layouts"</li>
                            <li>"CSV export functionality (coming next)"</li>
                            <li>"Date range filtering"</li>
                        </ul>
                    </div>
                </div>
            </Card>
        </Container>
    }
}
