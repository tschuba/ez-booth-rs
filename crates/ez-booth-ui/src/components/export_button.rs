use wasm_bindgen::JsCast;
use web_sys::{window, Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use crate::components::{use_toast, Button, ButtonSize, ButtonVariant};
use crate::state::use_app_state;
use crate::t;
use domain::BoothId;
use leptos::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportScope {
    All,
    Booth(BoothId),
}

#[component]
pub fn ExportButton(
    scope: ExportScope,
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let app_state = use_app_state();
    let toast = use_toast();
    let (is_exporting, set_is_exporting) = create_signal(false);

    let label = move || match scope {
        ExportScope::All => t!("backup.export_all")(),
        ExportScope::Booth(_) => t!("backup.export_booth")(),
    };

    let success_message = move || match scope {
        ExportScope::All => t!("backup.export_success_all")(),
        ExportScope::Booth(_) => t!("backup.export_success_booth")(),
    };

    let handle_export = move || {
        if is_exporting.get_untracked() {
            return;
        }

        let state_result = app_state.get();
        set_is_exporting.set(true);

        spawn_local(async move {
            let result: Result<(), String> = async move {
                let state = match state_result {
                    Some(Ok(state)) => state,
                    Some(Err(error)) => return Err(error),
                    None => return Err(t!("common.loading")()),
                };

                let serialized = match scope {
                    ExportScope::All => {
                        let data = state
                            .export_service
                            .export_all()
                            .await
                            .map_err(|err| err.to_string())?;
                        state
                            .export_service
                            .serialize_full_backup(&data)
                            .map_err(|err| err.to_string())?
                    }
                    ExportScope::Booth(booth_id) => {
                        let data = state
                            .export_service
                            .export_booth(&booth_id)
                            .await
                            .map_err(|err| err.to_string())?;
                        state
                            .export_service
                            .serialize_booth_backup(&data)
                            .map_err(|err| err.to_string())?
                    }
                };

                trigger_download(&serialized.file_name, &serialized.json)
                    .map_err(|err| err.to_string())?;

                Ok::<(), String>(())
            }
            .await;

            set_is_exporting.set(false);

            match result {
                Ok(()) => toast.success(success_message()),
                Err(error) => {
                    let message = t!("backup.export_failed")();
                    toast.error(format!("{message}: {error}"));
                }
            }
        });
    };

    view! {
        <Button
            on_click=Box::new(handle_export)
            variant=variant.unwrap_or(ButtonVariant::Primary)
            size=size.unwrap_or(ButtonSize::Medium)
            class=class.unwrap_or_default()
            disabled=is_exporting.get()
            title=label()
        >
            {move || if is_exporting.get() {
                t!("backup.export_in_progress")()
            } else {
                label()
            }}
        </Button>
    }
}

fn trigger_download(file_name: &str, contents: &str) -> Result<(), String> {
    let window = window().ok_or_else(|| "window not available".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "document not available".to_string())?;

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(contents));

    let options = BlobPropertyBag::new();
    options.set_type("application/json;charset=utf-8");

    let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &options)
        .map_err(|err| format!("failed to create blob: {err:?}"))?;
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|err| format!("failed to create object URL: {err:?}"))?;

    let anchor = document
        .create_element("a")
        .map_err(|err| format!("failed to create anchor: {err:?}"))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "failed to cast anchor element".to_string())?;

    anchor.set_href(&url);
    anchor.set_download(file_name);

    let body = document
        .body()
        .ok_or_else(|| "document body not available".to_string())?;

    body.append_child(&anchor)
        .map_err(|err| format!("failed to append anchor: {err:?}"))?;
    anchor.click();
    let _ = body.remove_child(&anchor);
    Url::revoke_object_url(&url).map_err(|err| format!("failed to revoke object URL: {err:?}"))?;

    Ok(())
}
