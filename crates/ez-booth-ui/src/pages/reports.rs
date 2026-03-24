use crate::components::{Card, Container, use_toast};
use crate::state::use_app_state;
use crate::t;
use crate::selected_booth_context;
use domain::models::{BoothSummary, VendorId};
use domain::services::VendorReportData;
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportType {
    BoothSummary,
    VendorReports,
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    let app_state = use_app_state();
    let selected_booth = selected_booth_context::use_selected_booth();
    let toast = use_toast();

    // Report configuration state
    let (report_type, set_report_type) = create_signal(ReportType::BoothSummary);
    let (selected_vendors, set_selected_vendors) = create_signal(Vec::<VendorId>::new());
    let (active_vendors, set_active_vendors) = create_signal(Vec::<VendorId>::new());
    
    // Report data state
    let (booth_summary, set_booth_summary) = create_signal(Option::<BoothSummary>::None);
    let (vendor_reports, set_vendor_reports) = create_signal(Vec::<VendorReportData>::new());
    let (is_generating, set_is_generating) = create_signal(false);

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

    // Handle report type change
    let on_report_type_change = move |new_type: ReportType| {
        set_report_type.set(new_type);
        // Clear previous reports when switching types
        set_booth_summary.set(None);
        set_vendor_reports.set(Vec::new());
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

    // Select all vendors
    let select_all_vendors = move |_| {
        set_selected_vendors.set(active_vendors.get());
    };

    // Clear vendor selection
    let clear_vendor_selection = move |_| {
        set_selected_vendors.set(Vec::new());
    };

    // Generate report
    let generate_report = move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();

        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_generating.set(true);
            let booth_id = booth.id.clone();
            let current_report_type = report_type.get();
            let vendors_to_report = selected_vendors.get();

            spawn_local(async move {
                match current_report_type {
                    ReportType::BoothSummary => {
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
                    ReportType::VendorReports => {
                        let vendor_ids = if vendors_to_report.is_empty() {
                            // Get all active vendors if none selected
                            match state.report_service.get_active_vendors(&booth_id, None).await {
                                Ok(vendors) => vendors,
                                Err(_) => Vec::new(),
                            }
                        } else {
                            vendors_to_report
                        };

                        if vendor_ids.is_empty() {
                            toast.error(&t!("report.errors.no_vendors_found")());
                        } else {
                            match state.report_service.generate_vendor_reports(&booth_id, vendor_ids, None).await {
                                Ok(reports) => {
                                    set_vendor_reports.set(reports);
                                    set_booth_summary.set(None);
                                }
                                Err(e) => {
                                    toast.error(&format!("{}: {:?}", t!("report.errors.generate_failed")(), e));
                                }
                            }
                        }
                    }
                }
                set_is_generating.set(false);
            });
        }
    };

    view! {
        <Container>
            <Card title_view={t!("report.title").into_view()}>
                <div class="space-y-6">
                    // Show prompt if no booth selected
                    <Show
                        when=move || selected_booth.get().is_none()
                        fallback=move || view! {
                            <div class="space-y-6">
                                // Report Type Selection
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        {t!("report.select_report_type")}
                                    </label>
                                    <div class="flex gap-4">
                                        <button
                                            on:click=move |_| on_report_type_change(ReportType::BoothSummary)
                                            class=move || {
                                                if report_type.get() == ReportType::BoothSummary {
                                                    "px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                                                } else {
                                                    "px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                                                }
                                            }
                                        >
                                            {t!("report.booth_summary_report")}
                                        </button>
                                        <button
                                            on:click=move |_| on_report_type_change(ReportType::VendorReports)
                                            class=move || {
                                                if report_type.get() == ReportType::VendorReports {
                                                    "px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                                                } else {
                                                    "px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                                                }
                                            }
                                        >
                                            {t!("report.vendor_report")}
                                        </button>
                                    </div>
                                </div>

                                // Vendor Selection (only for vendor reports)
                                <Show when=move || report_type.get() == ReportType::VendorReports>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                            {t!("report.select_vendors")}
                                        </label>
                                        <div class="flex gap-2 mb-2">
                                            <button
                                                on:click=select_all_vendors
                                                class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded"
                                            >
                                                {t!("report.all_vendors")}
                                            </button>
                                            <button
                                                on:click=clear_vendor_selection
                                                class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded"
                                            >
                                                {t!("common.clear")}
                                            </button>
                                        </div>
                                        <Show
                                            when=move || !active_vendors.get().is_empty()
                                            fallback=|| view! {
                                                <p class="text-gray-500 text-sm">{t!("report.errors.no_vendors_found")}</p>
                                            }
                                        >
                                            <div class="border rounded p-3 max-h-48 overflow-y-auto space-y-1">
                                                {move || {
                                                    active_vendors.get()
                                                        .into_iter()
                                                        .map(|vendor_id| {
                                                            let vid = vendor_id.clone();
                                                            let vid_for_toggle = vendor_id.clone();
                                                            let vendor_id_str = vendor_id.to_string();
                                                            view! {
                                                                <label class="flex items-center gap-2 cursor-pointer hover:bg-gray-50 p-1 rounded">
                                                                    <input
                                                                        type="checkbox"
                                                                        checked=move || selected_vendors.get().contains(&vid)
                                                                        on:change=move |_| toggle_vendor(vid_for_toggle.clone())
                                                                        class="rounded text-blue-600 focus:ring-blue-500"
                                                                    />
                                                                    <span class="text-sm">{vendor_id_str}</span>
                                                                </label>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </div>
                                        </Show>
                                    </div>
                                </Show>

                                // Generate Button
                                <div>
                                    <button
                                        on:click=generate_report
                                        disabled=move || is_generating.get()
                                        class="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed font-medium"
                                    >
                                        {move || {
                                            if is_generating.get() {
                                                t!("report.loading")()
                                            } else {
                                                t!("report.generate_report")()
                                            }
                                        }}
                                    </button>
                                </div>

                                // Report Display
                                <Show when=move || booth_summary.get().is_some()>
                                    {move || booth_summary.get().map(|summary| view! { <BoothSummaryDisplay summary=summary /> })}
                                </Show>

                                <Show when=move || !vendor_reports.get().is_empty()>
                                    {move || {
                                        let reports = vendor_reports.get();
                                        view! { <VendorReportsDisplay reports=reports /> }
                                    }}
                                </Show>
                            </div>
                        }
                    >
                        <div class="p-4 bg-blue-50 rounded">
                            <p class="text-gray-700">{t!("report.no_booth_selected")}</p>
                        </div>
                    </Show>
                </div>
            </Card>
        </Container>
    }
}

#[component]
fn BoothSummaryDisplay(summary: BoothSummary) -> impl IntoView {
    let total_revenue = summary.total_revenue;
    let total_purchases = summary.total_purchases;
    let unique_vendors = summary.unique_vendors;

    let handle_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };

    view! {
        <div class="space-y-4 print:space-y-6 border-t pt-6">
            <div class="flex justify-between items-center print:mb-4">
                <h3 class="text-lg font-semibold">{t!("report.booth_summary")}</h3>
                <button
                    on:click=handle_print
                    class="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded print:hidden text-sm"
                >
                    {t!("report.print_report")}
                </button>
            </div>

            // Summary Statistics
            <div class="grid grid-cols-3 gap-4">
                <div class="p-4 bg-blue-50 rounded">
                    <p class="text-sm text-gray-600">{t!("report.sales_total")}</p>
                    <p class="text-2xl font-bold text-blue-600">
                        {format!("€{:.2}", total_revenue)}
                    </p>
                </div>
                <div class="p-4 bg-green-50 rounded">
                    <p class="text-sm text-gray-600">{t!("report.purchase_count")}</p>
                    <p class="text-2xl font-bold text-green-600">{total_purchases}</p>
                </div>
                <div class="p-4 bg-purple-50 rounded">
                    <p class="text-sm text-gray-600">{t!("report.vendor_count")}</p>
                    <p class="text-2xl font-bold text-purple-600">{unique_vendors}</p>
                </div>
            </div>

            // Vendor Summaries Table
            <div class="overflow-x-auto">
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
                                {t!("report.purchase_count")}
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        {summary.vendor_summaries
                            .into_iter()
                            .map(|vs| {
                                let vendor_id_str = vs.vendor_id.to_string();
                                view! {
                                    <tr>
                                        <td class="px-4 py-3 text-sm font-medium text-gray-900">
                                            {vendor_id_str}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {format!("€{:.2}", vs.gross_sales)}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {format!("€{:.2}", vs.fees_due)}
                                        </td>
                                        <td class="px-4 py-3 text-sm font-semibold text-gray-900 text-right">
                                            {format!("€{:.2}", vs.net_payout)}
                                        </td>
                                        <td class="px-4 py-3 text-sm text-gray-700 text-right">
                                            {vs.purchase_count}
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
    let handle_print = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.print();
        }
    };

    view! {
        <div class="space-y-6 border-t pt-6">
            <div class="flex justify-between items-center print:mb-4">
                <h3 class="text-lg font-semibold">{t!("report.vendor_report")}</h3>
                <button
                    on:click=handle_print
                    class="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded print:hidden text-sm"
                >
                    {t!("report.print_report")}
                </button>
            </div>

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
                        <div class="border rounded p-4 print:break-inside-avoid bg-white">
                            <h4 class="text-md font-semibold mb-3">
                                {t!("report.vendor_id")} ": " {vendor_id}
                                {vendor_name.map(|name| format!(" ({})", name))}
                            </h4>

                            <div class="grid grid-cols-2 gap-3 mb-4">
                                <div>
                                    <p class="text-sm text-gray-600">{t!("report.gross_sales")}</p>
                                    <p class="text-lg font-semibold">
                                        {format!("€{:.2}", sales_sum)}
                                    </p>
                                </div>
                                <div>
                                    <p class="text-sm text-gray-600">{t!("report.item_count")}</p>
                                    <p class="text-lg font-semibold">{item_count}</p>
                                </div>
                                <div>
                                    <p class="text-sm text-gray-600">{t!("report.participation_fee")}</p>
                                    <p class="text-lg">{format!("€{:.2}", participation_fee)}</p>
                                </div>
                                <div>
                                    <p class="text-sm text-gray-600">{t!("report.sales_fee")}</p>
                                    <p class="text-lg">{format!("€{:.2}", sales_fee)}</p>
                                </div>
                                <div class="col-span-2 pt-2 border-t">
                                    <p class="text-sm text-gray-600">{t!("report.net_payout")}</p>
                                    <p class="text-xl font-bold text-green-600">
                                        {format!("€{:.2}", total_revenue)}
                                    </p>
                                </div>
                            </div>

                            // Items List
                            <div class="mt-3">
                                <p class="text-sm font-medium text-gray-700 mb-2">
                                    {t!("checkout.current_items")} ": " {item_count}
                                </p>
                                <div class="text-xs text-gray-600 space-y-1 max-h-32 overflow-y-auto bg-gray-50 p-2 rounded">
                                    {items
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, item)| {
                                            view! {
                                                <div class="flex justify-between">
                                                    <span>{"#"}{idx + 1}</span>
                                                    <span>{format!("€{:.2}", item.amount)}</span>
                                                </div>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
