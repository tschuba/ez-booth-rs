use crate::booth_ordering::sort_booths;
use crate::components::*;
use crate::error_translator::translate_domain_error;
use crate::formatting::format_date as format_display_date;
use crate::i18n::{translate_with_params, use_locale};
use crate::selected_booth_context::use_selected_booth;
use crate::state::*;
use crate::t;
use domain::models::booth::Booth;
use domain::models::{BoothId, BoothSummary};
use leptos::html;
use leptos::*;
use std::collections::HashMap;
use web_sys::window;

fn confirmation_token_from_booth(booth_id: &BoothId) -> String {
    confirmation_token_from_id_str(&booth_id.as_str())
}

fn confirmation_token_from_id_str(id: &str) -> String {
    let sanitized: String = id.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

    const FALLBACK_TOKEN: &str = "DELETE";

    if sanitized.len() < 4 {
        return FALLBACK_TOKEN.to_string();
    }

    let len = sanitized.len();
    let start = len.saturating_sub(4);
    sanitized[start..].to_uppercase()
}

fn focus_and_select_input(input_ref: &NodeRef<html::Input>) {
    if let Some(input) = input_ref.get() {
        let _ = input.focus();
        let _ = input.select();
    }
}

#[component]
pub fn BoothListPage() -> impl IntoView {
    let app_state = use_app_state();
    let locale = use_locale();
    let selected_booth = use_selected_booth();
    let booth_list_version = crate::selected_booth_context::use_booth_list_version();
    let (booths, set_booths) = create_signal(Vec::<Booth>::new());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_copy_modal, set_show_copy_modal) = create_signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = create_signal(false);
    let (editing_booth, set_editing_booth) = create_signal(None::<Booth>);
    let (copying_booth, set_copying_booth) = create_signal(None::<Booth>);
    let (deleting_booth, set_deleting_booth) = create_signal(None::<Booth>);
    let (is_checking_delete_requirements, set_is_checking_delete_requirements) =
        create_signal(false);
    let (delete_confirmation_token, set_delete_confirmation_token) = create_signal(String::new());
    let (delete_confirmation_input, set_delete_confirmation_input) = create_signal(String::new());
    let delete_confirmation_ref = create_node_ref::<html::Input>();
    let (is_loading, set_is_loading) = create_signal(true);
    let (expanded_booth_id, set_expanded_booth_id) = create_signal(None::<BoothId>);
    let (expanded_booth_summary, set_expanded_booth_summary) = create_signal(None::<BoothSummary>);
    let (is_loading_report, set_is_loading_report) = create_signal(false);

    let deletion_token_matches = create_memo(move |_| {
        let required = delete_confirmation_token.get().trim().to_uppercase();

        if required.is_empty() {
            return false;
        }

        let entered = delete_confirmation_input.get().trim().to_uppercase();
        entered == required
    });

    let toast = use_toast();

    let refresh_booths = move |state: AppState| {
        spawn_local(async move {
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
        });
    };

    let format_date =
        move |date: chrono::NaiveDate| -> String { format_display_date(date, locale.get()) };

    let close_report_modal = move || {
        set_expanded_booth_id.set(None);
        set_expanded_booth_summary.set(None);
        set_is_loading_report.set(false);
    };

    create_effect(move |_| {
        let _ = booth_list_version.get();
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
                            close_report_modal();
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
                    Ok(booth) => match state
                        .booth_service
                        .create_configured_booth(booth.clone())
                        .await
                    {
                        Ok(_) => {
                            toast.success(
                                &t!("booth.success.created")()
                                    .replace("{description}", &booth.description),
                            );
                            set_show_create_modal.set(false);
                            booth_list_version.update(|v| *v += 1);
                            refresh_booths(state.clone());
                        }
                        Err(err) => {
                            toast.error(&translate_domain_error(&err));
                        }
                    },
                    Err(err) => {
                        toast.error(&translate_domain_error(&err));
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
                        Ok(_) => match state.booth_service.update_booth(booth.clone()).await {
                            Ok(_) => {
                                toast.success(
                                    &t!("booth.success.updated")()
                                        .replace("{description}", &booth.description),
                                );
                                set_show_edit_modal.set(false);
                                set_editing_booth.set(None);
                                booth_list_version.update(|v| *v += 1);
                                refresh_booths(state.clone());
                            }
                            Err(err) => {
                                toast.error(&translate_domain_error(&err));
                            }
                        },
                        Err(err) => {
                            toast.error(&translate_domain_error(&err));
                        }
                    }
                }
            }
        });
    };

    let handle_copy_booth = move |data: CopyBoothFormData| {
        let state_result = app_state.get();
        let booth_to_copy = copying_booth.get();

        spawn_local(async move {
            if let Some(Ok(state)) = state_result {
                if let Some(source_booth) = booth_to_copy {
                    let Some(new_date) = data.parse_date() else {
                        toast.error(&t!("validation.date_invalid")());
                        return;
                    };

                    match state
                        .booth_service
                        .copy_booth(source_booth.id, data.description.clone(), new_date)
                        .await
                    {
                        Ok(copied) => {
                            toast.success(
                                &t!("booth.success.copied")()
                                    .replace("{description}", &copied.description),
                            );
                            set_show_copy_modal.set(false);
                            set_copying_booth.set(None);
                            booth_list_version.update(|v| *v += 1);
                            refresh_booths(state.clone());
                        }
                        Err(err) => {
                            toast.error(&translate_domain_error(&err));
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
                            set_delete_confirmation_token.set(String::new());
                            set_delete_confirmation_input.set(String::new());
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

    let close_delete_modals = move || {
        set_show_delete_confirm.set(false);
        set_delete_confirmation_token.set(String::new());
        set_delete_confirmation_input.set(String::new());
        set_deleting_booth.set(None);
        set_is_checking_delete_requirements.set(false);
    };

    let prompt_delete_booth = move |booth: Booth| {
        let state_result = app_state.get();
        set_deleting_booth.set(Some(booth.clone()));
        set_is_checking_delete_requirements.set(true);
        set_show_delete_confirm.set(false);
        set_delete_confirmation_token.set(String::new());
        set_delete_confirmation_input.set(String::new());

        spawn_local(async move {
            let Some(Ok(state)) = state_result else {
                set_is_checking_delete_requirements.set(false);
                return;
            };

            match state.purchase_repository.find_by_booth(&booth.id).await {
                Ok(purchases) => {
                    set_is_checking_delete_requirements.set(false);

                    if purchases.is_empty() {
                        set_show_delete_confirm.set(true);
                    } else {
                        set_delete_confirmation_token.set(confirmation_token_from_booth(&booth.id));
                        set_delete_confirmation_input.set(String::new());
                        set_timeout(
                            move || focus_and_select_input(&delete_confirmation_ref),
                            std::time::Duration::from_millis(0),
                        );
                    }
                }
                Err(_) => {
                    set_is_checking_delete_requirements.set(false);
                    set_deleting_booth.set(None);
                    toast.error(&t!("booth.errors_detail.delete_failed")());
                }
            }
        });
    };

    let delete_message = move || {
        deleting_booth
            .get()
            .map(|b| t!("booth.delete_confirm_message")().replace("{description}", &b.description))
            .unwrap_or_else(|| t!("booth.delete_confirm")())
    };

    let report_modal_title = move || {
        expanded_booth_id.get().and_then(|booth_id| {
            booths
                .get()
                .into_iter()
                .find(|booth| booth.id == booth_id)
                .map(|booth| {
                    format!(
                        "{} - {}",
                        booth.description,
                        t!("report.booth_summary_report")()
                    )
                })
        })
    };

    let handle_print = move |_| {
        set_timeout(
            move || {
                if let Some(window) = window() {
                    let _ = window.print();
                }
            },
            std::time::Duration::from_millis(100),
        );
    };

    let print_header_action = move || {
        view! {
            <button
                type="button"
                on:click=handle_print
                class="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                title={t!("report.print_report")()}
                aria-label={t!("report.print_report")()}
            >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path>
                </svg>
                <span>{t!("report.print_report")}</span>
            </button>
        }
            .into_view()
    };

    view! {
        <>
            <div class="print:hidden">
                <Container>
                    <div class="py-8">
                        <div class="mb-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                            <div>
                                <h1 class="text-3xl font-bold text-gray-900">{t!("booth.list_title")}</h1>
                                <p class="mt-1 text-sm text-gray-600">{t!("backup.booth_list_hint")}</p>
                            </div>
                            <div class="flex flex-wrap items-center gap-3">
                                <ExportButton
                                    scope=ExportScope::All
                                    variant=ButtonVariant::Secondary
                                />
                                <ImportButton variant=ButtonVariant::Secondary />
                                <Button
                                    on_click=Box::new(move || set_show_create_modal.set(true))
                                    aria_label=t!("booth.create_aria_label")()
                                >
                                    {t!("booth.create")}
                                </Button>
                            </div>
                        </div>

                        <div class="mb-6">
                            <StorageWarningInfo />
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
                                                            let booth_date = booth.date;
                                                             let booth_id_stored = store_value(booth.id);
                                                             let booth_for_edit = store_value(booth.clone());
                                                             let booth_for_copy = store_value(booth.clone());
                                                             let booth_for_delete = store_value(booth.clone());

                                                            view! {
                                                                <Card class="booth-card h-full">
                                                                    <div class="flex flex-col gap-4 h-full">
                                                                        <div class="flex gap-4">
                                                                            <div class="min-w-0 flex-1 space-y-3">
                                                                                <h3 class="text-lg font-semibold text-gray-900">
                                                                                    {booth_description.get_value()}
                                                                                </h3>
                                                                                <div class="min-w-0 flex-1">
                                                                                    <p class="text-gray-600">
                                                                                        {t!("booth.date_prefix")} " " {move || format_date(booth_date)}
                                                                                    </p>
                                                                                </div>
                                                                            </div>
                                                                            <button
                                                                                on:click=move |_| {
                                                                                    set_expanded_booth_summary.set(None);
                                                                                    set_expanded_booth_id.set(Some(booth_id_stored.get_value()));
                                                                                }
                                                                                disabled=move || is_loading_report.get()
                                                                                title={t!("booth.view_report")()}
                                                                                aria-label={t!("booth.view_report_aria")()}
                                                                                class="inline-flex h-20 w-20 shrink-0 items-center justify-center self-stretch rounded-lg bg-blue-600 text-white shadow-sm transition-colors hover:bg-blue-700 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                                                                            >
                                                                                <svg class="h-10 w-10" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z"></path>
                                                                                </svg>
                                                                            </button>
                                                                        </div>
                                                                        <div class="mt-auto flex flex-wrap items-center justify-end gap-3">
                                                                            <ExportButton
                                                                                scope=ExportScope::Booth(booth_id_stored.get_value())
                                                                                variant=ButtonVariant::Secondary
                                                                                size=ButtonSize::Small
                                                                            />
                                                                            <Button
                                                                                on_click=Box::new(move || {
                                                                                    set_copying_booth.set(Some(booth_for_copy.get_value()));
                                                                                    set_show_copy_modal.set(true);
                                                                                })
                                                                                variant=ButtonVariant::Ghost
                                                                                class="h-12 w-20 p-0".to_string()
                                                                                title=t!("booth.copy_button")()
                                                                                aria_label=t!("booth.copy_button")()
                                                                            >
                                                                                <svg class="h-7 w-7" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                                                                                </svg>
                                                                            </Button>
                                                                            <Button
                                                                                on_click=Box::new(move || {
                                                                                    set_editing_booth.set(Some(booth_for_edit.get_value()));
                                                                                    set_show_edit_modal.set(true);
                                                                                })
                                                                                variant=ButtonVariant::Ghost
                                                                                class="h-12 w-20 p-0".to_string()
                                                                                title=t!("booth.edit_button")()
                                                                                aria_label=t!("booth.edit_button")()
                                                                            >
                                                                                <svg class="h-7 w-7" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                                                                                </svg>
                                                                            </Button>
                                                                            <Button
                                                                                on_click=Box::new(move || {
                                                                                    prompt_delete_booth(booth_for_delete.get_value());
                                                                                })
                                                                                variant=ButtonVariant::Danger
                                                                                class="ml-6 h-12 w-20 p-0".to_string()
                                                                                title=t!("booth.delete_button")()
                                                                                aria_label=t!("booth.delete_button")()
                                                                            >
                                                                                <svg class="h-7 w-7" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                                                                                </svg>
                                                                            </Button>
                                                                        </div>
                                                                    </div>
                                                                </Card>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                        }
                                    >
                                        <Card>
                                            <div class="px-6 py-12 text-center">
                                                <p class="mb-4 text-lg font-semibold text-gray-700">
                                                    {t!("booth.empty_state.heading")}
                                                </p>
                                                <p class="mb-3 text-gray-600">
                                                    <span class="font-medium text-gray-800">
                                                        {t!("booth.empty_state.requirement")}
                                                    </span>
                                                    " "
                                                    {t!("booth.empty_state.features")}
                                                </p>
                                                <p class="mb-6 text-gray-600">
                                                    {t!("booth.empty_state.get_started")}
                                                </p>
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

                    <Show when=move || expanded_booth_id.get().is_some()>
                        {move || {
                            let modal_title = report_modal_title().unwrap_or_else(|| t!("report.booth_summary_report")());

                            view! {
                                <Modal
                                    show=Signal::derive(move || expanded_booth_id.get().is_some())
                                    on_close=close_report_modal
                                    title=Signal::derive(move || modal_title.clone())
                                    header_actions=print_header_action()
                                    size=ModalSize::XLarge
                                >
                                    <div class="space-y-6">
                                        <Show
                                            when=move || is_loading_report.get()
                                            fallback=move || {
                                                view! {
                                                    <Show when=move || expanded_booth_summary.get().is_some()>
                                                        {move || {
                                                            expanded_booth_summary.get().map(|summary| {
                                                                view! {
                                                                    <BoothSummaryDisplay summary=summary />
                                                                }
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
                                </Modal>
                            }
                        }}
                    </Show>

                    <Modal
                        show=show_create_modal
                        on_close=move || set_show_create_modal.set(false)
                        title=Signal::derive(move || create_booth_title())
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
                        title=Signal::derive(move || edit_booth_title())
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

                    <Modal
                        show=show_copy_modal
                        on_close=move || {
                            set_show_copy_modal.set(false);
                            set_copying_booth.set(None);
                        }
                        title=Signal::derive(move || t!("booth.copy_title")())
                        size=ModalSize::Medium
                    >
                        {move || copying_booth.get().map(|booth| {
                            view! {
                                <CopyBoothDialog
                                    source_booth=booth
                                    on_submit=handle_copy_booth
                                    on_cancel=move || {
                                        set_show_copy_modal.set(false);
                                        set_copying_booth.set(None);
                                    }
                                />
                            }
                        })}
                    </Modal>

                    <ConfirmModal
                        show=show_delete_confirm
                        on_close=close_delete_modals
                        on_confirm=handle_delete_booth
                        title=Signal::derive(move || t!("booth.delete")())
                        message=Signal::derive(delete_message)
                        confirm_text=Signal::derive(move || t!("common.delete")())
                        cancel_text=Signal::derive(move || t!("common.cancel")())
                        is_destructive=true
                    />

                    <Modal
                        show=Signal::derive(move || {
                            is_checking_delete_requirements.get()
                                || (!delete_confirmation_token.get().is_empty()
                                    && deleting_booth.get().is_some())
                        })
                        on_close=close_delete_modals
                        title=Signal::derive(move || {
                            if is_checking_delete_requirements.get() {
                                t!("booth.delete")()
                            } else {
                                t!("booth.delete_modal.title")()
                            }
                        })
                        size=ModalSize::Medium
                    >
                        <Show
                            when=move || is_checking_delete_requirements.get()
                            fallback=move || {
                                view! {
                                    <Show when=move || deleting_booth.get().is_some()>
                                        <div class="space-y-4">
                                            <p class="text-gray-700">
                                                {move || {
                                                    translate_with_params(
                                                        "booth.delete_modal.instructions",
                                                        HashMap::from([(
                                                            "token",
                                                            delete_confirmation_token.get(),
                                                        )]),
                                                    )
                                                }}
                                            </p>
                                            <input
                                                class="w-full rounded-lg border border-gray-300 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-red-500"
                                                placeholder=t!("booth.delete_modal.placeholder")()
                                                value=move || delete_confirmation_input.get()
                                                node_ref=delete_confirmation_ref
                                                on:input=move |ev| {
                                                    set_delete_confirmation_input
                                                        .set(event_target_value(&ev));
                                                }
                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                    if ev.key() == "Enter" && deletion_token_matches.get() {
                                                        ev.prevent_default();
                                                        handle_delete_booth();
                                                    }
                                                }
                                            />
                                            <div class="flex justify-end gap-2">
                                                <Button
                                                    variant=ButtonVariant::Secondary
                                                    on_click=Box::new(close_delete_modals)
                                                >
                                                    {t!("common.cancel")}
                                                </Button>
                                                <Button
                                                    variant=ButtonVariant::Danger
                                                    disabled=!deletion_token_matches.get()
                                                    on_click=Box::new(handle_delete_booth)
                                                >
                                                    {t!("booth.delete_modal.confirm")}
                                                </Button>
                                            </div>
                                        </div>
                                    </Show>
                                }
                            }
                        >
                            <div class="flex items-center justify-center py-12">
                                <div class="text-center">
                                    <div class="inline-block h-12 w-12 animate-spin rounded-full border-b-2 border-blue-600"></div>
                                    <p class="mt-4 text-gray-600">{t!("common.loading")}</p>
                                </div>
                            </div>
                        </Show>
                    </Modal>
                </Container>
            </div>

            <div class="hidden print:block">
                <Show when=move || expanded_booth_summary.get().is_some()>
                    {move || {
                        expanded_booth_summary.get().and_then(|summary| {
                            expanded_booth_id.get().and_then(|booth_id| {
                                booths
                                    .get()
                                    .into_iter()
                                    .find(|booth| booth.id == booth_id)
                                    .map(|booth| {
                                        view! {
                                            <PrintBoothSummary
                                                summary=summary
                                                booth_name=booth.description.clone()
                                                booth_date=booth.date
                                            />
                                        }
                                    })
                            })
                        })
                    }}
                </Show>
            </div>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::confirmation_token_from_id_str;

    #[test]
    fn confirmation_token_uses_last_four_alphanumeric_characters() {
        assert_eq!(
            confirmation_token_from_id_str("12345678-1234-1234-1234-1234567890ab"),
            "90AB"
        );
    }

    #[test]
    fn confirmation_token_falls_back_when_id_has_too_few_alphanumeric_characters() {
        assert_eq!(confirmation_token_from_id_str("--"), "DELETE");
        assert_eq!(confirmation_token_from_id_str("a-1"), "DELETE");
    }
}
