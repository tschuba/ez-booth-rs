use std::collections::HashMap;

use domain::BoothId;
use ez_booth_storage::export::{BoothBackupData, ExportScope, RenderedQrChunk, MAX_QR_CODES};
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast};

use crate::components::{Button, ButtonVariant, Modal, ModalSize};
use crate::i18n::translate_with_params;
use crate::state::use_app_state;
use crate::t;

const QR_WARNING_THRESHOLD_UI: usize = 5;
const QR_ROTATION_INTERVAL_MS: i32 = 2_000;
const QR_ROTATION_TICK_MS: i32 = 100;
const ESTIMATED_QR_BASE_BINARY_BYTES: usize = 400;
const ESTIMATED_QR_VENDOR_BINARY_BYTES: usize = 80;
const ESTIMATED_QR_PURCHASE_BINARY_BYTES: usize = 200;
const ESTIMATED_QR_COMPRESSION_PERCENT: usize = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QrExportStage {
    Configure,
    ConfirmLarge,
    Display,
}

#[component]
pub fn QrExportModal(
    booth_id: BoothId,
    #[prop(into)] show: Signal<bool>,
    on_close: impl Fn() + Clone + 'static,
    on_use_json: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let app_state = use_app_state();
    let on_close_action = on_close.clone();
    let on_close_footer = on_close.clone();
    let on_use_json_action = on_use_json.clone();
    let (stage, set_stage) = create_signal(QrExportStage::Configure);
    let (scope, set_scope) = create_signal(ExportScope::Week);
    let (backup, set_backup) = create_signal(None::<BoothBackupData>);
    let (chunks, set_chunks) = create_signal(Vec::<RenderedQrChunk>::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (is_generating, set_is_generating) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (current_index, set_current_index) = create_signal(0_usize);
    let (is_paused, set_is_paused) = create_signal(false);
    let (countdown_ms, set_countdown_ms) = create_signal(QR_ROTATION_INTERVAL_MS);

    let reset_display = move || {
        set_chunks.set(Vec::new());
        set_current_index.set(0);
        set_is_paused.set(false);
        set_countdown_ms.set(QR_ROTATION_INTERVAL_MS);
    };

    create_effect(move |_| {
        if show.get() {
            set_stage.set(QrExportStage::Configure);
            set_scope.set(ExportScope::Week);
            set_error.set(None);
            reset_display();
            set_backup.set(None);
            set_is_loading.set(true);

            let state_result = app_state.get();
            spawn_local(async move {
                let result = async move {
                    let state = match state_result {
                        Some(Ok(state)) => state,
                        Some(Err(error)) => return Err(error),
                        None => return Err(t!("common.loading")()),
                    };

                    state
                        .export_service
                        .export_booth(&booth_id)
                        .await
                        .map_err(|err| err.to_string())
                }
                .await;

                set_is_loading.set(false);
                match result {
                    Ok(value) => set_backup.set(Some(value)),
                    Err(message) => set_error.set(Some(message)),
                }
            });
        } else {
            set_stage.set(QrExportStage::Configure);
            set_scope.set(ExportScope::Week);
            set_error.set(None);
            set_backup.set(None);
            reset_display();
        }
    });

    create_effect(move |_| {
        let should_rotate = show.get()
            && stage.get() == QrExportStage::Display
            && !is_paused.get()
            && chunks.with(|items| items.len() > 1);

        if !should_rotate {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };

        let closure = Closure::wrap(Box::new(move || {
            let total = chunks.with(|items| items.len());
            if total <= 1 {
                return;
            }

            set_countdown_ms.update(|remaining| {
                if *remaining <= QR_ROTATION_TICK_MS {
                    *remaining = QR_ROTATION_INTERVAL_MS;
                    set_current_index.update(|index| *index = (*index + 1) % total);
                } else {
                    *remaining -= QR_ROTATION_TICK_MS;
                }
            });
        }) as Box<dyn FnMut()>);

        let handle = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                QR_ROTATION_TICK_MS,
            )
            .ok();

        on_cleanup(move || {
            if let Some(handle) = handle {
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(handle);
                }
            }
            drop(closure);
        });
    });

    let description = Signal::derive(move || {
        backup
            .get()
            .map(|value| value.booth.description)
            .unwrap_or_default()
    });
    let vendor_count =
        Signal::derive(move || backup.get().map(|value| value.vendors.len()).unwrap_or(0));
    let total_purchase_count =
        Signal::derive(move || backup.get().map(|value| value.purchases.len()).unwrap_or(0));
    let filtered_purchase_count = Signal::derive(move || {
        backup
            .get()
            .map(|value| scope.get().filter_purchases(&value.purchases).len())
            .unwrap_or(0)
    });
    let estimated_codes = Signal::derive(move || {
        ez_booth_storage::export::estimate_qr_count(
            vendor_count.get(),
            filtered_purchase_count.get(),
            scope.get(),
        )
    });
    let warning_needed = Signal::derive(move || {
        let count = estimated_codes.get();
        count >= QR_WARNING_THRESHOLD_UI && count <= MAX_QR_CODES
    });
    let exceeds_limit = Signal::derive(move || estimated_codes.get() > MAX_QR_CODES);
    let size_label = Signal::derive(move || {
        // This mirrors the current QR transfer payload shape closely enough for UI guidance:
        // booth metadata plus per-vendor/per-purchase MessagePack records, then an approximate
        // 70% gzip reduction. Keep these values aligned with the storage-layer QR format.
        let binary_size = ESTIMATED_QR_BASE_BINARY_BYTES
            + vendor_count.get() * ESTIMATED_QR_VENDOR_BINARY_BYTES
            + filtered_purchase_count.get() * ESTIMATED_QR_PURCHASE_BINARY_BYTES;
        let bytes = binary_size
            .saturating_mul(ESTIMATED_QR_COMPRESSION_PERCENT)
            .div_ceil(100);
        if bytes >= 1_000 {
            format!("~{:.1} KB", bytes as f64 / 1_000.0)
        } else {
            format!("~{} B", bytes)
        }
    });
    let progress_label = Signal::derive(move || {
        translate_with_params(
            "backup.qr_progress",
            HashMap::from([
                ("current", (current_index.get() + 1).to_string()),
                ("total", chunks.with(|items| items.len()).to_string()),
            ]),
        )
    });
    let countdown_label =
        Signal::derive(move || format!("{:.1}s", countdown_ms.get() as f64 / 1000.0));
    let countdown_width = Signal::derive(move || {
        let elapsed = QR_ROTATION_INTERVAL_MS.saturating_sub(countdown_ms.get());
        format!(
            "width: {:.2}%;",
            ((elapsed as f64 / QR_ROTATION_INTERVAL_MS as f64) * 100.0).clamp(0.0, 100.0)
        )
    });
    let dot_indexes =
        Signal::derive(move || (0..chunks.with(|items| items.len())).collect::<Vec<_>>());

    let generate_qr = move || {
        if is_generating.get_untracked() || exceeds_limit.get_untracked() {
            return;
        }

        let state_result = app_state.get();
        let selected_scope = scope.get_untracked();
        set_is_generating.set(true);
        set_error.set(None);

        spawn_local(async move {
            let result = async move {
                let state = match state_result {
                    Some(Ok(state)) => state,
                    Some(Err(error)) => return Err(error),
                    None => return Err(t!("common.loading")()),
                };

                let export = state
                    .qr_export_service
                    .export_booth_as_qr(&booth_id, selected_scope)
                    .await
                    .map_err(|err| err.to_string())?;

                state
                    .qr_export_service
                    .render_svg_chunks(&export.chunks)
                    .map_err(|err| err.to_string())
            }
            .await;

            set_is_generating.set(false);
            match result {
                Ok(rendered) => {
                    set_chunks.set(rendered);
                    set_current_index.set(0);
                    set_is_paused.set(false);
                    set_countdown_ms.set(QR_ROTATION_INTERVAL_MS);
                    set_stage.set(QrExportStage::Display);
                }
                Err(message) => {
                    set_error.set(Some(message));
                    set_stage.set(QrExportStage::Configure);
                }
            }
        });
    };

    let request_generate = move || {
        if warning_needed.get_untracked() {
            set_stage.set(QrExportStage::ConfirmLarge);
        } else {
            generate_qr();
        }
    };

    let previous_code = move || {
        let total = chunks.with(|items| items.len());
        if total <= 1 {
            return;
        }
        set_current_index.update(|index| *index = if *index == 0 { total - 1 } else { *index - 1 });
        set_countdown_ms.set(QR_ROTATION_INTERVAL_MS);
    };

    let next_code = move || {
        let total = chunks.with(|items| items.len());
        if total <= 1 {
            return;
        }
        set_current_index.update(|index| *index = (*index + 1) % total);
        set_countdown_ms.set(QR_ROTATION_INTERVAL_MS);
    };

    let action_bar_view = move || {
        match stage.get() {
            QrExportStage::Configure => view! {
                <div class="contents">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(on_close.clone())>
                        {t!("common.close")}
                    </Button>
                    <Button
                        on_click=Box::new(request_generate)
                        disabled=is_loading.get() || is_generating.get() || backup.get().is_none() || exceeds_limit.get()
                    >
                        {move || if is_generating.get() { t!("backup.qr_generating")() } else { t!("backup.qr_generate")() }}
                    </Button>
                </div>
            }
            .into_view(),
            QrExportStage::ConfirmLarge => view! {
                <div class="contents">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move || set_stage.set(QrExportStage::Configure))>
                        {t!("common.back")}
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click=Box::new(move || {
                            on_close_action.clone()();
                            on_use_json_action.clone()();
                        })
                    >
                        {t!("backup.qr_use_json")}
                    </Button>
                    <Button on_click=Box::new(generate_qr) disabled=is_generating.get()>
                        {move || if is_generating.get() { t!("backup.qr_generating")() } else { t!("backup.qr_continue")() }}
                    </Button>
                </div>
            }
            .into_view(),
            QrExportStage::Display => view! {
                <div class="contents">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move || set_stage.set(QrExportStage::Configure))>
                        {t!("backup.qr_reconfigure")}
                    </Button>
                    <Button variant=ButtonVariant::Primary on_click=Box::new(on_close.clone())>
                        {t!("common.close")}
                    </Button>
                </div>
            }
            .into_view(),
        }
    };

    view! {
        <Modal
            show=show
            on_close=move || on_close_footer.clone()()
            title=Signal::derive(move || t!("backup.qr_export_title")())
            size=ModalSize::XLarge
            action_bar=action_bar_view()
        >
            <div class="space-y-5 text-gray-700">
                <Show when=move || !description.get().is_empty()>
                    <div class="rounded-xl border border-gray-200 bg-gray-50 px-4 py-3">
                        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500">{t!("backup.qr_export_event_label")}</p>
                        <p class="mt-1 text-lg font-semibold text-gray-900">{move || description.get()}</p>
                    </div>
                </Show>

                <Show when=move || error.get().is_some()>
                    {move || error.get().map(|message| view! {
                        <div class="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">{message}</div>
                    })}
                </Show>

                <Show when=move || is_loading.get()>
                    <div class="rounded-xl border border-gray-200 bg-white px-4 py-12 text-center shadow-sm">
                        <div class="mx-auto h-10 w-10 animate-spin rounded-full border-4 border-gray-200 border-t-blue-600"></div>
                        <p class="mt-4 text-sm text-gray-600">{t!("backup.qr_loading_event")}</p>
                    </div>
                </Show>

                <Show when=move || !is_loading.get() && stage.get() == QrExportStage::Configure>
                    <div class="space-y-5">
                        <div class="grid gap-3 md:grid-cols-2">
                            <ScopeCard active=Signal::derive(move || scope.get() == ExportScope::Today) title=Signal::derive(move || t!("backup.qr_scope_today")()) subtitle=Signal::derive(move || t!("backup.qr_scope_today_hint")()) on_click=Callback::new(move |_| set_scope.set(ExportScope::Today)) />
                            <ScopeCard active=Signal::derive(move || scope.get() == ExportScope::Week) title=Signal::derive(move || t!("backup.qr_scope_week")()) subtitle=Signal::derive(move || t!("backup.qr_scope_week_hint")()) on_click=Callback::new(move |_| set_scope.set(ExportScope::Week)) />
                            <ScopeCard active=Signal::derive(move || scope.get() == ExportScope::Month) title=Signal::derive(move || t!("backup.qr_scope_month")()) subtitle=Signal::derive(move || t!("backup.qr_scope_month_hint")()) on_click=Callback::new(move |_| set_scope.set(ExportScope::Month)) />
                            <ScopeCard active=Signal::derive(move || scope.get() == ExportScope::Full) title=Signal::derive(move || t!("backup.qr_scope_full")()) subtitle=Signal::derive(move || t!("backup.qr_scope_full_hint")()) on_click=Callback::new(move |_| set_scope.set(ExportScope::Full)) />
                        </div>

                        <div class="rounded-2xl border border-slate-200 bg-gradient-to-br from-slate-50 via-white to-blue-50 px-5 py-5 shadow-sm">
                            <div class="flex flex-wrap items-start justify-between gap-4">
                                <div>
                                    <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500">{t!("backup.qr_estimate_label")}</p>
                                    <p class="mt-2 text-3xl font-semibold text-slate-900">{move || format!("{} {}", estimated_codes.get(), t!("backup.qr_codes_short")())}</p>
                                    <p class="mt-1 text-sm text-slate-600">{move || size_label.get()}</p>
                                </div>
                                <div class="min-w-[14rem] rounded-xl border border-slate-200 bg-white/90 px-4 py-3 text-sm text-slate-700 shadow-sm">
                                    <p>
                                        {move || translate_with_params(
                                            "backup.qr_purchase_window",
                                            HashMap::from([
                                                ("filtered", filtered_purchase_count.get().to_string()),
                                                ("total", total_purchase_count.get().to_string()),
                                            ]),
                                        )}
                                    </p>
                                    <p class="mt-1">
                                        {move || translate_with_params(
                                            "backup.qr_vendor_count",
                                            HashMap::from([("count", vendor_count.get().to_string())]),
                                        )}
                                    </p>
                                </div>
                            </div>

                            <Show when=move || warning_needed.get()>
                                <div class="mt-4 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
                                    <p class="font-medium">{t!("backup.qr_warning_many_title")}</p>
                                    <p class="mt-1">{t!("backup.qr_warning_many_body")}</p>
                                </div>
                            </Show>

                            <Show when=move || exceeds_limit.get()>
                                <div class="mt-4 rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900">
                                    <p class="font-medium">{t!("backup.qr_limit_title")}</p>
                                    <p class="mt-1">
                                        {move || translate_with_params(
                                            "backup.qr_limit_body",
                                            HashMap::from([
                                                ("count", estimated_codes.get().to_string()),
                                                ("max", MAX_QR_CODES.to_string()),
                                            ]),
                                        )}
                                    </p>
                                </div>
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || !is_loading.get() && stage.get() == QrExportStage::ConfirmLarge>
                    <div class="space-y-4 rounded-2xl border border-amber-200 bg-gradient-to-br from-amber-50 via-white to-orange-50 px-5 py-5 shadow-sm">
                        <p class="text-sm font-semibold uppercase tracking-[0.2em] text-amber-700">{t!("backup.qr_warning_many_title")}</p>
                        <p class="text-lg font-semibold text-gray-900">
                            {move || translate_with_params(
                                "backup.qr_warning_confirm_heading",
                                HashMap::from([("count", estimated_codes.get().to_string())]),
                            )}
                        </p>
                        <p>{t!("backup.qr_warning_confirm_body")}</p>
                        <div class="rounded-xl border border-amber-200 bg-white/80 px-4 py-3 text-sm text-amber-950">{t!("backup.qr_warning_confirm_tip")}</div>
                    </div>
                </Show>

                <Show when=move || !is_loading.get() && stage.get() == QrExportStage::Display>
                    <div class="space-y-5">
                        <div class="rounded-[1.75rem] border border-slate-200 bg-gradient-to-br from-slate-50 via-white to-slate-100 px-5 py-5 shadow-sm">
                            <div class="flex flex-wrap items-center justify-between gap-3">
                                <div>
                                    <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500">{t!("backup.qr_display_label")}</p>
                                    <p class="mt-1 text-lg font-semibold text-slate-900">{move || progress_label.get()}</p>
                                </div>
                                <Show when=move || chunks.with(|items| items.len() > 1)>
                                    <button
                                        type="button"
                                        class="inline-flex items-center gap-2 rounded-full border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:bg-slate-50"
                                        on:click=move |_| set_is_paused.update(|paused| *paused = !*paused)
                                    >
                                        <span>{move || if is_paused.get() { t!("backup.qr_resume")() } else { t!("backup.qr_pause")() }}</span>
                                    </button>
                                </Show>
                            </div>

                            <div class="mt-5 flex items-center justify-center">
                                <div class="w-full max-w-[24rem] rounded-[2rem] border border-slate-200 bg-white p-5 shadow-inner">
                                    <div
                                        class="aspect-square overflow-hidden rounded-2xl bg-white"
                                        inner_html=move || {
                                            chunks.with(|items| {
                                                items
                                                    .get(current_index.get())
                                                    .map(|chunk| chunk.svg.clone())
                                                    .unwrap_or_default()
                                            })
                                        }
                                    ></div>
                                </div>
                            </div>

                            <div class="mt-5 flex justify-center gap-2">
                                <For
                                    each=move || dot_indexes.get()
                                    key=|index| *index
                                    children=move |index| {
                                        view! {
                                            <span class=move || if current_index.get() == index {
                                                "h-2.5 w-8 rounded-full bg-blue-600 transition-all"
                                            } else {
                                                "h-2.5 w-2.5 rounded-full bg-slate-300 transition-all"
                                            }></span>
                                        }
                                    }
                                />
                            </div>

                            <Show when=move || chunks.with(|items| items.len() > 1)>
                                <div class="mt-5 space-y-2">
                                    <div class="flex items-center justify-between text-sm text-slate-600">
                                        <span>{t!("backup.qr_countdown_label")}</span>
                                        <span>{move || countdown_label.get()}</span>
                                    </div>
                                    <div class="h-2 overflow-hidden rounded-full bg-slate-200">
                                        <div class="h-full rounded-full bg-gradient-to-r from-blue-500 via-cyan-500 to-teal-500 transition-[width] duration-100" style=move || countdown_width.get()></div>
                                    </div>
                                </div>
                            </Show>
                        </div>

                        <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-700 shadow-sm">
                            <p>{t!("backup.qr_scan_instruction")}</p>
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Secondary on_click=Box::new(previous_code) disabled=chunks.with(|items| items.len() <= 1)>{t!("common.back")}</Button>
                                <Button variant=ButtonVariant::Secondary on_click=Box::new(next_code) disabled=chunks.with(|items| items.len() <= 1)>{t!("common.next")}</Button>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </Modal>
    }
}

#[component]
fn ScopeCard(
    #[prop(into)] active: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] subtitle: Signal<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=move || if active.get() {
                "rounded-2xl border border-blue-300 bg-blue-50 px-4 py-4 text-left shadow-sm ring-2 ring-blue-200 transition"
            } else {
                "rounded-2xl border border-gray-200 bg-white px-4 py-4 text-left shadow-sm transition hover:border-blue-200 hover:bg-blue-50/40"
            }
            on:click=move |_| on_click.call(())
        >
            <p class="text-sm font-semibold text-gray-900">{move || title.get()}</p>
            <p class="mt-1 text-sm text-gray-600">{move || subtitle.get()}</p>
        </button>
    }
}
