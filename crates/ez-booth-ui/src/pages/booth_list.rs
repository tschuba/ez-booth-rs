use crate::booth_ordering::sort_booths;
use crate::components::*;
use crate::i18n::{translate_with_params, use_locale, Locale};
use crate::selected_booth_context::use_selected_booth;
use crate::state::*;
use crate::t;
use chrono::Datelike;
use domain::models::booth::Booth;
use domain::models::{BoothId, BoothSummary};
use leptos::*;
use web_sys::window;

#[component]
pub fn BoothListPage() -> impl IntoView {
    let app_state = use_app_state();
    let locale = use_locale();
    let selected_booth = use_selected_booth();
    let booth_list_version = crate::selected_booth_context::use_booth_list_version();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (editing_booth, set_editing_booth) = create_signal(None::<Booth>);
    let (deleting_booth, set_deleting_booth) = create_signal(None::<Booth>);
    let (is_loading, set_is_loading) = create_signal(true);
    let (expanded_booth_id, set_expanded_booth_id) = create_signal(None::<BoothId>);
    let (expanded_booth_summary, set_expanded_booth_summary) = create_signal(None::<BoothSummary>);
    let (is_loading_report, set_is_loading_report) = create_signal(false);
    let (saved_scroll_y, set_saved_scroll_y) = create_signal(0.0_f64);

    let toast = use_toast();

    let format_date = move |date: chrono::NaiveDate| -> String {
        match locale.get() {
            Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => {
                let day = date.day();
                let month = match date.month() {
                    1 => "Jan",
                    2 => "Feb",
                    3 => "Mär",
                    4 => "Apr",
                    5 => "Mai",
                    6 => "Jun",
                    7 => "Jul",
                    8 => "Aug",
                    9 => "Sep",
                    10 => "Okt",
                    11 => "Nov",
                    12 => "Dez",
                    _ => "?",
                };
                format!("{}. {}", day, month)
            }
            Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => date.format("%b %d").to_string(),
        }
    };

    let save_scroll_position = move || {
        if let Some(window) = window() {
            set_saved_scroll_y.set(window.scroll_y().unwrap_or(0.0));
        }
    };

    let restore_scroll_position = move || {
        if let Some(window) = window() {
            let y = saved_scroll_y.get_untracked();
            window.scroll_to_with_x_and_y(0.0, y);
        }
    };

    let collapse_expanded_report = move || {
        if expanded_booth_id.get_untracked().is_some() {
            set_expanded_booth_id.set(None);
            set_expanded_booth_summary.set(None);
            set_is_loading_report.set(false);
            restore_scroll_position();
        }
    };

    create_effect(move |_| {
        let state_result = app_state.get();

        if let Some(Ok(state)) = state_result {
            spawn_local(async move {
                match state.booth_repository.find_all().await {
                    Ok(loaded_booths) => {
                        let mut sorted = loaded_booths;
                        sort_booths(&mut sorted);
                        set_booths.set(sorted);
                        set_is_loading.set(false);
                    }
                    Err(_) => {
                        toast.error(&t!("booth.errors.load_failed")());
                        set_is_loading.set(false);
                    }
                }
            });
        } else if let Some(Err(e)) = state_result {
            toast.error(&t!("booth.errors_detail.init_failed")().replace("{error}", &e));
            set_is_loading.set(false);
        }
    });

    create_effect(move |_| {
        let state_result = app_state.get();
        let booth_id = expanded_booth_id.get();

        if let (Some(Ok(state)), Some(booth_id)) = (state_result, booth_id) {
            set_is_loading_report.set(true);
            let booth_id_for_request = booth_id.clone();
            spawn_local(async move {
                match state
                    .report_service
                    .generate_booth_summary(&booth_id_for_request, None)
                    .await
                {
                    Ok(summary) => {
                        if expanded_booth_id.get_untracked() == Some(booth_id_for_request) {
                            set_expanded_booth_summary.set(Some(summary));
                        }
                    }
                    Err(e) => {
                        if expanded_booth_id.get_untracked() == Some(booth_id_for_request) {
                            let error_msg = translate_with_params(
                                "report.errors.generate_failed",
                                std::collections::HashMap::from([(
                                    "error",
                                    format_error_message(&e),
                                )]),
                            );
                            toast.error(&error_msg);
                            set_expanded_booth_id.set(None);
                            set_expanded_booth_summary.set(None);
                            restore_scroll_position();
                        }
                    }
                }

                if expanded_booth_id.get_untracked() == Some(booth_id_for_request) {
                    set_is_loading_report.set(false);
                }
            });
        } else if booth_id.is_none() {
            set_expanded_booth_summary.set(None);
            set_is_loading_report.set(false);
        }
    });

    let handle_create_booth = move |data: BoothFormData| {
        let state_result = app_state.get();
        let locale = use_locale().get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                match data.to_booth(locale) {
                    Ok(booth) => match state.booth_repository.save(&booth).await {
                        Ok(_) => {
                            toast.success(
                                &t!("booth.success.created")()
                                    .replace("{description}", &booth.description),
                            );
                            set_show_create_modal.set(false);
                            booth_list_version.update(|v| *v += 1);

                            match state.booth_repository.find_all().await {
                                Ok(loaded_booths) => {
                                    let mut sorted = loaded_booths;
                                    sort_booths(&mut sorted);
                                    set_booths.set(sorted);
                                }
                                Err(_) => {
                                    toast.error(&t!("booth.errors.load_failed")());
                                }
                            }
                        }
                        Err(_) => {
                            toast.error(&t!("booth.errors_detail.save_failed")());
                        }
                    },
                    Err(_) => {
                        toast.error(&t!("booth.errors_detail.invalid_data")());
                    }
                }
            }
        });
    };

    let handle_edit_booth = move |data: BoothFormData| {
        let state_result = app_state.get();
        let booth_to_edit = editing_booth.get();
        let locale = use_locale().get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                if let Some(mut booth) = booth_to_edit {
                    match data.update_booth(&mut booth, locale) {
                        Ok(_) => match state.booth_repository.save(&booth).await {
                            Ok(_) => {
                                toast.success(
                                    &t!("booth.success.updated")()
                                        .replace("{description}", &booth.description),
                                );
                                set_show_edit_modal.set(false);
                                set_editing_booth.set(None);
                                booth_list_version.update(|v| *v += 1);

                                match state.booth_repository.find_all().await {
                                    Ok(loaded_booths) => {
                                        let mut sorted = loaded_booths;
                                        sort_booths(&mut sorted);
                                        set_booths.set(sorted);
                                    }
                                    Err(_) => {
                                        toast.error(&t!("booth.errors.load_failed")());
                                    }
                                }
                            }
                            Err(_) => {
                                toast.error(&t!("booth.errors_detail.update_failed")());
                            }
                        },
                        Err(_) => {
                            toast.error(&t!("booth.errors_detail.invalid_data")());
                        }
                    }
                }
            }
        });
    };

    let handle_delete_booth = move || {
        let state_result = app_state.get();
        let booth_to_delete = deleting_booth.get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                if let Some(booth) = booth_to_delete {
                    match state.booth_repository.delete(&booth.id).await {
                        Ok(_) => {
                            if expanded_booth_id.get_untracked() == Some(booth.id) {
                                set_expanded_booth_id.set(None);
                                set_expanded_booth_summary.set(None);
                                set_is_loading_report.set(false);
                            }

                            if let Some(current_booth) = selected_booth.get() {
                                if current_booth.id == booth.id {
                                    selected_booth.set(None);
                                }
                            }

                            toast.success(
                                &t!("booth.success.deleted")()
                                    .replace("{description}", &booth.description),
                            );
                            set_show_delete_confirm.set(false);
                            set_deleting_booth.set(None);
                            booth_list_version.update(|v| *v += 1);

                            match state.booth_repository.find_all().await {
                                Ok(loaded_booths) => {
                                    let mut sorted = loaded_booths;
                                    sort_booths(&mut sorted);
                                    set_booths.set(sorted);
                                }
                                Err(_) => {
                                    toast.error(&t!("booth.errors.load_failed")());
                                }
                            }
                        }
                        Err(_) => {
                            toast.error(&t!("booth.errors_detail.delete_failed")());
                        }
                    }
                }
            }
        });
    };

    let translations = crate::i18n::use_translations();
    let create_booth_title = move || translations.with(|t| t.get("booth.create"));
    let edit_booth_title = move || translations.with(|t| t.get("booth.edit"));

    let delete_message = move || {
        deleting_booth
            .get()
            .map(|b| t!("booth.delete_confirm_message")().replace("{description}", &b.description))
            .unwrap_or_else(|| t!("booth.delete_confirm")())
    };

    let handle_print = move |_| {
        if let Some(window) = window() {
            let _ = window.print();
        }
    };

    view! {
        <>
            <div class="print:hidden">
                <Container>
                    <div class="py-8" on:click=move |_| collapse_expanded_report()>
                        <div class="flex justify-between items-center mb-6">
                            <h1 class="text-3xl font-bold text-gray-900">{t!("booth.list_title")}</h1>
                            <Button
                                on_click=Box::new(move || set_show_create_modal.set(true))
                                aria_label=t!("booth.create_aria_label")()
                            >
                                {t!("booth.create")}
                            </Button>
                        </div>

                        <Show
                            when=move || is_loading.get()
                            fallback=move || {
                                view! {
                                    <Show
                                        when=move || booths.get().is_empty()
                                        fallback=move || {
                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                                    <For
                                                        each=move || booths.get()
                                                        key=|booth| booth.id.as_str().to_string()
                                                        children=move |booth| {
                                                            let booth_description = store_value(booth.description.clone());
                                                            let booth_is_open = booth.is_open();
                                                            let booth_date = booth.date;
                                                            let booth_id = booth.id;
                                                            let booth_id_stored = store_value(booth_id);
                                                            let booth_for_edit = store_value(booth.clone());
                                                            let booth_for_delete = store_value(booth.clone());

                                                            view! {
                                                                <div on:click=move |e| e.stop_propagation()>
                                                                    <Show
                                                                        when=move || {
                                                                            expanded_booth_id.get() == Some(booth_id_stored.get_value())
                                                                        }
                                                                        fallback=move || {
                                                                            view! {
                                                                                <Card class="booth-card h-full">
                                                                                    <div class="flex flex-col gap-4 h-full">
                                                                                        <div class="space-y-2">
                                                                                            <h3 class="text-lg font-semibold text-gray-900">
                                                                                                {booth_description.get_value()}
                                                                                            </h3>
                                                                                            <p class="text-gray-600">
                                                                                                {t!("booth.date_prefix")} " " {move || format_date(booth_date)}
                                                                                            </p>
                                                                                            <p class="text-sm text-gray-500">
                                                                                                {t!("booth.status_label")} " "
                                                                                                {move || if booth_is_open {
                                                                                                    t!("booth.status_open")()
                                                                                                } else {
                                                                                                    t!("booth.status_closed")()
                                                                                                }}
                                                                                            </p>
                                                                                        </div>
                                                                                        <div class="flex items-center justify-end gap-2 mt-auto">
                                                                                            <button
                                                                                                on:click=move |e| {
                                                                                                    e.stop_propagation();
                                                                                                    save_scroll_position();
                                                                                                    set_expanded_booth_summary.set(None);
                                                                                                    set_expanded_booth_id.set(Some(booth_id_stored.get_value()));
                                                                                                }
                                                                                                disabled=move || is_loading_report.get()
                                                                                                title={t!("booth.view_report")()}
                                                                                                aria-label={t!("booth.view_report_aria")()}
                                                                                                class="w-10 h-10 inline-flex items-center justify-center rounded-lg bg-gray-100 text-gray-700 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                                                            >
                                                                                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                                    <line x1="18" y1="20" x2="18" y2="10"></line>
                                                                                                    <line x1="12" y1="20" x2="12" y2="4"></line>
                                                                                                    <line x1="6" y1="20" x2="6" y2="14"></line>
                                                                                                </svg>
                                                                                            </button>
                                                                                            <Button
                                                                                                on_click=Box::new(move || {
                                                                                                    set_editing_booth.set(Some(booth_for_edit.get_value()));
                                                                                                    set_show_edit_modal.set(true);
                                                                                                })
                                                                                                variant=ButtonVariant::Secondary
                                                                                            >
                                                                                                {t!("booth.edit_button")}
                                                                                            </Button>
                                                                                            <Button
                                                                                                on_click=Box::new(move || {
                                                                                                    set_deleting_booth.set(Some(booth_for_delete.get_value()));
                                                                                                    set_show_delete_confirm.set(true);
                                                                                                })
                                                                                                variant=ButtonVariant::Danger
                                                                                            >
                                                                                                {t!("booth.delete_button")}
                                                                                            </Button>
                                                                                        </div>
                                                                                    </div>
                                                                                </Card>
                                                                            }
                                                                        }
                                                                    >
                                                                        <Card class="expanded-booth-report lg:col-span-2 xl:col-span-3">
                                                                            <div class="space-y-6">
                                                                                <div class="flex items-center justify-between gap-4">
                                                                                    <div>
                                                                                        <h3 class="text-xl font-semibold text-gray-900">
                                                                                            {booth_description.get_value()}
                                                                                        </h3>
                                                                                        <p class="text-sm text-gray-500 mt-1">
                                                                                            {t!("booth.date_prefix")} " " {move || format_date(booth_date)}
                                                                                        </p>
                                                                                    </div>
                                                                                    <Button
                                                                                        variant=ButtonVariant::Secondary
                                                                                        on_click=Box::new(move || collapse_expanded_report())
                                                                                    >
                                                                                        {t!("common.back")}
                                                                                    </Button>
                                                                                </div>

                                                                                <Show
                                                                                    when=move || is_loading_report.get()
                                                                                    fallback=move || {
                                                                                        view! {
                                                                                            <Show when=move || expanded_booth_summary.get().is_some()>
                                                                                                {move || {
                                                                                                    expanded_booth_summary.get().map(|summary| {
                                                                                                        view! { <BoothSummaryDisplay summary=summary /> }
                                                                                                    })
                                                                                                }}
                                                                                            </Show>
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    <div class="flex items-center justify-center py-12">
                                                                                        <div class="text-center">
                                                                                            <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                                                                                            <p class="mt-4 text-gray-600">{t!("report.loading")}</p>
                                                                                        </div>
                                                                                    </div>
                                                                                </Show>
                                                                            </div>
                                                                        </Card>
                                                                    </Show>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
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

                    <Show
                        when=move || expanded_booth_id.get().is_some() && expanded_booth_summary.get().is_some() && !is_loading_report.get()
                    >
                        <button
                            on:click=handle_print
                            class="fixed bottom-8 right-8 min-w-[4rem] min-h-[4rem] w-16 h-16 flex items-center justify-center rounded-full shadow-2xl bg-blue-600 text-white hover:bg-blue-700 hover:scale-105 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                            title={t!("report.print_report")()}
                            aria-label={t!("report.print_report")()}
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                            </svg>
                        </button>
                    </Show>

                    <Modal
                        show=show_create_modal
                        on_close=move || set_show_create_modal.set(false)
                        title=create_booth_title()
                        size=ModalSize::Large
                    >
                        {move || {
                            if show_create_modal.get() {
                                let current_locale = locale.get();
                                Some(view! {
                                    <BoothForm
                                        initial_data=BoothFormData::default_with_locale(current_locale)
                                        on_submit=handle_create_booth
                                        on_cancel=move || {
                                            set_show_create_modal.set(false);
                                        }
                                    />
                                })
                            } else {
                                None
                            }
                        }}
                    </Modal>

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
                            let current_locale = locale.get();
                            let initial_data = BoothFormData::from_booth(&booth, current_locale);
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
            </div>

            <div class="hidden print:block">
                <Show when=move || expanded_booth_summary.get().is_some()>
                    {move || {
                        expanded_booth_summary
                            .get()
                            .map(|summary| view! { <PrintBoothSummary summary=summary /> })
                    }}
                </Show>
            </div>
        </>
    }
}
