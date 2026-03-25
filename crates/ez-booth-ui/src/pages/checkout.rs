use crate::components::*;
use crate::formatting::{format_currency, parse_decimal_input};
use crate::i18n::{translate_with_params, use_locale, Locale};
use crate::selected_booth_context;
use crate::state::use_app_state;
use crate::t;
use chrono::{DateTime, Local, Utc};
use domain::models::purchase::{Purchase, PurchaseItem};
use domain::models::shared::{PurchaseId, VendorId};
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
            Locale::De => "%H:%M",
            Locale::En => "%I:%M %p",
        };
        let local_time = added_at.with_timezone(&Local).format(format_str).to_string();
        translate_with_params(
            "checkout.time.time_of_day",
            HashMap::from([("time", local_time)]),
        )
    }
}

fn format_item_tooltip(added_at: DateTime<Utc>, locale: Locale) -> String {
    let format_str = match locale {
        Locale::De => "%d.%m.%Y %H:%M",
        Locale::En => "%Y-%m-%d %I:%M %p",
    };
    let local_time = added_at.with_timezone(&Local).format(format_str).to_string();
    translate_with_params(
        "checkout.time.tooltip",
        HashMap::from([("datetime", local_time)]),
    )
}

fn format_purchase_timestamp(timestamp: DateTime<Utc>, locale: Locale) -> String {
    let format_str = match locale {
        Locale::De => "%d.%m.%Y %H:%M",
        Locale::En => "%Y-%m-%d %I:%M %p",
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
            persist_form_data(booth_id, &CheckoutFormData {
                vendor_id: current_vendor_id.clone(),
                ..Default::default()
            });
            
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

        let vendor_id = VendorId::new(data.vendor_id.clone());

        let purchase_items: Vec<PurchaseItem> = data
            .items
            .into_iter()
            .map(|item| PurchaseItem::new(item.amount))
            .collect();

        let purchase = Purchase::new(booth.id.clone(), vendor_id.clone(), purchase_items);

        if let Some(Ok(state)) = state_result {
            let booth_id_clone = booth.id.clone();
            let vendor_id_clone = vendor_id.clone();
            let vendor_id_str = vendor_id_clone.as_str().to_string();
            let purchase_clone = purchase.clone();
            spawn_local(async move {
                // Use VendorService to get or create vendor
                let vendor_existed = state
                    .vendor_repository
                    .find_by_id(&booth_id_clone, &vendor_id_clone)
                    .await
                    .ok()
                    .flatten()
                    .is_some();

                match state
                    .vendor_service
                    .get_or_create(booth_id_clone.clone(), vendor_id_str.clone())
                    .await
                {
                    Ok(_vendor) => {
                        // Show info toast if vendor was auto-created
                        if !vendor_existed {
                            let info_msg = translate_with_params(
                                "checkout.info.vendor_auto_created",
                                HashMap::from([("vendor_id", vendor_id_str.clone())]),
                            );
                            toast.info(&info_msg);
                        }
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

                match state.purchase_repository.save(&purchase_clone).await {
                    Ok(_) => {
                        set_purchases.update(|list| {
                            list.push(purchase_clone.clone());
                            list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        });
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
        move || {
            set_pending_deletion.set(PendingDeletion::default());
            set_delete_confirmation_input.set(String::new());
        }
    };

    view! {
        <Container>
            <div class="space-y-6">
                <div class="flex flex-col gap-6 lg:flex-row">
                    <div class="flex-1 space-y-6">
                        <Card title_view={t!("checkout.title").into_view()}>
                            <Show
                                when=move || selected_booth.get().is_none()
                                fallback=move || view! {
                                    <Show
                                        when=move || is_loading.get()
                                        fallback=move || view! {
                                            <div class="flex flex-col gap-6 lg:grid lg:grid-cols-[minmax(0,1fr)_18rem]">
                                                        <div class="space-y-6 lg:pr-4">
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
                                                                            let value = event_target_value(&ev);
                                                                            set_form_data.update(|data| {
                                                                                data.vendor_id = value;
                                                                                data.vendor_error = None;
                                                                            });
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
                                                                                    set_form_data.update(|data| data.vendor_error = None);
                                                                                    if let Some(amount_input) = amount_input_ref.get() {
                                                                                        let _ = amount_input.focus();
                                                                                        let _ = amount_input.select();
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
                                                                        <Button on_click=Box::new(add_item)>
                                                                            {t!("checkout.add_item")}
                                                                        </Button>
                                                                        <Show when=move || form_data.get().amount_error.is_some()>
                                                                            <p class="text-sm text-red-600">{move || form_data.get().amount_error.clone().unwrap_or_default()}</p>
                                                                        </Show>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        <div class="space-y-6">
                                                            <div class="space-y-6 rounded-lg border bg-gray-50 p-4 shadow-sm">
                                                                <div class="flex justify-between text-lg font-semibold">
                                                                    <span>{t!("checkout.total")}</span>
                                                                    <span>{move || {
                                                                        let locale = use_locale().get();
                                                                        format_currency(form_data.get().total(), locale)
                                                                    }}</span>
                                                                </div>

                                                                <div class="flex flex-col gap-2 sm:flex-row">
                                                                    <Button class="flex-1".to_string() on_click=Box::new(submit_purchase)>
                                                                        <span class="inline-flex items-center justify-center gap-4">
                                                                            <svg
                                                                                class="w-8 h-8"
                                                                                viewBox="0 0 24 24"
                                                                                fill="none"
                                                                                stroke="currentColor"
                                                                                stroke-width="2"
                                                                                stroke-linecap="round"
                                                                                stroke-linejoin="round"
                                                                                aria-hidden="true"
                                                                            >
                                                                                <polyline points="20 6 9 17 4 12" />
                                                                            </svg>
                                                                            <span>{t!("checkout.confirm")}</span>
                                                                        </span>
                                                                    </Button>
                                                                    <Button
                                                                        class="flex-1".to_string()
                                                                        variant=ButtonVariant::Secondary
                                                                        on_click=Box::new(move || confirm_clear_form())
                                                                        aria_label=t!("checkout.confirm_cancel_confirm")()
                                                                    >
                                                                        <span
                                                                            class="inline-flex items-center justify-center"
                                                                            title={t!("checkout.confirm_cancel_confirm")()}
                                                                        >
                                                                            <svg
                                                                                class="w-8 h-8"
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
                                                                        </span>
                                                                    </Button>
                                                                </div>
                                                    </div>
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

                        <Card title_view={t!("checkout.recent_transactions_title").into_view()}>
                            <Show
                                when=move || filtered_purchases.get().is_empty()
                                fallback=move || {
                                    let list = filtered_purchases.get();
                                    view! {
                                        <div class="space-y-2">
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
                                                    <div class="flex items-center justify-between p-3 border rounded-lg">
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
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-lg font-bold">{
                                                                let locale = use_locale().get();
                                                                format_currency(amount, locale)
                                                            }</span>
                                                            <Button
                                                                variant=ButtonVariant::Ghost
                                                                on_click=Box::new(move || delete_purchase(purchase_id))
                                                            >
                                                                {t!("common.delete")}
                                                            </Button>
                                                        </div>
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

                    <div class="w-full lg:w-96 space-y-6">
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

                        <Card title_view={t!("checkout.current_items").into_view()}>
                            <div class="space-y-2">
                                <Show
                                    when=move || form_data.get().items.is_empty()
                                    fallback=move || {
                                        let data = form_data.get();
                                        let items = data.items;
                                        let total_items = items.len();
                                        view! {
                                            <ul class="space-y-2">
                                                {items.into_iter().enumerate().map(move |(index, item)| {
                                                    let display_number = total_items - index;
                                                    let vendor_label = if item.vendor_id.trim().is_empty() {
                                                        "—".to_string()
                                                    } else {
                                                        item.vendor_id.clone()
                                                    };
                                                    view! {
                                                        <li class="flex items-start justify-between text-sm p-2 border rounded-lg bg-gray-50">
                                                            <div>
                                                                <p class="font-medium">{format!("Item {}", display_number)}</p>
                                                                <p class="text-xs text-gray-500">{format!("Vendor {}", vendor_label)}</p>
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
