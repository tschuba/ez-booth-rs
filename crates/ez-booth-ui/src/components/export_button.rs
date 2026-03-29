use wasm_bindgen::JsCast;
use web_sys::{window, Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use crate::components::{
    use_toast, Button, ButtonSize, ButtonVariant, DropdownMenuItem, QrExportModal,
};
use crate::components::dropdown_menu::close_all_dropdown_menus;
use crate::state::use_app_state;
use crate::t;
use domain::BoothId;
use leptos::*;

fn booth_id_for_scope(scope: ExportScope) -> Option<BoothId> {
    match scope {
        ExportScope::Booth(id) => Some(id),
        ExportScope::All => None,
    }
}

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
    #[prop(default = false)] menu_item: bool,
) -> impl IntoView {
    let app_state = use_app_state();
    let toast = use_toast();
    let (is_exporting, set_is_exporting) = create_signal(false);
    let (show_qr_modal, set_show_qr_modal) = create_signal(false);
    let booth_id = booth_id_for_scope(scope);

    let label = move || match scope {
        ExportScope::All => t!("backup.export_all")(),
        ExportScope::Booth(_) => t!("backup.export_booth")(),
    };

    let success_message = move || match scope {
        ExportScope::All => t!("backup.export_success_all")(),
        ExportScope::Booth(_) => t!("backup.export_success_booth")(),
    };

    let menu_icon = move || {
        if is_exporting.get() {
            view! {
                <svg class="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                    <path class="opacity-75" fill="currentColor" d="M12 2a10 10 0 0 1 10 10h-3a7 7 0 0 0-7-7V2z"></path>
                </svg>
            }
                .into_view()
        } else {
            view! {
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                </svg>
            }
                .into_view()
        }
    };

    let qr_menu_icon = view! {
        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4z"></path>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 14h2m2 0h2m-6 3h6m-6 3h2m2 0h2"></path>
        </svg>
    }
    .into_view();

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

    let open_qr_export = move || {
        if booth_id.is_some() {
            set_show_qr_modal.set(true);
        }
    };

    let close_qr_export = move || {
        set_show_qr_modal.set(false);
        close_all_dropdown_menus();
    };

    if menu_item {
        view! {
            <>
                <DropdownMenuItem
                    on_click=Callback::new(move |_| handle_export())
                    icon=menu_icon()
                >
                    {move || if is_exporting.get() {
                        t!("backup.export_in_progress")()
                    } else {
                        label()
                    }}
                </DropdownMenuItem>

                <Show when=move || matches!(scope, ExportScope::Booth(_))>
                    <DropdownMenuItem
                        on_click=Callback::new(move |event: ev::MouseEvent| {
                            event.stop_propagation();
                            open_qr_export();
                        })
                        icon=qr_menu_icon.clone()
                        class="js-qr-export-menu-item".to_string()
                    >
                        {t!("backup.qr_export_menu")}
                    </DropdownMenuItem>
                </Show>

                {move || {
                    booth_id.map(|booth_id| {
                        view! {
                            <QrExportModal
                                booth_id
                                show=show_qr_modal
                                on_close=close_qr_export
                                on_use_json=handle_export
                            />
                        }
                    })
                }}
            </>
        }
        .into_view()
    } else {
        view! {
            <>
                <Button
                    on_click=Box::new(handle_export)
                    variant=variant.unwrap_or(ButtonVariant::Primary)
                    size=size.unwrap_or(ButtonSize::Medium)
                    class=class.unwrap_or_default()
                    disabled=is_exporting.get()
                    title=label()
                    aria_label=label()
                >
                    {move || if is_exporting.get() {
                        t!("backup.export_in_progress")()
                    } else {
                        label()
                    }}
                </Button>

                {move || {
                    booth_id.map(|booth_id| {
                        view! {
                            <QrExportModal
                                booth_id
                                show=show_qr_modal
                                on_close=close_qr_export
                                on_use_json=handle_export
                            />
                        }
                    })
                }}
            </>
        }
        .into_view()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booth_scope_returns_booth_id() {
        let booth_id = BoothId::new();

        assert_eq!(booth_id_for_scope(ExportScope::Booth(booth_id)), Some(booth_id));
    }

    #[test]
    fn all_scope_has_no_booth_id() {
        assert_eq!(booth_id_for_scope(ExportScope::All), None);
    }
}
