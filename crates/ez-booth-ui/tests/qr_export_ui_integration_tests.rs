wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use domain::BoothId;
use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

async fn sleep_ms(timeout_ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window available");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            resolve.unchecked_ref(),
            timeout_ms,
        );
    });

    JsFuture::from(promise).await.expect("timeout should resolve");
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

#[wasm_bindgen_test]
async fn booth_dropdown_qr_export_opens_and_closes_modal() {
    let document = document();
    let root = document
        .create_element("div")
        .expect("root creation should succeed")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("root should be an html element");
    root.set_class_name("js-qr-export-test-root");
    document
        .body()
        .expect("body available")
        .append_child(&root)
        .expect("should append root");

    let booth_id = BoothId::new();

    leptos::mount_to(root.clone(), move || {
        ez_booth_ui::provide_i18n();
        let app_state: Resource<(), Result<ez_booth_ui::AppState, String>> =
            create_local_resource(|| (), |_| async { Err("test app state".to_string()) });
        provide_context(app_state);

        view! {
            <ez_booth_ui::ToastProvider>
                <ez_booth_ui::DropdownMenu
                    trigger=view! {
                        <ez_booth_ui::Button
                            variant=ez_booth_ui::ButtonVariant::Secondary
                            class="js-open-actions".to_string()
                            title="Actions".to_string()
                            aria_label="Actions".to_string()
                        >
                            "Actions"
                        </ez_booth_ui::Button>
                    }
                >
                    <ez_booth_ui::ExportButton
                        scope=ez_booth_ui::ExportScope::Booth(booth_id)
                        menu_item=true
                    />
                </ez_booth_ui::DropdownMenu>
            </ez_booth_ui::ToastProvider>
        }
    });

    let action_button = wait_for_selector(".js-open-actions").await;
    click(&action_button);

    let qr_export_item = wait_for_selector(".js-qr-export-menu-item").await;
    click(&qr_export_item);

    let modal = wait_for_selector("[role=\"dialog\"]").await;
    let title = wait_for_selector("#modal-title").await;
    assert!(title.text_content().unwrap_or_default().len() > 0);
    assert!(modal.class_name().contains("fixed"));

    let close_button = wait_for_selector("button[aria-label=\"Close modal\"]").await;
    click(&close_button);
    wait_for_selector_to_disappear("[role=\"dialog\"]").await;

    root.remove();
}
