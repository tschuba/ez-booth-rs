use crate::components::*;
use crate::formatting::{format_currency, parse_decimal_input};
use crate::i18n::{translate_with_params, use_locale, Locale};
use crate::selected_booth_context;
use crate::state::use_app_state;
use crate::t;
use chrono::{DateTime, Local, Utc};
use domain::models::purchase::{Purchase, PurchaseItem};
use domain::models::shared::{PurchaseId, VendorId};
use domain::validation::validate_vendor_id;
use leptos::html;
use leptos::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use web_sys::window;

#[derive(Clone, Debug)]
struct CheckoutItem {
    amount: Decimal,
    vendor_id: String,
    added_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
struct CheckoutFormData {
    vendor_id: String,
    current_amount: String,
    items: Vec<CheckoutItem>,
    vendor_error: Option<String>,
    amount_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct PendingDeletion {
    purchase_id: Option<PurchaseId>,
    token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredCheckoutItem {
    amount: String,
    vendor_id: String,
    added_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredCheckoutForm {
    booth_id: Option<String>,
    vendor_id: String,
    current_amount: String,
    items: Vec<StoredCheckoutItem>,
}

const CHECKOUT_DRAFT_STORAGE_KEY: &str = "ez-booth-checkout-draft";

impl CheckoutFormData {
    fn total(&self) -> Decimal {
        self.items.iter().map(|item| item.amount).sum()
    }
}

fn format_item_timestamp(added_at: DateTime<Utc>, locale: Locale) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(added_at);

    if duration.num_seconds() < 60 {
        let secs = duration.num_seconds().max(1);
        let key = if secs == 1 {
            "checkout.time.seconds_one"
        } else {
            "checkout.time.seconds"
        };
        translate_with_params(key, HashMap::from([("count", secs.to_string())]))
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes().max(1);
        let key = if mins == 1 {
            "checkout.time.minutes_one"
        } else {
            "checkout.time.minutes"
        };
        translate_with_params(key, HashMap::from([("count", mins.to_string())]))
    } else {
        let format_str = match locale {
            Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "%H:%M",
            Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "%I:%M %p",
        };
        let local_time = added_at
            .with_timezone(&Local)
            .format(format_str)
            .to_string();
        translate_with_params(
            "checkout.time.time_of_day",
            HashMap::from([("time", local_time)]),
        )
    }
}

fn format_item_tooltip(added_at: DateTime<Utc>, locale: Locale) -> String {
    let format_str = match locale {
        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "%d.%m.%Y %H:%M",
        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "%Y-%m-%d %I:%M %p",
    };
    let local_time = added_at
        .with_timezone(&Local)
        .format(format_str)
        .to_string();
    translate_with_params(
        "checkout.time.tooltip",
        HashMap::from([("datetime", local_time)]),
    )
}

fn format_purchase_timestamp(timestamp: DateTime<Utc>, locale: Locale) -> String {
    let format_str = match locale {
        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => "%d.%m.%Y %H:%M",
        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => "%Y-%m-%d %I:%M %p",
    };
    timestamp
        .with_timezone(&Local)
        .format(format_str)
        .to_string()
}

fn confirmation_token_from_purchase(purchase_id: &PurchaseId) -> String {
    let id = purchase_id.as_str();
    let sanitized: String = id.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

    if sanitized.is_empty() {
        return String::new();
    }

    let len = sanitized.len();
    let start = len.saturating_sub(4);
    sanitized[start..].to_uppercase()
}

fn focus_and_select_input(input_ref: &NodeRef<html::Input>) {
    if let Some(input) = input_ref.get() {
        let _ = input.focus();
        let _ = input.select();
    }
}

fn get_local_storage() -> Option<web_sys::Storage> {
    let window = window()?;
    window.local_storage().ok().flatten()
}

fn load_saved_form_data() -> Option<(Option<String>, CheckoutFormData)> {
    let storage = get_local_storage()?;
    let raw = storage.get_item(CHECKOUT_DRAFT_STORAGE_KEY).ok()??;
    let parsed: StoredCheckoutForm = serde_json::from_str(&raw).ok()?;

    let mut items = Vec::with_capacity(parsed.items.len());
    for stored in parsed.items {
        let amount = Decimal::from_str(&stored.amount).ok()?;
        let added_at = DateTime::<Utc>::from_timestamp_millis(stored.added_at_ms)
            .unwrap_or_else(|| Utc::now());
        items.push(CheckoutItem {
            amount,
            vendor_id: stored.vendor_id,
            added_at,
        });
    }

    Some((
        parsed.booth_id,
        CheckoutFormData {
            vendor_id: parsed.vendor_id,
            current_amount: parsed.current_amount,
            items,
            ..Default::default()
        },
    ))
}

fn persist_form_data(booth_id: Option<String>, data: &CheckoutFormData) {
    let is_empty = data.vendor_id.trim().is_empty()
        && data.current_amount.trim().is_empty()
        && data.items.is_empty();

    if let Some(storage) = get_local_storage() {
        if is_empty {
            let _ = storage.remove_item(CHECKOUT_DRAFT_STORAGE_KEY);
            return;
        }

        let stored = StoredCheckoutForm {
            booth_id,
            vendor_id: data.vendor_id.clone(),
            current_amount: data.current_amount.clone(),
            items: data
                .items
                .iter()
                .map(|item| StoredCheckoutItem {
                    amount: item.amount.to_string(),
                    vendor_id: item.vendor_id.clone(),
                    added_at_ms: item.added_at.timestamp_millis(),
                })
                .collect(),
        };

        if let Ok(serialized) = serde_json::to_string(&stored) {
            let _ = storage.set_item(CHECKOUT_DRAFT_STORAGE_KEY, &serialized);
        }
    }
}

fn format_error_message<E: std::fmt::Debug>(error: &E) -> String {
    const MAX_LEN: usize = 140;
    let mut formatted = format!("{:?}", error).replace(['\n', '\r'], " ");

    if formatted.len() > MAX_LEN {
        formatted.truncate(MAX_LEN - 3);
        formatted.push_str("...");
    }

    formatted
}

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let app_state = use_app_state();
    let toast = use_toast();

    // Use global selected booth context
    let selected_booth = selected_booth_context::use_selected_booth();

    // Purchases for current booth
    let (purchases, set_purchases) = create_signal(Vec::<Purchase>::new());

    // Checkout form data
    let (_, initial_form_data) =
        load_saved_form_data().unwrap_or((None, CheckoutFormData::default()));
    let (form_data, set_form_data) = create_signal(initial_form_data);

    // Cancel confirmation modal
    let (show_cancel_modal, set_show_cancel_modal) = create_signal(false);

    // Delete confirmation modal state
    let (pending_deletion, set_pending_deletion) = create_signal(PendingDeletion::default());
    let (delete_confirmation_input, set_delete_confirmation_input) = create_signal(String::new());
    let delete_confirmation_ref = create_node_ref::<html::Input>();

    // Item deletion state - tracks which item (by index) is armed for deletion
    let (item_to_delete, set_item_to_delete) = create_signal::<Option<usize>>(None);

    // Purchase deletion state - tracks which purchase (by ID) is armed for deletion
    let (purchase_to_delete, set_purchase_to_delete) = create_signal::<Option<PurchaseId>>(None);

    // Transaction detail expansion state - tracks which purchase is expanded
    let (expanded_purchase_id, set_expanded_purchase_id) =
        create_signal::<Option<PurchaseId>>(None);

    let deletion_token_matches = create_memo(move |_| {
        let required = pending_deletion.get().token.trim().to_uppercase();

        if required.is_empty() {
            return false;
        }

        let entered = delete_confirmation_input.get().trim().to_uppercase();
        !entered.is_empty() && entered == required
    });

    // Persist form data anytime it changes
    {
        let form_data = form_data.clone();
        let selected_booth = selected_booth.clone();
        create_effect(move |_| {
            let data = form_data.get();
            let booth_id = selected_booth.get().map(|b| b.id.as_str());
            persist_form_data(booth_id, &data);
        });
    }

    // Input references for focus management
    let vendor_input_ref = create_node_ref::<html::Input>();
    let amount_input_ref = create_node_ref::<html::Input>();

    // Loading state
    let (is_loading, set_is_loading) = create_signal(true);

    // Vendor validation rule for current booth (changes when booth changes)
    let vendor_validation_rule = create_memo(move |_| {
        selected_booth.get().map(|booth| booth.vendor_id_validation.clone())
    });

    // Focus vendor input when view is ready and data is loaded
    {
        let vendor_input_ref = vendor_input_ref.clone();
        let is_loading = is_loading.clone();
        let selected_booth = selected_booth.clone();

        create_effect(move |_| {
            if !is_loading.get() && selected_booth.get().is_some() {
                if let Some(input) = vendor_input_ref.get() {
                    let _ = input.focus();
                    let _ = input.select();
                }
            }
        });
    }

    // Load purchases for selected booth
    create_effect(move |_| {
        let state_result = app_state.get();
        let booth = selected_booth.get();
        if let (Some(Ok(state)), Some(booth)) = (state_result, booth) {
            set_is_loading.set(true);
            let set_purchases = set_purchases.clone();
            let toast = toast.clone();
            let booth_id = booth.id.clone();
            spawn_local(async move {
                match state.purchase_repository.find_by_booth(&booth_id).await {
                    Ok(existing_purchases) => {
                        let mut sorted = existing_purchases;
                        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        set_purchases.set(sorted);
                    }
                    Err(e) => {
                        let error_msg = translate_with_params(
                            "checkout.errors.load_purchases_failed",
                            HashMap::from([("error", format_error_message(&e))]),
                        );
                        toast.error(&error_msg);
                    }
                }
                set_is_loading.set(false);
            });
        }
    });

    let filtered_purchases = Memo::new(move |_| {
        selected_booth
            .get()
            .map(|booth| {
                let mut list: Vec<Purchase> = purchases
                    .get()
                    .into_iter()
                    .filter(|purchase| purchase.booth_id == booth.id)
                    .collect();
                list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                list
            })
            .unwrap_or_default()
    });

    let running_totals = Memo::new(move |_| {
        let purchases_list = filtered_purchases.get();
        let total = purchases_list
            .iter()
            .map(|p| p.total_amount())
            .sum::<Decimal>();
        let item_count = purchases_list.iter().map(|p| p.items.len()).sum::<usize>();
        let checkout_count = purchases_list.len();

        (total, item_count, checkout_count)
    });

    // Form actions
    let vendor_input_ref_for_add = vendor_input_ref.clone();
    let amount_input_ref_for_add = amount_input_ref.clone();

    let add_item = move || {
        let mut data = form_data.get();
        let vendor_id_for_item = data.vendor_id.trim().to_string();

        if vendor_id_for_item != data.vendor_id {
            let message = t!("checkout.info.vendor_trimmed")();
            toast.info(&message);
            set_form_data.update(|form| form.vendor_id = vendor_id_for_item.clone());
            data.vendor_id = vendor_id_for_item.clone();
            if let Some(vendor_input) = vendor_input_ref_for_add.get() {
                vendor_input.set_value(&vendor_id_for_item);
            }
        }

        if vendor_id_for_item.is_empty() {
            let message = t!("checkout.errors.vendor_required")();
            toast.warning(&message);
            set_form_data.update(|form| form.vendor_error = Some(message));
            focus_and_select_input(&vendor_input_ref_for_add);
            return;
        }

        // Validate vendor ID against booth rules (if booth selected)
        if let Some(rule) = vendor_validation_rule.get() {
            if let Err(e) = validate_vendor_id(&vendor_id_for_item, &rule) {
                let error_msg = format!("{}", e);
                set_form_data.update(|form| form.vendor_error = Some(error_msg));
                focus_and_select_input(&vendor_input_ref_for_add);
                return;
            }
        }
        // If no booth selected, defer validation to server

        if data.current_amount.trim().is_empty() {
            let message = t!("checkout.errors.amount_required")();
            toast.warning(&message);
            set_form_data.update(|form| form.amount_error = Some(message));
            focus_and_select_input(&amount_input_ref_for_add);
            return;
        }

        let normalized_amount = data.current_amount.trim().to_string();

        match parse_decimal_input(&normalized_amount) {
            Ok(amount) => {
                if amount <= Decimal::ZERO {
                    let message = t!("checkout.errors.amount_positive")();
                    toast.warning(&message);
                    set_form_data.update(|form| {
                        form.amount_error = Some(message.clone());
                    });
                    focus_and_select_input(&amount_input_ref_for_add);
                    return;
                }

                let mut new_items = data.items.clone();
                new_items.insert(
                    0,
                    CheckoutItem {
                        amount,
                        vendor_id: vendor_id_for_item.clone(),
                        added_at: Utc::now(),
                    },
                );

                set_form_data.set(CheckoutFormData {
                    current_amount: String::new(),
                    items: new_items,
                    amount_error: None,
                    vendor_error: None,
                    ..data
                });

                if let Some(vendor_input) = vendor_input_ref_for_add.get() {
                    let _ = vendor_input.select();
                    let _ = vendor_input.focus();
                }

                if let Some(amount_input) = amount_input_ref_for_add.get() {
                    let _ = amount_input.set_value("");
                }
            }
            Err(_) => {
                let message = t!("checkout.errors.amount_invalid")();
                toast.error(&message);
                set_form_data.update(|form| form.amount_error = Some(message));
                focus_and_select_input(&amount_input_ref_for_add);
            }
        }
    };

    // Item deletion functions

    // Remove an item from the checkout list by index
    let remove_item = move |index: usize| {
        set_form_data.update(|data| {
            if index < data.items.len() {
                data.items.remove(index);
            }
        });
        set_item_to_delete.set(None);
    };

    // Handle clicking on an item or its overlay
    // First click: arm the item for deletion (show red overlay)
    // Second click (on overlay): delete the item
    let handle_item_click = move |index: usize| {
        if item_to_delete.get() == Some(index) {
            // Clicking armed overlay - delete the item
            remove_item(index);
        } else {
            // Clicking normal item - arm it for deletion
            set_item_to_delete.set(Some(index));
        }
    };

    // Cancel the armed deletion state
    let cancel_delete = move || {
        set_item_to_delete.set(None);
        set_purchase_to_delete.set(None);
    };

    let confirm_clear_form = move || {
        let items_present = !form_data.get().items.is_empty();
        if items_present {
            set_show_cancel_modal.set(true);
        } else {
            set_form_data.set(CheckoutFormData::default());
            let booth_id = selected_booth.get().map(|b| b.id.as_str());
            persist_form_data(booth_id, &CheckoutFormData::default());
        }
    };

    let handle_cancel_confirm = {
        let set_form_data = set_form_data.clone();
        let vendor_input_ref = vendor_input_ref.clone();
        let amount_input_ref = amount_input_ref.clone();
        move || {
            // Read current vendor_id from input field before clearing
            let current_vendor_id = if let Some(input) = vendor_input_ref.get() {
                input.value()
            } else {
                String::new()
            };

            let mut new_form = CheckoutFormData::default();
            new_form.vendor_id = current_vendor_id.clone();
            set_form_data.set(new_form);
            let booth_id = selected_booth.get().map(|b| b.id.as_str());
            persist_form_data(
                booth_id,
                &CheckoutFormData {
                    vendor_id: current_vendor_id.clone(),
                    ..Default::default()
                },
            );

            // Set focus appropriately
            if current_vendor_id.trim().is_empty() {
                focus_and_select_input(&vendor_input_ref);
            } else {
                focus_and_select_input(&amount_input_ref);
            }
        }
    };

    let submit_purchase = move || {
        let state_result = app_state.get();
        let booth = selected_booth.get();
        let mut data = form_data.get();

        if booth.is_none() {
            toast.warning(&t!("checkout.errors.booth_required")());
            return;
        }

        let booth = booth.unwrap();

        let trimmed_vendor_id = data.vendor_id.trim().to_string();
        if trimmed_vendor_id != data.vendor_id {
            let message = t!("checkout.info.vendor_trimmed")();
            toast.info(&message);
            let trimmed_clone = trimmed_vendor_id.clone();
            set_form_data.update(|form| form.vendor_id = trimmed_clone);
            data.vendor_id = trimmed_vendor_id;
        }

        if data.vendor_id.is_empty() {
            let message = t!("checkout.errors.vendor_required")();
            toast.warning(&message);
            set_form_data.update(|form| form.vendor_error = Some(message));
            focus_and_select_input(&vendor_input_ref_for_add);
            return;
        }

        if data.items.is_empty() {
            toast.warning(&t!("checkout.errors.items_required")());
            return;
        }

        // Create purchase items with vendor_id from each item
        let purchase_items: Vec<PurchaseItem> = data
            .items
            .into_iter()
            .map(|item| PurchaseItem::new(item.amount, VendorId::new(item.vendor_id)))
            .collect();

        let purchase = Purchase::new(booth.id.clone(), purchase_items);

        if let Some(Ok(state)) = state_result {
            let booth_id_clone = booth.id.clone();
            let purchase_clone = purchase.clone();
            let vendor_input_ref_clone = vendor_input_ref_for_add.clone();
            let amount_input_ref_clone = amount_input_ref_for_add.clone();
            spawn_local(async move {
                // Collect unique vendor IDs from all items
                let unique_vendor_ids: Vec<VendorId> = {
                    use std::collections::HashSet;
                    let mut ids: Vec<VendorId> = purchase_clone
                        .items
                        .iter()
                        .map(|item| item.vendor_id.clone())
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect();
                    ids.sort();
                    ids
                };

                // Get or create all vendors involved in this purchase
                for vendor_id in unique_vendor_ids {
                    let vendor_id_str = vendor_id.as_str().to_string();
                    match state
                        .vendor_service
                        .get_or_create(booth_id_clone.clone(), vendor_id_str)
                        .await
                    {
                        Ok(_vendor) => {
                            // Vendor successfully retrieved or created
                        }
                        Err(e) => {
                            let error_msg = translate_with_params(
                                "checkout.errors.vendor_create_failed",
                                HashMap::from([("error", format_error_message(&e))]),
                            );
                            toast.error(&error_msg);
                            return;
                        }
                    }
                }

                match state.purchase_repository.save(&purchase_clone).await {
                    Ok(_) => {
                        set_purchases.update(|list| {
                            list.push(purchase_clone.clone());
                            list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        });

                        // Clear input fields explicitly and focus vendor input for next checkout
                        if let Some(vendor_input) = vendor_input_ref_clone.get() {
                            let _ = vendor_input.set_value("");
                            let _ = vendor_input.focus();
                            let _ = vendor_input.select();
                        }
                        if let Some(amount_input) = amount_input_ref_clone.get() {
                            let _ = amount_input.set_value("");
                        }

                        toast.success(&t!("checkout.success")());
                        set_form_data.set(CheckoutFormData::default());
                        let booth_id = selected_booth.get().map(|b| b.id.as_str());
                        persist_form_data(booth_id, &CheckoutFormData::default());
                    }
                    Err(e) => {
                        let error_msg = translate_with_params(
                            "checkout.errors.save_purchase_failed",
                            HashMap::from([("error", format_error_message(&e))]),
                        );
                        toast.error(&error_msg);
                    }
                }
            });
        }
    };

    let delete_purchase = move |purchase_id| {
        let token = confirmation_token_from_purchase(&purchase_id);
        set_pending_deletion.set(PendingDeletion {
            purchase_id: Some(purchase_id),
            token,
        });
        set_delete_confirmation_input.set(String::new());
        spawn_local(async move {
            if let Some(input) = delete_confirmation_ref.get() {
                let _ = input.set_value("");
                let _ = input.focus();
                let _ = input.select();
            }
        });
    };

    // Handle clicking on a purchase or its overlay
    // First click: arm the purchase for deletion (show red overlay)
    // Second click (on overlay): trigger deletion modal with token confirmation
    let handle_purchase_click = move |purchase_id: PurchaseId| {
        if purchase_to_delete.get() == Some(purchase_id) {
            // Clicking armed overlay - trigger deletion modal
            delete_purchase(purchase_id);
        } else {
            // Clicking normal purchase - arm it for deletion
            set_purchase_to_delete.set(Some(purchase_id));
        }
    };

    // Handle clicking on transaction card to expand/collapse details
    let handle_transaction_detail_click = move |purchase_id: PurchaseId| {
        // Priority 1: Don't interfere with armed deletion state
        if purchase_to_delete.get() == Some(purchase_id) {
            return; // Let deletion overlay handle the click
        }

        // Priority 2: Toggle expansion (accordion behavior)
        let current = expanded_purchase_id.get();
        if current == Some(purchase_id) {
            set_expanded_purchase_id.set(None); // Collapse if already expanded
        } else {
            set_expanded_purchase_id.set(Some(purchase_id)); // Expand, auto-collapsing any other
        }
    };

    let perform_delete_purchase = {
        let app_state = app_state.clone();
        let set_purchases = set_purchases.clone();
        let toast = toast.clone();
        move || {
            let pending = pending_deletion.get();
            if pending.purchase_id.is_none() {
                return;
            }

            let purchase_id = pending.purchase_id.unwrap();
            let state_result = app_state.get();

            if let Some(Ok(state)) = state_result {
                spawn_local(async move {
                    if let Err(e) = state.purchase_repository.delete(&purchase_id).await {
                        let error_msg = translate_with_params(
                            "checkout.errors.delete_purchase_failed",
                            HashMap::from([("error", format_error_message(&e))]),
                        );
                        toast.error(&error_msg);
                    }
                });

                set_purchases.update(|list| {
                    list.retain(|p| p.id != purchase_id);
                    list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                });
            }

            set_pending_deletion.set(PendingDeletion::default());
            set_delete_confirmation_input.set(String::new());
        }
    };

    let cancel_delete_purchase = {
        let set_pending_deletion = set_pending_deletion.clone();
        let set_delete_confirmation_input = set_delete_confirmation_input.clone();
        let set_purchase_to_delete = set_purchase_to_delete.clone();
        move || {
            set_pending_deletion.set(PendingDeletion::default());
            set_delete_confirmation_input.set(String::new());
            set_purchase_to_delete.set(None);
        }
    };

    view! {
        <Container>
            <div class="space-y-6" on:click=move |_| cancel_delete()>
                <div class="flex flex-col gap-6 lg:flex-row">
                    <div class="flex-1 space-y-6">
                        <Card title_view={t!("checkout.title").into_view()}>
                            <Show
                                when=move || selected_booth.get().is_none()
                                fallback=move || view! {
                                    <Show
                                        when=move || is_loading.get()
                                        fallback=move || view! {
                                            <div class="space-y-6">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        {t!("checkout.vendor_id")}
                                                    </label>
                                                    <input
                                                        class={move || format!(
                                                            "w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 {}",
                                                            if form_data.get().vendor_error.is_some() {
                                                                "border-red-500 focus:ring-red-500"
                                                            } else {
                                                                "border-gray-300"
                                                            }
                                                        )}
                                                        placeholder={t!("checkout.vendor_placeholder")}
                                                        value=move || form_data.get().vendor_id
                                                        node_ref=vendor_input_ref
                                                    on:input=move |ev| {
                                                            cancel_delete();
                                                            let value = event_target_value(&ev);
                                                            let trimmed = value.trim().to_string();
                                                            
                                                            // Update vendor_id with raw value
                                                            set_form_data.update(|data| {
                                                                data.vendor_id = value.clone();
                                                            });
                                                            
                                                            // Validate against booth rules
                                                            if trimmed.is_empty() {
                                                                // Clear error if empty (required check happens on add/submit)
                                                                set_form_data.update(|data| data.vendor_error = None);
                                                            } else if let Some(rule) = vendor_validation_rule.get() {
                                                                // Validate using domain validation
                                                                match validate_vendor_id(&trimmed, &rule) {
                                                                    Ok(()) => {
                                                                        set_form_data.update(|data| data.vendor_error = None);
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("{}", e);
                                                                        set_form_data.update(|data| {
                                                                            data.vendor_error = Some(error_msg);
                                                                        });
                                                                    }
                                                                }
                                                            } else {
                                                                // No booth selected - clear error (will be caught on submit)
                                                                set_form_data.update(|data| data.vendor_error = None);
                                                            }
                                                        }
                                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                            if ev.key() == "Enter" {
                                                                ev.prevent_default();

                                                                let current_value = form_data.get().vendor_id;
                                                                let trimmed = current_value.trim().to_string();

                                                                if trimmed != current_value {
                                                                    let message = t!("checkout.info.vendor_trimmed")();
                                                                    toast.info(&message);
                                                                    set_form_data.update(|data| data.vendor_id = trimmed.clone());
                                                                    if let Some(input) = vendor_input_ref.get() {
                                                                        input.set_value(&trimmed);
                                                                    }
                                                                }

                                                                if trimmed.is_empty() {
                                                                    let message = t!("checkout.errors.vendor_required")();
                                                                    toast.warning(&message);
                                                                    set_form_data.update(|data| data.vendor_error = Some(message));
                                                                    if let Some(input) = vendor_input_ref.get() {
                                                                        let _ = input.focus();
                                                                        let _ = input.select();
                                                                    }
                                                                } else {
                                                                    // Validate against booth rules before advancing
                                                                    let is_valid = if let Some(rule) = vendor_validation_rule.get() {
                                                                        validate_vendor_id(&trimmed, &rule).is_ok()
                                                                    } else {
                                                                        // No booth selected - treat as invalid
                                                                        false
                                                                    };
                                                                    
                                                                    if is_valid {
                                                                        // Valid: clear error and advance to amount field
                                                                        set_form_data.update(|data| data.vendor_error = None);
                                                                        if let Some(amount_input) = amount_input_ref.get() {
                                                                            let _ = amount_input.focus();
                                                                            let _ = amount_input.select();
                                                                        }
                                                                    } else {
                                                                        // Invalid: keep focus on vendor, select text, ensure error is set
                                                                        if let Some(rule) = vendor_validation_rule.get() {
                                                                            if let Err(e) = validate_vendor_id(&trimmed, &rule) {
                                                                                let error_msg = format!("{}", e);
                                                                                set_form_data.update(|data| {
                                                                                    data.vendor_error = Some(error_msg);
                                                                                });
                                                                            }
                                                                        }
                                                                        if let Some(input) = vendor_input_ref.get() {
                                                                            let _ = input.focus();
                                                                            let _ = input.select();
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        on:blur=move |_| {
                                                            // Auto-trim and re-validate on blur
                                                            let current_value = form_data.get().vendor_id;
                                                            let trimmed = current_value.trim().to_string();
                                                            
                                                            if trimmed != current_value {
                                                                set_form_data.update(|data| data.vendor_id = trimmed.clone());
                                                                if let Some(input) = vendor_input_ref.get() {
                                                                    input.set_value(&trimmed);
                                                                }
                                                            }
                                                            
                                                            // Re-validate after trimming
                                                            if trimmed.is_empty() {
                                                                set_form_data.update(|data| data.vendor_error = None);
                                                            } else if let Some(rule) = vendor_validation_rule.get() {
                                                                match validate_vendor_id(&trimmed, &rule) {
                                                                    Ok(()) => {
                                                                        set_form_data.update(|data| data.vendor_error = None);
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("{}", e);
                                                                        set_form_data.update(|data| {
                                                                            data.vendor_error = Some(error_msg);
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    />
                                                    <Show when=move || form_data.get().vendor_error.is_some()>
                                                        <p class="mt-1 text-sm text-red-600">{move || form_data.get().vendor_error.clone().unwrap_or_default()}</p>
                                                    </Show>
                                                </div>

                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        {t!("checkout.amount")}
                                                    </label>
                                                    <div class="flex flex-col gap-2">
                                                        <input
                                                            class={move || format!(
                                                                "flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 {}",
                                                                if form_data.get().amount_error.is_some() {
                                                                    "border-red-500 focus:ring-red-500"
                                                                } else {
                                                                    "border-gray-300"
                                                                }
                                                            )}
                                                            placeholder={t!("checkout.amount_placeholder")}
                                                            inputmode="decimal"
                                                            value=move || form_data.get().current_amount
                                                            node_ref=amount_input_ref
                                                            on:input=move |ev| {
                                                                cancel_delete();
                                                                let value = event_target_value(&ev);
                                                                set_form_data.update(|data| {
                                                                    data.current_amount = value;
                                                                    data.amount_error = None;
                                                                });
                                                            }
                                                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                                if ev.key() == "Enter" {
                                                                    ev.prevent_default();
                                                                    add_item();
                                                                }
                                                            }
                                                        />
                                                        <Show when=move || form_data.get().amount_error.is_some()>
                                                            <p class="text-sm text-red-600">{move || form_data.get().amount_error.clone().unwrap_or_default()}</p>
                                                        </Show>
                                                    </div>
                                                </div>

                                                {/* Action buttons - side by side on desktop, stacked on mobile */}
                                                <div class="flex flex-col sm:flex-row gap-4">
                                                    <Button
                                                        variant=ButtonVariant::Secondary
                                                        class="sm:flex-[3]".to_string()
                                                        on_click=Box::new(move || {
                                                            cancel_delete();
                                                            add_item();
                                                        })
                                                    >
                                                        {t!("checkout.add_item")}
                                                    </Button>

                                                    <Button
                                                        variant=ButtonVariant::Success
                                                        class="sm:flex-[7] shadow-lg ring-2 ring-green-300/50".to_string()
                                                        on_click=Box::new(move || {
                                                            cancel_delete();
                                                            submit_purchase();
                                                        })
                                                    >
                                                        <span class="inline-flex items-center justify-center gap-4">
                                                        <svg
                                                            class="w-8 h-8"
                                                            viewBox="0 0 32 32"
                                                            xmlns="http://www.w3.org/2000/svg"
                                                            aria-hidden="true"
                                                        >
                                                            <path
                                                                fill="currentColor"
                                                                d="M28.918,15.952c-0.051-0.378-0.272-0.697-0.592-0.902c-0.212-0.136-0.475-0.137-0.697-0.042 c-0.003-0.816,0-1.632-0.021-2.447c-0.021-0.915,0.029-1.829,0.063-2.742c0.012-0.301-0.181-0.561-0.44-0.689 c-0.023-0.344-0.047-0.688-0.077-1.031c-0.019-0.205-0.036-0.41-0.065-0.613c-0.006-0.04-0.011-0.08-0.017-0.12 c-0.015-0.106-0.063-0.289-0.123-0.37c-0.087-0.114-0.161-0.241-0.296-0.306c-0.099-0.047-0.194-0.093-0.302-0.108 c-0.105-0.016-0.213-0.016-0.319-0.028c-0.023-0.337-0.041-0.674-0.056-1.011c0.038-0.087,0.059-0.181,0.059-0.279 c0-0.395-0.329-0.718-0.72-0.722c-0.805-0.006-1.61,0.006-2.415-0.028c-0.724-0.03-1.447-0.072-2.174-0.087 c-1.527-0.028-3.057-0.017-4.586-0.028c-1.5-0.01-2.998-0.04-4.501-0.044c-0.733-0.001-1.466-0.015-2.198-0.015 c-0.706,0-1.413,0.013-2.118,0.062C6.545,4.457,5.757,4.635,5.041,4.947C4.342,5.249,3.714,5.851,3.497,6.593 c-0.059,0.203-0.089,0.41-0.116,0.621C3.358,7.385,3.338,7.56,3.358,7.732c0.025,0.215,0.06,0.423,0.117,0.629 C3.24,8.48,3.075,8.72,3.072,8.997c-0.01,0.798-0.006,1.595-0.021,2.391c-0.013,0.724-0.03,1.447-0.038,2.169 c-0.013,1.462-0.021,2.922,0,4.385c0.015,1.016,0.038,2.032,0.042,3.048c0.006,1.02,0.019,2.041-0.004,3.063 c-0.003,0.127,0.042,0.241,0.104,0.345c0.151,0.39,0.309,0.782,0.514,1.146c0.084,0.148,0.19,0.285,0.292,0.422 c0.095,0.125,0.192,0.249,0.289,0.372c0.148,0.188,0.37,0.336,0.568,0.463c0.442,0.289,0.957,0.473,1.476,0.564 c0.45,0.078,0.904,0.154,1.36,0.182c0.828,0.049,1.66,0.055,2.491,0.07c0.79,0.013,1.58,0.028,2.368,0.027 c0.805,0,1.608-0.009,2.412-0.004c0.807,0.008,1.612,0.032,2.419,0.015c0.832-0.015,1.662-0.049,2.493-0.061 c0.799-0.013,1.603-0.009,2.402-0.009c0.763,0,1.527,0,2.288-0.01c0.357-0.004,0.714,0,1.073,0.004 c0.376,0.004,0.752,0.008,1.13,0.002c0.257-0.003,0.481-0.131,0.633-0.319c0.193-0.138,0.33-0.351,0.325-0.6 c-0.017-0.86-0.008-1.72-0.017-2.581c-0.013-0.988,0.002-1.974,0.019-2.961c0.237-0.036,0.449-0.105,0.646-0.27 c0.439-0.368,0.543-0.999,0.589-1.538c0.029-0.334,0.029-0.67,0.042-1.006c0.021-0.513,0.046-1.031,0.028-1.546 C28.985,16.49,28.954,16.22,28.918,15.952z M7.703,8.835C7.706,8.762,7.71,8.688,7.694,8.613C7.69,8.594,7.687,8.574,7.683,8.555 c-0.02-0.306-0.01-0.614-0.007-0.922c1.463,0.042,2.927,0.043,4.392,0.05c0.754,0.004,1.508,0.023,2.262,0.042 c0.699,0.019,1.396,0.07,2.095,0.093c0.727,0.025,1.456,0.036,2.186,0.051c0.729,0.015,1.46,0.03,2.189,0.036 c0.742,0.006,1.485,0.013,2.226,0.015c0.653,0.002,1.307,0.03,1.96,0.034c0.242,0.001,0.483,0.01,0.723,0.028 c0.04,0.353,0.066,0.707,0.077,1.063c-0.389,0.008-0.779,0.004-1.166,0c-0.727-0.01-1.455-0.032-2.182-0.036 c-1.46-0.008-2.917-0.057-4.377-0.068c-1.553-0.011-3.105,0.002-4.658-0.053c-0.999-0.034-2-0.032-2.998-0.03 c-0.534,0-1.067,0.002-1.599-0.004C8.438,8.849,8.07,8.844,7.703,8.835z M27.418,19.574L27.418,19.574 c0.001-0.001,0.003-0.003,0.005-0.005C27.422,19.571,27.42,19.573,27.418,19.574z M27.224,16.425 c0.052,0.004,0.103,0.001,0.152-0.007c0.03,0.439,0.025,0.879,0.017,1.32c-0.004,0.279-0.009,0.558-0.015,0.839 c-0.003,0.273,0.001,0.546-0.024,0.817c-0.012,0.063-0.026,0.125-0.043,0.187c-0.277,0.014-0.557,0.005-0.833-0.002 c-0.336-0.008-0.672-0.011-1.008-0.011c-0.745,0-1.493,0.002-2.232-0.082c-0.225-0.04-0.443-0.097-0.657-0.176 c-0.215-0.102-0.412-0.22-0.603-0.358c-0.072-0.063-0.141-0.128-0.207-0.197c-0.04-0.062-0.075-0.125-0.109-0.19 c-0.035-0.094-0.067-0.189-0.093-0.288c-0.016-0.223-0.009-0.441,0.015-0.663c0.04-0.238,0.089-0.474,0.171-0.702 c0.027-0.053,0.056-0.103,0.087-0.152c0.038-0.038,0.076-0.075,0.116-0.109c0.076-0.047,0.154-0.09,0.235-0.13 c0.197-0.071,0.397-0.125,0.605-0.162c0.325-0.033,0.649-0.04,0.976-0.048c0.349-0.01,0.699-0.023,1.048-0.023 C25.624,16.284,26.427,16.364,27.224,16.425z M4.895,7.3c0.023-0.124,0.051-0.245,0.091-0.364C5.014,6.88,5.045,6.827,5.078,6.775 c0.087-0.098,0.18-0.19,0.279-0.276c0.127-0.087,0.259-0.159,0.399-0.226C6.231,6.092,6.73,5.979,7.23,5.903 C7.913,5.83,8.593,5.797,9.279,5.792c0.725-0.004,1.451-0.011,2.176-0.013c0.676,0,1.352,0.004,2.03,0.008 c0.811,0.004,1.624,0.009,2.436,0.004c1.523-0.009,3.046-0.013,4.569,0.017c0.729,0.015,1.456,0.03,2.186,0.078 c0.601,0.042,1.203,0.056,1.805,0.072c0.014,0.177,0.026,0.355,0.046,0.533c-0.067,0-0.134-0.002-0.201-0.001 c-1.424,0.009-2.848,0.03-4.273,0.009c-0.754-0.009-1.508-0.025-2.26-0.049c-0.712-0.021-1.422-0.027-2.133-0.055 c-0.727-0.029-1.453-0.059-2.18-0.068c-0.727-0.011-1.455-0.01-2.182-0.019c-1.356-0.021-2.712-0.07-4.066-0.127 c-0.009,0-0.018-0.001-0.027-0.001c-0.35,0-0.631,0.296-0.678,0.631c-0.211,0.135-0.361,0.359-0.36,0.623 c0.002,0.44,0.01,0.883,0.066,1.321C6.203,8.749,6.171,8.75,6.139,8.746C5.907,8.71,5.677,8.671,5.454,8.597 C5.382,8.56,5.314,8.519,5.246,8.473C5.21,8.441,5.177,8.408,5.145,8.372C5.091,8.285,5.046,8.196,5.004,8.104 C4.954,7.972,4.915,7.84,4.886,7.702C4.877,7.567,4.882,7.435,4.895,7.3z M24.46,25.97c-0.777,0.019-1.555,0.017-2.332,0.034 c-1.662,0.032-3.323,0.089-4.987,0.101c-0.771,0.004-1.544,0.021-2.315,0.015c-0.82-0.004-1.639-0.015-2.457-0.017 c-0.773-0.002-1.546,0.008-2.319,0.011c-0.832,0.002-1.669,0.006-2.497-0.076c-0.066-0.007-0.132-0.013-0.198-0.02 c-0.451-0.064-0.905-0.144-1.331-0.302c-0.17-0.084-0.334-0.177-0.49-0.284c-0.108-0.095-0.203-0.198-0.294-0.308 c-0.08-0.106-0.158-0.215-0.231-0.328c-0.079-0.126-0.141-0.265-0.201-0.402c-0.069-0.17-0.135-0.342-0.2-0.515 c-0.039-0.103-0.087-0.194-0.148-0.275c-0.01-1.288-0.01-2.575-0.027-3.861c-0.011-0.714-0.025-1.43-0.034-2.144 c-0.011-0.733-0.002-1.468-0.004-2.201C4.395,14.349,4.393,13.3,4.42,12.25c0.02-0.816,0.045-1.631,0.067-2.447 c0.048,0.03,0.095,0.063,0.143,0.09c0.399,0.22,0.845,0.275,1.289,0.329c0.372,0.046,0.746,0.066,1.122,0.08 c0.746,0.027,1.495,0.017,2.243,0.017c0.788,0,1.574-0.009,2.362-0.013c0.775-0.006,1.546,0.03,2.32,0.055 c0.746,0.023,1.493,0.023,2.241,0.023c0.775-0.002,1.551-0.008,2.326-0.006c0.76,0.002,1.519,0.047,2.277,0.093 c1.424,0.084,2.854,0.089,4.28,0.106c0.337,0.004,0.679,0,1.02-0.014c-0.021,1.427-0.002,2.855,0.004,4.282 c-0.467-0.022-0.934-0.055-1.402-0.06c-0.359-0.004-0.718-0.015-1.075-0.021c-0.036-0.001-0.071-0.001-0.107-0.001 c-0.398,0-0.791,0.038-1.186,0.09c-0.429,0.059-0.866,0.234-1.234,0.46c-0.275,0.169-0.469,0.395-0.663,0.648 c-0.122,0.159-0.194,0.384-0.262,0.566c-0.074,0.196-0.114,0.406-0.154,0.612c-0.103,0.528-0.152,1.063-0.042,1.595 c0.027,0.133,0.08,0.262,0.129,0.389c0.122,0.317,0.287,0.579,0.515,0.832c0.18,0.201,0.422,0.357,0.642,0.511 c0.142,0.099,0.298,0.18,0.456,0.253c0.224,0.101,0.456,0.207,0.697,0.266c0.427,0.106,0.873,0.141,1.312,0.165 c0.492,0.027,0.987,0.023,1.483,0.019c0.209-0.002,0.42-0.004,0.63-0.004c0.088,0,0.175,0.001,0.263,0.002 c-0.011,1.592,0.04,3.183,0.053,4.774C25.6,25.947,25.03,25.957,24.46,25.97z M24.132,17.865c0,0.431-0.359,0.791-0.791,0.791 c-0.431,0-0.791-0.359-0.791-0.791c0-0.431,0.359-0.791,0.791-0.791C23.772,17.074,24.132,17.434,24.132,17.865z"
                                                            />
                                                        </svg>
                                                            <span class="text-2xl font-semibold">{move || {
                                                                let locale = use_locale().get();
                                                                format_currency(form_data.get().total(), locale)
                                                            }}</span>
                                                        </span>
                                                    </Button>
                                                </div>
                                            </div>
                                        }
                                    >
                                        <p class="text-gray-600">{t!("checkout.loading_message")}</p>
                                    </Show>
                                }
                            >
                                <p class="text-gray-600">{t!("checkout.prompt_select_booth")}</p>
                            </Show>
                        </Card>

                        <Card>
                            {/* Custom header with title and Clear Items button */}
                            <div class="flex items-center justify-between mb-4">
                                <h2 class="text-xl font-semibold">{t!("checkout.current_items")}</h2>
                                <Show when=move || !form_data.get().items.is_empty()>
                                    <Button
                                        variant=ButtonVariant::Danger
                                        on_click=Box::new(move || confirm_clear_form())
                                        class="px-3 py-2 !bg-red-100 hover:!bg-red-200 !text-red-700".to_string()
                                        aria_label={t!("checkout.confirm_cancel_confirm")()}
                                    >
                                        {/* Trash icon */}
                                        <svg
                                            class="w-6 h-6"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            aria-hidden="true"
                                        >
                                            <polyline points="3 6 5 6 21 6" />
                                            <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
                                            <path d="M10 11v6" />
                                            <path d="M14 11v6" />
                                            <path d="M9 6V4a2 2 0 012-2h2a2 2 0 012 2v2" />
                                        </svg>
                                    </Button>
                                </Show>
                            </div>
                            <div class="space-y-2">
                                <Show
                                    when=move || form_data.get().items.is_empty()
                                    fallback=move || {
                                        let data = form_data.get();
                                        let items = data.items;
                                        let total_items = items.len();
                                        view! {
                                            {/* Explanatory hint text */}
                                            <p class="text-xs text-gray-500 mb-3 px-1">
                                                {t!("checkout.items_list_hint")}
                                            </p>

                                            <ul class="space-y-2">
                                                {items.into_iter().enumerate().map(move |(index, item)| {
                                                    let display_number = total_items - index;
                                                    let vendor_label = if item.vendor_id.trim().is_empty() {
                                                        "—".to_string()
                                                    } else {
                                                        item.vendor_id.clone()
                                                    };
                                                    view! {
                                                        <li
                                                            class="relative text-sm p-2 border rounded-lg bg-gray-50
                                                                   cursor-pointer hover:bg-gray-100 transition-colors select-none"
                                                            on:click=move |e| {
                                                                e.stop_propagation();
                                                                handle_item_click(index);
                                                            }
                                                        >
                                                            {/* Item content - pointer-events-none to make entire item the click target */}
                                                            <div class="flex items-start justify-between pointer-events-none">
                                                                <div>
                                                                    <p class="font-medium">{format!("{} {}", t!("checkout.item_label")(), display_number)}</p>
                                                                    <p class="text-xs text-gray-500">{format!("{} {}", t!("checkout.vendor_label")(), vendor_label)}</p>
                                                                </div>
                                                                <div class="text-right">
                                                                    <span class="block font-semibold">{
                                                                        let locale = use_locale().get();
                                                                        format_currency(item.amount, locale)
                                                                    }</span>
                                                                    <p class="text-xs text-gray-400" title={
                                                                        let locale = use_locale().get();
                                                                        format_item_tooltip(item.added_at, locale)
                                                                    }>{
                                                                        let locale = use_locale().get();
                                                                        format!("{}", format_item_timestamp(item.added_at, locale))
                                                                    }</p>
                                                                </div>
                                                            </div>

                                                            {/* RED OVERLAY - shown when item is armed for deletion */}
                                                            <Show when=move || item_to_delete.get() == Some(index)>
                                                                <div
                                                                    class="absolute inset-0 rounded-lg cursor-pointer z-10 transition-all pointer-events-auto"
                                                                    style="background: rgba(220, 38, 38, 0.7); backdrop-filter: blur(2px);"
                                                                    on:click=move |e| {
                                                                        e.stop_propagation();
                                                                        handle_item_click(index)
                                                                    }
                                                                    role="alertdialog"
                                                                    aria-label={t!("checkout.remove_item_confirm")}
                                                                >
                                                                    <div class="flex items-center justify-center gap-3 h-full">
                                                                        {/* Trash icon */}
                                                                        <svg
                                                                            class="w-8 h-8 text-white flex-shrink-0"
                                                                            viewBox="0 0 24 24"
                                                                            fill="none"
                                                                            stroke="currentColor"
                                                                            stroke-width="2"
                                                                            stroke-linecap="round"
                                                                            stroke-linejoin="round"
                                                                            aria-hidden="true"
                                                                        >
                                                                            <polyline points="3 6 5 6 21 6" />
                                                                            <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
                                                                            <path d="M10 11v6" />
                                                                            <path d="M14 11v6" />
                                                                            <path d="M9 6V4a2 2 0 012-2h2a2 2 0 012 2v2" />
                                                                        </svg>

                                                                        {/* Confirmation text */}
                                                                        <p class="text-white text-base font-semibold">
                                                                            {t!("checkout.remove_item_confirm")}
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                            </Show>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        }
                                    }
                                >
                                    <p class="text-gray-500 text-sm">{t!("checkout.no_items")}</p>
                                </Show>
                            </div>
                        </Card>
                    </div>

                    <div class="flex-1 space-y-6">
                        <Card title_view={t!("checkout.running_totals_title").into_view()}>
                            <div class="space-y-4">
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{t!("checkout.running_totals.sales")}</span>
                                    <span class="text-lg font-semibold">{move || {
                                        let locale = use_locale().get();
                                        format_currency(running_totals.get().0, locale)
                                    }}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{t!("checkout.running_totals.items")}</span>
                                    <span class="text-lg font-semibold">{move || running_totals.get().1.to_string()}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{t!("checkout.running_totals.checkouts")}</span>
                                    <span class="text-lg font-semibold">{move || running_totals.get().2.to_string()}</span>
                                </div>
                            </div>
                        </Card>

                        <Card title_view={t!("checkout.recent_transactions_title").into_view()}>
                            <Show
                                when=move || filtered_purchases.get().is_empty()
                                fallback=move || {
                                    let list = filtered_purchases.get();
                                    view! {
                                        <div class="space-y-2">
                                            {/* Explanatory hint text */}
                                            <p class="text-xs text-gray-500 mb-3 px-1">
                                                {t!("checkout.transactions_hint")}
                                            </p>

                                            {list.into_iter().map(|purchase| {
                                                let amount = purchase.total_amount();
                                                let purchase_id = purchase.id;
                                                let purchase_id_label = purchase_id.as_str().to_string();
                                                let item_count = purchase.items.len();
                                                let items_label = if item_count == 1 {
                                                    t!("checkout.recent.items_label_one")()
                                                } else {
                                                    translate_with_params(
                                                        "checkout.recent.items_label",
                                                        HashMap::from([(
                                                            "count",
                                                            item_count.to_string(),
                                                        )]),
                                                    )
                                                };
                                                view! {
                                                    <div
                                                        class=move || format!(
                                                            "relative group text-sm border rounded-lg bg-gray-50 transition-all duration-300 select-none {}",
                                                            if expanded_purchase_id.get() == Some(purchase_id) {
                                                                "border-blue-500 shadow-sm" // Active state: blue border
                                                            } else {
                                                                "border-gray-200 hover:border-blue-300" // Default/hover state
                                                            }
                                                        )
                                                    >
                                                        {/* Card header - clickable area */}
                                                        <div
                                                            class=move || format!(
                                                                "p-3 transition-colors duration-300 cursor-pointer {}",
                                                                if expanded_purchase_id.get() == Some(purchase_id) {
                                                                    "bg-blue-50" // Active state: blue background
                                                                } else {
                                                                    ""
                                                                }
                                                            )
                                                            on:click=move |e| {
                                                                e.stop_propagation();
                                                                handle_transaction_detail_click(purchase_id);
                                                            }
                                                        >

                                                            {/* Purchase content */}
                                                            <div class="flex items-center justify-between pr-12">
                                                                <div class="space-y-1">
                                                                    <p class="text-sm font-semibold">{items_label.clone()}</p>
                                                                     <p class="text-xs text-gray-500">
                                                                        {let locale = use_locale().get();
                                                                        translate_with_params(
                                                                            "checkout.recent.timestamp",
                                                                            HashMap::from([(
                                                                                "datetime",
                                                                                format_purchase_timestamp(purchase.timestamp, locale)
                                                                            )])
                                                                        )
                                                                        }
                                                                     </p>
                                                                    <p class="text-xs text-gray-500 font-mono break-all">
                                                                        {translate_with_params(
                                                                            "checkout.recent.purchase_id",
                                                                            HashMap::from([(
                                                                                "id",
                                                                                purchase_id_label.clone()
                                                                            )])
                                                                        )}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <span class="text-lg font-bold">{
                                                                        let locale = use_locale().get();
                                                                        format_currency(amount, locale)
                                                                    }</span>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        {/* Expanded detail section */}
                                                        <Show when=move || expanded_purchase_id.get() == Some(purchase_id)>
                                                            <div
                                                                class="pl-6 pr-14 pb-4 pt-3 space-y-3 animate-in slide-in-from-top-2 duration-300"
                                                                style="border-top: 1px dashed #e5e7eb;" // Subtle dashed separator
                                                            >
                                                                {/* Items list with vendor per item */}
                                                                <div class="space-y-2">
                                                                    {
                                                                        let vendor_label = format!("{}: ", t!("checkout.vendor_label")());
                                                                        purchase.items.iter().enumerate().map(|(idx, item)| {
                                                                            let position_num = idx + 1;
                                                                            let locale = use_locale().get();
                                                                            let vendor_text = format!("{}{}", vendor_label, item.vendor_id.as_str());
                                                                            view! {
                                                                                <div class="py-2 border-b border-gray-100 last:border-0">
                                                                                    {/* First line: Item number and amount */}
                                                                                    <div class="flex justify-between text-sm">
                                                                                        <span class="font-medium text-gray-900">
                                                                                            {translate_with_params(
                                                                                                "checkout.transaction_detail.item_number",
                                                                                                HashMap::from([("number", position_num.to_string())])
                                                                                            )}
                                                                                        </span>
                                                                                        <span class="font-medium text-gray-900">
                                                                                            {format_currency(item.amount, locale)}
                                                                                        </span>
                                                                                    </div>

                                                                                    {/* Second line: Vendor ID */}
                                                                                    <div class="text-xs text-gray-500 mt-0.5">
                                                                                        {vendor_text}
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        }).collect_view()
                                                                    }
                                                                </div>
                                                            </div>
                                                        </Show>

                                                        {/* Separator + Delete Icon (hover-visible on desktop, always visible on mobile) */}
                                                        <div
                                                            class="absolute right-0 top-0 h-full flex items-center
                                                                   opacity-0 group-hover:opacity-100 sm:opacity-100 
                                                                   transition-opacity cursor-pointer"
                                                            on:click=move |e| {
                                                                e.stop_propagation();
                                                                handle_purchase_click(purchase_id);
                                                            }
                                                            aria-label={t!("checkout.confirm_cancel_confirm")}
                                                        >
                                                            {/* Vertical separator */}
                                                            <div class="h-full w-px bg-gray-300"></div>

                                                            {/* Delete icon area */}
                                                            <div class="px-3 py-2 hover:bg-red-50 transition-colors h-full flex items-center">
                                                                <svg
                                                                    class="w-5 h-5 text-gray-400 hover:text-red-600 transition-colors"
                                                                    viewBox="0 0 24 24"
                                                                    fill="none"
                                                                    stroke="currentColor"
                                                                    stroke-width="2"
                                                                    stroke-linecap="round"
                                                                    stroke-linejoin="round"
                                                                    aria-hidden="true"
                                                                >
                                                                    <polyline points="3 6 5 6 21 6" />
                                                                    <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
                                                                    <path d="M10 11v6" />
                                                                    <path d="M14 11v6" />
                                                                    <path d="M9 6V4a2 2 0 012-2h2a2 2 0 012 2v2" />
                                                                </svg>
                                                            </div>
                                                        </div>

                                                        {/* RED OVERLAY - shown when purchase is armed for deletion */}
                                                        <Show when=move || purchase_to_delete.get() == Some(purchase_id)>
                                                            <div
                                                                class="absolute inset-0 rounded-lg cursor-pointer z-10 transition-all pointer-events-auto"
                                                                style="background: rgba(220, 38, 38, 0.7); backdrop-filter: blur(2px);"
                                                                on:click=move |e| {
                                                                    e.stop_propagation();
                                                                    handle_purchase_click(purchase_id)
                                                                }
                                                                role="alertdialog"
                                                                aria-label={t!("checkout.remove_transaction_confirm")}
                                                            >
                                                                <div class="flex items-center justify-center gap-3 h-full">
                                                                    {/* Trash icon */}
                                                                    <svg
                                                                        class="w-8 h-8 text-white flex-shrink-0"
                                                                        viewBox="0 0 24 24"
                                                                        fill="none"
                                                                        stroke="currentColor"
                                                                        stroke-width="2"
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        aria-hidden="true"
                                                                    >
                                                                        <polyline points="3 6 5 6 21 6" />
                                                                        <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
                                                                        <path d="M10 11v6" />
                                                                        <path d="M14 11v6" />
                                                                        <path d="M9 6V4a2 2 0 012-2h2a2 2 0 012 2v2" />
                                                                    </svg>

                                                                    {/* Confirmation text */}
                                                                    <p class="text-white text-base font-semibold">
                                                                        {t!("checkout.remove_transaction_confirm")}
                                                                    </p>
                                                                </div>
                                                            </div>
                                                        </Show>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }
                                }
                            >
                                <p class="text-gray-500">{t!("checkout.no_transactions_message")}</p>
                            </Show>
                        </Card>
                    </div>
                </div>

            </div>
        </Container>

        <ConfirmModal
            show=show_cancel_modal
            on_close=move || set_show_cancel_modal.set(false)
            on_confirm=handle_cancel_confirm.clone()
            title=t!("checkout.confirm_cancel_title")()
            message=Signal::derive(|| t!("checkout.confirm_cancel")())
            confirm_text=t!("checkout.confirm_cancel_confirm")()
            cancel_text=t!("common.cancel")()
            is_destructive=true
        />

        <Modal
            show=Signal::derive(move || pending_deletion.get().purchase_id.is_some())
            on_close=cancel_delete_purchase.clone()
            title=t!("checkout.delete_modal.title")()
            size=ModalSize::Medium
        >
            <Show when=move || pending_deletion.get().purchase_id.is_some()>
                <div class="space-y-4">
                    <p class="text-gray-700">
                        {move || {
                            let token = pending_deletion.get().token;
                            translate_with_params(
                                "checkout.delete_modal.instructions",
                                HashMap::from([("token", token)]),
                            )
                        }}
                    </p>
                    <input
                        class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500 border-gray-300"
                        placeholder=t!("checkout.delete_modal.placeholder")()
                        value=move || delete_confirmation_input.get()
                        node_ref=delete_confirmation_ref
                        on:input=move |ev| {
                            set_delete_confirmation_input.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" && deletion_token_matches.get() {
                                ev.prevent_default();
                                perform_delete_purchase();
                            }
                        }
                    />
                    <div class="flex justify-end gap-2">
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click=Box::new(cancel_delete_purchase.clone())
                    >
                        {t!("common.cancel")}
                    </Button>
                    <Button
                        variant=ButtonVariant::Danger
                        disabled=!deletion_token_matches.get()
                        on_click=Box::new(perform_delete_purchase.clone())
                    >
                        {t!("checkout.delete_modal.confirm")}
                    </Button>
                </div>
            </div>
        </Show>
        </Modal>
    }
}
