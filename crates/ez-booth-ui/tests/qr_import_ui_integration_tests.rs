wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use leptos::*;
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
