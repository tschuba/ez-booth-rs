use leptos::html;
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, File as WebFile, FileReader, ProgressEvent};

use crate::components::{use_toast, Button, ButtonVariant};
use crate::selected_booth_context::{use_booth_list_version, use_selected_booth};
use crate::state::use_app_state;
use crate::t;
use crate::utils::{current_device_info, download_text_file};
use ez_booth_storage::{MigrationIssueStrategy, MigrationParseSummary, ValidationIssue};

#[component]
pub fn MigrationWizard() -> impl IntoView {
    let app_state = use_app_state();
    let toast = use_toast();
    let input_ref = create_node_ref::<html::Input>();
    let booth_list_version = use_booth_list_version();
    let selected_booth = use_selected_booth();

    let (selected_file_name, set_selected_file_name) = create_signal(None::<String>);
    let (validation_summary, set_validation_summary) = create_signal(None::<MigrationParseSummary>);
    let (status_message, set_status_message) = create_signal(None::<String>);
    let (fatal_error, set_fatal_error) = create_signal(None::<String>);
    let (is_validating, set_is_validating) = create_signal(false);
    let (is_importing, set_is_importing) = create_signal(false);

    let open_file_picker = {
        let input_ref = input_ref.clone();
        move || {
            if let Some(input) = input_ref.get() {
                input.set_value("");
                input.click();
            }
        }
    };

    let on_file_change = {
        let input_ref = input_ref.clone();
        move |_event: Event| {
            let Some(input) = input_ref.get() else {
                return;
            };
            let Some(files) = input.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };

            set_selected_file_name.set(Some(file.name()));
            set_validation_summary.set(None);
            set_fatal_error.set(None);
            set_status_message.set(Some(t!("migration.status.reading")()));
            set_is_validating.set(true);

            let app_state = app_state.clone();
            let toast = toast.clone();
            spawn_local(async move {
                let bytes = match read_file_as_bytes(&file).await {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        set_is_validating.set(false);
                        set_status_message.set(None);
                        set_fatal_error.set(Some(error.clone()));
                        toast.error(error);
                        return;
                    }
                };

                let Some(state_result) = app_state.get() else {
                    let error = t!("migration.errors.state_unavailable")();
                    set_is_validating.set(false);
                    set_status_message.set(None);
                    set_fatal_error.set(Some(error.clone()));
                    toast.error(error);
                    return;
                };

                let state = match state_result {
                    Ok(state) => state,
                    Err(error) => {
                        set_is_validating.set(false);
                        set_status_message.set(None);
                        set_fatal_error.set(Some(error.clone()));
                        toast.error(error);
                        return;
                    }
                };

                match state.migration_service.parse_and_validate(bytes) {
                    Ok(summary) => {
                        let message = if summary.validation.issues.is_empty() {
                            t!("migration.status.ready_clean")()
                        } else {
                            t!("migration.status.ready_issues")()
                        };
                        set_validation_summary.set(Some(summary));
                        set_status_message.set(Some(message));
                    }
                    Err(error) => {
                        let message = error.to_string();
                        set_status_message.set(None);
                        set_fatal_error.set(Some(message.clone()));
                        toast.error(message);
                    }
                }

                set_is_validating.set(false);
            });
        }
    };

    let perform_import = move |strategy: MigrationIssueStrategy| {
        if is_importing.get_untracked() {
            return;
        }

        let Some(summary) = validation_summary.get_untracked() else {
            return;
        };
        let Some(state_result) = app_state.get() else {
            toast.error(t!("migration.errors.state_unavailable")());
            return;
        };

        let state = match state_result {
            Ok(state) => state,
            Err(error) => {
                toast.error(error);
                return;
            }
        };

        let original_counts = (
            summary.validation.booth_count,
            summary.validation.vendor_count,
            summary.validation.purchase_count,
        );

        set_is_importing.set(true);
        set_fatal_error.set(None);
        set_status_message.set(Some(t!("migration.status.backing_up")()));

        let toast = toast.clone();
        let booth_list_version = booth_list_version;
        let selected_booth = selected_booth;
        spawn_local(async move {
            let backup_data = match state.export_service.export_all().await {
                Ok(data) => data,
                Err(error) => {
                    let message = t!("migration.errors.backup_failed")()
                        .replace("{error}", &error.to_string());
                    set_is_importing.set(false);
                    set_status_message.set(None);
                    set_fatal_error.set(Some(message.clone()));
                    toast.error(message);
                    return;
                }
            };

            let device_info = current_device_info();
            let serialized = match state
                .export_service
                .serialize_full_backup_with_device_identifier(
                    &backup_data,
                    Some(device_info.identifier.as_str()),
                ) {
                Ok(serialized) => serialized,
                Err(error) => {
                    let message = t!("migration.errors.backup_failed")()
                        .replace("{error}", &error.to_string());
                    set_is_importing.set(false);
                    set_status_message.set(None);
                    set_fatal_error.set(Some(message.clone()));
                    toast.error(message);
                    return;
                }
            };

            if let Err(error) = download_text_file(
                &serialized.file_name,
                &serialized.json,
                "application/json;charset=utf-8",
            ) {
                let message = t!("migration.errors.backup_failed")().replace("{error}", &error);
                set_is_importing.set(false);
                set_status_message.set(None);
                set_fatal_error.set(Some(message.clone()));
                toast.error(message);
                return;
            }

            set_status_message.set(Some(t!("migration.status.importing")()));

            let prepared = match state.migration_service.prepare_import(summary, strategy) {
                Ok(prepared) => prepared,
                Err(error) => {
                    let message = error.to_string();
                    set_is_importing.set(false);
                    set_status_message.set(None);
                    set_fatal_error.set(Some(message.clone()));
                    toast.error(message);
                    return;
                }
            };

            match state
                .migration_service
                .replace_all(
                    prepared.booths.clone(),
                    prepared.vendors.clone(),
                    prepared.purchases.clone(),
                )
                .await
            {
                Ok(result) => {
                    let skipped_vendors = original_counts.1.saturating_sub(prepared.vendors.len());
                    let skipped_purchases =
                        original_counts.2.saturating_sub(prepared.purchases.len());

                    booth_list_version.update(|value| *value += 1);
                    selected_booth.set(prepared.booths.first().cloned());
                    set_validation_summary.set(Some(prepared));
                    set_is_importing.set(false);
                    set_status_message.set(Some(t!("migration.status.done")()));
                    toast.success(
                        t!("migration.success.summary")()
                            .replace("{booths}", &result.booths_migrated.to_string())
                            .replace("{vendors}", &result.vendors_migrated.to_string())
                            .replace("{purchases}", &result.purchases_migrated.to_string())
                            .replace("{skipped_vendors}", &skipped_vendors.to_string())
                            .replace("{skipped_purchases}", &skipped_purchases.to_string()),
                    );
                }
                Err(error) => {
                    let message = t!("migration.errors.import_failed")()
                        .replace("{error}", &error.to_string());
                    set_is_importing.set(false);
                    set_status_message.set(None);
                    set_fatal_error.set(Some(message.clone()));
                    toast.error(message);
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            <section class="space-y-3">
                <div class="space-y-2">
                    <h3 class="text-lg font-semibold text-slate-900">{t!("migration.title")}</h3>
                    <p class="text-sm text-gray-600">{t!("migration.description")}</p>
                </div>

                <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-950">
                    <p class="font-medium">{t!("migration.warning_title")}</p>
                    <p class="mt-1">{t!("migration.warning_body")}</p>
                </div>

                <div class="flex flex-wrap items-center gap-3">
                    <Button
                        on_click=Box::new(open_file_picker)
                        variant=ButtonVariant::Primary
                        disabled=Signal::derive(move || is_validating.get() || is_importing.get())
                    >
                        {move || {
                            if is_validating.get() {
                                t!("migration.actions.validating")()
                            } else if is_importing.get() {
                                t!("migration.actions.importing")()
                            } else {
                                t!("migration.actions.choose_file")()
                            }
                        }}
                    </Button>
                    <input
                        _ref=input_ref
                        type="file"
                        class="hidden"
                        accept=".db,application/octet-stream"
                        on:change=on_file_change
                    />
                    <Show when=move || selected_file_name.get().is_some()>
                        <span class="text-sm text-gray-600">{move || selected_file_name.get().unwrap_or_default()}</span>
                    </Show>
                </div>
            </section>

            <Show when=move || status_message.get().is_some()>
                <div class="rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                    {move || status_message.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || fatal_error.get().is_some()>
                <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">
                    {move || fatal_error.get().unwrap_or_default()}
                </div>
            </Show>

            {move || {
                let Some(summary) = validation_summary.get() else {
                    return ().into_view();
                };

                let issue_count = summary.validation.issues.len();
                let has_issues = issue_count > 0;
                let skipped_vendors = summary
                    .validation
                    .vendor_count
                    .saturating_sub(summary.vendors.len());
                let skipped_purchases = summary
                    .validation
                    .purchase_count
                    .saturating_sub(summary.purchases.len());
                let issue_views = summary
                    .validation
                    .issues
                    .iter()
                    .map(|issue| {
                        view! {
                            <li class="rounded border border-amber-200 bg-white px-3 py-2">
                                {format_issue(issue)}
                            </li>
                        }
                    })
                    .collect_view();
                let clean_import_disabled = is_importing.get() || has_issues;

                view! {
                    <section class="space-y-4 border-t border-gray-200 pt-6">
                        <div class="grid gap-4 sm:grid-cols-3">
                            <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
                                <p class="text-sm font-medium text-gray-700">{t!("migration.summary.booths")}</p>
                                <p class="mt-1 text-lg font-semibold text-slate-900">{summary.validation.booth_count}</p>
                            </div>
                            <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
                                <p class="text-sm font-medium text-gray-700">{t!("migration.summary.vendors")}</p>
                                <p class="mt-1 text-lg font-semibold text-slate-900">{summary.validation.vendor_count}</p>
                            </div>
                            <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
                                <p class="text-sm font-medium text-gray-700">{t!("migration.summary.purchases")}</p>
                                <p class="mt-1 text-lg font-semibold text-slate-900">{summary.validation.purchase_count}</p>
                            </div>
                        </div>

                        {if has_issues {
                            view! {
                                <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-950 space-y-3">
                                    <p class="font-medium">
                                        {t!("migration.validation.issues_found")().replace("{count}", &issue_count.to_string())}
                                    </p>
                                    <ul class="space-y-2">{issue_views}</ul>
                                    <p class="text-xs text-amber-900">
                                        {t!("migration.validation.skip_summary")()
                                            .replace("{vendors}", &skipped_vendors.to_string())
                                            .replace("{purchases}", &skipped_purchases.to_string())}
                                    </p>
                                </div>
                            }
                                .into_view()
                        } else {
                            view! {
                                <div class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900">
                                    {t!("migration.validation.clean")}
                                </div>
                            }
                                .into_view()
                        }}

                        <div class="flex flex-wrap gap-3">
                            <Button
                                on_click=Box::new(move || perform_import(MigrationIssueStrategy::Cancel))
                                variant=ButtonVariant::Secondary
                                disabled=Signal::derive(move || clean_import_disabled)
                            >
                                {t!("migration.actions.import_clean_only")}
                            </Button>
                            {if has_issues {
                                view! {
                                    <Button
                                        on_click=Box::new(move || perform_import(MigrationIssueStrategy::SkipInvalid))
                                        variant=ButtonVariant::Primary
                                        disabled=is_importing
                                    >
                                        {t!("migration.actions.skip_invalid")}
                                    </Button>
                                }
                                    .into_view()
                            } else {
                                ().into_view()
                            }}
                        </div>
                    </section>
                }
                    .into_view()
            }}
        </div>
    }
}

fn format_issue(issue: &ValidationIssue) -> String {
    match issue {
        ValidationIssue::PurchaseTotalMismatch {
            purchase_id,
            expected,
            actual,
        } => t!("migration.issue.purchase_total_mismatch")()
            .replace("{purchase_id}", purchase_id)
            .replace("{expected}", &expected.to_string())
            .replace("{actual}", &actual.to_string()),
        ValidationIssue::VendorMissingBooth {
            booth_id,
            vendor_id,
        } => t!("migration.issue.vendor_missing_booth")()
            .replace("{vendor_id}", vendor_id)
            .replace("{booth_id}", booth_id),
        ValidationIssue::PurchaseMissingBooth {
            booth_id,
            purchase_id,
        } => t!("migration.issue.purchase_missing_booth")()
            .replace("{purchase_id}", purchase_id)
            .replace("{booth_id}", booth_id),
        ValidationIssue::PurchaseWithoutItems { purchase_id } => {
            t!("migration.issue.purchase_without_items")().replace("{purchase_id}", purchase_id)
        }
        ValidationIssue::PurchaseItemMissingVendor {
            booth_id,
            purchase_id,
            item_id,
            vendor_id,
        } => t!("migration.issue.purchase_item_missing_vendor")()
            .replace("{booth_id}", booth_id)
            .replace("{purchase_id}", purchase_id)
            .replace("{item_id}", item_id)
            .replace("{vendor_id}", vendor_id),
    }
}

async fn read_file_as_bytes(file: &WebFile) -> Result<Vec<u8>, String> {
    let reader = FileReader::new()
        .map_err(|err| format!("{}: {:?}", t!("migration.errors.file_read_failed")(), err))?;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader_for_load = reader.clone();
        let resolve_fn = resolve.clone();
        let reject_for_load = reject.clone();

        let onload =
            Closure::once(Box::new(
                move |_event: ProgressEvent| match reader_for_load.result() {
                    Ok(result) => {
                        let _ = resolve_fn.call1(&JsValue::NULL, &result);
                    }
                    Err(_) => {
                        let _ = reject_for_load.call1(
                            &JsValue::NULL,
                            &JsValue::from_str(&t!("migration.errors.file_read_failed")()),
                        );
                    }
                },
            ) as Box<dyn FnOnce(_)>);

        let reject_for_error = reject.clone();
        let onerror = Closure::once(Box::new(move |_event: ProgressEvent| {
            let _ = reject_for_error.call1(
                &JsValue::NULL,
                &JsValue::from_str(&t!("migration.errors.file_read_failed")()),
            );
        }) as Box<dyn FnOnce(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        if let Err(err) = reader.read_as_array_buffer(file) {
            let _ = reject.call1(
                &JsValue::NULL,
                &JsValue::from_str(&format!(
                    "{}: {:?}",
                    t!("migration.errors.file_read_failed")(),
                    err
                )),
            );
        }

        onload.forget();
        onerror.forget();
    });

    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| t!("migration.errors.file_read_failed")())
        })?;

    Ok(js_sys::Uint8Array::new(&result).to_vec())
}
