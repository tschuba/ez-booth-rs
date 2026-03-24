use crate::components::{Card, Container, use_toast};
use crate::state::use_app_state;
use crate::t;
use crate::selected_booth_context;
use domain::models::{BoothSummary, VendorId, PurchaseId};
use domain::services::VendorReportData;
use leptos::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportTab {
    BoothSummary,
    VendorReports,
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    let app_state = use_app_state();
    let selected_booth = selected_booth_context::use_selected_booth();
    let toast = use_toast();

    // Tab and selection state
    let (active_tab, set_active_tab) = create_signal(ReportTab::BoothSummary);
    let (selected_vendors, set_selected_vendors) = create_signal(Vec::<VendorId>::new());
    let (active_vendors, set_active_vendors) = create_signal(Vec::<VendorId>::new());
    let (show_custom_selection, set_show_custom_selection) = create_signal(false);
    
    // Report data state
    let (booth_summary, set_booth_summary) = create_signal(Option::<BoothSummary>::None);
    let (vendor_reports, set_vendor_reports) = create_signal(Vec::<VendorReportData>::new());
    let (is_loading, set_is_loading) = create_signal(false);

    // Load active vendors when booth is selected
    create_effect(move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();

        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            let booth_id = booth.id.clone();
            spawn_local(async move {
                match state.report_service.get_active_vendors(&booth_id, None).await {
                    Ok(vendors) => {
                        set_active_vendors.set(vendors);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Failed to load active vendors: {:?}", e).into());
                    }
                }
            });
        } else {
            set_active_vendors.set(Vec::new());
        }
    });

    // Auto-generate report when tab changes or booth changes
    create_effect(move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();
        let current_tab = active_tab.get();
        
        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            let booth_id = booth.id.clone();
            set_is_loading.set(true);
            
            spawn_local(async move {
                match current_tab {
                    ReportTab::BoothSummary => {
                        match state.report_service.generate_booth_summary(&booth_id, None).await {
                            Ok(summary) => {
                                set_booth_summary.set(Some(summary));
                                set_vendor_reports.set(Vec::new());
                            }
                            Err(e) => {
                                toast.error(&format!("{}: {:?}", t!("report.errors.generate_failed")(), e));
                            }
                        }
                    }
                    ReportTab::VendorReports => {
                        // For vendor reports tab, only load when we have "Print All" action
                        // This will be triggered separately
                        set_booth_summary.set(None);
                    }
                }
                set_is_loading.set(false);
            });
        }
    });

    // Generate vendor reports (for Print All or custom selection)
    let generate_vendor_reports = move |vendor_ids: Vec<VendorId>| {
        let state_result = app_state.get();
        let booth = selected_booth.get();

        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let booth_id = booth.id.clone();

            spawn_local(async move {
                if vendor_ids.is_empty() {
                    toast.error(&t!("report.errors.no_vendors_found")());
                    set_is_loading.set(false);
                } else {
                    match state.report_service.generate_vendor_reports(&booth_id, vendor_ids, None).await {
                        Ok(reports) => {
                            set_vendor_reports.set(reports);
                        }
                        Err(e) => {
                            toast.error(&format!("{}: {:?}", t!("report.errors.generate_failed")(), e));
                        }
                    }
                    set_is_loading.set(false);
                }
            });
        }
    };

    // Print All Vendors action
    let print_all_vendors = move |_| {
        let all_vendors = active_vendors.get();
        generate_vendor_reports(all_vendors);
    };

    // Print Custom Selection action
    let print_custom_selection = move |_| {
        let selected = selected_vendors.get();
        if selected.is_empty() {
            toast.error(&t!("report.errors.no_vendors_selected")());
        } else {
            generate_vendor_reports(selected);
        }
    };

    // Handle vendor selection toggle
    let toggle_vendor = move |vendor_id: VendorId| {
        let mut current = selected_vendors.get();
        if let Some(pos) = current.iter().position(|v| v == &vendor_id) {
            current.remove(pos);
        } else {
            current.push(vendor_id);
        }
        current.sort();
        set_selected_vendors.set(current);
    };

    // Handle print for booth summary
    let handle_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };

    view! {
        <>
            // Screen-only UI (hidden during print)
            <div class="print:hidden">
                <Container>
                    <Card title_view={t!("report.title").into_view()}>
                        <Show
                            when=move || selected_booth.get().is_none()
                            fallback=move || view! {
                                <div class="space-y-6">
                                    // Tab Navigation
                                    <div class="border-b border-gray-200">
                                        <nav class="-mb-px flex space-x-8">
                                            <button
                                                on:click=move |_| set_active_tab.set(ReportTab::BoothSummary)
                                                class=move || {
                                                    if active_tab.get() == ReportTab::BoothSummary {
                                                        "border-blue-600 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                                                    } else {
                                                        "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                                                    }
                                                }
                                            >
                                                {t!("report.booth_summary_report")}
                                            </button>
                                            <button
                                                on:click=move |_| set_active_tab.set(ReportTab::VendorReports)
                                                class=move || {
                                                    if active_tab.get() == ReportTab::VendorReports {
                                                        "border-blue-600 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                                                    } else {
                                                        "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                                                    }
                                                }
                                            >
                                                {t!("report.vendor_report")}
                                            </button>
                                        </nav>
                                    </div>

                                    // Tab Content
                                    <div class="min-h-[400px]">
                                        <Show
                                            when=move || active_tab.get() == ReportTab::BoothSummary
                                            fallback=move || view! {
                                                // Vendor Reports Tab
                                                <div class="space-y-4">
                                                    // Quick Action: Print All
                                                    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                                                        <div class="flex items-center justify-between">
                                                            <div>
                                                                <h3 class="text-lg font-semibold text-gray-900">
                                                                    {t!("report.print_all_vendors")}
                                                                </h3>
                                                                <p class="text-sm text-gray-600 mt-1">
                                                                    {move || format!("{} {} {}", 
                                                                        t!("report.print_reports_for")(),
                                                                        active_vendors.get().len(),
                                                                        t!("report.active_vendors")()
                                                                    )}
                                                                </p>
                                                            </div>
                                                            <button
                                                                on:click=print_all_vendors
                                                                disabled=move || is_loading.get() || active_vendors.get().is_empty()
                                                                class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed font-medium shadow-sm"
                                                            >
                                                                {move || if is_loading.get() {
                                                                    t!("report.loading")()
                                                                } else {
                                                                    t!("report.print_all")()
                                                                }}
                                                            </button>
                                                        </div>
                                                    </div>

                                                    // Custom Selection (Optional)
                                                    <div class="border border-gray-200 rounded-lg p-4">
                                                        <button
                                                            on:click=move |_| set_show_custom_selection.update(|v| *v = !*v)
                                                            class="flex items-center justify-between w-full text-left"
                                                        >
                                                            <span class="font-medium text-gray-900">
                                                                {t!("report.custom_vendor_selection")}
                                                            </span>
                                                            <svg
                                                                class=move || if show_custom_selection.get() { "transform rotate-180 w-5 h-5" } else { "w-5 h-5" }
                                                                fill="none"
                                                                stroke="currentColor"
                                                                viewBox="0 0 24 24"
                                                            >
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                                            </svg>
                                                        </button>

                                                        <Show when=move || show_custom_selection.get()>
                                                            <div class="mt-4 space-y-3">
                                                                <div class="max-h-64 overflow-y-auto border rounded p-3 space-y-2 bg-gray-50">
                                                                    {move || {
                                                                        active_vendors.get()
                                                                            .into_iter()
                                                                            .map(|vendor_id| {
                                                                                let vid = vendor_id.clone();
                                                                                let vid_for_toggle = vendor_id.clone();
                                                                                let vendor_id_str = vendor_id.to_string();
                                                                                view! {
                                                                                    <label class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-2 rounded">
                                                                                        <input
                                                                                            type="checkbox"
                                                                                            checked=move || selected_vendors.get().contains(&vid)
                                                                                            on:change=move |_| toggle_vendor(vid_for_toggle.clone())
                                                                                            class="rounded text-blue-600 focus:ring-blue-500"
                                                                                        />
                                                                                        <span class="text-sm font-medium">{vendor_id_str}</span>
                                                                                    </label>
                                                                                }
                                                                            })
                                                                            .collect_view()
                                                                    }}
                                                                </div>
                                                                <div class="flex gap-2">
                                                                    <button
                                                                        on:click=move |_| set_selected_vendors.set(active_vendors.get())
                                                                        class="px-3 py-2 text-sm bg-gray-100 hover:bg-gray-200 rounded"
                                                                    >
                                                                        {t!("report.select_all")}
                                                                    </button>
                                                                    <button
                                                                        on:click=move |_| set_selected_vendors.set(Vec::new())
                                                                        class="px-3 py-2 text-sm bg-gray-100 hover:bg-gray-200 rounded"
                                                                    >
                                                                        {t!("report.clear_selection")}
                                                                    </button>
                                                                </div>
                                                                <button
                                                                    on:click=print_custom_selection
                                                                    disabled=move || is_loading.get() || selected_vendors.get().is_empty()
                                                                    class="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed font-medium"
                                                                >
                                                                    {move || if is_loading.get() {
                                                                        t!("report.loading")()
                                                                    } else {
                                                                        format!("{} ({} {})", 
                                                                            t!("report.print_selected")(),
                                                                            selected_vendors.get().len(),
                                                                            t!("report.vendors")()
                                                                        )
                                                                    }}
                                                                </button>
                                                            </div>
                                                        </Show>
                                                    </div>

                                                    // Vendor Reports Preview
                                                    <Show when=move || !vendor_reports.get().is_empty()>
                                                        <div class="border-t pt-6">
                                                            <VendorReportsDisplay reports=vendor_reports.get() />
                                                        </div>
                                                    </Show>
                                                </div>
                                            }
                                        >
                                            // Booth Summary Tab
                                            <div>
                                                <Show
                                                    when=move || is_loading.get()
                                                    fallback=move || view! {
                                                        <Show when=move || booth_summary.get().is_some()>
                                                            {move || booth_summary.get().map(|summary| {
                                                                view! { <BoothSummaryDisplay summary=summary /> }
                                                            })}
                                                        </Show>
                                                    }
                                                >
                                                    <div class="flex items-center justify-center py-12">
                                                        <div class="text-center">
                                                            <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                                                            <p class="mt-4 text-gray-600">{t!("report.loading")}</p>
                                                        </div>
                                                    </div>
                                                </Show>
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            }
                        >
                            <div class="p-4 bg-blue-50 rounded">
                                <p class="text-gray-700">{t!("report.no_booth_selected")}</p>
                            </div>
                        </Show>
                    </Card>
                </Container>

                // Floating Print Button (only for Booth Summary with data)
                <Show when=move || active_tab.get() == ReportTab::BoothSummary && booth_summary.get().is_some()>
                    <button
                        on:click=handle_print
                        class="fixed bottom-8 right-8 px-6 py-4 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 hover:shadow-xl transition-all font-medium flex items-center gap-2"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                        </svg>
                        {t!("report.print_report")}
                    </button>
                </Show>

                // Floating Print Button for Vendor Reports (when reports are loaded)
                <Show when=move || active_tab.get() == ReportTab::VendorReports && !vendor_reports.get().is_empty()>
                    <button
                        on:click=handle_print
                        class="fixed bottom-8 right-8 px-6 py-4 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 hover:shadow-xl transition-all font-medium flex items-center gap-2"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                        </svg>
                        {t!("report.print_report")}
                    </button>
                </Show>
            </div>

            // Print-only layout (hidden on screen, visible during print)
            <div class="hidden print:block">
                <Show when=move || booth_summary.get().is_some()>
                    {move || booth_summary.get().map(|summary| view! { <PrintBoothSummary summary=summary /> })}
                </Show>

                <Show when=move || !vendor_reports.get().is_empty()>
                    {move || {
                        let reports = vendor_reports.get();
                        view! { <PrintVendorReports reports=reports /> }
                    }}
                </Show>
            </div>
        </>
    }
}

#[component]
fn BoothSummaryDisplay(summary: BoothSummary) -> impl IntoView {
    let total_revenue = summary.total_revenue;
    let total_purchases = summary.total_purchases;
    let unique_vendors = summary.unique_vendors;
    let total_participation_fees = summary.total_participation_fees;
    let total_sales_fees = summary.total_sales_fees;
    let total_booth_revenue = summary.total_booth_revenue;

    view! {
        <div class="space-y-6">
            // Summary Statistics
            <div class="grid grid-cols-3 gap-4">
                <div class="p-4 bg-blue-50 rounded-lg">
                    <p class="text-sm text-gray-600">{t!("report.sales_total")}</p>
                    <p class="text-2xl font-bold text-blue-600">
                        {format!("€ {:.2}", total_revenue)}
                    </p>
                </div>
                <div class="p-4 bg-green-50 rounded-lg">
                    <p class="text-sm text-gray-600">{t!("report.purchase_count")}</p>
                    <p class="text-2xl font-bold text-green-600">{total_purchases}</p>
                </div>
                <div class="p-4 bg-purple-50 rounded-lg">
                    <p class="text-sm text-gray-600">{t!("report.vendors_count")}</p>
                    <p class="text-2xl font-bold text-purple-600">{unique_vendors}</p>
                </div>
            </div>

            // Booth Revenue Section
            <div class="border rounded-lg p-6 bg-gradient-to-br from-amber-50 to-yellow-50">
                <h3 class="text-lg font-bold text-gray-800 mb-4">{t!("report.booth_revenue")}</h3>
                <div class="space-y-3">
                    <div class="flex justify-between items-center">
                        <span class="text-gray-700">{t!("report.total_participation_fees")}</span>
                        <span class="font-semibold text-gray-900">{format!("€ {:.2}", total_participation_fees)}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-gray-700">{t!("report.total_sales_fees")}</span>
                        <span class="font-semibold text-gray-900">{format!("€ {:.2}", total_sales_fees)}</span>
                    </div>
                    <div class="flex justify-between items-center pt-3 border-t-2 border-amber-200">
                        <span class="text-lg font-bold text-gray-900">{t!("report.total_booth_revenue")}</span>
                        <span class="text-2xl font-bold text-amber-700">{format!("€ {:.2}", total_booth_revenue)}</span>
                    </div>
                </div>
            </div>

            // Vendor Breakdown Table
            <div class="border rounded-lg overflow-hidden">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                {t!("report.vendor_id")}
                            </th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                {t!("report.gross_sales")}
                            </th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                {t!("report.fees_due")}
                            </th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                {t!("report.net_payout")}
                            </th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                {t!("report.item_count")}
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        {summary
                            .vendor_summaries
                            .into_iter()
                            .map(|vs| {
                                let vendor_id_str = vs.vendor_id.to_string();
                                view! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-4 py-3 text-sm font-medium text-gray-900">
                                            {vendor_id_str}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {format!("€ {:.2}", vs.gross_sales)}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {format!("€ {:.2}", vs.fees_due)}
                                        </td>
                                        <td class="px-4 py-3 text-sm font-semibold text-gray-900 text-right">
                                            {format!("€ {:.2}", vs.net_payout)}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {vs.item_count}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn VendorReportsDisplay(reports: Vec<VendorReportData>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            {reports
                .into_iter()
                .map(|report| {
                    let vendor_id = report.vendor.vendor_id.as_str().to_string();
                    let vendor_name = report.vendor.name.clone();
                    let sales_sum = report.sales_sum;
                    let item_count = report.items.len();
                    let participation_fee = report.participation_fee;
                    let sales_fee = report.sales_fee;
                    let total_revenue = report.total_revenue;
                    let items = report.items;

                    view! {
                        <div class="border rounded-lg p-4 bg-white shadow-sm">
                            <div class="mb-4">
                                <h4 class="text-lg font-semibold text-gray-900">
                                    {t!("report.vendor_id")} ": " {vendor_id}
                                    {vendor_name.map(|name| format!(" ({})", name))}
                                </h4>
                            </div>

                            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
                                <div class="bg-gray-50 p-3 rounded">
                                    <p class="text-xs text-gray-600">{t!("report.gross_sales")}</p>
                                    <p class="text-lg font-semibold">{format!("€ {:.2}", sales_sum)}</p>
                                </div>
                                <div class="bg-gray-50 p-3 rounded">
                                    <p class="text-xs text-gray-600">{t!("report.item_count")}</p>
                                    <p class="text-lg font-semibold">{item_count}</p>
                                </div>
                                <div class="bg-gray-50 p-3 rounded">
                                    <p class="text-xs text-gray-600">{t!("report.fees_due")}</p>
                                    <p class="text-lg">{format!("€ {:.2}", participation_fee + sales_fee)}</p>
                                </div>
                                <div class="bg-green-50 p-3 rounded">
                                    <p class="text-xs text-gray-600">{t!("report.net_payout")}</p>
                                    <p class="text-lg font-bold text-green-700">{format!("€ {:.2}", total_revenue)}</p>
                                </div>
                            </div>

                            // Items preview (collapsible)
                            <details class="mt-3">
                                <summary class="cursor-pointer text-sm font-medium text-gray-700 hover:text-gray-900">
                                    {t!("report.view_items")} " (" {item_count} ")"
                                </summary>
                                <div class="mt-2 text-xs text-gray-600 space-y-1 max-h-96 overflow-y-auto bg-gray-50 p-2 rounded border border-gray-300">
                                    {
                                        // Group items by transaction_id
                                        let mut grouped: HashMap<PurchaseId, Vec<_>> = HashMap::new();
                                        for item in items.iter() {
                                            grouped.entry(item.transaction_id.clone()).or_default().push(item.clone());
                                        }
                                        
                                        let mut transactions: Vec<_> = grouped.into_iter().collect();
                                        transactions.sort_by(|a, b| {
                                            // Sort by the first item's position in the original list
                                            items.iter().position(|i| i.transaction_id == a.0)
                                                .cmp(&items.iter().position(|i| i.transaction_id == b.0))
                                        });
                                        
                                        let mut item_counter = 0;
                                        transactions
                                            .into_iter()
                                            .map(|(transaction_id, transaction_items)| {
                                                let is_multi_item = transaction_items.len() > 1;
                                                let transaction_total: rust_decimal::Decimal = transaction_items.iter()
                                                    .map(|item| item.item.amount)
                                                    .sum();
                                                
                                                if is_multi_item {
                                                    // Multi-item transaction: show grouped with subtotal
                                                    let txn_label = t!("report.transaction_id");
                                                    view! {
                                                        <div class="mb-2 border-l-2 border-blue-400 pl-2">
                                                            {transaction_items
                                                                .into_iter()
                                                                .enumerate()
                                                                .map(|(idx, report_item)| {
                                                                    item_counter += 1;
                                                                    let time_str = report_item.timestamp.format("%H:%M").to_string();
                                                                    let txn_id_str = if idx == 0 {
                                                                        format!("{}: {}", txn_label(), transaction_id.to_string())
                                                                    } else {
                                                                        String::new()
                                                                    };
                                                                    view! {
                                                                        <div class="grid grid-cols-[1fr_auto_auto] gap-4 items-center py-1">
                                                                            <span class="text-gray-500 text-xs">{txn_id_str}</span>
                                                                            <span class="text-gray-500 text-xs text-right">{time_str}</span>
                                                                            <span class="font-medium text-right">{format!("€ {:.2}", report_item.item.amount)}</span>
                                                                        </div>
                                                                    }.into_view()
                                                                })
                                                                .collect_view()}
                                                            <div class="grid grid-cols-[1fr_auto_auto] gap-4 items-center py-1 border-t border-gray-300 font-semibold">
                                                                <span></span>
                                                                <span class="text-right">{t!("report.subtotal")}</span>
                                                                <span class="text-right">{format!("€ {:.2}", transaction_total)}</span>
                                                            </div>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    // Single-item transaction: show transaction ID in first column
                                                    item_counter += 1;
                                                    let report_item = &transaction_items[0];
                                                    let time_str = report_item.timestamp.format("%H:%M").to_string();
                                                    view! {
                                                        <div class="mb-2 border-l-2 border-blue-400 pl-2 py-1">
                                                            <div class="grid grid-cols-[1fr_auto_auto] gap-4 items-center">
                                                                <span class="text-gray-500 text-xs">
                                                                    {t!("report.transaction_id")}{": "}{report_item.transaction_id.to_string()}
                                                                </span>
                                                                <span class="text-gray-500 text-xs text-right">{time_str}</span>
                                                                <span class="font-medium text-right">{format!("€ {:.2}", report_item.item.amount)}</span>
                                                            </div>
                                                        </div>
                                                    }.into_view()
                                                }
                                            })
                                            .collect_view()
                                    }
                                </div>
                            </details>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}

// Print-only component for Booth Summary
#[component]
fn PrintBoothSummary(summary: BoothSummary) -> impl IntoView {
    let total_revenue = summary.total_revenue;
    let total_purchases = summary.total_purchases;
    let unique_vendors = summary.unique_vendors;
    let total_participation_fees = summary.total_participation_fees;
    let total_sales_fees = summary.total_sales_fees;
    let total_booth_revenue = summary.total_booth_revenue;

    view! {
        <div class="p-8 max-w-4xl mx-auto">
            // Report Header
            <div class="mb-8 pb-4 border-b-2 border-gray-800">
                <h1 class="text-3xl font-bold mb-2">{t!("report.booth_summary_report")}</h1>
            </div>

            // Summary Statistics
            <div class="mb-8">
                <h2 class="text-xl font-bold mb-4">{t!("report.summary_statistics")}</h2>
                <div class="grid grid-cols-3 gap-6 mb-6">
                    <div class="border-2 border-gray-300 p-4 rounded">
                        <p class="text-sm text-gray-600 mb-1">{t!("report.sales_total")}</p>
                        <p class="text-3xl font-bold">{format!("€ {:.2}", total_revenue)}</p>
                    </div>
                    <div class="border-2 border-gray-300 p-4 rounded">
                        <p class="text-sm text-gray-600 mb-1">{t!("report.purchase_count")}</p>
                        <p class="text-3xl font-bold">{total_purchases}</p>
                    </div>
                    <div class="border-2 border-gray-300 p-4 rounded">
                        <p class="text-sm text-gray-600 mb-1">{t!("report.vendors_count")}</p>
                        <p class="text-3xl font-bold">{unique_vendors}</p>
                    </div>
                </div>
            </div>

            // Booth Revenue Section
            <div class="mb-8 border-2 border-gray-400 p-6 rounded bg-gray-50">
                <h2 class="text-xl font-bold mb-4">{t!("report.booth_revenue")}</h2>
                <div class="space-y-2">
                    <div class="flex justify-between text-base">
                        <span class="text-gray-700">{t!("report.total_participation_fees")}"："</span>
                        <span class="font-semibold">{format!("€ {:.2}", total_participation_fees)}</span>
                    </div>
                    <div class="flex justify-between text-base">
                        <span class="text-gray-700">{t!("report.total_sales_fees")}"："</span>
                        <span class="font-semibold">{format!("€ {:.2}", total_sales_fees)}</span>
                    </div>
                    <div class="flex justify-between pt-3 border-t-2 border-gray-800 text-lg">
                        <span class="font-bold">{t!("report.total_booth_revenue")}"："</span>
                        <span class="font-bold text-2xl">{format!("€ {:.2}", total_booth_revenue)}</span>
                    </div>
                </div>
            </div>

            // Vendor Breakdown Table
            <div>
                <h2 class="text-xl font-bold mb-4">{t!("report.vendor_breakdown")}</h2>
                <table class="w-full border-collapse">
                    <thead>
                        <tr class="border-b-2 border-gray-800">
                            <th class="px-4 py-3 text-left font-bold">{t!("report.vendor_id")}</th>
                            <th class="px-4 py-3 text-right font-bold">{t!("report.gross_sales")}</th>
                            <th class="px-4 py-3 text-right font-bold">{t!("report.fees_due")}</th>
                            <th class="px-4 py-3 text-right font-bold">{t!("report.net_payout")}</th>
                            <th class="px-4 py-3 text-right font-bold">{t!("report.item_count")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {summary
                            .vendor_summaries
                            .into_iter()
                            .map(|vs| {
                                let vendor_id_str = vs.vendor_id.to_string();
                                view! {
                                    <tr class="border-b border-gray-300">
                                        <td class="px-4 py-3 font-medium">{vendor_id_str}</td>
                                        <td class="px-4 py-3 text-right">{format!("€ {:.2}", vs.gross_sales)}</td>
                                        <td class="px-4 py-3 text-right">{format!("€ {:.2}", vs.fees_due)}</td>
                                        <td class="px-4 py-3 text-right font-semibold">{format!("€ {:.2}", vs.net_payout)}</td>
                                        <td class="px-4 py-3 text-right">{vs.item_count}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>

            // Footer
            <div class="mt-8 pt-4 border-t border-gray-400 text-sm text-gray-600">
                <p>{t!("report.generated_at")}" "
                    {chrono::Local::now().format("%d.%m.%Y %H:%M").to_string()}
                </p>
            </div>
        </div>
    }
}

// Print-only component for Vendor Reports
#[component]
fn PrintVendorReports(reports: Vec<VendorReportData>) -> impl IntoView {
    view! {
        <div class="p-8 max-w-4xl mx-auto">
            {reports
                .into_iter()
                .enumerate()
                .map(|(idx, report)| {
                    let vendor_id = report.vendor.vendor_id.as_str().to_string();
                    let vendor_name = report.vendor.name.clone();
                    let sales_sum = report.sales_sum;
                    let participation_fee = report.participation_fee;
                    let sales_fee = report.sales_fee;
                    let total_revenue = report.total_revenue;
                    let items = report.items;
                    let booth_description = report.booth.description.clone();
                    let booth_date = report.booth.date;
                    
                    // Use CSS page-break-before property
                    let page_break_style = if idx > 0 { "page-break-before: always;" } else { "" };

                    view! {
                        <div class="mb-8" style=page_break_style>
                            // Report Header
                            <div class="mb-6 pb-4 border-b-2 border-gray-800">
                                <h1 class="text-3xl font-bold mb-2">{t!("report.vendor_report")}</h1>
                                <div class="text-lg">
                                    <p class="font-semibold">
                                        {t!("report.vendor_id")}": "{vendor_id.clone()}
                                        {vendor_name.map(|name| format!(" - {}", name))}
                                    </p>
                                    <p class="text-sm text-gray-700">{booth_description}</p>
                                    <p class="text-sm text-gray-600">{booth_date.format("%d.%m.%Y").to_string()}</p>
                                </div>
                            </div>

                            // Financial Summary
                            <div class="mb-6">
                                <h2 class="text-xl font-bold mb-4">{t!("report.financial_summary")}</h2>
                                <div class="border-2 border-gray-300 p-4 rounded space-y-3">
                                    <div class="flex justify-between py-2">
                                        <span class="font-medium">{t!("report.gross_sales")}"："</span>
                                        <span class="text-lg font-semibold">{format!("€ {:.2}", sales_sum)}</span>
                                    </div>
                                    <div class="flex justify-between py-2 border-t">
                                        <span class="text-gray-600">{t!("report.participation_fee")}"："</span>
                                        <span>{format!("-€ {:.2}", participation_fee)}</span>
                                    </div>
                                    <div class="flex justify-between py-2">
                                        <span class="text-gray-600">{t!("report.sales_fee")}"："</span>
                                        <span>{format!("-€ {:.2}", sales_fee)}</span>
                                    </div>
                                    <div class="flex justify-between py-3 border-t-2 border-gray-800">
                                        <span class="text-xl font-bold">{t!("report.net_payout")}"："</span>
                                        <span class="text-2xl font-bold text-green-700">{format!("€ {:.2}", total_revenue)}</span>
                                    </div>
                                </div>
                            </div>

                            // Items List
                            <div>
                                <h2 class="text-xl font-bold mb-4">
                                    {t!("report.sales_details")}" ("{items.len()}" "{t!("report.items")}")"
                                </h2>
                                 <table class="w-full border-collapse">
                                    <thead>
                                        <tr class="border-b-2 border-gray-800">
                                            <th class="px-4 py-2 text-left font-bold">{t!("report.transaction_id")}</th>
                                            <th class="px-4 py-2 text-left font-bold">{t!("report.time")}</th>
                                            <th class="px-4 py-2 text-right font-bold">{t!("report.amount")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {
                                            // Group items by transaction_id
                                            let mut grouped: HashMap<PurchaseId, Vec<_>> = HashMap::new();
                                            for item in items.iter() {
                                                grouped.entry(item.transaction_id.clone()).or_default().push(item.clone());
                                            }
                                            
                                            let mut transactions: Vec<_> = grouped.into_iter().collect();
                                            transactions.sort_by(|a, b| {
                                                items.iter().position(|i| i.transaction_id == a.0)
                                                    .cmp(&items.iter().position(|i| i.transaction_id == b.0))
                                            });
                                            
                                            let mut item_counter = 0;
                                            transactions
                                                .into_iter()
                                                .flat_map(|(transaction_id, transaction_items)| {
                                                    let is_multi_item = transaction_items.len() > 1;
                                                    let transaction_total: rust_decimal::Decimal = transaction_items.iter()
                                                        .map(|item| item.item.amount)
                                                        .sum();
                                                    
                                                    if is_multi_item {
                                                        // Multi-item transaction: show items with subtotal (no item numbers)
                                                        let mut rows = vec![];
                                                        
                                                        for report_item in transaction_items {
                                                            item_counter += 1;
                                                            let time_str = report_item.timestamp
                                                                .with_timezone(&chrono::Local)
                                                                .format("%H:%M")
                                                                .to_string();
                                                            rows.push(view! {
                                                                <tr class="border-b border-gray-200">
                                                                    <td class="px-4 py-2 text-gray-600 font-mono text-sm">{transaction_id.to_string()}</td>
                                                                    <td class="px-4 py-2 text-gray-600 text-sm">{time_str}</td>
                                                                    <td class="px-4 py-2 text-right font-medium">{format!("€ {:.2}", report_item.item.amount)}</td>
                                                                </tr>
                                                            }.into_view());
                                                        }
                                                        
                                                        // Add subtotal row
                                                        rows.push(view! {
                                                            <tr class="border-b-2 border-gray-400 bg-gray-50">
                                                                <td colspan="2" class="px-4 py-2 font-semibold text-right">{t!("report.subtotal")}</td>
                                                                <td class="px-4 py-2 text-right font-semibold">{format!("€ {:.2}", transaction_total)}</td>
                                                            </tr>
                                                        }.into_view());
                                                        
                                                        rows
                                                    } else {
                                                        // Single-item transaction: show without item number
                                                        item_counter += 1;
                                                        let report_item = &transaction_items[0];
                                                        let time_str = report_item.timestamp
                                                            .with_timezone(&chrono::Local)
                                                            .format("%H:%M")
                                                            .to_string();
                                                        vec![view! {
                                                            <tr class="border-b border-gray-300">
                                                                <td class="px-4 py-2 text-gray-600 font-mono text-sm">{report_item.transaction_id.to_string()}</td>
                                                                <td class="px-4 py-2 text-gray-600 text-sm">{time_str}</td>
                                                                <td class="px-4 py-2 text-right font-medium">{format!("€ {:.2}", report_item.item.amount)}</td>
                                                            </tr>
                                                        }.into_view()]
                                                    }
                                                })
                                                .collect_view()
                                        }
                                    </tbody>
                                    <tfoot>
                                        <tr class="border-t-2 border-gray-800 font-bold">
                                            <td colspan="2" class="px-4 py-3">{t!("report.total")}</td>
                                            <td class="px-4 py-3 text-right text-lg">{format!("€ {:.2}", sales_sum)}</td>
                                        </tr>
                                    </tfoot>
                                </table>
                            </div>

                            // Footer
                            <div class="mt-8 pt-4 border-t border-gray-400 text-sm text-gray-600">
                                <p>{t!("report.generated_at")}" "
                                    {chrono::Local::now().format("%d.%m.%Y %H:%M").to_string()}
                                </p>
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
