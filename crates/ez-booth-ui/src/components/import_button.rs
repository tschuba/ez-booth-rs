use std::rc::Rc;

use leptos::html;
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, FileReader, ProgressEvent};

use crate::components::{use_toast, Button, ButtonSize, ButtonVariant, Modal, ModalSize};
use crate::t;
use ez_booth_storage::export::{ImportError, ImportValidator, ValidationFailure};

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

#[component]
pub fn ImportButton(
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let toast = use_toast();
    let input_ref = create_node_ref::<html::Input>();
    let (is_reading, set_is_reading) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (selected_file_name, set_selected_file_name) = create_signal(String::new());
    let (preview, set_preview) = create_signal(None::<ImportPreview>);
    let (validation_failures, set_validation_failures) =
        create_signal(Vec::<ValidationFailure>::new());
    let (structure_error, set_structure_error) = create_signal(None::<String>);

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
        let set_validation_failures = set_validation_failures;
        let set_structure_error = set_structure_error;
        move || {
            set_preview.set(None);
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

                let validation =
                    validator_for_load
                        .validate_backup(&contents)
                        .map(|data| ImportPreview::Full {
                            booths: data.booths.len(),
                            vendors: data.vendors.len(),
                            purchases: data.purchases.len(),
                        });

                let resolved = match validation {
                    Ok(preview) => Ok(preview),
                    Err(ImportError::InvalidJson(_)) => validator_for_load
                        .validate_booth_backup(&contents)
                        .map(|data| ImportPreview::Booth {
                            description: data.booth.description,
                            vendors: data.vendors.len(),
                            purchases: data.purchases.len(),
                        }),
                    Err(other) => Err(other),
                };

                set_is_reading.set(false);
                match resolved {
                    Ok(next_preview) => {
                        set_preview.set(Some(next_preview));
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
        set_validation_failures.set(Vec::new());
        set_structure_error.set(None);
    };

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
                disabled=is_reading.get()
                title=t!("backup.import")()
            >
                {move || if is_reading.get() {
                    t!("backup.import_in_progress")()
                } else {
                    t!("backup.import")()
                }}
            </Button>

            <Modal
                show=Signal::derive(move || show_modal.get())
                on_close=close_modal
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
                                            <p class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
                                                {t!("backup.import_next_step_placeholder")}
                                            </p>
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
                                            <p class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
                                                {t!("backup.import_next_step_placeholder")}
                                            </p>
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

                    <div class="flex justify-end">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(close_modal)>
                            {t!("common.close")}
                        </Button>
                    </div>
                </div>
            </Modal>
        </>
    }
}
