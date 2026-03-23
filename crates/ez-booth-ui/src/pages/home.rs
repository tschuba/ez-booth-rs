use crate::components::*;
use crate::t;
use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    // Modal state
    let (show_modal, set_show_modal) = create_signal(false);
    let (show_confirm, set_show_confirm) = create_signal(false);
    
    // Static message for demo confirm modal
    let confirm_message = Signal::derive(move || 
        "Are you sure you want to proceed with this action?".to_string()
    );
    
    // Get toast context
    let toast = use_toast();
    
    view! {
        <Container>
            <div class="py-12">
                <Card title="EZ Booth">
                    <p class="text-gray-600 mb-4">
                        {t!("app.welcome")}
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
                        <a href="/booths" class="block">
                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                <h3 class="text-lg font-semibold mb-2">{t!("booth.list_title")}</h3>
                                <p class="text-gray-600 text-sm">"Manage your booths"</p>
                            </div>
                        </a>
                        <a href="/checkout" class="block">
                            <div class="p-6 border border-gray-200 rounded-lg hover:shadow-md transition-shadow">
                                <h3 class="text-lg font-semibold mb-2">{t!("checkout.title")}</h3>
                                <p class="text-gray-600 text-sm">"Process sales"</p>
                            </div>
                        </a>
                    </div>
                </Card>
                
                // Component Demo Section
                <div class="mt-8">
                    <Card title="Component Demo">
                        <div class="space-y-4">
                            // Toast Demo
                            <div>
                                <h3 class="text-md font-semibold mb-2">"Toast Notifications"</h3>
                                <div class="flex gap-2 flex-wrap">
                                    <Button on_click=Box::new(move || toast.success("Success! Operation completed."))>
                                        "Show Success"
                                    </Button>
                                    <Button on_click=Box::new(move || toast.error("Error! Something went wrong."))>
                                        "Show Error"
                                    </Button>
                                    <Button on_click=Box::new(move || toast.warning("Warning! Please check your input."))>
                                        "Show Warning"
                                    </Button>
                                    <Button on_click=Box::new(move || toast.info("Info: Here's some information."))>
                                        "Show Info"
                                    </Button>
                                </div>
                            </div>
                            
                            // Modal Demo
                            <div>
                                <h3 class="text-md font-semibold mb-2">"Modals"</h3>
                                <div class="flex gap-2">
                                    <Button on_click=Box::new(move || set_show_modal.set(true))>
                                        "Open Modal"
                                    </Button>
                                    <Button on_click=Box::new(move || set_show_confirm.set(true))>
                                        "Open Confirm Dialog"
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
                title="Example Modal".to_string()
            >
                <div class="space-y-4">
                    <p>"This is an example modal with custom content."</p>
                    <p class="text-sm text-gray-600">"You can press Escape or click the overlay to close it."</p>
                    <Button on_click=Box::new(move || set_show_modal.set(false))>
                        "Close Modal"
                    </Button>
                </div>
            </Modal>
            
            <ConfirmModal
                show=show_confirm
                on_close=move || set_show_confirm.set(false)
                on_confirm=move || {
                    toast.success("Action confirmed!");
                }
                title="Confirm Action".to_string()
                message=confirm_message
                is_destructive=false
            />
        </Container>
    }
}
