use std::rc::Rc;

use leptos::html;
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, File as WebFile, FileReader, ProgressEvent};

use crate::components::{use_toast, Button, ButtonSize, ButtonVariant, Modal, ModalSize};
use crate::error_logging::{current_route, stack_trace, use_error_logger, ErrorLogDraft};
use crate::selected_booth_context::use_booth_list_version;
use crate::state::use_app_state;
use crate::t;
use crate::utils::current_device_info;
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

#[derive(Clone, Debug, PartialEq)]
struct ImportCandidate {
    file_name: String,
    preview: Option<ImportPreview>,
    parsed_data: Option<ParsedImportData>,
    validation_failures: Vec<ValidationFailure>,
    structure_error: Option<String>,
}

impl ImportCandidate {
    fn is_ready(&self) -> bool {
        self.preview.is_some() && self.parsed_data.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImportResultItem {
    file_name: String,
    success: bool,
    message: String,
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
    let log_error = use_error_logger();
    let input_ref = create_node_ref::<html::Input>();
    let (is_reading, set_is_reading) = create_signal(false);
    let (is_importing, set_is_importing) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (selected_file_names, set_selected_file_names) = create_signal(Vec::<String>::new());
    let (candidates, set_candidates) = create_signal(Vec::<ImportCandidate>::new());
    let (conflict_strategy, set_conflict_strategy) = create_signal(ConflictStrategy::Merge);
    let (import_progress, set_import_progress) = create_signal(None::<(usize, usize)>);
    let (import_results, set_import_results) = create_signal(Vec::<ImportResultItem>::new());

    let validator = Rc::new(ImportValidator::new());

    let open_file_picker = {
        let input_ref = input_ref.clone();
        move || {
            if is_reading.get_untracked() || is_importing.get_untracked() {
                return;
            }

            if let Some(input) = input_ref.get() {
                input.set_value("");
                input.click();
            }
        }
    };

    let reset_results = {
        let set_candidates = set_candidates;
        let set_selected_file_names = set_selected_file_names;
        let set_import_progress = set_import_progress;
        let set_import_results = set_import_results;
        move || {
            set_candidates.set(Vec::new());
            set_selected_file_names.set(Vec::new());
            set_import_progress.set(None);
            set_import_results.set(Vec::new());
        }
    };

    let log_error_for_file_change = log_error.clone();
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

            if files.length() == 0 {
                return;
            }

            reset_results();
            set_is_reading.set(true);

            let file_names = (0..files.length())
                .filter_map(|index| files.get(index))
                .map(|file| file.name())
                .collect::<Vec<_>>();
            set_selected_file_names.set(file_names);

            let candidates_signal = set_candidates;
            let show_modal_signal = set_show_modal;
            let reading_signal = set_is_reading;
            let toast = toast.clone();
            let log_error = log_error_for_file_change.clone();
            let validator_for_parse = Rc::clone(&validator);

            spawn_local(async move {
                let mut parsed_candidates = Vec::new();
                for index in 0..files.length() {
                    let Some(file) = files.get(index) else {
                        continue;
                    };

                    let candidate =
                        read_and_parse_file(file, Rc::clone(&validator_for_parse)).await;
                    parsed_candidates.push(candidate);
                }

                reading_signal.set(false);
                if parsed_candidates.is_empty() {
                    log_error(ErrorLogDraft {
                        error_type: "import_read_failed".to_string(),
                        error_message: t!("backup.import_failed")(),
                        stack_trace: stack_trace(),
                        user_action: Some("read import files".to_string()),
                        route: current_route(),
                        vendor_id: None,
                        purchase_id: None,
                        details: vec!["no import candidates produced".to_string()],
                    });
                    toast.error(t!("backup.import_failed")());
                    return;
                }

                candidates_signal.set(parsed_candidates);
                show_modal_signal.set(true);
            });
        }
    };

    let close_modal = move || {
        set_show_modal.set(false);
        set_candidates.set(Vec::new());
        set_selected_file_names.set(Vec::new());
        set_import_progress.set(None);
        set_import_results.set(Vec::new());
        set_conflict_strategy.set(ConflictStrategy::Merge);
    };

    let close_modal_action = store_value(close_modal);

    let handle_apply_import = move || {
        if is_importing.get_untracked() {
            return;
        }

        let state_result = app_state.get();
        let ready_candidates = candidates
            .get_untracked()
            .into_iter()
            .filter_map(|candidate| {
                candidate
                    .parsed_data
                    .clone()
                    .map(|parsed_data| (candidate.file_name, parsed_data))
            })
            .collect::<Vec<_>>();
        let strategy = conflict_strategy.get_untracked();

        if ready_candidates.is_empty() {
            return;
        }

        set_is_importing.set(true);
        set_import_progress.set(Some((0, ready_candidates.len())));
        set_import_results.set(Vec::new());
        let log_error = log_error.clone();

        spawn_local(async move {
            let state = match state_result {
                Some(Ok(state)) => state,
                Some(Err(error)) => {
                    set_is_importing.set(false);
                    set_import_progress.set(None);
                    toast.error(format!("{}: {}", t!("backup.import_apply_failed")(), error));
                    return;
                }
                None => {
                    set_is_importing.set(false);
                    set_import_progress.set(None);
                    toast.error(format!(
                        "{}: {}",
                        t!("backup.import_apply_failed")(),
                        t!("common.loading")()
                    ));
                    return;
                }
            };

            let mut combined = ImportSummary::default();
            let mut result_items = Vec::new();

            for (index, (file_name, payload)) in ready_candidates.iter().cloned().enumerate() {
                set_import_progress.set(Some((index + 1, ready_candidates.len())));

                let result = match payload {
                    ParsedImportData::Full(data) => {
                        state.import_service.import_all(data, strategy).await
                    }
                    ParsedImportData::Booth(data) => {
                        state
                            .import_service
                            .import_booth_backup_restoring_archived(
                                data,
                                strategy,
                                current_device_info(),
                            )
                            .await
                    }
                };

                match result {
                    Ok(summary) => {
                        combined.booths_imported += summary.booths_imported;
                        combined.vendors_imported += summary.vendors_imported;
                        combined.purchases_imported += summary.purchases_imported;
                        combined.conflicts_resolved += summary.conflicts_resolved;
                        combined.skipped_records.extend(summary.skipped_records);
                        result_items.push(ImportResultItem {
                            file_name,
                            success: true,
                            message: t!("backup.import_file_success")(),
                        });
                    }
                    Err(err) => {
                        log_error(ErrorLogDraft {
                            error_type: "import_apply_failed".to_string(),
                            error_message: err.to_string(),
                            stack_trace: stack_trace(),
                            user_action: Some("apply import".to_string()),
                            route: current_route(),
                            vendor_id: None,
                            purchase_id: None,
                            details: vec![format!("file={file_name}")],
                        });
                        result_items.push(ImportResultItem {
                            file_name,
                            success: false,
                            message: err.to_string(),
                        });
                    }
                }
            }

            set_is_importing.set(false);
            set_import_progress.set(None);
            set_import_results.set(result_items.clone());

            let success_count = result_items.iter().filter(|item| item.success).count();
            let failure_count = result_items.len().saturating_sub(success_count);

            if combined.booths_imported > 0
                || combined.vendors_imported > 0
                || combined.purchases_imported > 0
                || combined.conflicts_resolved > 0
            {
                booth_list_version.update(|version| *version += 1);
            }

            if failure_count == 0 {
                let message = t!("backup.import_apply_success")()
                    .replace("{booths}", &combined.booths_imported.to_string())
                    .replace("{vendors}", &combined.vendors_imported.to_string())
                    .replace("{purchases}", &combined.purchases_imported.to_string())
                    .replace("{resolved}", &combined.conflicts_resolved.to_string())
                    .replace("{skipped}", &combined.skipped_records.len().to_string());
                toast.success(message);
                close_modal_action.with_value(|close| close());
            } else {
                let message = t!("backup.import_partial_success")()
                    .replace("{success}", &success_count.to_string())
                    .replace("{failed}", &failure_count.to_string());
                toast.error(message);
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
    let handle_apply_import_action = store_value(handle_apply_import);

    let ready_count = Signal::derive(move || {
        candidates
            .get()
            .iter()
            .filter(|item| item.is_ready())
            .count()
    });
    let invalid_count = Signal::derive(move || {
        candidates
            .get()
            .iter()
            .filter(|item| !item.is_ready())
            .count()
    });
    let has_ready_candidates = Signal::derive(move || ready_count.get() > 0);

    view! {
        <>
            <input
                node_ref=input_ref
                type="file"
                accept="application/json,.json"
                class="hidden"
                multiple
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
                action_bar=
                    view! {
                        <div class="contents">
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Box::new(move || close_modal_action.with_value(|close| close()))
                            >
                                {t!("common.close")}
                            </Button>
                            <Show when=move || has_ready_candidates.get()>
                                <Button
                                    on_click=Box::new(move || handle_apply_import_action.with_value(|handler| handler()))
                                    disabled=is_importing.get()
                                >
                                    {move || if is_importing.get() {
                                        t!("backup.import_in_progress")()
                                    } else {
                                        t!("backup.import_apply")()
                                    }}
                                </Button>
                            </Show>
                        </div>
                    }
                    .into_view()
            >
                <div class="space-y-4 text-gray-700">
                    <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3 text-sm">
                        <span class="font-medium text-gray-900">{t!("backup.selected_files")}</span>
                        <span>{move || format!(" {}", selected_file_names.get().join(", "))}</span>
                    </div>

                    <div class="grid gap-3 sm:grid-cols-3">
                        <div class="rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-900">
                            <p class="font-semibold">{t!("backup.import_ready_count")}</p>
                            <p>{move || ready_count.get().to_string()}</p>
                        </div>
                        <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">
                            <p class="font-semibold">{t!("backup.import_blocked_count")}</p>
                            <p>{move || invalid_count.get().to_string()}</p>
                        </div>
                        <Show when=move || import_progress.get().is_some()>
                            {move || import_progress.get().map(|(current, total)| view! {
                                <div class="rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                                    <p class="font-semibold">{t!("backup.import_progress_label")}</p>
                                    <p>{t!("backup.import_progress_status")()
                                        .replace("{current}", &current.to_string())
                                        .replace("{total}", &total.to_string())}</p>
                                </div>
                            })}
                        </Show>
                    </div>

                    <Show when=move || has_ready_candidates.get()>
                        <div class="space-y-2 rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-900">
                            <p class="font-medium">{t!("backup.import_strategy_label")}</p>
                            <div class="space-y-1">
                                {[
                                    ("merge", ConflictStrategy::Merge, t!("backup.strategy_merge"), t!("backup.strategy_merge_desc")),
                                    ("skip",  ConflictStrategy::Skip,  t!("backup.strategy_skip"),  t!("backup.strategy_skip_desc")),
                                    ("replace", ConflictStrategy::Replace, t!("backup.strategy_replace"), t!("backup.strategy_replace_desc")),
                                ].into_iter().map(|(val, strat, label, desc)| {
                                    let is_selected = move || conflict_strategy.get() == strat;
                                    view! {
                                        <label class="flex cursor-pointer items-start gap-2 rounded-md px-2 py-1.5 hover:bg-blue-100">
                                            <input
                                                type="radio"
                                                name="conflict_strategy"
                                                value=val
                                                class="mt-0.5 accent-blue-600"
                                                prop:checked=is_selected
                                                on:change=move |ev| on_strategy_change_action.with_value(|handler| handler(ev))
                                            />
                                            <div>
                                                <p class="font-medium leading-snug">{label}</p>
                                                <p class="text-xs text-blue-700 leading-snug">{desc}</p>
                                            </div>
                                        </label>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <p class="text-xs text-blue-700">{t!("backup.import_apply_ready")}</p>
                        </div>
                    </Show>

                    <div class="space-y-3 max-h-[28rem] overflow-y-auto pr-1">
                        <For
                            each=move || candidates.get()
                            key=|candidate| candidate.file_name.clone()
                            children=move |candidate| {
                                let file_name = candidate.file_name;
                                let preview = candidate.preview;
                                let validation_failures = candidate.validation_failures;
                                let structure_error = candidate.structure_error;
                                let is_ready = preview.is_some();
                                let preview_view = match preview.clone() {
                                    Some(ImportPreview::Full {
                                        booths,
                                        vendors,
                                        purchases,
                                    }) => Some(
                                        view! {
                                            <p class="mt-2 text-sm text-gray-600">
                                                {t!("backup.import_counts")()
                                                    .replace("{booths}", &booths.to_string())
                                                    .replace("{vendors}", &vendors.to_string())
                                                    .replace("{purchases}", &purchases.to_string())}
                                            </p>
                                        }
                                            .into_view(),
                                    ),
                                    Some(ImportPreview::Booth {
                                        description,
                                        vendors,
                                        purchases,
                                    }) => Some(
                                        view! {
                                            <div class="mt-2 space-y-1 text-sm text-gray-600">
                                                <p>{t!("backup.import_valid_booth")().replace("{description}", &description)}</p>
                                                <p>
                                                    {t!("backup.import_booth_counts")()
                                                        .replace("{vendors}", &vendors.to_string())
                                                        .replace("{purchases}", &purchases.to_string())}
                                                </p>
                                            </div>
                                        }
                                            .into_view(),
                                    ),
                                    None => None,
                                };
                                let validation_view = if validation_failures.is_empty() {
                                    None
                                } else {
                                    Some(
                                        view! {
                                            <div class="mt-3 space-y-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-900">
                                                <p class="font-medium">{t!("backup.import_validation_summary")}</p>
                                                <For
                                                    each=move || validation_failures.clone()
                                                    key=|failure| format!("{}:{}:{}", failure.record_type, failure.record_id, failure.reason)
                                                    children=move |failure| {
                                                        view! {
                                                            <div class="rounded border border-red-100 bg-white px-3 py-2">
                                                                <p class="font-medium">{format!("{} {}", failure.record_type, failure.record_id)}</p>
                                                                <p>{failure.reason}</p>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }
                                            .into_view(),
                                    )
                                };
                                let structure_view = structure_error.map(|message| {
                                    view! {
                                        <div class="mt-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">
                                            <p class="font-medium">{t!("backup.import_structure_summary")}</p>
                                            <pre class="mt-2 whitespace-pre-wrap">{message}</pre>
                                        </div>
                                    }
                                        .into_view()
                                });
                                view! {
                                    <div class="rounded-lg border border-gray-200 bg-white px-4 py-3 shadow-sm">
                                        <div class="flex items-start justify-between gap-3">
                                            <div>
                                                <p class="font-medium text-gray-900">{file_name.clone()}</p>
                                                <Show when=move || is_ready>
                                                    <p class="text-sm text-green-700">{t!("backup.import_ready_label")}</p>
                                                </Show>
                                                <Show when=move || !is_ready>
                                                    <p class="text-sm text-red-700">{t!("backup.import_invalid_label")}</p>
                                                </Show>
                                            </div>
                                        </div>

                                        {preview_view}
                                        {validation_view}
                                        {structure_view}
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || !import_results.get().is_empty()>
                        <div class="space-y-3 rounded-lg border border-gray-200 bg-gray-50 px-4 py-3 text-sm text-gray-800">
                            <p class="font-medium text-gray-900">{t!("backup.import_results_label")}</p>
                            <For
                                each=move || import_results.get()
                                key=|item| item.file_name.clone()
                                children=move |item| {
                                    let status_class = if item.success {
                                        "text-green-700"
                                    } else {
                                        "text-red-700"
                                    };
                                    view! {
                                        <div class="rounded border border-gray-200 bg-white px-3 py-2">
                                            <p class="font-medium text-gray-900">{item.file_name}</p>
                                            <p class=status_class>{item.message}</p>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>
            </Modal>
        </>
    }
}

async fn read_and_parse_file(file: WebFile, validator: Rc<ImportValidator>) -> ImportCandidate {
    let file_name = file.name();

    match read_file_as_text(&file).await {
        Ok(contents) => match parse_import_data(&validator, &contents) {
            Ok((preview, parsed_data)) => ImportCandidate {
                file_name,
                preview: Some(preview),
                parsed_data: Some(parsed_data),
                validation_failures: Vec::new(),
                structure_error: None,
            },
            Err(ImportError::ValidationFailed { failures }) => ImportCandidate {
                file_name,
                preview: None,
                parsed_data: None,
                validation_failures: failures,
                structure_error: None,
            },
            Err(other) => ImportCandidate {
                file_name,
                preview: None,
                parsed_data: None,
                validation_failures: Vec::new(),
                structure_error: Some(format_import_error(&other)),
            },
        },
        Err(message) => ImportCandidate {
            file_name,
            preview: None,
            parsed_data: None,
            validation_failures: Vec::new(),
            structure_error: Some(message),
        },
    }
}

fn parse_import_data(
    validator: &ImportValidator,
    contents: &str,
) -> Result<(ImportPreview, ParsedImportData), ImportError> {
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
        Err(ImportError::InvalidJson(_)) | Err(ImportError::InvalidStructure(_)) => {
            // Only fall back when the payload does not match the booth backup shape.
            // Once a file parses as a booth backup, checksum/version/validation errors
            // should be shown directly instead of reinterpreting the same payload as a
            // different backup type.
            validator.validate_backup(contents).map(|data| {
                let preview = ImportPreview::Full {
                    booths: data.booths.len(),
                    vendors: data.vendors.len(),
                    purchases: data.purchases.len(),
                };
                (preview, ParsedImportData::Full(data))
            })
        }
        Err(other) => Err(other),
    }
}

fn format_import_error(error: &ImportError) -> String {
    match error {
        ImportError::UnsupportedVersion { found, supported } => t!("backup.import_version_error")()
            .replace("{found}", &found.to_string())
            .replace("{supported}", &supported.to_string()),
        ImportError::OrphanedRecords { details } => details.join("\n"),
        ImportError::InvalidStructure(message) | ImportError::InvalidJson(message) => {
            message.clone()
        }
        ImportError::ChecksumMismatch { expected, actual } => t!("backup.checksum_failed")()
            .replace(
                "{detail}",
                &t!("backup.checksum_mismatch_detail")()
                    .replace("{expected}", expected)
                    .replace("{actual}", actual),
            ),
        _ => error.to_string(),
    }
}

async fn read_file_as_text(file: &WebFile) -> Result<String, String> {
    let reader =
        FileReader::new().map_err(|err| format!("{}: {:?}", t!("backup.import_failed")(), err))?;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader_for_load = reader.clone();
        let resolve_fn = resolve.clone();
        let reject_for_load = reject.clone();

        let onload =
            Closure::once(Box::new(
                move |_event: ProgressEvent| match reader_for_load.result() {
                    Ok(result) => {
                        if let Some(contents) = result.as_string() {
                            let _ = resolve_fn.call1(&JsValue::NULL, &JsValue::from_str(&contents));
                        } else {
                            let _ = reject_for_load.call1(
                                &JsValue::NULL,
                                &JsValue::from_str(&t!("backup.import_invalid_encoding")()),
                            );
                        }
                    }
                    Err(_) => {
                        let _ = reject_for_load.call1(
                            &JsValue::NULL,
                            &JsValue::from_str(&t!("backup.import_failed")()),
                        );
                    }
                },
            ) as Box<dyn FnOnce(_)>);

        let reject_for_error = reject.clone();
        let onerror = Closure::once(Box::new(move |_event: ProgressEvent| {
            let _ = reject_for_error.call1(
                &JsValue::NULL,
                &JsValue::from_str(&t!("backup.import_failed")()),
            );
        }) as Box<dyn FnOnce(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        if let Err(err) = reader.read_as_text(file) {
            let _ = reject.call1(
                &JsValue::NULL,
                &JsValue::from_str(&format!("{}: {:?}", t!("backup.import_failed")(), err)),
            );
        }

        onload.forget();
        onerror.forget();
    });

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| t!("backup.import_failed")())
        })?
        .as_string()
        .ok_or_else(|| t!("backup.import_invalid_encoding")())
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use domain::{Booth, FeeConfig};
    use ez_booth_storage::export::{
        BackupData, BoothBackupData, DeviceInfo, ImportError, ImportValidator,
        BACKUP_FORMAT_VERSION,
    };
    use rust_decimal_macros::dec;

    use super::{parse_import_data, ImportPreview, ParsedImportData};

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
        backup.device_info = Some(DeviceInfo {
            identifier: "marias-laptop".to_string(),
            platform: "Windows".to_string(),
            browser: "Chrome".to_string(),
        });

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
                assert_eq!(
                    data.device_info.unwrap().identifier,
                    "marias-laptop".to_string()
                );
            }
            other => panic!("expected booth backup parse result, got {other:?}"),
        }
    }

    #[test]
    fn full_backup_with_empty_lists_is_detected_as_full_backup() {
        let validator = ImportValidator::new();
        let booth = sample_booth();
        let backup = BackupData {
            version: BACKUP_FORMAT_VERSION,
            created_at: Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0).unwrap(),
            app_version: "test-version".to_string(),
            checksum: None,
            device_info: Some(DeviceInfo {
                identifier: "marias-laptop".to_string(),
                platform: "Windows".to_string(),
                browser: "Chrome".to_string(),
            }),
            booths: vec![booth.clone()],
            vendors: Vec::new(),
            purchases: Vec::new(),
            metadata: Default::default(),
        };

        let contents = serde_json::to_string(&backup).unwrap();
        let result = parse_import_data(&validator, &contents).unwrap();

        match result {
            (
                ImportPreview::Full {
                    booths,
                    vendors,
                    purchases,
                },
                ParsedImportData::Full(data),
            ) => {
                assert_eq!(booths, 1);
                assert_eq!(vendors, 0);
                assert_eq!(purchases, 0);
                assert_eq!(data.booths[0].id, booth.id);
                assert_eq!(
                    data.device_info.unwrap().identifier,
                    "marias-laptop".to_string()
                );
            }
            other => panic!("expected full backup parse result, got {other:?}"),
        }
    }

    #[test]
    fn booth_backup_checksum_mismatch_is_reported_without_fallback() {
        let validator = ImportValidator::new();
        let booth = sample_booth();
        let mut backup = BoothBackupData::new(booth, "test-version");
        backup.created_at = Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0).unwrap();
        backup.checksum = Some("wrong".to_string());

        let contents = serde_json::to_string(&backup).unwrap();
        let error = parse_import_data(&validator, &contents).unwrap_err();

        assert!(matches!(error, ImportError::ChecksumMismatch { .. }));
    }
}
