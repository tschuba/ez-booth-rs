use crate::components::*;
use crate::components::pagination::Pagination;
use crate::components::pagination_prefs::use_pagination_preference;
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
    
    // Accordion state - only one vendor can be expanded at a time
    let (expanded_vendor_id, set_expanded_vendor_id) = create_signal(None::<VendorId>);
    
    // Selection state
    let (selected_vendor_ids, set_selected_vendor_ids) = create_signal(Vec::<VendorId>::new());
    let (reports_for_print, set_reports_for_print) = create_signal(Vec::<VendorReportData>::new());
    
    // Accessibility announcements
    let (aria_announcement, set_aria_announcement) = create_signal(String::new());
    
    // Pagination state with readiness flag
    let (page_size, set_page_size, page_size_ready) = use_pagination_preference("vendor_page_size", 10);
    let (current_page, set_current_page) = create_signal(0);
    
    // Create paginated vendor reports - only slice when preference is ready
    let paginated_vendor_reports = create_memo(move |_| {
        // Wait for page size preference to be ready
        if !page_size_ready.get() {
            return Vec::new();
        }
        
        let reports = vendor_reports.get();
        let size = page_size.get();
        let page = current_page.get();
        let start = page * size;
        let end = (start + size).min(reports.len());
        
        if start >= reports.len() {
            Vec::new()
        } else {
            reports[start..end].to_vec()
        }
    });

    let toast = use_toast();

    // Reset to first page when vendor_reports or page_size changes
    create_effect(move |_| {
        let _ = vendor_reports.get();
        let _ = page_size.get();
        set_current_page.set(0);
    });

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

    view! {
        <>
            // Aria-live region for accessibility announcements
            <div
                role="status"
                aria-live="polite"
                aria-atomic="true"
                class="sr-only"
            >
                {move || aria_announcement.get()}
            </div>
            
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
                                    
                                    // Top pagination controls
                                    <Show when=move || !vendor_reports.get().is_empty()>
                                        <div class="mb-4">
                                            <Pagination
                                                current_page=current_page
                                                total_items=Signal::derive(move || vendor_reports.get().len())
                                                page_size=page_size
                                                on_page_change=move |page| set_current_page.set(page)
                                                on_page_size_change=move |size| set_page_size.set(size)
                                                translation_prefix="vendor.pagination"
                                                show_page_size_selector=true
                                            />
                                        </div>
                                    </Show>

                                    // Vendor list
                                    <Show
                                        when=move || !vendor_reports.get().is_empty() || !vendors_without_purchases.get().is_empty()
                                        fallback=|| view! { <p class="text-gray-500 text-center py-8">{t!("vendor.no_vendors")}</p> }
                                    >
                                        <div class="space-y-4">
                                            {/* Vendors with purchases */}
                                            {move || paginated_vendor_reports.get().into_iter().map(|report| {
                                                // Clone all needed data upfront
                                                let report_data = report.clone();
                                                let vendor_id = report.vendor.vendor_id.clone();
                                                let vendor_id_for_class1 = vendor_id.clone();
                                                let vendor_id_for_expanded = vendor_id.clone();
                                                let vendor_id_str = vendor_id.as_str().to_string();
                                                let locale = use_locale();
                                                
                                                // Create stores for detail data
                                                let (detail_report, _) = create_signal(report_data.clone());
                                                
                                                let is_expanded = create_memo(move |_| {
                                                    expanded_vendor_id.get() == Some(vendor_id_for_expanded.clone())
                                                });
                                                
                                                let is_selected = create_memo(move |_| {
                                                    selected_vendor_ids.get().contains(&vendor_id_for_class1)
                                                });
                                                
                                                // IDs for keyboard support closure captures
                                                let vendor_id_for_select = vendor_id.clone();
                                                let vendor_id_for_select_key = vendor_id.clone();
                                                let vendor_id_for_keydown = vendor_id.clone();
                                                let vendor_id_for_details_btn = vendor_id.clone();
                                                
                                                view! {
                                                    <div
                                                        class=move || {
                                                            let base = "group relative rounded-lg transition-all duration-200 overflow-hidden";
                                                            let border_color = if is_selected.get() {
                                                                "border-2 border-blue-500 shadow-md"
                                                            } else {
                                                                "border border-gray-200 hover:border-gray-300 hover:shadow-md"
                                                            };
                                                            let bg = if is_expanded.get() {
                                                                "bg-blue-50"
                                                            } else {
                                                                "bg-white"
                                                            };
                                                            format!("{} {} {}", base, border_color, bg)
                                                        }
                                                    >
                                                        {/* Left accent bar */}
                                                        <div
                                                            class=move || {
                                                                let color = if is_selected.get() {
                                                                    "bg-blue-500"
                                                                } else {
                                                                    "bg-gray-200"
                                                                };
                                                                format!("absolute left-0 top-0 bottom-0 w-1 transition-colors duration-200 {}", color)
                                                            }
                                                        />
                                                        
                                                        {/* Summary section - clickable for selection */}
                                                        <div
                                                            class="pl-4 pr-4 py-4 cursor-pointer"
                                                            tabindex="0"
                                                            role="button"
                                                            aria-pressed=move || if is_selected.get() { "true" } else { "false" }
                                                            on:click=move |_| {
                                                                toggle_vendor_selection(vendor_id_for_select.clone());
                                                                let msg = if selected_vendor_ids.get().contains(&vendor_id_for_select) {
                                                                    format!("Vendor {} selected", vendor_id_for_select.as_str())
                                                                } else {
                                                                    format!("Vendor {} deselected", vendor_id_for_select.as_str())
                                                                };
                                                                set_aria_announcement.set(msg);
                                                            }
                                                            on:keydown=move |e| {
                                                                match e.key().as_str() {
                                                                    " " | "Enter" => {
                                                                        e.prevent_default();
                                                                        toggle_vendor_selection(vendor_id_for_select_key.clone());
                                                                        let msg = if selected_vendor_ids.get().contains(&vendor_id_for_select_key) {
                                                                            format!("Vendor {} selected", vendor_id_for_select_key.as_str())
                                                                        } else {
                                                                            format!("Vendor {} deselected", vendor_id_for_select_key.as_str())
                                                                        };
                                                                        set_aria_announcement.set(msg);
                                                                    }
                                                                    _ => {}
                                                                }
                                                            }
                                                        >
                                                            {/* Content wrapper - Identity left, Print+Payout grouped right */}
                                                            <div class="flex items-center pr-16">
                                                                {/* Zone 1: Identity (Vendor ID + item count) - grows to fill space */}
                                                                <div class="flex flex-col gap-1 flex-1">
                                                                    <div class="flex items-baseline gap-2">
                                                                        <span class="text-xs text-gray-500 uppercase tracking-wide">{t!("vendor.id_label")()}</span>
                                                                        <span class="text-lg font-bold text-gray-900">{vendor_id_str.clone()}</span>
                                                                    </div>
                                                                    <div class="text-sm text-gray-600">
                                                                        <span class="font-medium">{report.items.len()}</span>
                                                                        <span class="ml-1">{t!("checkout.running_totals.items")()}</span>
                                                                    </div>
                                                                </div>
                                                                
                                                                {/* Zone 2+3: Print button + Payout metrics grouped together */}
                                                                <div class="flex items-center gap-3">
                                                                    {/* Print button - hover visible when collapsed, always visible when expanded */}
                                                                    <button
                                                                        on:click=move |e| {
                                                                            e.stop_propagation();
                                                                            set_reports_for_print.set(vec![report_data.clone()]);
                                                                            set_timeout(
                                                                                move || {
                                                                                    if let Some(window) = web_sys::window() {
                                                                                        let _ = window.print();
                                                                                    }
                                                                                },
                                                                                std::time::Duration::from_millis(100),
                                                                            );
                                                                        }
                                                                        title={t!("vendor.print_report")}
                                                                        aria-label={t!("vendor.print_report")}
                                                                        class=move || format!(
                                                                            "min-w-[4rem] min-h-[4rem] w-16 h-16 flex items-center justify-center rounded-full transition-all shadow-lg bg-blue-600 text-white hover:bg-blue-700 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 {}",
                                                                            if is_expanded.get() {
                                                                                "opacity-100"
                                                                            } else {
                                                                                "opacity-0 group-hover:opacity-100"
                                                                            }
                                                                        )
                                                                    >
                                                                        {/* Print icon - matches FAB */}
                                                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                                                                        </svg>
                                                                    </button>
                                                                    
                                                                    {/* Performance metrics - directly adjacent to print button */}
                                                                    <div class="flex flex-col items-end gap-1">
                                                                        <div class="text-right">
                                                                            <div class="text-xs text-gray-500 uppercase tracking-wide mb-0.5">
                                                                                {t!("vendor.net_payout")()}
                                                                            </div>
                                                                            <div class="text-2xl font-bold text-green-700">
                                                                                {format_currency(report.total_revenue, locale.get())}
                                                                            </div>
                                                                        </div>
                                                                        <div class="text-xs text-gray-600">
                                                                            <span>{t!("vendor.total_sales")()}</span>
                                                                            <span class="ml-1 font-semibold">{format_currency(report.sales_sum, locale.get())}</span>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        
                                                        {/* Checkout-style expansion toggle on right edge */}
                                                        <div class="absolute right-0 top-0 h-full flex items-center">
                                                            {/* Vertical separator */}
                                                            <div class="h-full w-px bg-gray-300"></div>
                                                            
                                                            {/* Expansion toggle button */}
                                                            <div
                                                                class="px-3 hover:bg-blue-50 transition-colors h-full flex items-center cursor-pointer"
                                                                on:click=move |e| {
                                                                    e.stop_propagation();
                                                                    set_expanded_vendor_id.update(|current| {
                                                                        let was_expanded = current.as_ref() == Some(&vendor_id_for_details_btn);
                                                                        *current = if was_expanded {
                                                                            None
                                                                        } else {
                                                                            Some(vendor_id_for_details_btn.clone())
                                                                        };
                                                                        let msg = if was_expanded {
                                                                            format!("Details collapsed for Vendor {}", vendor_id_for_details_btn.as_str())
                                                                        } else {
                                                                            format!("Details expanded for Vendor {}", vendor_id_for_details_btn.as_str())
                                                                        };
                                                                        set_aria_announcement.set(msg);
                                                                    });
                                                                }
                                                                on:keydown=move |e| {
                                                                    match e.key().as_str() {
                                                                        "ArrowRight" => {
                                                                            e.prevent_default();
                                                                            e.stop_propagation();
                                                                            set_expanded_vendor_id.set(Some(vendor_id_for_keydown.clone()));
                                                                            set_aria_announcement.set(format!("Details expanded for Vendor {}", vendor_id_for_keydown.as_str()));
                                                                        }
                                                                        "ArrowLeft" => {
                                                                            e.prevent_default();
                                                                            e.stop_propagation();
                                                                            set_expanded_vendor_id.set(None);
                                                                            set_aria_announcement.set(format!("Details collapsed for Vendor {}", vendor_id_for_keydown.as_str()));
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                                tabindex="0"
                                                                role="button"
                                                                aria-expanded=move || if is_expanded.get() { "true" } else { "false" }
                                                                aria-controls=format!("vendor-details-{}", vendor_id_str.clone())
                                                                aria-label=move || if is_expanded.get() { t!("common.close")() } else { t!("vendor.view_details")() }
                                                            >
                                                                {/* Chevron icon */}
                                                                <svg
                                                                    class=move || format!("w-5 h-5 text-gray-400 hover:text-blue-600 transition-all {}", if is_expanded.get() { "rotate-180" } else { "" })
                                                                    fill="none"
                                                                    stroke="currentColor"
                                                                    viewBox="0 0 24 24"
                                                                >
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                                                </svg>
                                                            </div>
                                                        </div>
                                                        
                                                        {/* Expanded detail drawer */}
                                                        <div
                                                            id=format!("vendor-details-{}", vendor_id_str.clone())
                                                            role="region"
                                                            aria-label=format!("Vendor {} details", vendor_id_str.clone())
                                                            class=move || format!(
                                                                "overflow-hidden transition-all duration-200 {}",
                                                                if is_expanded.get() { "max-h-[1000px] opacity-100" } else { "max-h-0 opacity-0" }
                                                            )
                                                        >
                                                            <div class="border-t border-gray-200 p-4 pr-16 bg-gray-50">
                                                                {
                                                                    let report = detail_report.get();
                                                                    let locale = use_locale();
                                                                    
                                                                     view! {
                                                                        <div class="space-y-4">
                                                                            {/* Two-column responsive grid layout */}
                                                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                                                {/* Left column: Fee breakdown with sum visualization */}
                                                                                <div class="space-y-3">
                                                                                    <h4 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                                                                                        {t!("vendor.financial_summary")()}
                                                                                    </h4>
                                                                                    <div class="space-y-3">
                                                                                        {/* Individual fees side-by-side with plus indicator */}
                                                                                        <div class="flex items-center gap-2 flex-wrap sm:flex-nowrap">
                                                                                            <div class="bg-white p-3 rounded shadow-sm flex-1 min-w-[120px]">
                                                                                                <p class="text-xs text-gray-600">{t!("vendor.participation_fee")()}</p>
                                                                                                <p class="text-lg font-semibold">{format_currency(report.participation_fee, locale.get())}</p>
                                                                                            </div>
                                                                                            
                                                                                            {/* Plus symbol */}
                                                                                            <div class="flex-shrink-0">
                                                                                                <div class="w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center">
                                                                                                    <span class="text-gray-600 text-sm font-bold">+</span>
                                                                                                </div>
                                                                                            </div>
                                                                                            
                                                                                            <div class="bg-white p-3 rounded shadow-sm flex-1 min-w-[120px]">
                                                                                                <p class="text-xs text-gray-600">{t!("vendor.sales_fee")()}</p>
                                                                                                <p class="text-lg font-semibold">{format_currency(report.sales_fee, locale.get())}</p>
                                                                                            </div>
                                                                                        </div>
                                                                                        
                                                                                        {/* Equals symbol */}
                                                                                        <div class="flex justify-center">
                                                                                            <div class="w-6 h-6 rounded-full bg-blue-100 flex items-center justify-center">
                                                                                                <span class="text-blue-600 text-sm font-bold">=</span>
                                                                                            </div>
                                                                                        </div>
                                                                                        
                                                                                        {/* Cumulative total with emphasis */}
                                                                                        <div class="bg-gradient-to-br from-blue-50 to-blue-100 p-3 rounded shadow-md border-2 border-blue-500">
                                                                                            <p class="text-xs text-gray-700 font-semibold">{t!("vendor.fees_due")()}</p>
                                                                                            <p class="text-xl font-bold text-blue-700">{format_currency(report.participation_fee + report.sales_fee, locale.get())}</p>
                                                                                            <p class="text-xs text-gray-600 mt-1">{t!("vendor.fees_sum_explanation")()}</p>
                                                                                        </div>
                                                                                    </div>
                                                                                </div>
                                                                                
                                                                                {/* Right column: Transactions */}
                                                                                <div class="space-y-3">
                                                                                    <h4 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                                                                                        {t!("vendor.transaction_id")()}{" "}({report.items.len()}{" "}{t!("vendor.items")()})
                                                                                    </h4>
                                                                                    
                                                                                    {/* Transactions list */}
                                                                                    <div class="bg-white p-3 rounded shadow-sm max-h-96 overflow-y-auto">
                                                                                        <div class="text-xs text-gray-600 space-y-2">
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
                                                                                                    let item_count = transaction_items.len();
                                                                                                    
                                                                                                    view! {
                                                                                                        <div class="border-l-2 border-blue-400 pl-2 py-1">
                                                                                                            <div class="flex justify-between items-center gap-2">
                                                                                                                <div class="flex items-center gap-2 flex-1">
                                                                                                                    <span class="text-gray-500 text-xs">
                                                                                                                        {t!("vendor.transaction_id")()}{": "}{transaction_id.to_string()}
                                                                                                                    </span>
                                                                                                                    <span class="text-gray-400">"·"</span>
                                                                                                                    <span class="text-xs text-gray-600">
                                                                                                                        {item_count}{" "}{t!("vendor.items")()}
                                                                                                                    </span>
                                                                                                                </div>
                                                                                                                <span class="text-gray-500 text-xs">{time_str}</span>
                                                                                                                <span class="font-medium">{format_currency(total, locale.get())}</span>
                                                                                                            </div>
                                                                                                        </div>
                                                                                                    }
                                                                                                })
                                                                                                .collect_view()
                                                                                        }
                                                                                        </div>
                                                                                    </div>
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                            
                                            {/* Bottom pagination */}
                                            <Show when=move || !vendor_reports.get().is_empty()>
                                                <div class="mt-4">
                                                    <Pagination
                                                        current_page=current_page
                                                        total_items=Signal::derive(move || vendor_reports.get().len())
                                                        page_size=page_size
                                                        on_page_change=move |page| set_current_page.set(page)
                                                        on_page_size_change=move |size| set_page_size.set(size)
                                                        translation_prefix="vendor.pagination"
                                                        show_page_size_selector=true
                                                    />
                                                </div>
                                            </Show>
                                            
                                            {/* Vendors without purchases heading */}
                                            <Show when=move || !vendors_without_purchases.get().is_empty()>
                                                <div class="mt-8 mb-4">
                                                    <h2 class="text-lg font-semibold text-gray-700 border-b border-gray-300 pb-2">
                                                        {t!("vendor.vendors_without_purchases_heading")}
                                                    </h2>
                                                </div>
                                            </Show>
                                            
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
                        {/* Clear selection button - icon only with improved size and icon */}
                        <button
                            on:click=move |_| set_selected_vendor_ids.set(Vec::new())
                            disabled=move || selected_vendor_ids.get().is_empty()
                            title={t!("vendor.clear_selection")}
                            aria-label={t!("vendor.clear_selection")}
                            class="min-w-[4rem] min-h-[4rem] w-16 h-16 flex items-center justify-center rounded-full transition-all shadow-2xl bg-gray-800/80 backdrop-blur text-white hover:bg-gray-900 hover:scale-110 disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:scale-100 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                        >
                            <span class="sr-only">{t!("vendor.clear_selection")}</span>
                            {/* Clear selection icon - list with X marks */}
                            <svg class="w-7 h-7" viewBox="0 0 16 16" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                                <path d="M10 12.6l.7.7 1.6-1.6 1.6 1.6.8-.7L13 11l1.7-1.6-.8-.8-1.6 1.7-1.6-1.7-.7.8 1.6 1.6-1.6 1.6zM1 4h14V3H1v1zm0 3h14V6H1v1zm8 2.5V9H1v1h8v-.5zM9 13v-1H1v1h8z" />
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
