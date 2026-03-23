use crate::components::*;
use crate::state::*;
use crate::t;
use leptos::*;
use domain::models::booth::Booth;

#[component]
pub fn BoothListPage() -> impl IntoView {
    let app_state = use_app_state();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (is_loading, set_is_loading) = create_signal(true);
    
    let toast = use_toast();
    
    // Load booths from storage
    create_effect(move |_| {
        spawn_local(async move {
            let state_result = app_state.get();
            match state_result {
                Some(Ok(state)) => {
                    match state.booth_repository.find_all().await {
                        Ok(loaded_booths) => {
                            set_booths.set(loaded_booths);
                            set_is_loading.set(false);
                        }
                        Err(e) => {
                            toast.error(&format!("Failed to load booths: {:?}", e));
                            set_is_loading.set(false);
                        }
                    }
                }
                Some(Err(e)) => {
                    toast.error(&format!("App initialization failed: {}", e));
                    set_is_loading.set(false);
                }
                None => {
                    // Still loading app state
                }
            }
        });
    });
    
    // Handle booth creation
    let handle_create_booth = move |data: BoothFormData| {
        spawn_local(async move {
            let state_result = app_state.get();
            if let Some(Ok(state)) = state_result {
                // Convert form data to domain model
                match data.to_booth() {
                    Ok(booth) => {
                        // Save to storage
                        match state.booth_repository.save(&booth).await {
                            Ok(_) => {
                                toast.success(&format!("Booth created: {}", booth.description));
                                set_show_create_modal.set(false);
                                
                                // Reload booths
                                if let Ok(loaded_booths) = state.booth_repository.find_all().await {
                                    set_booths.set(loaded_booths);
                                }
                            }
                            Err(e) => {
                                toast.error(&format!("Failed to save booth: {:?}", e));
                            }
                        }
                    }
                    Err(e) => {
                        toast.error(&format!("Invalid booth data: {:?}", e));
                    }
                }
            }
        });
    };
    
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
                
                // Loading state
                <Show
                    when=move || is_loading.get()
                    fallback=move || view! {
                        // Booth list or empty state
                        <Show
                            when=move || booths.get().is_empty()
                            fallback=move || view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    <For
                                        each=move || booths.get()
                                        key=|booth| booth.id.as_str().to_string()
                                        children=move |booth| {
                                            view! {
                                                <Card>
                                                    <h3 class="text-lg font-semibold mb-2">{booth.description.clone()}</h3>
                                                    <p class="text-gray-600 mb-2">
                                                        "Date: " {booth.date.to_string()}
                                                    </p>
                                                    <p class="text-sm text-gray-500">
                                                        "Status: " {if booth.is_open() { "Open" } else { "Closed" }}
                                                    </p>
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
                    }
                >
                    <Card>
                        <div class="text-center py-12">
                            <p class="text-gray-600">"Loading booths..."</p>
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
                    on_submit=handle_create_booth
                    on_cancel=move || {
                        set_show_create_modal.set(false);
                    }
                />
            </Modal>
        </Container>
    }
}
