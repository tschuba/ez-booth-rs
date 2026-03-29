use std::rc::Rc;

use leptos::html;
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, FileReader, ProgressEvent};

use crate::components::{use_toast, Button, ButtonSize, ButtonVariant, Modal, ModalSize};
use crate::selected_booth_context::use_booth_list_version;
use crate::state::use_app_state;
use crate::t;
use ez_booth_storage::export::{
    BackupData, BoothBackupData, ConflictStrategy, ImportError, ImportSummary, ImportValidator,
    ValidationFailure,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum ImportPreview {
    Full {
        booths: usize,
        vendors: usize,
        purchases: usize,
    },
    Booth {
        description: String,
        vendors: usize,
        purchases: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum ParsedImportData {
    Full(BackupData),
    Booth(BoothBackupData),
}

#[component]
pub fn ImportButton(
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let app_state = use_app_state();
    let booth_list_version = use_booth_list_version();
    let toast = use_toast();
    let input_ref = create_node_ref::<html::Input>();
    let (is_reading, set_is_reading) = create_signal(false);
    let (is_importing, set_is_importing) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (selected_file_name, set_selected_file_name) = create_signal(String::new());
    let (preview, set_preview) = create_signal(None::<ImportPreview>);
    let (parsed_data, set_parsed_data) = create_signal(None::<ParsedImportData>);
    let (validation_failures, set_validation_failures) =
        create_signal(Vec::<ValidationFailure>::new());
    let (structure_error, set_structure_error) = create_signal(None::<String>);
    let (conflict_strategy, set_conflict_strategy) = create_signal(ConflictStrategy::Merge);

    let validator = Rc::new(ImportValidator::new());

    let open_file_picker = {
        let input_ref = input_ref.clone();
        move || {
            if is_reading.get_untracked() {
                return;
            }

            if let Some(input) = input_ref.get() {
                input.set_value("");
                input.click();
            }
        }
    };

    let reset_results = {
        let set_preview = set_preview;
        let set_parsed_data = set_parsed_data;
        let set_validation_failures = set_validation_failures;
        let set_structure_error = set_structure_error;
        move || {
            set_preview.set(None);
            set_parsed_data.set(None);
            set_validation_failures.set(Vec::new());
            set_structure_error.set(None);
        }
    };

    let on_file_change = {
        let input_ref = input_ref.clone();
        let validator = Rc::clone(&validator);
        move |_ev: Event| {
            let Some(input) = input_ref.get() else {
                return;
            };

            let Some(files) = input.files() else {
                return;
            };

            let Some(file) = files.get(0) else {
                return;
            };

            reset_results();
            set_selected_file_name.set(file.name());
            set_is_reading.set(true);

            let reader = match FileReader::new() {
                Ok(reader) => reader,
                Err(err) => {
                    set_is_reading.set(false);
                    toast.error(format!("{}: {:?}", t!("backup.import_failed")(), err));
                    return;
                }
            };

            let reader_for_load = reader.clone();
            let validator_for_load = Rc::clone(&validator);
            let onload = Closure::wrap(Box::new(move |_event: ProgressEvent| {
                let Ok(result) = reader_for_load.result() else {
                    set_is_reading.set(false);
                    toast.error(t!("backup.import_failed")());
                    return;
                };

                let Some(contents) = result.as_string() else {
                    set_is_reading.set(false);
                    toast.error(t!("backup.import_invalid_encoding")());
                    return;
                };

                let booth_validation = validator_for_load
                    .validate_booth_backup(&contents)
                    .map(|data| {
                        let preview = ImportPreview::Booth {
                            description: data.booth.description.clone(),
                            vendors: data.vendors.len(),
                            purchases: data.purchases.len(),
                        };
                        (preview, ParsedImportData::Booth(data))
                    });

                let resolved = match booth_validation {
                    Ok(booth) => Ok(booth),
                    Err(ImportError::InvalidJson(_)) | Err(ImportError::InvalidStructure(_)) => {
                        validator_for_load.validate_backup(&contents).map(|data| {
                            let preview = ImportPreview::Full {
                                booths: data.booths.len(),
                                vendors: data.vendors.len(),
                                purchases: data.purchases.len(),
                            };
                            (preview, ParsedImportData::Full(data))
                        })
                    }
                    Err(other) => Err(other),
                };

                set_is_reading.set(false);
                match resolved {
                    Ok((next_preview, next_data)) => {
                        set_preview.set(Some(next_preview));
                        set_parsed_data.set(Some(next_data));
                        set_show_modal.set(true);
                    }
                    Err(ImportError::ValidationFailed { failures }) => {
                        set_validation_failures.set(failures);
                        set_show_modal.set(true);
                    }
                    Err(ImportError::UnsupportedVersion { found, supported }) => {
                        set_structure_error.set(Some(
                            t!("backup.import_version_error")()
                                .replace("{found}", &found.to_string())
                                .replace("{supported}", &supported.to_string()),
                        ));
                        set_show_modal.set(true);
                    }
                    Err(ImportError::OrphanedRecords { details }) => {
                        set_structure_error.set(Some(details.join("\n")));
                        set_show_modal.set(true);
                    }
                    Err(ImportError::InvalidStructure(message))
                    | Err(ImportError::InvalidJson(message)) => {
                        set_structure_error.set(Some(message));
                        set_show_modal.set(true);
                    }
                    Err(other) => {
                        set_structure_error.set(Some(other.to_string()));
                        set_show_modal.set(true);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let onerror = Closure::wrap(Box::new(move |_event: ProgressEvent| {
                set_is_reading.set(false);
                toast.error(t!("backup.import_failed")());
            }) as Box<dyn FnMut(_)>);

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));

            if let Err(err) = reader.read_as_text(&file) {
                set_is_reading.set(false);
                toast.error(format!("{}: {:?}", t!("backup.import_failed")(), err));
            }

            onload.forget();
            onerror.forget();
        }
    };

    let close_modal = move || {
        set_show_modal.set(false);
        set_preview.set(None);
        set_parsed_data.set(None);
        set_validation_failures.set(Vec::new());
        set_structure_error.set(None);
        set_conflict_strategy.set(ConflictStrategy::Merge);
    };

    let close_modal_action = store_value(close_modal);

    let handle_apply_import = move || {
        if is_importing.get_untracked() {
            return;
        }

        let state_result = app_state.get();
        let parsed = parsed_data.get_untracked();
        let strategy = conflict_strategy.get_untracked();

        let Some(payload) = parsed else {
            return;
        };

        set_is_importing.set(true);

        spawn_local(async move {
            let result: Result<ImportSummary, String> = async move {
                let state = match state_result {
                    Some(Ok(state)) => state,
                    Some(Err(error)) => return Err(error),
                    None => return Err(t!("common.loading")()),
                };

                match payload {
                    ParsedImportData::Full(data) => state
                        .import_service
                        .import_all(data, strategy)
                        .await
                        .map_err(|err| err.to_string()),
                    ParsedImportData::Booth(data) => state
                        .import_service
                        .import_booth_backup(data, strategy)
                        .await
                        .map_err(|err| err.to_string()),
                }
            }
            .await;

            set_is_importing.set(false);

            match result {
                Ok(summary) => {
                    if summary.booths_imported > 0
                        || summary.vendors_imported > 0
                        || summary.purchases_imported > 0
                        || summary.conflicts_resolved > 0
                    {
                        booth_list_version.update(|version| *version += 1);
                    }

                    let message = t!("backup.import_apply_success")()
                        .replace("{booths}", &summary.booths_imported.to_string())
                        .replace("{vendors}", &summary.vendors_imported.to_string())
                        .replace("{purchases}", &summary.purchases_imported.to_string())
                        .replace("{resolved}", &summary.conflicts_resolved.to_string())
                        .replace("{skipped}", &summary.skipped_records.len().to_string());
                    toast.success(message);
                    close_modal_action.with_value(|close| close());
                }
                Err(error) => {
                    toast.error(format!("{}: {}", t!("backup.import_apply_failed")(), error));
                }
            }
        });
    };

    let on_strategy_change = move |ev: Event| {
        let value = event_target_value(&ev);
        let strategy = match value.as_str() {
            "skip" => ConflictStrategy::Skip,
            "replace" => ConflictStrategy::Replace,
            _ => ConflictStrategy::Merge,
        };
        set_conflict_strategy.set(strategy);
    };

    let on_strategy_change_action = store_value(on_strategy_change);

    view! {
        <>
            <input
                node_ref=input_ref
                type="file"
                accept="application/json,.json"
                class="hidden"
                on:change=on_file_change
            />

            <Button
                on_click=Box::new(open_file_picker)
                variant=variant.unwrap_or(ButtonVariant::Secondary)
                size=size.unwrap_or(ButtonSize::Medium)
                class=class.unwrap_or_default()
                disabled=is_reading.get() || is_importing.get()
                title=t!("backup.import")()
            >
                {move || if is_reading.get() || is_importing.get() {
                    t!("backup.import_in_progress")()
                } else {
                    t!("backup.import")()
                }}
            </Button>

            <Modal
                show=Signal::derive(move || show_modal.get())
                on_close=move || close_modal_action.with_value(|close| close())
                title=Signal::derive(move || t!("backup.import_review_title")())
                size=ModalSize::Large
            >
                <div class="space-y-4 text-gray-700">
                    <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3 text-sm">
                        <span class="font-medium text-gray-900">{t!("backup.selected_file")}</span>
                        <span>{move || format!(" {}", selected_file_name.get())}</span>
                    </div>

                    <Show when=move || preview.get().is_some()>
                        {move || {
                            preview.get().map(|preview| {
                                match preview {
                                    ImportPreview::Full { booths, vendors, purchases } => view! {
                                        <div class="space-y-3">
                                            <p class="text-sm uppercase tracking-wide text-green-700 font-semibold">
                                                {t!("backup.import_ready_label")}
                                            </p>
                                            <p>{t!("backup.import_valid_full")}</p>
                                            <p class="text-sm text-gray-600">
                                                {t!("backup.import_counts")()
                                                    .replace("{booths}", &booths.to_string())
                                                    .replace("{vendors}", &vendors.to_string())
                                                    .replace("{purchases}", &purchases.to_string())}
                                            </p>
                                            <div class="space-y-2 rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                                                <p class="font-medium">{t!("backup.import_strategy_label")}</p>
                                                <select
                                                    class="w-full rounded-lg border border-blue-200 bg-white px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                    on:change=move |ev| on_strategy_change_action.with_value(|handler| handler(ev))
                                                >
                                                    <option value="merge" selected=move || conflict_strategy.get() == ConflictStrategy::Merge>{t!("backup.strategy_merge")}</option>
                                                    <option value="skip" selected=move || conflict_strategy.get() == ConflictStrategy::Skip>{t!("backup.strategy_skip")}</option>
                                                    <option value="replace" selected=move || conflict_strategy.get() == ConflictStrategy::Replace>{t!("backup.strategy_replace")}</option>
                                                </select>
                                                <p>{t!("backup.import_apply_ready")}</p>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    ImportPreview::Booth { description, vendors, purchases } => view! {
                                        <div class="space-y-3">
                                            <p class="text-sm uppercase tracking-wide text-green-700 font-semibold">
                                                {t!("backup.import_ready_label")}
                                            </p>
                                            <p>{t!("backup.import_valid_booth")().replace("{description}", &description)}</p>
                                            <p class="text-sm text-gray-600">
                                                {t!("backup.import_booth_counts")()
                                                    .replace("{vendors}", &vendors.to_string())
                                                    .replace("{purchases}", &purchases.to_string())}
                                            </p>
                                            <div class="space-y-2 rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                                                <p class="font-medium">{t!("backup.import_strategy_label")}</p>
                                                <select
                                                    class="w-full rounded-lg border border-blue-200 bg-white px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                    on:change=move |ev| on_strategy_change_action.with_value(|handler| handler(ev))
                                                >
                                                    <option value="merge" selected=move || conflict_strategy.get() == ConflictStrategy::Merge>{t!("backup.strategy_merge")}</option>
                                                    <option value="skip" selected=move || conflict_strategy.get() == ConflictStrategy::Skip>{t!("backup.strategy_skip")}</option>
                                                    <option value="replace" selected=move || conflict_strategy.get() == ConflictStrategy::Replace>{t!("backup.strategy_replace")}</option>
                                                </select>
                                                <p>{t!("backup.import_apply_ready")}</p>
                                            </div>
                                        </div>
                                    }.into_view(),
                                }
                            })
                        }}
                    </Show>

                    <Show when=move || !validation_failures.get().is_empty()>
                        <div class="space-y-3">
                            <p class="text-sm uppercase tracking-wide text-red-700 font-semibold">
                                {t!("backup.import_invalid_label")}
                            </p>
                            <p>{t!("backup.import_validation_summary")}</p>
                            <div class="max-h-80 space-y-2 overflow-y-auto rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-900">
                                <For
                                    each=move || validation_failures.get()
                                    key=|failure| format!("{}:{}:{}", failure.record_type, failure.record_id, failure.reason)
                                    children=move |failure| {
                                        view! {
                                            <div class="rounded border border-red-100 bg-white px-3 py-2">
                                                <p class="font-medium">
                                                    {format!("{} {}", failure.record_type, failure.record_id)}
                                                </p>
                                                <p>{failure.reason}</p>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </Show>

                    <Show when=move || structure_error.get().is_some()>
                        {move || structure_error.get().map(|message| {
                            view! {
                                <div class="space-y-3">
                                    <p class="text-sm uppercase tracking-wide text-red-700 font-semibold">
                                        {t!("backup.import_invalid_label")}
                                    </p>
                                    <p>{t!("backup.import_structure_summary")}</p>
                                    <pre class="whitespace-pre-wrap rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">{message}</pre>
                                </div>
                            }
                        })}
                    </Show>

                    <div class="flex justify-end gap-2">
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click=Box::new(move || close_modal_action.with_value(|close| close()))
                        >
                            {t!("common.close")}
                        </Button>
                        <Show when=move || preview.get().is_some()>
                            <Button on_click=Box::new(handle_apply_import) disabled=is_importing.get()>
                                {move || if is_importing.get() {
                                    t!("backup.import_in_progress")()
                                } else {
                                    t!("backup.import_apply")()
                                }}
                            </Button>
                        </Show>
                    </div>
                </div>
            </Modal>
        </>
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use domain::{Booth, FeeConfig};
    use ez_booth_storage::export::{BoothBackupData, ImportValidator};
    use rust_decimal_macros::dec;

    use super::{ImportPreview, ParsedImportData};

    fn parse_import_data(
        validator: &ImportValidator,
        contents: &str,
    ) -> Result<(ImportPreview, ParsedImportData), String> {
        let booth_validation = validator.validate_booth_backup(contents).map(|data| {
            let preview = ImportPreview::Booth {
                description: data.booth.description.clone(),
                vendors: data.vendors.len(),
                purchases: data.purchases.len(),
            };
            (preview, ParsedImportData::Booth(data))
        });

        match booth_validation {
            Ok(booth) => Ok(booth),
            Err(ez_booth_storage::export::ImportError::InvalidJson(_))
            | Err(ez_booth_storage::export::ImportError::InvalidStructure(_)) => validator
                .validate_backup(contents)
                .map(|data| {
                    let preview = ImportPreview::Full {
                        booths: data.booths.len(),
                        vendors: data.vendors.len(),
                        purchases: data.purchases.len(),
                    };
                    (preview, ParsedImportData::Full(data))
                })
                .map_err(|err| err.to_string()),
            Err(err) => Err(err.to_string()),
        }
    }

    fn sample_booth() -> Booth {
        Booth::new(
            "Spring Market 2026".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
            FeeConfig {
                participation_fee: dec!(10.00),
                sales_fee_percent: dec!(15.00),
                rounding_step: dec!(0.50),
            },
        )
        .unwrap()
    }

    #[test]
    fn booth_backup_with_empty_lists_is_detected_as_booth_backup() {
        let validator = ImportValidator::new();
        let booth = sample_booth();
        let mut backup = BoothBackupData::new(booth.clone(), "test-version");
        backup.created_at = Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0).unwrap();

        let contents = serde_json::to_string(&backup).unwrap();
        let result = parse_import_data(&validator, &contents).unwrap();

        match result {
            (
                ImportPreview::Booth {
                    description,
                    vendors,
                    purchases,
                },
                ParsedImportData::Booth(data),
            ) => {
                assert_eq!(description, booth.description);
                assert_eq!(vendors, 0);
                assert_eq!(purchases, 0);
                assert_eq!(data.booth.id, booth.id);
            }
            other => panic!("expected booth backup parse result, got {other:?}"),
        }
    }
}
