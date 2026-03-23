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
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (editing_booth, set_editing_booth) = create_signal(None::<Booth>);
    let (deleting_booth, set_deleting_booth) = create_signal(None::<Booth>);
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
    
    // Handle booth editing
    let handle_edit_booth = move |data: BoothFormData| {
        spawn_local(async move {
            let state_result = app_state.get();
            if let Some(Ok(state)) = state_result {
                if let Some(mut booth) = editing_booth.get() {
                    // Update the booth with form data
                    match data.update_booth(&mut booth) {
                        Ok(_) => {
                            // Save updated booth to storage
                            match state.booth_repository.save(&booth).await {
                                Ok(_) => {
                                    toast.success(&format!("Booth updated: {}", booth.description));
                                    set_show_edit_modal.set(false);
                                    set_editing_booth.set(None);
                                    
                                    // Reload booths
                                    if let Ok(loaded_booths) = state.booth_repository.find_all().await {
                                        set_booths.set(loaded_booths);
                                    }
                                }
                                Err(e) => {
                                    toast.error(&format!("Failed to update booth: {:?}", e));
                                }
                            }
                        }
                        Err(e) => {
                            toast.error(&format!("Invalid booth data: {:?}", e));
                        }
                    }
                }
            }
        });
    };
    
    // Handle booth deletion
    let handle_delete_booth = move || {
        spawn_local(async move {
            let state_result = app_state.get();
            if let Some(Ok(state)) = state_result {
                if let Some(booth) = deleting_booth.get() {
                    match state.booth_repository.delete(&booth.id).await {
                        Ok(_) => {
                            toast.success(&format!("Booth deleted: {}", booth.description));
                            set_show_delete_confirm.set(false);
                            set_deleting_booth.set(None);
                            
                            // Reload booths
                            if let Ok(loaded_booths) = state.booth_repository.find_all().await {
                                set_booths.set(loaded_booths);
                            }
                        }
                        Err(e) => {
                            toast.error(&format!("Failed to delete booth: {:?}", e));
                        }
                    }
                }
            }
        });
    };
    
    // Get translated strings
    let translations = crate::i18n::use_translations();
    let create_booth_title = move || translations.with(|t| t.get("booth.create"));
    let edit_booth_title = move || translations.with(|t| t.get("booth.edit"));
    
    // Reactive delete message
    let delete_message = move || {
        deleting_booth.get()
            .map(|b| format!("Are you sure you want to delete '{}'? This action cannot be undone.", b.description))
            .unwrap_or_else(|| "Are you sure you want to delete this booth?".to_string())
    };
    
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
                                            let booth_for_edit = booth.clone();
                                            let booth_for_delete = booth.clone();
                                            
                                            view! {
                                                <Card>
                                                    <h3 class="text-lg font-semibold mb-2">{booth.description.clone()}</h3>
                                                    <p class="text-gray-600 mb-2">
                                                        "Date: " {booth.date.to_string()}
                                                    </p>
                                                    <p class="text-sm text-gray-500 mb-4">
                                                        "Status: " {if booth.is_open() { "Open" } else { "Closed" }}
                                                    </p>
                                                    <div class="flex gap-2">
                                                        <Button
                                                            on_click=Box::new(move || {
                                                                set_editing_booth.set(Some(booth_for_edit.clone()));
                                                                set_show_edit_modal.set(true);
                                                            })
                                                            variant=crate::components::ButtonVariant::Secondary
                                                        >
                                                            "Edit"
                                                        </Button>
                                                        <Button
                                                            on_click=Box::new(move || {
                                                                set_deleting_booth.set(Some(booth_for_delete.clone()));
                                                                set_show_delete_confirm.set(true);
                                                            })
                                                            variant=crate::components::ButtonVariant::Danger
                                                        >
                                                            "Delete"
                                                        </Button>
                                                    </div>
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
            
            // Edit booth modal
            <Modal
                show=show_edit_modal
                on_close=move || {
                    set_show_edit_modal.set(false);
                    set_editing_booth.set(None);
                }
                title=edit_booth_title()
                size=ModalSize::Large
            >
                {move || editing_booth.get().map(|booth| {
                    let initial_data = BoothFormData::from_booth(&booth);
                    view! {
                        <BoothForm
                            initial_data=initial_data
                            on_submit=handle_edit_booth
                            on_cancel=move || {
                                set_show_edit_modal.set(false);
                                set_editing_booth.set(None);
                            }
                        />
                    }
                })}
            </Modal>
            
            // Delete confirmation modal
            <ConfirmModal
                show=show_delete_confirm
                on_close=move || {
                    set_show_delete_confirm.set(false);
                    set_deleting_booth.set(None);
                }
                on_confirm=handle_delete_booth
                title="Delete Booth".to_string()
                message=Signal::derive(delete_message)
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                is_destructive=true
            />
        </Container>
    }
}
