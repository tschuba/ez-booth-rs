use crate::components::*;
use crate::t;
use leptos::*;

// Temporary booth type until we integrate with domain crate
#[derive(Clone, Debug)]
struct BoothListItem {
    id: String,
    description: String,
    date: String,
    status: String,
}

#[component]
pub fn BoothListPage() -> impl IntoView {
    // TODO: Load booths from storage using BoothService
    let (booths, _set_booths) = create_signal(Vec::<BoothListItem>::new());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    
    let toast = use_toast();
    
    // Get translated strings
    let translations = crate::i18n::use_translations();
    let create_booth_title = move || translations.with(|t| t.get("booth.create"));
    
    view! {
        <Container>
            <div class="py-8">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-3xl font-bold text-gray-900">{t!("booth.list_title")}</h1>
                    <Button 
                        on_click=Box::new(move || set_show_create_modal.set(true))
                        aria_label="Create new booth".to_string()
                    >
                        {t!("booth.create")}
                    </Button>
                </div>
                
                // Booth list
                <Show
                    when=move || booths.get().is_empty()
                    fallback=move || view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            <For
                                each=move || booths.get()
                                key=|_booth| 0 // TODO: Use booth.id
                                children=move |_booth| {
                                    view! {
                                        <Card>
                                            <p class="text-gray-600">"Booth item"</p>
                                        </Card>
                                    }
                                }
                            />
                        </div>
                    }
                >
                    <Card>
                        <div class="text-center py-12">
                            <p class="text-gray-600 mb-4">"No booths yet"</p>
                            <Button 
                                on_click=Box::new(move || set_show_create_modal.set(true))
                            >
                                {t!("booth.create")}
                            </Button>
                        </div>
                    </Card>
                </Show>
            </div>
            
            // Create booth modal
            <Modal
                show=show_create_modal
                on_close=move || set_show_create_modal.set(false)
                title=create_booth_title()
                size=ModalSize::Large
            >
                <BoothForm
                    on_submit=move |data| {
                        // TODO: Save booth using BoothService
                        toast.success(&format!("Booth created: {}", data.description));
                        set_show_create_modal.set(false);
                    }
                    on_cancel=move || {
                        set_show_create_modal.set(false);
                    }
                />
            </Modal>
        </Container>
    }
}
