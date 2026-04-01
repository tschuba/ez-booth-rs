use leptos::*;

use crate::components::*;
use crate::t;
use crate::utils::{
    current_device_info, reset_device_identifier, save_device_identifier,
    validate_device_identifier,
};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let toast = use_toast();
    let device_info = current_device_info();
    let platform = device_info.platform.clone();
    let browser = device_info.browser.clone();
    let initial_identifier = device_info.identifier;

    let (saved_identifier, set_saved_identifier) = create_signal(initial_identifier.clone());
    let device_identifier = create_rw_signal(initial_identifier);
    let (validation_error, set_validation_error) = create_signal(None::<String>);

    create_effect(move |_| {
        let value = device_identifier.get();
        let message = match validate_device_identifier(&value) {
            Ok(()) => None,
            Err(_) => Some(t!("settings.device_validation_error")()),
        };
        set_validation_error.set(message);
    });

    let is_dirty = Signal::derive(move || device_identifier.get() != saved_identifier.get());
    let can_save = Signal::derive(move || is_dirty.get() && validation_error.get().is_none());

    let handle_save = {
        let toast = toast.clone();
        move || {
            let value = device_identifier.get_untracked();
            match save_device_identifier(value) {
                Ok(saved_value) => {
                    set_saved_identifier.set(saved_value.clone());
                    device_identifier.set(saved_value);
                    toast.success(t!("settings.device_save_success")());
                }
                Err(error) => {
                    toast.error(format!("{}: {error}", t!("common.error")()));
                }
            }
        }
    };

    let handle_reset = {
        let toast = toast.clone();
        move || match reset_device_identifier() {
            Ok(generated) => {
                set_saved_identifier.set(generated.clone());
                device_identifier.set(generated);
                toast.success(t!("settings.device_reset_success")());
            }
            Err(error) => {
                toast.error(format!("{}: {error}", t!("common.error")()));
            }
        }
    };

    view! {
        <Container class="mt-6 pb-24" aria_label=t!("settings.title")() as_landmark=true>
            <div class="mx-auto max-w-3xl">
                <Card title_view={t!("settings.title").into_view()}>
                    <div class="space-y-6">
                        <div class="space-y-2">
                            <h3 class="text-lg font-semibold text-slate-900">{t!("settings.device_section_title")}</h3>
                            <p class="text-sm text-gray-600">{t!("settings.device_help_text")}</p>
                        </div>

                        <div class="grid gap-4 sm:grid-cols-2">
                            <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
                                <p class="text-sm font-medium text-gray-700">{t!("settings.device_current_label")}</p>
                                <p class="mt-1 font-mono text-sm text-slate-900 break-all">{move || saved_identifier.get()}</p>
                            </div>
                            <div class="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
                                <p class="text-sm font-medium text-gray-700">{t!("settings.device_detected_label")}</p>
                                <p class="mt-1 text-sm text-slate-900">
                                    {t!("settings.device_platform_info")()
                                        .replace("{platform}", &platform)
                                        .replace("{browser}", &browser)}
                                </p>
                            </div>
                        </div>

                        <div class="space-y-2">
                            <Input
                                value=device_identifier
                                label=t!("settings.device_edit_label")()
                                error=validation_error
                                placeholder="marias-laptop".to_string()
                                aria_label=t!("settings.device_edit_label")()
                            />
                            <p class="text-sm text-gray-500">{t!("settings.device_format_help")}</p>
                        </div>

                        <div class="flex flex-wrap gap-3">
                            <Button on_click=Box::new(handle_save) disabled=!can_save.get()>
                                {t!("settings.device_save")}
                            </Button>
                            <Button
                                on_click=Box::new(handle_reset)
                                variant=ButtonVariant::Secondary
                            >
                                {t!("settings.device_reset")}
                            </Button>
                        </div>
                    </div>
                </Card>
            </div>
        </Container>
    }
}
