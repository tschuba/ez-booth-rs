use js_sys::{Array, Function, Object, Promise, Reflect};
use leptos::html;
use leptos::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{HtmlVideoElement, MediaStream, MediaStreamConstraints};

use crate::components::{Button, ButtonVariant, Modal, ModalSize};
use crate::t;
use ez_booth_storage::export::{
    parse_chunk_payload, BoothBackupData, CollectorStatus, ImportValidator, QrChunkCollector,
};

const SCAN_POLL_INTERVAL_MS: i32 = 350;
const DUPLICATE_FEEDBACK_COOLDOWN_MS: f64 = 1_200.0;
const TRANSIENT_SCAN_ERROR_NOTICE_THRESHOLD: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScannerStage {
    RequestingCamera,
    Scanning,
    PermissionDenied,
    Unsupported,
    Error,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoticeTone {
    Info,
    Success,
    Error,
}

fn create_qr_detector() -> Result<JsValue, String> {
    let global = js_sys::global();
    let ctor = Reflect::get(&global, &JsValue::from_str("BarcodeDetector"))
        .map_err(|_| "BarcodeDetector unavailable".to_string())?
        .dyn_into::<Function>()
        .map_err(|_| "BarcodeDetector unavailable".to_string())?;

    let options = Object::new();
    let formats = Array::new();
    formats.push(&JsValue::from_str("qr_code"));
    Reflect::set(&options, &JsValue::from_str("formats"), &formats)
        .map_err(|_| "Failed to configure BarcodeDetector".to_string())?;

    Reflect::construct(&ctor, &Array::of1(&options.into()))
        .map_err(|_| "Failed to create BarcodeDetector".to_string())
}

fn supports_barcode_detector() -> bool {
    Reflect::has(&js_sys::global(), &JsValue::from_str("BarcodeDetector")).unwrap_or(false)
}

async fn detect_qr_payload(
    detector: JsValue,
    video: HtmlVideoElement,
) -> Result<Option<String>, String> {
    let detect = Reflect::get(&detector, &JsValue::from_str("detect"))
        .map_err(|_| "Scanner API unavailable".to_string())?
        .dyn_into::<Function>()
        .map_err(|_| "Scanner API unavailable".to_string())?;

    let result = detect
        .call1(&detector, video.as_ref())
        .map_err(|_| "Unable to scan current frame".to_string())?;
    let detected = JsFuture::from(Promise::from(result))
        .await
        .map_err(|_| "Unable to scan current frame".to_string())?;
    let items = Array::from(&detected);

    for item in items.iter() {
        if let Ok(raw_value) = Reflect::get(&item, &JsValue::from_str("rawValue")) {
            if let Some(raw) = raw_value.as_string() {
                return Ok(Some(raw));
            }
        }
    }

    Ok(None)
}

fn stop_media_stream(stream: Option<MediaStream>) {
    if let Some(stream) = stream {
        let tracks = stream.get_tracks();
        for track in tracks.iter() {
            if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
    }
}

#[component]
pub fn QrImportScanner(
    #[prop(into)] show: Signal<bool>,
    on_close: impl Fn() + Clone + 'static,
    on_import_ready: impl Fn(BoothBackupData) + Clone + 'static,
    on_use_file_import: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let video_ref = create_node_ref::<html::Video>();
    let validator = ImportValidator::new();
    let (stage, set_stage) = create_signal(ScannerStage::RequestingCamera);
    let (notice, set_notice) = create_signal(None::<(NoticeTone, String)>);
    let (collector, set_collector) = create_signal(QrChunkCollector::new());
    let (media_stream, set_media_stream) = create_signal(None::<MediaStream>);
    let (detector, set_detector) = create_signal(None::<JsValue>);
    let (is_detecting, set_is_detecting) = create_signal(false);
    let (last_payload, set_last_payload) = create_signal(None::<String>);
    let (last_payload_at, set_last_payload_at) = create_signal(0.0_f64);
    let (completed_backup, set_completed_backup) = create_signal(None::<BoothBackupData>);
    let (is_mounted, set_is_mounted) = create_signal(true);
    let (detect_error_streak, set_detect_error_streak) = create_signal(0_u32);

    let stop_stream = move || {
        stop_media_stream(media_stream.get_untracked());
        set_media_stream.set(None);
    };

    let reset_scanner = move || {
        stop_stream();
        set_detector.set(None);
        set_stage.set(ScannerStage::RequestingCamera);
        set_notice.set(None);
        set_collector.set(QrChunkCollector::new());
        set_is_detecting.set(false);
        set_last_payload.set(None);
        set_last_payload_at.set(0.0);
        set_completed_backup.set(None);
        set_detect_error_streak.set(0);
    };

    let close_scanner = {
        let on_close = on_close.clone();
        move || {
            reset_scanner();
            on_close();
        }
    };
    let close_scanner_for_preview = close_scanner.clone();
    let close_scanner_for_actions = close_scanner.clone();
    let close_scanner_for_modal = close_scanner.clone();
    let use_file_import = {
        let on_use_file_import = on_use_file_import.clone();
        move || {
            reset_scanner();
            on_use_file_import();
        }
    };

    let start_scanner = move || {
        reset_scanner();

        if !supports_barcode_detector() {
            set_stage.set(ScannerStage::Unsupported);
            set_notice.set(Some((
                NoticeTone::Error,
                t!("backup.import_qr_browser_unsupported_body")(),
            )));
            return;
        }

        let Some(window) = web_sys::window() else {
            set_stage.set(ScannerStage::Error);
            set_notice.set(Some((
                NoticeTone::Error,
                t!("backup.import_qr_camera_error_body")(),
            )));
            return;
        };

        let Ok(media_devices) = window.navigator().media_devices() else {
            set_stage.set(ScannerStage::Unsupported);
            set_notice.set(Some((
                NoticeTone::Error,
                t!("backup.import_qr_browser_unsupported_body")(),
            )));
            return;
        };

        let video_ref = video_ref.clone();

        spawn_local(async move {
            let constraints = MediaStreamConstraints::new();
            let video_constraints = Object::new();
            let _ = Reflect::set(
                &video_constraints,
                &JsValue::from_str("facingMode"),
                &JsValue::from_str("environment"),
            );
            constraints.set_video(&video_constraints.into());
            constraints.set_audio(&JsValue::FALSE);

            let stream = match media_devices.get_user_media_with_constraints(&constraints) {
                Ok(promise) => match JsFuture::from(promise).await {
                    Ok(stream) => match stream.dyn_into::<MediaStream>() {
                        Ok(stream) => stream,
                        Err(_) => {
                            set_stage.set(ScannerStage::Error);
                            set_notice.set(Some((
                                NoticeTone::Error,
                                t!("backup.import_qr_camera_error_body")(),
                            )));
                            return;
                        }
                    },
                    Err(err) => {
                        let denied = err
                            .dyn_ref::<js_sys::Object>()
                            .and_then(|value| Reflect::get(value, &JsValue::from_str("name")).ok())
                            .and_then(|value| value.as_string())
                            .map(|name| name == "NotAllowedError" || name == "SecurityError")
                            .unwrap_or(false);

                        set_stage.set(if denied {
                            ScannerStage::PermissionDenied
                        } else {
                            ScannerStage::Error
                        });
                        set_notice.set(Some((
                            NoticeTone::Error,
                            if denied {
                                t!("backup.import_qr_permission_help")()
                            } else {
                                t!("backup.import_qr_camera_error_body")()
                            },
                        )));
                        return;
                    }
                },
                Err(_) => {
                    set_stage.set(ScannerStage::Error);
                    set_notice.set(Some((
                        NoticeTone::Error,
                        t!("backup.import_qr_camera_error_body")(),
                    )));
                    return;
                }
            };

            if !is_mounted.get_untracked() {
                stop_media_stream(Some(stream));
                return;
            }

            set_media_stream.set(Some(stream.clone()));

            let Some(video) = video_ref.get() else {
                stop_stream();
                return;
            };

            video.set_muted(true);
            video.set_autoplay(true);
            let _ = video.set_attribute("playsinline", "true");
            video.set_src_object(Some(&stream));

            match video.play() {
                Ok(promise) => {
                    let _ = JsFuture::from(promise).await;
                }
                Err(_) => {
                    stop_stream();
                    set_stage.set(ScannerStage::Error);
                    set_notice.set(Some((
                        NoticeTone::Error,
                        t!("backup.import_qr_camera_error_body")(),
                    )));
                    return;
                }
            }

            match create_qr_detector() {
                Ok(detector) => {
                    set_media_stream.set(Some(stream));
                    set_detector.set(Some(detector));
                    set_stage.set(ScannerStage::Scanning);
                    set_notice.set(Some((NoticeTone::Info, t!("backup.import_qr_ready")())));
                }
                Err(message) => {
                    stop_stream();
                    set_stage.set(ScannerStage::Unsupported);
                    set_notice.set(Some((NoticeTone::Error, message)));
                }
            }
        });
    };

    create_effect(move |_| {
        if show.get() {
            set_is_mounted.set(true);
            start_scanner();
        } else {
            set_is_mounted.set(false);
            reset_scanner();
        }
    });

    on_cleanup(move || {
        set_is_mounted.set(false);
        reset_scanner();
    });

    create_effect(move |_| {
        let should_scan = show.get() && stage.get() == ScannerStage::Scanning;
        if !should_scan {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };

        let closure = Closure::wrap(Box::new(move || {
            if is_detecting.get_untracked() {
                return;
            }

            let Some(detector) = detector.get_untracked() else {
                return;
            };
            let Some(video) = video_ref.get() else {
                return;
            };
            let video: HtmlVideoElement = (*video).clone();

            if video.ready_state() < 2 {
                return;
            }

            set_is_detecting.set(true);
            spawn_local(async move {
                let result = detect_qr_payload(detector, video).await;
                set_is_detecting.set(false);

                match result {
                    Ok(Some(raw)) => {
                        set_detect_error_streak.set(0);
                        let now = js_sys::Date::now();
                        if last_payload.get_untracked().as_deref() == Some(raw.as_str())
                            && now - last_payload_at.get_untracked() < DUPLICATE_FEEDBACK_COOLDOWN_MS
                        {
                            return;
                        }

                        set_last_payload.set(Some(raw.clone()));
                        set_last_payload_at.set(now);

                        let chunk = match parse_chunk_payload(&raw) {
                            Ok(chunk) => chunk,
                            Err(_) => {
                                set_notice.set(Some((
                                    NoticeTone::Error,
                                    t!("backup.import_qr_invalid")(),
                                )));
                                return;
                            }
                        };

                        let mut next_collector = collector.get_untracked();
                        match next_collector.add_chunk(chunk.clone()) {
                            Ok(status) => {
                                let scanned_index = chunk.i + 1;
                                set_collector.set(next_collector.clone());
                                match status {
                                    CollectorStatus::ChunkAdded => {
                                        set_notice.set(Some((
                                            NoticeTone::Success,
                                            t!("backup.import_qr_chunk_added")()
                                                .replace("{index}", &scanned_index.to_string()),
                                        )));
                                    }
                                    CollectorStatus::Duplicate => {
                                        set_notice.set(Some((
                                            NoticeTone::Info,
                                            t!("backup.import_qr_duplicate")()
                                                .replace("{index}", &scanned_index.to_string()),
                                        )));
                                    }
                                    CollectorStatus::Complete => match next_collector.reassemble_backup() {
                                        Ok(backup) => match validator.validate_booth_backup_data(backup) {
                                            Ok(backup) => {
                                                stop_stream();
                                                set_completed_backup.set(Some(backup));
                                                set_stage.set(ScannerStage::Complete);
                                                set_notice.set(Some((
                                                    NoticeTone::Success,
                                                    t!("backup.import_qr_complete")(),
                                                )));
                                            }
                                            Err(err) => {
                                                stop_stream();
                                                set_stage.set(ScannerStage::Error);
                                                set_notice.set(Some((NoticeTone::Error, err.to_string())));
                                            }
                                        },
                                        Err(err) => {
                                            stop_stream();
                                            set_stage.set(ScannerStage::Error);
                                            set_notice.set(Some((NoticeTone::Error, err.to_string())));
                                        }
                                    },
                                }
                            }
                            Err(err) => {
                                stop_stream();
                                set_stage.set(ScannerStage::Error);
                                set_notice.set(Some((NoticeTone::Error, err.to_string())));
                            }
                        }
                    }
                    Ok(None) => {
                        set_detect_error_streak.set(0);
                    }
                    Err(message) => {
                        let streak = detect_error_streak.get_untracked() + 1;
                        set_detect_error_streak.set(streak);
                        if streak >= TRANSIENT_SCAN_ERROR_NOTICE_THRESHOLD {
                            set_notice.set(Some((NoticeTone::Error, message)));
                        }
                    }
                }
            });
        }) as Box<dyn FnMut()>);

        let handle = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                SCAN_POLL_INTERVAL_MS,
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

    let progress = Signal::derive(move || collector.get().progress());
    let progress_label = Signal::derive(move || {
        let (received, total) = progress.get();
        match total {
            Some(total) => t!("backup.import_qr_progress_known")()
                .replace("{received}", &received.to_string())
                .replace("{total}", &total.to_string()),
            None => t!("backup.import_qr_progress_unknown")()
                .replace("{received}", &received.to_string()),
        }
    });
    let dot_count = Signal::derive(move || {
        let (received, total) = progress.get();
        total.unwrap_or_else(|| (received + 2).clamp(3, 6))
    });
    let dot_indexes = Signal::derive(move || (0..dot_count.get()).collect::<Vec<_>>());

    let preview_import = {
        let on_import_ready = on_import_ready.clone();
        move || {
            if let Some(backup) = completed_backup.get_untracked() {
                on_import_ready(backup);
                close_scanner_for_preview();
            }
        }
    };

    let close_scanner_for_actions = store_value(close_scanner_for_actions);
    let preview_import = store_value(preview_import);
    let use_file_import = store_value(use_file_import);
    let start_scanner = store_value(start_scanner);

    let action_bar = Callback::new(move |_| view! {
        {move || match stage.get() {
            ScannerStage::Complete => {
                view! {
                    <div class="contents">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move || close_scanner_for_actions.with_value(|close_scanner| close_scanner()))>
                            {t!("common.close")}
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            class="js-qr-preview-import".to_string()
                            on_click=Box::new(move || preview_import.with_value(|preview_import| preview_import()))
                        >
                            {t!("backup.import_qr_preview")}
                        </Button>
                    </div>
                }
                    .into_view()
            }
            ScannerStage::PermissionDenied | ScannerStage::Error => {
                view! {
                    <div class="contents">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move || close_scanner_for_actions.with_value(|close_scanner| close_scanner()))>
                            {t!("common.close")}
                        </Button>
                        <Button
                            variant=ButtonVariant::Ghost
                            class="js-import-json-fallback".to_string()
                            on_click=Box::new(move || use_file_import.with_value(|use_file_import| use_file_import()))
                        >
                            {t!("backup.import_qr_use_json")}
                        </Button>
                        <Button variant=ButtonVariant::Primary on_click=Box::new(move || start_scanner.with_value(|start_scanner| start_scanner()))>
                            {t!("backup.import_qr_try_again")}
                        </Button>
                    </div>
                }
                    .into_view()
            }
            ScannerStage::Unsupported => {
                view! {
                    <div class="contents">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move || close_scanner_for_actions.with_value(|close_scanner| close_scanner()))>
                            {t!("common.close")}
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            class="js-import-json-fallback".to_string()
                            on_click=Box::new(move || use_file_import.with_value(|use_file_import| use_file_import()))
                        >
                            {t!("backup.import_qr_use_json")}
                        </Button>
                    </div>
                }
                    .into_view()
            }
            ScannerStage::RequestingCamera | ScannerStage::Scanning => {
                view! {
                    <div class="contents">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move || close_scanner_for_actions.with_value(|close_scanner| close_scanner()))>
                            {t!("backup.import_qr_cancel")}
                        </Button>
                    </div>
                }
                    .into_view()
            }
        }}
    }
    .into_view());

    view! {
        <Modal
            show=show
            on_close=close_scanner_for_modal.clone()
            title=Signal::derive(move || t!("backup.import_qr_title")())
            size=ModalSize::XLarge
            action_bar=action_bar
        >
            <div class="space-y-5 text-gray-700">
                <div class=move || {
                    if matches!(stage.get(), ScannerStage::RequestingCamera | ScannerStage::Scanning) {
                        "space-y-4"
                    } else {
                        "hidden"
                    }
                }>
                    <div class="overflow-hidden rounded-[1.75rem] border border-slate-200 bg-slate-950 shadow-xl">
                        <div class="relative aspect-[4/3] w-full overflow-hidden">
                            <video
                                node_ref=video_ref
                                class="h-full w-full object-cover"
                                autoplay=true
                                playsinline=true
                                muted=true
                            ></video>
                            <div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0,transparent_9rem,rgba(2,6,23,0.58)_9.15rem)]"></div>
                            <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
                                <div class="h-48 w-48 rounded-[2rem] border-2 border-dashed border-white/85 shadow-[0_0_0_9999px_rgba(15,23,42,0.14)]"></div>
                            </div>
                            <Show when=move || stage.get() == ScannerStage::RequestingCamera>
                                <div class="absolute inset-0 flex flex-col items-center justify-center bg-slate-950/70 text-white backdrop-blur-[1px]">
                                    <div class="h-10 w-10 animate-spin rounded-full border-4 border-white/20 border-t-white"></div>
                                    <p class="mt-4 text-sm font-medium">{t!("backup.import_qr_camera_request")}</p>
                                </div>
                            </Show>
                        </div>
                    </div>

                    <div class="rounded-2xl border border-slate-200 bg-gradient-to-br from-slate-50 via-white to-sky-50 px-5 py-4 shadow-sm">
                        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500">{t!("backup.import_qr_open")}</p>
                        <p class="mt-2 text-lg font-semibold text-slate-900">{move || progress_label.get()}</p>
                        <p class="mt-1 text-sm text-slate-600">
                            {move || if stage.get() == ScannerStage::RequestingCamera {
                                t!("backup.import_qr_camera_request")()
                            } else if progress.get().0 == 0 {
                                t!("backup.import_qr_ready")()
                            } else {
                                t!("backup.import_qr_scan_next")()
                            }}
                        </p>
                        <div class="mt-4 flex gap-2">
                            <For
                                each=move || dot_indexes.get()
                                key=|index| *index
                                children=move |index| {
                                    view! {
                                        <span class=move || {
                                            let (received, total) = progress.get();
                                            let active = index < received;
                                            let is_last_placeholder = total.is_none() && index == dot_count.get() - 1;
                                            if active {
                                                "h-2.5 w-8 rounded-full bg-emerald-500 transition-all"
                                            } else if is_last_placeholder {
                                                "h-2.5 w-8 rounded-full border border-dashed border-slate-300 bg-transparent transition-all"
                                            } else {
                                                "h-2.5 w-2.5 rounded-full bg-slate-300 transition-all"
                                            }
                                        }></span>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>

                <Show when=move || stage.get() == ScannerStage::Complete>
                    <div class="space-y-4">
                        <div class="rounded-2xl border border-emerald-200 bg-gradient-to-br from-emerald-50 via-white to-teal-50 px-5 py-5 shadow-sm">
                            <p class="text-sm font-semibold uppercase tracking-[0.2em] text-emerald-700">{t!("backup.import_qr_complete")}</p>
                            <Show when=move || completed_backup.get().is_some()>
                                {move || completed_backup.get().map(|backup| view! {
                                    <>
                                        <p class="mt-2 text-2xl font-semibold text-slate-900">{backup.booth.description.clone()}</p>
                                        <p class="mt-2 text-sm text-slate-700">
                                            {t!("backup.import_booth_counts")()
                                                .replace("{vendors}", &backup.vendors.len().to_string())
                                                .replace("{purchases}", &backup.purchases.len().to_string())}
                                        </p>
                                        <p class="mt-3 text-sm text-slate-600">{t!("backup.import_qr_complete_hint")}</p>
                                    </>
                                })}
                            </Show>
                        </div>
                    </div>
                </Show>

                <Show when=move || matches!(stage.get(), ScannerStage::PermissionDenied | ScannerStage::Unsupported | ScannerStage::Error)>
                    <div class="rounded-2xl border border-red-200 bg-red-50 px-5 py-5 shadow-sm">
                        <p class="text-sm font-semibold uppercase tracking-[0.2em] text-red-700">
                            {move || match stage.get() {
                                ScannerStage::PermissionDenied => t!("backup.import_qr_permission_denied")(),
                                ScannerStage::Unsupported => t!("backup.import_qr_browser_unsupported")(),
                                _ => t!("backup.import_qr_camera_error")(),
                            }}
                        </p>
                        <p class="mt-3 text-sm text-red-900">
                            {move || match stage.get() {
                                ScannerStage::PermissionDenied => t!("backup.import_qr_permission_help")(),
                                ScannerStage::Unsupported => t!("backup.import_qr_browser_unsupported_body")(),
                                _ => t!("backup.import_qr_camera_error_body")(),
                            }}
                        </p>
                    </div>
                </Show>

                <Show when=move || notice.get().is_some()>
                    {move || notice.get().map(|(tone, message)| {
                        let class_name = match tone {
                            NoticeTone::Info => "rounded-xl border border-sky-200 bg-sky-50 px-4 py-3 text-sm text-sky-900",
                            NoticeTone::Success => "rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900",
                            NoticeTone::Error => "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900",
                        };
                        view! { <div class=class_name>{message}</div> }
                    })}
                </Show>
            </div>
        </Modal>
    }
}
