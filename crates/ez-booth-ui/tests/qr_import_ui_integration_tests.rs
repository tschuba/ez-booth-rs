wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use leptos::*;
use chrono::NaiveDate;
use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
use ez_booth_storage::export::{create_chunks, serialize_and_compress_backup, serialize_chunk_payload, BoothBackupData};
use rust_decimal_macros::dec;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;

async fn sleep_ms(timeout_ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window available");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            resolve.unchecked_ref(),
            timeout_ms,
        );
    });

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout should resolve");
}

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("window available")
        .document()
        .expect("document available")
}

fn click(element: &web_sys::Element) {
    element
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("element should be clickable")
        .click();
}

async fn wait_for_selector(selector: &str) -> web_sys::Element {
    for _ in 0..80 {
        if let Some(element) = document()
            .query_selector(selector)
            .expect("selector query should succeed")
        {
            return element;
        }

        sleep_ms(50).await;
    }

    panic!("timed out waiting for selector: {selector}");
}

async fn wait_for_selector_to_disappear(selector: &str) {
    for _ in 0..80 {
        if document()
            .query_selector(selector)
            .expect("selector query should succeed")
            .is_none()
        {
            return;
        }

        sleep_ms(50).await;
    }

    panic!("selector still present after timeout: {selector}");
}

fn mount_import_button(root: web_sys::HtmlElement) {
    leptos::mount_to(root, move || {
        ez_booth_ui::provide_i18n();
        let app_state: Resource<(), Result<ez_booth_ui::AppState, String>> =
            create_local_resource(|| (), |_| async { Err("test app state".to_string()) });
        provide_context(app_state);

        view! {
            <ez_booth_ui::ToastProvider>
                <ez_booth_ui::ImportButton
                    variant=ez_booth_ui::ButtonVariant::Secondary
                    class="js-open-import".to_string()
                />
            </ez_booth_ui::ToastProvider>
        }
    });
}

fn file_input_click_counter(document: &web_sys::Document) -> (Rc<Cell<u32>>, Closure<dyn FnMut(web_sys::Event)>) {
    let file_input_clicks = Rc::new(Cell::new(0_u32));
    let file_input_clicks_for_listener = Rc::clone(&file_input_clicks);
    let click_listener = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        if input.type_() == "file" {
            file_input_clicks_for_listener.set(file_input_clicks_for_listener.get() + 1);
        }
    }) as Box<dyn FnMut(_)>);
    document
        .add_event_listener_with_callback("click", click_listener.as_ref().unchecked_ref())
        .expect("should register click listener");

    (file_input_clicks, click_listener)
}

fn test_root(document: &web_sys::Document, class_name: &str) -> web_sys::HtmlElement {
    let root = document
        .create_element("div")
        .expect("root creation should succeed")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("root should be an html element");
    root.set_class_name(class_name);
    document
        .body()
        .expect("body available")
        .append_child(&root)
        .expect("should append root");
    root
}

fn sample_booth_backup() -> BoothBackupData {
    let booth = Booth::new(
        "Spring Market 2026".to_string(),
        NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
        FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(15.00),
            rounding_step: dec!(0.50),
        },
    )
    .unwrap();
    let vendor = Vendor::new(VendorId::from("101"), booth.id).with_name("Alex Seller".to_string());
    let purchase = Purchase::new(
        booth.id,
        vec![PurchaseItem::new(dec!(12.50), vendor.vendor_id.clone()).unwrap()],
    )
    .unwrap();

    let mut backup = BoothBackupData::new(booth, "test-version");
    backup.vendors = vec![vendor];
    backup.purchases = vec![purchase];
    backup
}

fn fake_barcode_detector_factory(payloads: &[String]) -> js_sys::Function {
    let payload_array = js_sys::Array::new();
    for payload in payloads {
        payload_array.push(&JsValue::from_str(payload));
    }

    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__ezBoothTestQrPayloads"),
        payload_array.as_ref(),
    )
    .expect("should store fake QR payloads");
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__ezBoothBarcodeDetectorDetectCount"),
        &JsValue::from_f64(0.0),
    )
    .expect("should reset detect count");

    js_sys::eval(
        "class FakeBarcodeDetector {
            constructor() {
                this.index = 0;
            }

            detect() {
                const payloads = globalThis.__ezBoothTestQrPayloads || [];
                const rawValue = payloads[this.index] ?? null;
                globalThis.__ezBoothBarcodeDetectorDetectCount =
                    (globalThis.__ezBoothBarcodeDetectorDetectCount || 0) + 1;
                if (this.index < payloads.length) {
                    this.index += 1;
                }
                if (!rawValue) {
                    return Promise.resolve([]);
                }
                return Promise.resolve([{ rawValue }]);
            }
        }

        FakeBarcodeDetector;",
    )
    .expect("should build fake detector")
    .dyn_into::<js_sys::Function>()
    .expect("fake detector should be a function")
}

fn object_define_property(
    target: &JsValue,
    key: &str,
    descriptor: &js_sys::Object,
) -> Result<(), JsValue> {
    let object_ctor = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("Object"))?
        .dyn_into::<js_sys::Object>()?;
    let define_property = js_sys::Reflect::get(&object_ctor, &JsValue::from_str("defineProperty"))?
        .dyn_into::<js_sys::Function>()?;
    define_property.call3(
        &object_ctor,
        target,
        &JsValue::from_str(key),
        descriptor.as_ref(),
    )?;
    Ok(())
}

fn object_get_own_property_descriptor(target: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    let object_ctor = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("Object"))?
        .dyn_into::<js_sys::Object>()?;
    let get_own_property_descriptor =
        js_sys::Reflect::get(&object_ctor, &JsValue::from_str("getOwnPropertyDescriptor"))?
            .dyn_into::<js_sys::Function>()?;
    get_own_property_descriptor.call2(&object_ctor, target, &JsValue::from_str(key))
}

fn object_delete_property(target: &JsValue, key: &str) -> Result<(), JsValue> {
    js_sys::Reflect::delete_property(
        &target.clone().unchecked_into::<js_sys::Object>(),
        &JsValue::from_str(key),
    )?;
    Ok(())
}

#[wasm_bindgen_test]
async fn qr_import_unsupported_browser_offers_json_fallback() {
    let global = js_sys::global();
    let barcode_detector_key = JsValue::from_str("BarcodeDetector");
    let original_barcode_detector = js_sys::Reflect::get(&global, &barcode_detector_key)
        .expect("should read BarcodeDetector value");
    js_sys::Reflect::set(&global, &barcode_detector_key, &JsValue::UNDEFINED)
        .expect("should override BarcodeDetector");

    let result = async {
        let document = document();
        let root = test_root(&document, "js-qr-import-test-root");
        let (file_input_clicks, click_listener) = file_input_click_counter(&document);

        mount_import_button(root.clone());

        let import_button = wait_for_selector(".js-open-import").await;
        click(&import_button);

        let qr_import_item = wait_for_selector(".js-qr-import-menu-item").await;
        click(&qr_import_item);

        let modal = wait_for_selector("[role=\"dialog\"]").await;
        assert!(modal.class_name().contains("fixed"));

        let fallback_button = wait_for_selector(".js-import-json-fallback").await;
        click(&fallback_button);

        wait_for_selector_to_disappear(".js-import-json-fallback").await;
        assert_eq!(file_input_clicks.get(), 1);

        let _ = document.remove_event_listener_with_callback(
            "click",
            click_listener.as_ref().unchecked_ref(),
        );
        drop(click_listener);
        root.remove();
    }
    .await;

    js_sys::Reflect::set(&global, &barcode_detector_key, &original_barcode_detector)
        .expect("should restore BarcodeDetector");

    result
}

#[wasm_bindgen_test]
async fn qr_import_permission_denied_offers_json_fallback() {
    let global = js_sys::global();
    let barcode_detector_key = JsValue::from_str("BarcodeDetector");
    let original_barcode_detector = js_sys::Reflect::get(&global, &barcode_detector_key)
        .expect("should read BarcodeDetector value");
    let fake_detector = js_sys::Function::new_no_args("return []; ");
    js_sys::Reflect::set(&global, &barcode_detector_key, fake_detector.as_ref())
        .expect("should override BarcodeDetector");

    let window = web_sys::window().expect("window available");
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().expect("media devices available");
    let get_user_media_key = JsValue::from_str("getUserMedia");
    let original_get_user_media = js_sys::Reflect::get(media_devices.as_ref(), &get_user_media_key)
        .expect("should read getUserMedia value");
    let denied_get_user_media = js_sys::Function::new_no_args(
        "return Promise.reject({ name: 'NotAllowedError' });",
    );
    js_sys::Reflect::set(
        media_devices.as_ref(),
        &get_user_media_key,
        denied_get_user_media.as_ref(),
    )
    .expect("should override getUserMedia");

    let result = async {
        let document = document();
        let root = test_root(&document, "js-qr-import-permission-test-root");
        let (file_input_clicks, click_listener) = file_input_click_counter(&document);

        mount_import_button(root.clone());

        let import_button = wait_for_selector(".js-open-import").await;
        click(&import_button);

        let qr_import_item = wait_for_selector(".js-qr-import-menu-item").await;
        click(&qr_import_item);

        let fallback_button = wait_for_selector(".js-import-json-fallback").await;
        click(&fallback_button);

        wait_for_selector_to_disappear("[role=\"dialog\"]").await;
        assert_eq!(file_input_clicks.get(), 1);

        let _ = document.remove_event_listener_with_callback(
            "click",
            click_listener.as_ref().unchecked_ref(),
        );
        drop(click_listener);
        root.remove();
    }
    .await;

    js_sys::Reflect::set(media_devices.as_ref(), &get_user_media_key, &original_get_user_media)
        .expect("should restore getUserMedia");
    js_sys::Reflect::set(&global, &barcode_detector_key, &original_barcode_detector)
        .expect("should restore BarcodeDetector");

    result
}

#[wasm_bindgen_test]
async fn qr_import_scan_reaches_preview_modal() {
    let backup = sample_booth_backup();
    let compressed = serialize_and_compress_backup(&backup).expect("backup should compress");
    let chunks = create_chunks(&compressed).expect("backup should fit into QR chunks");
    let payloads = chunks
        .iter()
        .map(|chunk| serialize_chunk_payload(chunk).expect("chunk should serialize"))
        .collect::<Vec<_>>();

    let global = js_sys::global();
    let barcode_detector_key = JsValue::from_str("BarcodeDetector");
    let original_barcode_detector = js_sys::Reflect::get(&global, &barcode_detector_key)
        .expect("should read BarcodeDetector value");
    let fake_detector = fake_barcode_detector_factory(&payloads);
    js_sys::Reflect::set(&global, &barcode_detector_key, fake_detector.as_ref())
        .expect("should override BarcodeDetector");

    let window = web_sys::window().expect("window available");
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().expect("media devices available");
    let get_user_media_key = JsValue::from_str("getUserMedia");
    let original_get_user_media = js_sys::Reflect::get(media_devices.as_ref(), &get_user_media_key)
        .expect("should read getUserMedia value");
    let granted_get_user_media = js_sys::Function::new_no_args(
        "return new Promise((resolve) => setTimeout(() => resolve(new MediaStream()), 25));",
    );
    js_sys::Reflect::set(
        media_devices.as_ref(),
        &get_user_media_key,
        granted_get_user_media.as_ref(),
    )
    .expect("should override getUserMedia");

    let html_media_ctor = js_sys::Reflect::get(&global, &JsValue::from_str("HTMLMediaElement"))
        .expect("HTMLMediaElement should exist");
    let media_prototype = js_sys::Reflect::get(&html_media_ctor, &JsValue::from_str("prototype"))
        .expect("media prototype should exist");
    let original_ready_state_descriptor =
        object_get_own_property_descriptor(&media_prototype, "readyState")
            .expect("should read readyState descriptor");
    let ready_state_override = js_sys::Object::new();
    js_sys::Reflect::set(
        &ready_state_override,
        &JsValue::from_str("configurable"),
        &JsValue::TRUE,
    )
    .expect("configurable should set");
    js_sys::Reflect::set(
        &ready_state_override,
        &JsValue::from_str("get"),
        js_sys::Function::new_no_args("return 4;").as_ref(),
    )
    .expect("getter should set");
    object_define_property(&media_prototype, "readyState", &ready_state_override)
        .expect("should override readyState");

    let original_play = js_sys::Reflect::get(&media_prototype, &JsValue::from_str("play"))
        .expect("should read play value");
    let play_override = js_sys::Function::new_no_args("return Promise.resolve();");
    js_sys::Reflect::set(&media_prototype, &JsValue::from_str("play"), play_override.as_ref())
        .expect("should override play");

    let result = async {
        let document = document();
        let root = test_root(&document, "js-qr-import-success-test-root");

        mount_import_button(root.clone());

        let import_button = wait_for_selector(".js-open-import").await;
        click(&import_button);

        let qr_import_item = wait_for_selector(".js-qr-import-menu-item").await;
        click(&qr_import_item);

        for _ in 0..40 {
            let detect_count = js_sys::Reflect::get(
                &js_sys::global(),
                &JsValue::from_str("__ezBoothBarcodeDetectorDetectCount"),
            )
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
            if detect_count >= payloads.len() as f64 {
                break;
            }
            sleep_ms(50).await;
        }

        let preview_button = wait_for_selector(".js-qr-preview-import").await;
        click(&preview_button);

        let title = wait_for_selector("#modal-title").await;
        assert!(!title.text_content().unwrap_or_default().is_empty());

        let selected_source = wait_for_selector("[role=\"dialog\"] .rounded-lg.border.border-gray-200").await;
        assert!(selected_source.text_content().unwrap_or_default().contains("QR"));

        let strategy_select = wait_for_selector("select").await;
        let strategy_value = js_sys::Reflect::get(strategy_select.as_ref(), &JsValue::from_str("value"))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        assert_eq!(strategy_value, "merge");

        let strategy_text = strategy_select.text_content().unwrap_or_default();
        assert!(!strategy_text.is_empty());

        let preview_text = document.body().unwrap().text_content().unwrap_or_default();
        assert!(preview_text.contains("Spring Market 2026"));
        assert!(preview_text.contains("1"));
        assert!(preview_text.contains("Merge") || preview_text.contains("zusammen"));
        assert!(preview_text.contains("Apply") || preview_text.contains("anwenden"));

        root.remove();
    }
    .await;

    js_sys::Reflect::set(&media_prototype, &JsValue::from_str("play"), &original_play)
        .expect("should restore play");
    if original_ready_state_descriptor.is_undefined() {
        object_delete_property(&media_prototype, "readyState")
            .expect("should delete readyState override");
    } else {
        object_define_property(
            &media_prototype,
            "readyState",
            &original_ready_state_descriptor.unchecked_into::<js_sys::Object>(),
        )
        .expect("should restore readyState descriptor");
    }
    js_sys::Reflect::set(media_devices.as_ref(), &get_user_media_key, &original_get_user_media)
        .expect("should restore getUserMedia");
    js_sys::Reflect::set(&global, &barcode_detector_key, &original_barcode_detector)
        .expect("should restore BarcodeDetector");
    let global_object = global.clone().unchecked_into::<js_sys::Object>();
    let _ = js_sys::Reflect::delete_property(
        &global_object,
        &JsValue::from_str("__ezBoothTestQrPayloads"),
    );
    let _ = js_sys::Reflect::delete_property(
        &global_object,
        &JsValue::from_str("__ezBoothBarcodeDetectorDetectCount"),
    );

    result
}
