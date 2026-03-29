use crate::components::{Card, Container, ExportButton, ExportScope, ImportButton};
use crate::state::use_app_state;
use crate::t;
use leptos::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let app_state = use_app_state();
    let (counts, set_counts) = create_signal(None::<(usize, usize, usize)>);

    create_effect(move |_| {
        let Some(Ok(state)) = app_state.get() else {
            return;
        };

        let booth_repo = state.booth_repository.clone();
        let vendor_repo = state.vendor_repository.clone();
        let purchase_repo = state.purchase_repository.clone();

        spawn_local(async move {
            let next_counts = async {
                let booths = booth_repo.find_all().await.ok()?.len();
                let vendors = vendor_repo.find_all().await.ok()?.len();
                let purchases = purchase_repo.find_all().await.ok()?.len();
                Some((booths, vendors, purchases))
            }
            .await;

            set_counts.set(next_counts);
        });
    });

    view! {
        <Container max_width="max-w-4xl">
            <div class="space-y-6">
                <Card title_view={t!("settings.title").into_view()}>
                    <div class="space-y-4 text-gray-700">
                        <p class="text-sm uppercase tracking-wide text-blue-600 font-semibold">
                            {t!("backup.section_label")}
                        </p>
                        <div class="space-y-2">
                            <h3 class="text-lg font-semibold text-gray-900">{t!("backup.heading")}</h3>
                            <p>{t!("backup.storage_explanation")}</p>
                            <p>{t!("backup.recommendation")}</p>
                        </div>
                        <div class="rounded-lg border border-blue-100 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                            {t!("backup.counts_hint")}
                            <Show
                                when=move || counts.get().is_some()
                                fallback=move || view! { <span>{format!(" {}", t!("common.loading")())}</span> }
                            >
                                {move || {
                                    counts
                                        .get()
                                        .map(|(booths, vendors, purchases)| {
                                            view! {
                                                <span>
                                                    {format!(
                                                        " {} {} • {} {} • {} {}",
                                                        booths,
                                                        t!("backup.count_booths")(),
                                                        vendors,
                                                        t!("backup.count_vendors")(),
                                                        purchases,
                                                        t!("backup.count_purchases")(),
                                                    )}
                                                </span>
                                            }
                                        })
                                }}
                            </Show>
                        </div>
                        <div class="flex flex-wrap gap-3 pt-2">
                            <ExportButton scope=ExportScope::All />
                            <ImportButton />
                        </div>
                    </div>
                </Card>
            </div>
        </Container>
    }
}
