use crate::components::*;
use crate::i18n::translate_with_params;
use crate::state::use_app_state;
use crate::t;
use chrono::{DateTime, Local, Utc};
use domain::models::booth::Booth;
use domain::models::purchase::{Purchase, PurchaseItem};
use domain::models::shared::VendorId;
use leptos::html;
use leptos::*;
use rust_decimal::Decimal;
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

impl CheckoutFormData {
    fn total(&self) -> Decimal {
        self.items.iter().map(|item| item.amount).sum()
    }
}

fn format_item_timestamp(added_at: DateTime<Utc>) -> String {
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
        let local_time = added_at.with_timezone(&Local).format("%H:%M").to_string();
        translate_with_params(
            "checkout.time.time_of_day",
            HashMap::from([("time", local_time)]),
        )
    }
}

fn format_item_tooltip(added_at: DateTime<Utc>) -> String {
    let local_time = added_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string();
    translate_with_params(
        "checkout.time.tooltip",
        HashMap::from([("datetime", local_time)]),
    )
}

fn focus_and_select_input(input_ref: &NodeRef<html::Input>) {
    if let Some(input) = input_ref.get() {
        let _ = input.focus();
        let _ = input.select();
    }
}

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let app_state = use_app_state();
    let toast = use_toast();

    // Selected booth (for now, use the first booth)
    let (selected_booth, set_selected_booth) = create_signal(None::<Booth>);

    // Purchases for current booth
    let (purchases, set_purchases) = create_signal(Vec::<Purchase>::new());

    // Checkout form data
    let (form_data, set_form_data) = create_signal(CheckoutFormData::default());

    // Cancel confirmation modal
    let (show_cancel_modal, set_show_cancel_modal) = create_signal(false);

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

    // Load initial data
    create_effect(move |_| {
        let state_result = app_state.get();

        if let Some(Ok(state)) = state_result {
            set_is_loading.set(true);
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(booths) => {
                        if let Some(booth) = booths.first().cloned() {
                            set_selected_booth.set(Some(booth.clone()));

                            match state.purchase_repository.find_by_booth(&booth.id).await {
                                Ok(existing_purchases) => {
                                    let mut sorted = existing_purchases;
                                    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                                    set_purchases.set(sorted);
                                }
                                Err(e) => {
                                    toast.error(&format!("Failed to load purchases: {:?}", e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to load booths: {:?}", e));
                    }
                }

                set_is_loading.set(false);
            });
        }
    });

    // Derived signals
    let total_sales = Memo::new(move |_| {
        purchases
            .get()
            .iter()
            .map(|p| p.total_amount())
            .sum::<Decimal>()
    });

    let fee_summary = Memo::new(move |_| {
        selected_booth
            .get()
            .map(|booth| {
                let total = total_sales.get();
                let fee_percent = booth.fees.sales_fee_percent;
                let fee_amount = total * fee_percent / Decimal::from(100);
                let net_amount = total - fee_amount;

                (total, fee_amount, net_amount)
            })
            .unwrap_or((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO))
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

        let normalized_amount = data.current_amount.replace(',', ".");

        match Decimal::from_str(&normalized_amount) {
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
        }
    };

    let handle_cancel_confirm = {
        let set_form_data = set_form_data.clone();
        move || {
            set_form_data.set(CheckoutFormData::default());
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
            let purchase_clone = purchase.clone();
            spawn_local(async move {
                match state.purchase_repository.save(&purchase_clone).await {
                    Ok(_) => {
                        set_purchases.update(|list| {
                            list.push(purchase_clone.clone());
                            list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        });
                        toast.success(&t!("checkout.success")());
                        set_form_data.set(CheckoutFormData::default());
                    }
                    Err(e) => {
                        toast.error(&format!("Failed to save purchase: {:?}", e));
                    }
                }
            });
        }
    };

    let delete_purchase = move |purchase_id| {
        let state_result = app_state.get();
        if let Some(Ok(state)) = state_result {
            spawn_local(async move {
                if let Err(e) = state.purchase_repository.delete(&purchase_id).await {
                    toast.error(&format!("Failed to delete purchase: {:?}", e));
                }
            });

            set_purchases.update(|list| {
                list.retain(|p| p.id != purchase_id);
                list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            });
        }
    };

    view! {
        <Container>
            <div class="space-y-6">
                <div class="flex flex-col gap-4 lg:flex-row">
                    <div class="flex-1 space-y-4">
                        <Card title="Checkout">
                            <Show
                                when=move || is_loading.get()
                                fallback=move || view! {
                                    <Show
                                        when=move || selected_booth.get().is_none()
                                        fallback=move || view! {
                                            <div class="flex flex-col gap-6 lg:grid lg:grid-cols-[minmax(0,1fr)_18rem]">
                                                <div class="space-y-4 lg:pr-4">
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
                                                                placeholder="Vendor ID (e.g., 101)"
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
                                                                    placeholder="0.00"
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

                                                <div class="space-y-4">
                                                    <div>
                                                        <h3 class="text-sm font-semibold text-gray-700 mb-2">"Current Items"</h3>
                                                        <div class="space-y-2 border rounded-lg p-3 bg-gray-50">
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
                                                                                    <li class="flex items-start justify-between text-sm">
                                                                                        <div>
                                                                                            <p class="font-medium">{format!("Item {}", display_number)}</p>
                                                                                            <p class="text-xs text-gray-500">{format!("Vendor {}", vendor_label)}</p>
                                                                                        </div>
                                                                                        <div class="text-right">
                                                                                            <span class="block font-semibold">{format!("{:.2}", item.amount)}</span>
                                                                                            <p class="text-xs text-gray-400" title={format_item_tooltip(item.added_at)}>{format!("{}", format_item_timestamp(item.added_at))}</p>
                                                                                        </div>
                                                                                    </li>
                                                                                }
                                                                            }).collect_view()}
                                                                        </ul>
                                                                    }
                                                                }
                                                            >
                                                                <p class="text-gray-500 text-sm">"No items added yet"</p>
                                                            </Show>
                                                        </div>
                                                    </div>

                                                    <div class="space-y-4 rounded-lg border bg-gray-50 p-4 shadow-sm">
                                                        <div class="flex justify-between text-lg font-semibold">
                                                            <span>{t!("checkout.total")}</span>
                                                            <span>{move || format!("{:.2}", form_data.get().total())}</span>
                                                        </div>

                                                        <div class="flex flex-col gap-2 sm:flex-row">
                                                            <Button class="flex-1" on_click=Box::new(submit_purchase)>
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
                                                                class="flex-1"
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
                                        <p class="text-gray-600">"Please create a booth to start checkout."</p>
                                    </Show>
                                }
                            >
                                <p class="text-gray-600">"Loading checkout data..."</p>
                            </Show>
                        </Card>

                        <Card title="Recent Transactions">
                            <Show
                                when=move || purchases.get().is_empty()
                                fallback=move || {
                                    let list = purchases.get();
                                    view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|purchase| {
                                                let amount = purchase.total_amount();
                                                let purchase_id = purchase.id;
                                                view! {
                                                    <div class="flex items-center justify-between p-3 border rounded-lg">
                                                        <div>
                                                            <p class="text-sm font-semibold">{format!("Vendor {}", purchase.vendor_id.as_str())}</p>
                                                            <p class="text-xs text-gray-500">
                                                                {purchase.timestamp.format("%Y-%m-%d %H:%M").to_string()}
                                                            </p>
                                                        </div>
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-lg font-bold">{format!("{:.2}", amount)}</span>
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
                                <p class="text-gray-500">"No transactions recorded yet."</p>
                            </Show>
                        </Card>
                    </div>

                    <div class="w-full lg:w-96">
                        <Card title="Running Totals">
                            <div class="space-y-4">
                                <div class="flex justify-between">
                                    <span class="text-gray-600">"Total Sales"</span>
                                    <span class="text-lg font-semibold">{move || format!("{:.2}", fee_summary.get().0)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">"Commission"</span>
                                    <span class="text-lg font-semibold">{move || format!("{:.2}", fee_summary.get().1)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">"Net Revenue"</span>
                                    <span class="text-lg font-semibold">{move || format!("{:.2}", fee_summary.get().2)}</span>
                                </div>
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
    }
}
