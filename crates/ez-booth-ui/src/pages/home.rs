use crate::components::*;
use crate::state::use_app_state;
use crate::t;
use leptos::*;
use leptos_router::use_navigate;

fn format_error_message(message: &str) -> String {
    const MAX_LEN: usize = 140;
    let mut formatted = message.replace(['\n', '\r'], " ");

    if formatted.len() > MAX_LEN {
        formatted.truncate(MAX_LEN - 3);
        formatted.push_str("...");
    }

    formatted
}

#[component]
pub fn HomePage() -> impl IntoView {
    // Modal state
    let (show_modal, set_show_modal) = create_signal(false);
    let (show_confirm, set_show_confirm) = create_signal(false);

    // Static message for demo confirm modal
    let confirm_message = Signal::derive(move || t!("home.are_you_sure")());

    // Get toast context
    let toast = use_toast();

    // Smart redirect based on booth status
    let (is_redirecting, set_is_redirecting) = create_signal(true);

    {
        let app_state = use_app_state();
        create_effect(move |_| {
            // Wait for app state to be ready
            let Some(Ok(state)) = app_state.get() else {
                return;
            };
            
            let booth_repository = state.booth_repository.clone();
            let navigate = use_navigate();
            
            spawn_local(async move {
                // Load all booths and check if any are open
                match booth_repository.find_all().await {
                    Ok(booths) => {
                        let has_open_booth = booths.iter().any(|b| b.is_open());
                        
                        if has_open_booth {
                            // At least one open booth exists - go to checkout
                            navigate("/checkout", Default::default());
                        } else {
                            // No open booths - go to booth list
                            navigate("/booths", Default::default());
                        }
                    }
                    Err(_) => {
                        // Error loading booths - show home page
                        set_is_redirecting.set(false);
                    }
                }
            });
        });
    }

    view! {
                                        <Container>
                                            <Show
                                                when=move || !is_redirecting.get()
                                                fallback=move || view! {
                                                    <div class="py-12 text-center">
                                                        <p class="text-gray-600">{t!("common.loading")}</p>
                                                    </div>
                                                }
                                            >
                                            <div class="py-12">
                                                <Card title="EZ Booth">
                                                    <p class="text-gray-600 mb-4">
                                                        {t!("app.welcome")}
                                                    </p>
                                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
                                                        <a href="/booths" class="block">
                                                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                                                <h3 class="text-lg font-semibold mb-2">{t!("booth.list_title")}</h3>
                                                                <p class="text-gray-600 text-sm">{t!("home.manage_booths")}</p>
                                                            </div>
                                                        </a>
                                                        <a href="/checkout" class="block">
                                                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                                                <h3 class="text-lg font-semibold mb-2">{t!("checkout.title")}</h3>
                                                                <p class="text-gray-600 text-sm">{t!("home.process_sales")}</p>
                                                            </div>
                                                        </a>
                                                    </div>
                                                </Card>

                                                // Component Demo Section
                                                <div class="mt-8">
                                                    <Card title_view={t!("home.component_demo").into_view()}>
                                                        <div class="space-y-4">
                                                            // Toast Demo
                                                            <div>
                                                                <h3 class="text-md font-semibold mb-2">{t!("home.toast_notifications")}</h3>
                                                                <div class="flex gap-2 flex-wrap">
                                <Button on_click=Box::new(move || toast.success(t!("home.toast_success_msg")()))>
                                                                    {t!("home.show_success")}
                                                                </Button>
                            <Button on_click=Box::new(move || {
                                                                    let full_message = t!("home.toast_error_short_full")();
                                                                    let short_message = format_error_message(&full_message);
                                                                    toast.error_with_full(&short_message, &full_message)
                                                                })>
                                                                    {t!("home.show_error_short")}
                                                                </Button>
                        <Button on_click=Box::new(move || {
                                                                let full_message = t!("home.toast_error_long_full")();
                                                                let short_message = format_error_message(&full_message);
                                                                toast.error_with_full(&short_message, &full_message)
                                                            })>
                                                                {t!("home.show_error_long")}
                                                            </Button>
                    <Button on_click=Box::new(move || toast.warning(t!("home.toast_warning_msg")()))>
                                                        {t!("home.show_warning")}
                                                    </Button>
                <Button on_click=Box::new(move || toast.info(t!("home.toast_info_msg")()))>
                                                    {t!("home.show_info")}
                                                </Button>
                                                                </div>
                                                            </div>

                                                            // Modal Demo
                                                            <div>
                                                                <h3 class="text-md font-semibold mb-2">{t!("home.modals")}</h3>
                                                                <div class="flex gap-2">
            <Button on_click=Box::new(move || set_show_modal.set(true))>
                                                    {t!("home.open_modal")}
                                                </Button>
        <Button on_click=Box::new(move || set_show_confirm.set(true))>
                                                {t!("home.open_confirm")}
                                            </Button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </Card>
                                                </div>
                                            </div>

                                            // Modal Components
                                            <Modal
                                                show=show_modal
                                                on_close=move || set_show_modal.set(false)
                                                title=t!("home.example_modal_title")()
                                            >
                                                <div class="space-y-4">
    <p>{t!("home.example_modal_content")}</p>
                        <p class="text-sm text-gray-600">{t!("home.example_modal_hint")}</p>
                        <Button on_click=Box::new(move || set_show_modal.set(false))>
                            {t!("home.close_modal")}
                        </Button>
                                                </div>
                                            </Modal>

                                            <ConfirmModal
                                                show=show_confirm
                                                on_close=move || set_show_confirm.set(false)
                                                on_confirm=move || {
                                                    toast.success(t!("home.action_confirmed")());
                                                }
                                                title=t!("home.confirm_action")()
                                                message=confirm_message
                                                confirm_text=t!("common.confirm")()
                                                cancel_text=t!("common.cancel")()
                                                is_destructive=false
                                            />
                                            </Show>
                                        </Container>
                                    }
}
