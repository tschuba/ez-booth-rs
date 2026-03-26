use crate::components::*;
use crate::formatting::format_currency;
use crate::i18n::use_locale;
use crate::selected_booth_context;
use crate::state::*;
use crate::t;
use domain::models::{PurchaseId, Vendor, VendorId};
use domain::services::VendorReportData;
use leptos::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[component]
pub fn VendorListPage() -> impl IntoView {
    let app_state = use_app_state();
    let (vendor_reports, set_vendor_reports) = create_signal(Vec::<VendorReportData>::new());
    let (vendors_without_purchases, set_vendors_without_purchases) = create_signal(Vec::<Vendor>::new());
    
    // Use global selected booth context
    let selected_booth = selected_booth_context::use_selected_booth();
    let (is_loading, set_is_loading) = create_signal(true);
    let (show_vendor_detail, set_show_vendor_detail) = create_signal(false);
    let (selected_vendor_report, set_selected_vendor_report) = create_signal(None::<VendorReportData>);
    
    // Selection state
    let (selected_vendor_ids, set_selected_vendor_ids) = create_signal(Vec::<VendorId>::new());
    let (reports_for_print, set_reports_for_print) = create_signal(Vec::<VendorReportData>::new());

    let toast = use_toast();

    // Load vendors when booth is selected
    create_effect(move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();

        if booth.is_none() {
            // No booth selected, clear vendors
            set_vendor_reports.set(Vec::new());
            set_vendors_without_purchases.set(Vec::new());
            set_is_loading.set(false);
        } else if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let booth_id = booth.id.clone();
            spawn_local(async move {
                // Get all registered vendors for the booth
                let all_vendors_result = state.vendor_repository.find_by_booth(&booth_id).await;
                
                // Get active vendors (those with purchases)
                let active_vendors_result = state.report_service.get_active_vendors(&booth_id, None).await;
                
                match (all_vendors_result, active_vendors_result) {
                    (Ok(mut all_vendors), Ok(active_vendor_ids)) => {
                        // Sort all vendors
                        all_vendors.sort_by(|a, b| a.vendor_id.cmp(&b.vendor_id));
                        
                        // Generate reports for active vendors
                        if !active_vendor_ids.is_empty() {
                            match state.report_service.generate_vendor_reports(&booth_id, active_vendor_ids.clone(), None).await {
                                Ok(reports) => {
                                    set_vendor_reports.set(reports);
                                    
                                    // Find vendors without purchases
                                    let vendors_without: Vec<Vendor> = all_vendors
                                        .into_iter()
                                        .filter(|v| !active_vendor_ids.contains(&v.vendor_id))
                                        .collect();
                                    set_vendors_without_purchases.set(vendors_without);
                                }
                                Err(e) => {
                                    toast.error(&format!("Failed to load vendor reports: {:?}", e));
                                }
                            }
                        } else {
                            // No active vendors, all vendors have no purchases
                            set_vendor_reports.set(Vec::new());
                            set_vendors_without_purchases.set(all_vendors);
                        }
                    }
                    (Err(e), _) => {
                        toast.error(&format!("Failed to load vendors: {:?}", e));
                    }
                    (_, Err(e)) => {
                        toast.error(&format!("Failed to load active vendors: {:?}", e));
                    }
                }
                
                set_is_loading.set(false);
            });
        }
    });

    let handle_vendor_click = move |report: VendorReportData| {
        set_selected_vendor_report.set(Some(report));
        set_show_vendor_detail.set(true);
    };
    
    // Toggle vendor selection
    let toggle_vendor_selection = move |vendor_id: VendorId| {
        set_selected_vendor_ids.update(|ids| {
            if let Some(pos) = ids.iter().position(|v| v == &vendor_id) {
                ids.remove(pos);
            } else {
                ids.push(vendor_id);
            }
            ids.sort();
        });
    };
    
    // Generate reports for all vendors
    let generate_all_reports = move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();
        
        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let booth_id = booth.id.clone();
            
            spawn_local(async move {
                match state.report_service.get_active_vendors(&booth_id, None).await {
                    Ok(vendor_ids) => {
                        if vendor_ids.is_empty() {
                            toast.error(&t!("vendor.no_vendors_with_purchases")());
                            set_is_loading.set(false);
                        } else {
                            match state.report_service.generate_vendor_reports(&booth_id, vendor_ids, None).await {
                                Ok(reports) => {
                                    set_reports_for_print.set(reports);
                                    // Trigger print after a short delay to allow DOM update
                                    set_timeout(
                                        move || {
                                            if let Some(window) = web_sys::window() {
                                                let _ = window.print();
                                            }
                                        },
                                        std::time::Duration::from_millis(100),
                                    );
                                }
                                Err(e) => {
                                    toast.error(&format!("Failed to generate reports: {:?}", e));
                                }
                            }
                            set_is_loading.set(false);
                        }
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to get active vendors: {:?}", e));
                        set_is_loading.set(false);
                    }
                }
            });
        }
    };
    
    // Generate reports for selected vendors
    let generate_selected_reports = move |_| {
        let selected = selected_vendor_ids.get();
        if selected.is_empty() {
            toast.error(&t!("vendor.no_vendors_selected")());
            return;
        }
        
        let state_result = app_state.get();
        let booth = selected_booth.get();
        
        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let booth_id = booth.id.clone();
            
            spawn_local(async move {
                match state.report_service.generate_vendor_reports(&booth_id, selected, None).await {
                    Ok(reports) => {
                        set_reports_for_print.set(reports);
                        // Trigger print after a short delay to allow DOM update
                        set_timeout(
                            move || {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.print();
                                }
                            },
                            std::time::Duration::from_millis(100),
                        );
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to generate reports: {:?}", e));
                    }
                }
                set_is_loading.set(false);
            });
        }
    };
    
    // Print single vendor report from modal
    let print_vendor_report = move || {
        if let Some(report) = selected_vendor_report.get() {
            set_reports_for_print.set(vec![report]);
            // Trigger print after a short delay
            set_timeout(
                move || {
                    if let Some(window) = web_sys::window() {
                        let _ = window.print();
                    }
                },
                std::time::Duration::from_millis(100),
            );
        }
    };

    // Helper to get vendor detail modal title
    let vendor_detail_title = move || {
        selected_vendor_report
            .get()
            .map(|r| {
                format!(
                    "{} {}",
                    t!("vendor.detail_title")(),
                    r.vendor.vendor_id.as_str()
                )
            })
            .unwrap_or_else(|| t!("vendor.detail_title")())
    };

    view! {
        <>
            // Screen-only UI (hidden during print)
            <div class="print:hidden">
                <Container>
                    <div class="py-8">
                        <Card title_view={t!("vendor.list_title").into_view()}>
                            <Show
                                when=move || !is_loading.get()
                                fallback=|| view! { <p class="text-gray-600">{t!("common.loading")}</p> }
                            >
                                <Show
                                    when=move || selected_booth.get().is_some()
                                    fallback=|| view! { <p class="text-gray-500 text-center py-8">{t!("vendor.select_booth_prompt")}</p> }
                                >
                                    // Helper text section
                                    <div class="mb-6">
                                        <Show when=move || !vendor_reports.get().is_empty()>
                                            <p class="text-sm text-gray-600">
                                                {move || {
                                                    let total_count = vendor_reports.get().len();
                                                    let selected_count = selected_vendor_ids.get().len();
                                                    if selected_count > 0 {
                                                        format!("{} {} {} {}", 
                                                            selected_count,
                                                            t!("vendor.vendors_selected_of")(),
                                                            total_count,
                                                            t!("vendor.vendors_with_purchases")()
                                                        )
                                                    } else {
                                                        format!("{} {} {}", 
                                                            total_count,
                                                            t!("vendor.vendors_with_purchases")(),
                                                            t!("vendor.click_vendors_hint")()
                                                        )
                                                    }
                                                }}
                                            </p>
                                        </Show>
                                    </div>

                                    // Vendor list
                                    <Show
                                        when=move || !vendor_reports.get().is_empty() || !vendors_without_purchases.get().is_empty()
                                        fallback=|| view! { <p class="text-gray-500 text-center py-8">{t!("vendor.no_vendors")}</p> }
                                    >
                                        <div class="space-y-4">
                                            {/* Vendors with purchases */}
                                            {move || vendor_reports.get().into_iter().map(|report| {
                                                let report_clone = report.clone();
                                                let vendor_id = report.vendor.vendor_id.clone();
                                                let vendor_id_for_button = vendor_id.clone();
                                                let vendor_id_for_class1 = vendor_id.clone();
                                                let vendor_id_for_class2 = vendor_id.clone();
                                                let vendor_id_for_text = vendor_id.clone();
                                                let vendor_id_str = vendor_id.as_str().to_string();
                                                let locale = use_locale();
                                                
                                                view! {
                                                    <div
                                                        class=move || format!(
                                                            "border rounded-lg p-4 transition-all cursor-pointer {}",
                                                            if selected_vendor_ids.get().contains(&vendor_id_for_class1) {
                                                                "border-blue-500 bg-blue-50 shadow-md"
                                                            } else {
                                                                "border-gray-200 hover:shadow-md"
                                                            }
                                                        )
                                                        on:click=move |_| toggle_vendor_selection(vendor_id_for_button.clone())
                                                    >
                                                        <div class="flex items-start justify-between">
                                                            <div class="flex-1">
                                                                <h3 class="text-lg font-semibold text-gray-900">
                                                                    {vendor_id_str.clone()}
                                                                </h3>
                                                                <div class="mt-2 space-y-1 text-sm">
                                                                    <div class="flex justify-between">
                                                                        <span class="text-gray-600">{t!("checkout.total")()}</span>
                                                                        <span class="font-semibold">{format_currency(report.sales_sum, locale.get())}</span>
                                                                    </div>
                                                                    <div class="flex justify-between">
                                                                        <span class="text-gray-600">{t!("vendor.net_payout")()}</span>
                                                                        <span class="font-semibold text-green-700">{format_currency(report.total_revenue, locale.get())}</span>
                                                                    </div>
                                                                    <div class="flex justify-between">
                                                                        <span class="text-gray-600">{t!("checkout.running_totals.items")()}</span>
                                                                        <span class="font-semibold">{report.items.len()}</span>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                            <button
                                                                on:click=move |e| {
                                                                    e.stop_propagation();
                                                                    handle_vendor_click(report_clone.clone())
                                                                }
                                                                title={t!("vendor.view_details")}
                                                                aria-label={t!("vendor.view_details")}
                                                                class="ml-4 w-10 h-10 flex items-center justify-center rounded-full transition-all bg-gradient-to-br from-gray-50 to-gray-100 text-gray-700 hover:from-gray-100 hover:to-gray-200 hover:shadow-md border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1"
                                                            >
                                                                {/* Info icon */}
                                                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                            
                                            {/* Vendors without purchases */}
                                            {move || vendors_without_purchases.get().into_iter().map(|vendor| {
                                                let vendor_id_str = vendor.vendor_id.as_str().to_string();
                                                view! {
                                                    <div class="border border-gray-200 rounded-lg p-4 bg-gray-50">
                                                        <h3 class="text-lg font-semibold text-gray-600 flex items-center gap-2">
                                                            <span>{t!("vendor.id_label")()}</span>
                                                            <span>{vendor_id_str}</span>
                                                        </h3>
                                                        <p class="text-sm text-gray-500 mt-2">
                                                            {t!("vendor.no_purchases")}
                                                        </p>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </Show>
                                </Show>
                            </Show>
                        </Card>
                    </div>
                </Container>

                // Floating action buttons (bottom-right corner)
                <Show when=move || !vendor_reports.get().is_empty()>
                    <div class="fixed bottom-6 right-6 z-40 flex flex-col-reverse sm:flex-row items-end gap-3">
                        {/* Clear selection button - icon only */}
                        <button
                            on:click=move |_| set_selected_vendor_ids.set(Vec::new())
                            disabled=move || selected_vendor_ids.get().is_empty()
                            title={t!("vendor.clear_selection")}
                            aria-label={t!("vendor.clear_selection")}
                            class="w-14 h-14 flex items-center justify-center rounded-full transition-all shadow-2xl bg-gray-800/80 backdrop-blur text-white hover:bg-gray-900 hover:scale-110 disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:scale-100 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                        >
                            {/* Arrow rotate icon */}
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                            </svg>
                        </button>
                        
                        {/* Print button - with text */}
                        <button
                            on:click=move |_| {
                                let selected = selected_vendor_ids.get();
                                if selected.is_empty() {
                                    generate_all_reports(())
                                } else {
                                    generate_selected_reports(())
                                }
                            }
                            disabled=move || is_loading.get() || vendor_reports.get().is_empty()
                            title={move || {
                                let selected_count = selected_vendor_ids.get().len();
                                if selected_count > 0 {
                                    format!("{} ({})", t!("vendor.print_selection")(), selected_count)
                                } else {
                                    t!("vendor.print_all")()
                                }
                            }}
                            class="px-6 py-4 rounded-full font-semibold text-lg shadow-2xl transition-all bg-gradient-to-br from-blue-600 to-blue-700 text-white hover:from-blue-700 hover:to-blue-800 hover:shadow-2xl hover:scale-105 disabled:from-gray-400 disabled:to-gray-500 disabled:cursor-not-allowed disabled:hover:scale-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center gap-3"
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                            </svg>
                            {move || {
                                if is_loading.get() {
                                    t!("common.loading")()
                                } else {
                                    let selected_count = selected_vendor_ids.get().len();
                                    if selected_count > 0 {
                                        format!("{} ({})", t!("vendor.print_selection")(), selected_count)
                                    } else {
                                        t!("vendor.print_all")()
                                    }
                                }
                            }}
                        </button>
                    </div>
                </Show>

                // Vendor detail modal
                <Modal
                    show=show_vendor_detail
                    on_close=move || set_show_vendor_detail.set(false)
                    title=vendor_detail_title()
                >
                    {move || {
                        selected_vendor_report.get().map(|report| {
                            let vendor_id = report.vendor.vendor_id.as_str().to_string();
                            let vendor_name = report.vendor.name.clone();
                            let locale = use_locale();
                            
                            view! {
                                <div class="space-y-4">
                                    <div class="mb-4">
                                        <h4 class="text-lg font-semibold text-gray-900">
                                            {t!("vendor.id_label")()}" "{vendor_id}
                                            {vendor_name.map(|name| format!(" ({})", name))}
                                        </h4>
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="bg-gray-50 p-3 rounded">
                                            <p class="text-xs text-gray-600">{t!("vendor.gross_sales")()}</p>
                                            <p class="text-lg font-semibold">{format_currency(report.sales_sum, locale.get())}</p>
                                        </div>
                                        <div class="bg-gray-50 p-3 rounded">
                                            <p class="text-xs text-gray-600">{t!("vendor.item_count")()}</p>
                                            <p class="text-lg font-semibold">{report.items.len()}</p>
                                        </div>
                                        <div class="bg-gray-50 p-3 rounded">
                                            <p class="text-xs text-gray-600">{t!("vendor.fees_due")()}</p>
                                            <p class="text-lg">{format_currency(report.participation_fee + report.sales_fee, locale.get())}</p>
                                        </div>
                                        <div class="bg-green-50 p-3 rounded">
                                            <p class="text-xs text-gray-600">{t!("vendor.net_payout")()}</p>
                                            <p class="text-lg font-bold text-green-700">{format_currency(report.total_revenue, locale.get())}</p>
                                        </div>
                                    </div>

                                    // Items preview
                                    <details class="mt-3">
                                        <summary class="cursor-pointer text-sm font-medium text-gray-700 hover:text-gray-900">
                                            {t!("vendor.view_items")}" ("{report.items.len()}")"
                                        </summary>
                                        <div class="mt-2 text-xs text-gray-600 space-y-1 max-h-96 overflow-y-auto bg-gray-50 p-2 rounded border border-gray-300">
                                            {
                                                // Group items by transaction_id
                                                let mut grouped: HashMap<PurchaseId, Vec<_>> = HashMap::new();
                                                for item in report.items.iter() {
                                                    grouped.entry(item.transaction_id.clone()).or_default().push(item.clone());
                                                }

                                                let mut transactions: Vec<_> = grouped.into_iter().collect();
                                                transactions.sort_by(|a, b| {
                                                    report.items.iter().position(|i| i.transaction_id == a.0)
                                                        .cmp(&report.items.iter().position(|i| i.transaction_id == b.0))
                                                });

                                                transactions
                                                    .into_iter()
                                                    .map(|(transaction_id, transaction_items)| {
                                                        let time_str = transaction_items[0].timestamp.format("%H:%M").to_string();
                                                        let total: Decimal = transaction_items.iter().map(|i| i.item.amount).sum();
                                                        
                                                        view! {
                                                            <div class="mb-2 border-l-2 border-blue-400 pl-2 py-1">
                                                                <div class="flex justify-between items-center">
                                                                    <span class="text-gray-500 text-xs">
                                                                        {t!("vendor.transaction_id")()}{": "}{transaction_id.to_string()}
                                                                    </span>
                                                                    <span class="text-gray-500 text-xs">{time_str}</span>
                                                                    <span class="font-medium">{format_currency(total, locale.get())}</span>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view()
                                            }
                                        </div>
                                    </details>

                                    <div class="mt-6 flex justify-end gap-2">
                                        <Button on_click=Box::new(print_vendor_report)>
                                            {t!("vendor.print_report")}
                                        </Button>
                                        <Button on_click=Box::new(move || set_show_vendor_detail.set(false))>
                                            {t!("common.close")}
                                        </Button>
                                    </div>
                                </div>
                            }
                        })
                    }}
                </Modal>
            </div>

            // Print-only layout (hidden on screen, visible during print)
            <div class="hidden print:block">
                <Show when=move || !reports_for_print.get().is_empty()>
                    {move || {
                        let reports = reports_for_print.get();
                        view! { <PrintVendorReports reports=reports /> }
                    }}
                </Show>
            </div>
        </>
    }
}

// Reuse PrintVendorReports component from reports page
#[component]
fn PrintVendorReports(reports: Vec<VendorReportData>) -> impl IntoView {
    let locale = use_locale();
    view! {
        <div class="print-reports-container">
            {reports
                .into_iter()
                .map(|report| {
                    let vendor_id = report.vendor.vendor_id.as_str().to_string();
                    let vendor_name = report.vendor.name.clone();
                    let sales_sum = report.sales_sum;
                    let participation_fee = report.participation_fee;
                    let sales_fee = report.sales_fee;
                    let total_revenue = report.total_revenue;
                    let items = report.items;
                    let booth_description = report.booth.description.clone();
                    let booth_date = report.booth.date;

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
                    let has_multiple_transactions = transactions.len() > 1;

                    view! {
                        <div class="print-vendor-report">
                            // Vendor header - compact for efficiency
                            <div class="mb-3 pb-2 border-b border-gray-800">
                                <h1 class="text-2xl font-bold mb-1">{t!("vendor.report_title")}</h1>
                                <div class="text-base">
                                    <p class="font-semibold">
                                        {t!("vendor.id_label")()}": "{vendor_id.clone()}
                                        {vendor_name.as_ref().map(|name| format!(" - {}", name))}
                                    </p>
                                    <p class="text-sm text-gray-700">{booth_description.clone()}</p>
                                    <p class="text-sm text-gray-600">{booth_date.format("%d.%m.%Y").to_string()}</p>
                                </div>
                            </div>

                            // Financial summary - reduced spacing
                            <div class="mb-3">
                                <h2 class="text-lg font-bold mb-2">{t!("vendor.financial_summary")}</h2>
                                <div class="border border-gray-400 p-2">
                                    <div class="flex justify-between py-1">
                                        <span class="font-medium">{t!("vendor.gross_sales")}"："</span>
                                        <span class="text-base font-semibold">{move || format_currency(sales_sum, locale.get())}</span>
                                    </div>
                                    <div class="flex justify-between py-1 border-t border-gray-300">
                                        <span>{t!("vendor.participation_fee")}"："</span>
                                        <span>{move || format!("-{}", format_currency(participation_fee, locale.get()))}</span>
                                    </div>
                                    <div class="flex justify-between py-1 border-t border-gray-300">
                                        <span>{t!("vendor.sales_fee")}"："</span>
                                        <span>{move || format!("-{}", format_currency(sales_fee, locale.get()))}</span>
                                    </div>
                                    <div class="flex justify-between py-1 border-t-2 border-gray-800">
                                        <span class="text-base font-bold">{t!("vendor.net_payout")}"："</span>
                                        <span class="text-lg font-bold">{move || format_currency(total_revenue, locale.get())}</span>
                                    </div>
                                </div>
                            </div>

                            // Sales details - compact grid layout
                            <div class="print-sales-section">
                                <h2 class="text-lg font-bold mb-0">{t!("vendor.sales_details")}" ("{items.len()}" "{t!("vendor.items")}")"</h2>
                                <div class="print-transactions-container">
                                    {transactions
                                        .into_iter()
                                        .map(|(transaction_id, transaction_items)| {
                                            let is_multi_item = transaction_items.len() > 1;
                                            let transaction_total: Decimal = transaction_items.iter()
                                                .map(|item| item.item.amount)
                                                .sum();

                                            view! {
                                                <div class="print-transaction-group">
                                                    {if is_multi_item {
                                                        // Multi-item transaction
                                                        view! {
                                                            <div class="print-transaction-header">
                                                                <span class="text-xs text-gray-600">{t!("vendor.transaction_id")}": "{transaction_id.to_string()}</span>
                                                            </div>
                                                            <div class="print-items-grid">
                                                                {transaction_items
                                                                    .into_iter()
                                                                    .map(|report_item| {
                                                                        let time_str = report_item.timestamp
                                                                            .with_timezone(&chrono::Local)
                                                                            .format("%H:%M")
                                                                            .to_string();
                                                                        let amount = report_item.item.amount;
                                                                        view! {
                                                                            <div class="print-item">
                                                                                <span class="print-item-time">{time_str}</span>
                                                                                <span class="print-item-amount">{move || format_currency(amount, locale.get())}</span>
                                                                            </div>
                                                                        }
                                                                    })
                                                                    .collect_view()}
                                                            </div>
                                                            {if has_multiple_transactions {
                                                                view! {
                                                                    <div class="print-subtotal">
                                                                        <span>{t!("vendor.subtotal")}"："</span>
                                                                        <span class="font-semibold">{move || format_currency(transaction_total, locale.get())}</span>
                                                                    </div>
                                                                }.into_view()
                                                            } else {
                                                                View::default()
                                                            }}
                                                        }.into_view()
                                                    } else {
                                                        // Single-item transaction
                                                        let report_item = &transaction_items[0];
                                                        let time_str = report_item.timestamp
                                                            .with_timezone(&chrono::Local)
                                                            .format("%H:%M")
                                                            .to_string();
                                                        let amount = report_item.item.amount;
                                                        view! {
                                                            <div class="print-transaction-header">
                                                                <span class="text-xs text-gray-600">{t!("vendor.transaction_id")}": "{transaction_id.to_string()}</span>
                                                            </div>
                                                            <div class="print-items-grid">
                                                                <div class="print-item">
                                                                    <span class="print-item-time">{time_str}</span>
                                                                    <span class="print-item-amount">{move || format_currency(amount, locale.get())}</span>
                                                                </div>
                                                            </div>
                                                        }.into_view()
                                                    }}
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
