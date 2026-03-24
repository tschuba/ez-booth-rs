use crate::components::*;
use crate::i18n::{use_locale, Locale};
use crate::state::*;
use crate::t;
use chrono::Datelike;
use domain::models::booth::Booth;
use leptos::*;

#[component]
pub fn BoothListPage() -> impl IntoView {
    let app_state = use_app_state();
    let locale = use_locale();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (editing_booth, set_editing_booth) = create_signal(None::<Booth>);
    let (deleting_booth, set_deleting_booth) = create_signal(None::<Booth>);
    let (is_loading, set_is_loading) = create_signal(true);

    let toast = use_toast();

    // Format date based on locale
    // German: "24. Mär" (DD. MMM)
    // English: "Mar 24" (MMM DD)
    let format_date = move |date: chrono::NaiveDate| -> String {
        match locale.get() {
            Locale::De => {
                // German format: DD. MMM (e.g., "24. Mär")
                let day = date.day();
                let month = match date.month() {
                    1 => "Jan", 2 => "Feb", 3 => "Mär", 4 => "Apr",
                    5 => "Mai", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                    9 => "Sep", 10 => "Okt", 11 => "Nov", 12 => "Dez",
                    _ => "?",
                };
                format!("{}. {}", day, month)
            }
            Locale::En => {
                // English format: MMM DD (e.g., "Mar 24")
                date.format("%b %d").to_string()
            }
        }
    };

    // Load booths from storage - track app_state resource
    create_effect(move |_| {
        // Read app_state inside the effect so it's tracked
        let state_result = app_state.get();

        web_sys::console::log_1(
            &format!("Effect running, state_result: {:?}", state_result.is_some()).into(),
        );

        if let Some(Ok(state)) = state_result {
            web_sys::console::log_1(&"App state ready, loading booths...".into());
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(loaded_booths) => {
                        web_sys::console::log_1(
                            &format!("Loaded {} booths", loaded_booths.len()).into(),
                        );
                        set_booths.set(loaded_booths.clone());
                        set_is_loading.set(false);
                        web_sys::console::log_1(&format!("Set booths signal, is_loading now false. Booths count in signal: {}", loaded_booths.len()).into());
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Failed to load booths: {:?}", e).into());
                        toast.error(&t!("booth.errors.load_failed")());
                        set_is_loading.set(false);
                    }
                }
            });
        } else if let Some(Err(e)) = state_result {
            web_sys::console::log_1(&format!("App state error: {}", e).into());
            toast.error(&t!("booth.errors_detail.init_failed")().replace("{error}", &e));
            set_is_loading.set(false);
        } else {
            web_sys::console::log_1(&"App state still loading...".into());
        }
        // If None, still loading - keep is_loading true
    });

    // Handle booth creation
    let handle_create_booth = move |data: BoothFormData| {
        // Read app_state BEFORE spawn_local to avoid reactive tracking issues
        let state_result = app_state.get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                // Convert form data to domain model
                match data.to_booth() {
                    Ok(booth) => {
                        // Save to storage
                        match state.booth_repository.save(&booth).await {
                            Ok(_) => {
                                web_sys::console::log_1(
                                    &format!("Booth saved: {}", booth.description).into(),
                                );
                                toast.success(&t!("booth.success.created")().replace("{description}", &booth.description));
                                set_show_create_modal.set(false);

                                // Reload booths
                                match state.booth_repository.find_all().await {
                                    Ok(loaded_booths) => {
                                        web_sys::console::log_1(
                                            &format!(
                                                "After create: reloaded {} booths",
                                                loaded_booths.len()
                                            )
                                            .into(),
                                        );
                                        set_booths.set(loaded_booths);
                                    }
                                    Err(e) => {
                                        web_sys::console::log_1(
                                            &format!("Failed to reload after create: {:?}", e)
                                                .into(),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                toast.error(&t!("booth.errors_detail.save_failed")());
                            }
                        }
                    }
                    Err(e) => {
                        toast.error(&t!("booth.errors_detail.invalid_data")());
                    }
                }
            }
        });
    };

    // Handle booth editing
    let handle_edit_booth = move |data: BoothFormData| {
        // Read signals BEFORE spawn_local
        let state_result = app_state.get();
        let booth_to_edit = editing_booth.get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                if let Some(mut booth) = booth_to_edit {
                    // Update the booth with form data
                    match data.update_booth(&mut booth) {
                        Ok(_) => {
                            // Save updated booth to storage
                            match state.booth_repository.save(&booth).await {
                                Ok(_) => {
                                    web_sys::console::log_1(
                                        &format!("Booth updated: {}", booth.description).into(),
                                    );
                                    toast.success(&t!("booth.success.updated")().replace("{description}", &booth.description));
                                    set_show_edit_modal.set(false);
                                    set_editing_booth.set(None);

                                    // Reload booths
                                    match state.booth_repository.find_all().await {
                                        Ok(loaded_booths) => {
                                            web_sys::console::log_1(
                                                &format!(
                                                    "After edit: reloaded {} booths",
                                                    loaded_booths.len()
                                                )
                                                .into(),
                                            );
                                            set_booths.set(loaded_booths);
                                        }
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("Failed to reload after edit: {:?}", e)
                                                    .into(),
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    toast.error(&t!("booth.errors_detail.update_failed")());
                                }
                            }
                        }
                        Err(e) => {
                            toast.error(&t!("booth.errors_detail.invalid_data")());
                        }
                    }
                }
            }
        });
    };

    // Handle booth deletion
    let handle_delete_booth = move || {
        // Read signals BEFORE spawn_local
        let state_result = app_state.get();
        let booth_to_delete = deleting_booth.get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                if let Some(booth) = booth_to_delete {
                    match state.booth_repository.delete(&booth.id).await {
                        Ok(_) => {
                            web_sys::console::log_1(
                                &format!("Booth deleted: {}", booth.description).into(),
                            );
                            toast.success(&t!("booth.success.deleted")().replace("{description}", &booth.description));
                            set_show_delete_confirm.set(false);
                            set_deleting_booth.set(None);

                            // Reload booths
                            match state.booth_repository.find_all().await {
                                Ok(loaded_booths) => {
                                    web_sys::console::log_1(
                                        &format!(
                                            "After delete: reloaded {} booths",
                                            loaded_booths.len()
                                        )
                                        .into(),
                                    );
                                    set_booths.set(loaded_booths);
                                }
                                Err(e) => {
                                    web_sys::console::log_1(
                                        &format!("Failed to reload after delete: {:?}", e).into(),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            toast.error(&t!("booth.errors_detail.delete_failed")());
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
        deleting_booth
            .get()
            .map(|b| {
                t!("booth.delete_confirm_message")()
                    .replace("{description}", &b.description)
            })
            .unwrap_or_else(|| t!("booth.delete_confirm")())
    };

    view! {
        <Container>
            <div class="py-8">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-3xl font-bold text-gray-900">{t!("booth.list_title")}</h1>
                    <Button
                        on_click=Box::new(move || set_show_create_modal.set(true))
                        aria_label=t!("booth.create_aria_label")()
                    >
                        {t!("booth.create")}
                    </Button>
                </div>

                // Loading state
                <Show
                    when=move || {
                        let loading = is_loading.get();
                        web_sys::console::log_1(&format!("Rendering: is_loading={}", loading).into());
                        loading
                    }
                    fallback=move || {
                        let booth_count = booths.get().len();
                        web_sys::console::log_1(&format!("Rendering fallback: {} booths", booth_count).into());
                        view! {
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
                                            let booth_date = booth.date;

                                            view! {
                                                <Card>
                                                    <h3 class="text-lg font-semibold mb-2">{booth.description.clone()}</h3>
                                                    <p class="text-gray-600 mb-2">
                                                        {t!("booth.date_prefix")} " " {move || format_date(booth_date)}
                                                    </p>
                                                    <p class="text-sm text-gray-500 mb-4">
                                                        {t!("booth.status_label")} " " {move || if booth.is_open() { t!("booth.status_open")() } else { t!("booth.status_closed")() }}
                                                    </p>
                                                    <div class="flex gap-2">
                                                        <Button
                                                            on_click=Box::new(move || {
                                                                set_editing_booth.set(Some(booth_for_edit.clone()));
                                                                set_show_edit_modal.set(true);
                                                            })
                                                            variant=crate::components::ButtonVariant::Secondary
                                                        >
                                                            {t!("booth.edit_button")}
                                                        </Button>
                                                        <Button
                                                            on_click=Box::new(move || {
                                                                set_deleting_booth.set(Some(booth_for_delete.clone()));
                                                                set_show_delete_confirm.set(true);
                                                            })
                                                            variant=crate::components::ButtonVariant::Danger
                                                        >
                                                            {t!("booth.delete_button")}
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
                                    <p class="text-gray-600 mb-4">{t!("booth.no_booths_message")}</p>
                                    <Button
                                        on_click=Box::new(move || set_show_create_modal.set(true))
                                    >
                                        {t!("booth.create")}
                                    </Button>
                                </div>
                            </Card>
                        </Show>
                        }
                    }
                >
                    <Card>
                        <div class="text-center py-12">
                            <p class="text-gray-600">{t!("booth.loading_message")}</p>
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
                title=t!("booth.delete")()
                message=Signal::derive(delete_message)
                confirm_text=t!("common.delete")()
                cancel_text=t!("common.cancel")()
                is_destructive=true
            />
        </Container>
    }
}
