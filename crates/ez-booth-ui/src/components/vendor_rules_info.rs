use crate::components::{Modal, ModalSize};
use crate::i18n::translate_with_params;
use crate::t;
use domain::models::booth::{OmissionRule, VendorIdOmissionRules, VendorIdValidation};
use leptos::*;
use std::collections::HashMap;

fn format_validation_rule(rule: &VendorIdValidation) -> String {
    match rule {
        VendorIdValidation::Unrestricted => t!("checkout.rules_validation_unrestricted")(),
        VendorIdValidation::DigitsOnly => t!("checkout.rules_validation_digits_only")(),
        VendorIdValidation::Regex(pattern) => translate_with_params(
            "checkout.rules_validation_regex",
            HashMap::from([("pattern", pattern.clone())]),
        ),
    }
}

fn format_omission_rule(rule: &OmissionRule) -> String {
    match rule {
        OmissionRule::Exact(value) => {
            t!("booth.vendor_omission_display_exact")().replace("{value}", value)
        }
        OmissionRule::Wildcard(pattern) => {
            t!("booth.vendor_omission_display_wildcard")().replace("{pattern}", pattern)
        }
        OmissionRule::Regex(pattern) => {
            t!("booth.vendor_omission_display_regex")().replace("{pattern}", pattern)
        }
        OmissionRule::Range { start, end } => t!("booth.vendor_omission_display_range")()
            .replace("{range}", &format!("{start}-{end}")),
        OmissionRule::RangeWithStep { start, end, step } => {
            t!("booth.vendor_omission_display_range_step")()
                .replace("{range}", &format!("{start}-{end}"))
                .replace("{step}", &step.to_string())
        }
    }
}

#[component]
pub fn VendorRulesInfoModal(
    #[prop(into)] show: Signal<bool>,
    on_close: impl Fn() + 'static + Clone,
    #[prop(into)] vendor_validation_rule: Signal<Option<VendorIdValidation>>,
    #[prop(into)] vendor_omission_rules: Signal<VendorIdOmissionRules>,
) -> impl IntoView {
    let omission_summaries = Signal::derive(move || {
        vendor_omission_rules
            .get()
            .rules
            .into_iter()
            .map(|rule| format_omission_rule(&rule))
            .collect::<Vec<_>>()
    });

    view! {
        <Modal
            show=show
            on_close=on_close
            title=Signal::derive(move || t!("checkout.rules_modal_title")())
            size=ModalSize::Medium
        >
            <div class="space-y-5">
                <div>
                    <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
                        {t!("checkout.rules_validation_title")()}
                    </h3>
                    <p class="mt-2 text-sm text-slate-700">
                        {move || vendor_validation_rule.get().map(|rule| format_validation_rule(&rule)).unwrap_or_else(|| t!("checkout.rules_validation_unrestricted")())}
                    </p>
                </div>

                <div class="border-t border-slate-200 pt-4">
                    <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
                        {t!("checkout.rules_omission_title")()}
                    </h3>
                    <Show
                        when=move || !omission_summaries.get().is_empty()
                        fallback=move || view! {
                            <p class="mt-2 text-sm text-slate-700">{t!("checkout.rules_omission_empty")()}</p>
                        }
                    >
                        <ul class="mt-2 space-y-2 text-sm text-slate-700">
                            <For
                                each=move || omission_summaries.get().into_iter().enumerate()
                                key=|(index, _)| *index
                                children=move |(_, summary)| {
                                    view! {
                                        <li class="rounded-md bg-slate-50 px-3 py-2">{summary}</li>
                                    }
                                }
                            />
                        </ul>
                    </Show>
                </div>

                <div class="border-t border-slate-200 pt-4">
                    <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
                        {t!("checkout.rules_amount_title")()}
                    </h3>
                    <p class="mt-2 text-sm text-slate-700">{t!("checkout.rules_amount_summary")()}</p>
                </div>
            </div>
        </Modal>
    }
}
