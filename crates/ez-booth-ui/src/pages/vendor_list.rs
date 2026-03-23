use crate::components::*;
use crate::state::*;
use crate::t;
use domain::models::booth::Booth;
use domain::models::vendor::Vendor;
use leptos::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug)]
struct VendorSummary {
    vendor: Vendor,
    total_sales: Decimal,
    purchase_count: usize,
}

#[component]
pub fn VendorListPage() -> impl IntoView {
    let app_state = use_app_state();
    let (vendor_summaries, set_vendor_summaries) = create_signal(Vec::<VendorSummary>::new());
    let (selected_booth, set_selected_booth) = create_signal(None::<Booth>);
    let (is_loading, set_is_loading) = create_signal(true);
    let (show_vendor_detail, set_show_vendor_detail) = create_signal(false);
    let (selected_vendor, set_selected_vendor) = create_signal(None::<VendorSummary>);

    let toast = use_toast();

    // Load vendors when booth is selected
    create_effect(move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();

        if booth.is_none() {
            // No booth selected, clear vendors
            set_vendor_summaries.set(Vec::new());
            set_is_loading.set(false);
        } else if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let booth_id = booth.id;
            spawn_local(async move {
                // List vendors for the booth
                match state.vendor_repository.find_by_booth(&booth_id).await {
                    Ok(mut vendors) => {
                        // Sort vendors using smart sorting (already implemented in VendorId)
                        vendors.sort_by(|a, b| a.vendor_id.cmp(&b.vendor_id));

                        // Calculate summary for each vendor
                        let mut summaries = Vec::new();
                        for vendor in vendors {
                            // Get all purchases for this vendor
                            match state.purchase_repository.find_by_vendor(
                                &vendor.booth_id,
                                &vendor.vendor_id,
                            ).await {
                                Ok(purchases) => {
                                    let purchase_count = purchases.len();
                                    let total_sales: Decimal = purchases
                                        .iter()
                                        .flat_map(|p| &p.items)
                                        .map(|item| item.amount)
                                        .sum();

                                    summaries.push(VendorSummary {
                                        vendor,
                                        total_sales,
                                        purchase_count,
                                    });
                                }
                                Err(e) => {
                                    toast.error(&format!("Failed to load purchases for vendor: {:?}", e));
                                }
                            }
                        }

                        set_vendor_summaries.set(summaries);
                        set_is_loading.set(false);
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to load vendors: {:?}", e));
                        set_is_loading.set(false);
                    }
                }
            });
        }
    });

    // Load available booths for selection
    let (booths, set_booths) = create_signal(Vec::new());
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

    let handle_vendor_click = move |summary: VendorSummary| {
        set_selected_vendor.set(Some(summary));
        set_show_vendor_detail.set(true);
    };

    // Helper to get vendor detail modal title
    let vendor_detail_title = move || {
        selected_vendor.get()
            .map(|s| format!("{} {}", t!("vendor.detail_title")(), s.vendor.vendor_id.as_str()))
            .unwrap_or_else(|| t!("vendor.detail_title")())
    };

    view! {
        <Container>
            <div class="py-8">
                <Card title_view={t!("vendor.list_title").into_view()}>
                    // Booth selector
                    <div class="mb-6">
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {t!("vendor.select_booth")}
                        </label>
                        <select
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if value.is_empty() {
                                    set_selected_booth.set(None);
                                } else {
                                    // Find the booth by ID
                                    let booth = booths.get().into_iter().find(|b| b.id.as_str() == value);
                                    set_selected_booth.set(booth);
                                }
                            }
                        >
                            <option value="">{t!("vendor.no_booth_selected")}</option>
                            {move || booths.get().into_iter().map(|booth| {
                                let booth_id = booth.id.as_str().to_string();
                                view! {
                                    <option value={booth_id}>
                                        {booth.description.clone()}
                                    </option>
                                }
                            }).collect_view()}
                        </select>
                    </div>

                    // Vendor list
                    <Show
                        when=move || !is_loading.get()
                        fallback=|| view! { <p class="text-gray-600">{t!("common.loading")}</p> }
                    >
                        <Show
                            when=move || selected_booth.get().is_some()
                            fallback=|| view! { <p class="text-gray-500 text-center py-8">{t!("vendor.select_booth_prompt")}</p> }
                        >
                            <Show
                                when=move || !vendor_summaries.get().is_empty()
                                fallback=|| view! { <p class="text-gray-500 text-center py-8">{t!("vendor.no_vendors")}</p> }
                            >
                                <div class="space-y-4">
                                    {move || vendor_summaries.get().into_iter().map(|summary| {
                                        let summary_clone = summary.clone();
                                        let vendor_id = summary.vendor.vendor_id.as_str().to_string();
                                        view! {
                                            <div
                                                class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer"
                                                on:click=move |_| handle_vendor_click(summary_clone.clone())
                                            >
                                                <div class="flex justify-between items-start">
                                                    <div>
<h3 class="text-lg font-semibold text-gray-900 flex items-center">
    <span>{t!("vendor.id_label")}</span>
    <span class="ml-2">{vendor_id}</span>
</h3>
                                                        <p class="text-sm text-gray-600 mt-1">
                                                            {t!("vendor.purchase_count")} {summary.purchase_count}
                                                        </p>
                                                    </div>
                                                    <div class="text-right">
                                                        <p class="text-2xl font-bold text-blue-600">
                                                            {format!("{:.2}", summary.total_sales)}
                                                        </p>
                                                        <p class="text-xs text-gray-500">
                                                            {t!("vendor.total_sales")}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </Show>
                        </Show>
                    </Show>
                </Card>
            </div>

            // Vendor detail modal
            <Modal
                show=show_vendor_detail
                on_close=move || set_show_vendor_detail.set(false)
                title=vendor_detail_title()
            >
                {move || {
                    selected_vendor.get().map(|summary| {
                        let vendor_id = summary.vendor.vendor_id.as_str().to_string();
                        let created_at = summary.vendor.created_at.format("%Y-%m-%d %H:%M").to_string();
                        view! {
                            <div class="space-y-4">
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <p class="text-sm text-gray-600">{t!("vendor.id_label")}</p>
                                        <p class="text-lg font-semibold">{vendor_id}</p>
                                    </div>
                                    <div>
                                        <p class="text-sm text-gray-600">{t!("vendor.purchase_count")}</p>
                                        <p class="text-lg font-semibold">{summary.purchase_count}</p>
                                    </div>
                                    <div>
                                        <p class="text-sm text-gray-600">{t!("vendor.total_sales")}</p>
                                        <p class="text-lg font-semibold">{format!("{:.2}", summary.total_sales)}</p>
                                    </div>
                                    <div>
                                        <p class="text-sm text-gray-600">{t!("vendor.created_at")}</p>
                                        <p class="text-lg font-semibold">
                                            {created_at}
                                        </p>
                                    </div>
                                </div>

                                <div class="mt-6 flex justify-end">
                                    <Button on_click=Box::new(move || set_show_vendor_detail.set(false))>
                                        {t!("common.close")}
                                    </Button>
                                </div>
                            </div>
                        }
                    })
                }}
            </Modal>
        </Container>
    }
}
