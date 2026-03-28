use crate::formatting::{decimal_separator, format_decimal_for_input};
use crate::i18n::Locale;
use crate::t;
use leptos::*;
use rust_decimal::Decimal;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardKey {
    Digit(u8),
    Decimal,
    DoubleZero,
    Backspace,
    Clear,
    QuickAmount(Decimal),
}

#[component]
pub fn OnScreenKeyboard(
    #[prop(into)] is_visible: Signal<bool>,
    on_key: Callback<KeyboardKey>,
    quick_amounts: Vec<Decimal>,
    locale: Locale,
) -> impl IntoView {
    let decimal = decimal_separator(locale).to_string();
    let quick_amounts_label = t!("checkout.keyboard_quick_amounts");
    let backspace_label = t!("checkout.keyboard_backspace");
    let decimal_label = t!("checkout.keyboard_decimal");
    let double_zero_label = t!("checkout.keyboard_double_zero");
    let clear_label = t!("common.clear");
    let digit_rows = Rc::new(vec![vec![7_u8, 8, 9], vec![4_u8, 5, 6], vec![1_u8, 2, 3]]);
    let quick_amounts_source = Rc::new(quick_amounts);
    view! {
        <Show when=move || is_visible.get()>
            <div class="rounded-2xl border border-slate-200 bg-gradient-to-b from-slate-50 to-white p-4 shadow-xl ring-1 ring-slate-200/60">
                <div class="space-y-4">
                    <div class="space-y-2">
                        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">
                            {quick_amounts_label}
                        </p>
                        <div class="grid grid-cols-2 gap-2 sm:grid-cols-5">
                            {let quick_amounts_source = quick_amounts_source.clone(); move || {
                                quick_amounts_source
                                    .iter()
                                    .copied()
                                    .map(|amount| {
                                        let label = format_decimal_for_input(amount, locale, 2);
                                        view! {
                                            <button
                                                type="button"
                                                class="min-h-12 rounded-xl border border-teal-200 bg-teal-50 px-3 py-2 text-sm font-semibold text-teal-900 transition hover:bg-teal-100 focus:outline-none focus:ring-2 focus:ring-teal-500"
                                                aria-label=label.clone()
                                                on:click={
                                                    let on_key = on_key.clone();
                                                    move |_| on_key.call(KeyboardKey::QuickAmount(amount))
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </div>

                    <div class="grid grid-cols-[repeat(3,minmax(0,1fr))_minmax(0,1.15fr)] gap-2">
                        {let digit_rows = digit_rows.clone(); move || {
                            digit_rows
                                .iter()
                                .cloned()
                                .flat_map(|row| row.into_iter().map(Some).chain(std::iter::once(None)))
                                .enumerate()
                                .map(|(index, digit)| match digit {
                                    Some(value) => {
                                        let label = value.to_string();
                                        view! {
                                            <button
                                                type="button"
                                                class="min-h-14 rounded-xl border border-slate-200 bg-white text-lg font-semibold text-slate-900 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                aria-label=label.clone()
                                                on:click={
                                                    let on_key = on_key.clone();
                                                    move |_| on_key.call(KeyboardKey::Digit(value))
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                        .into_view()
                                    }
                                    None if index == 3 => view! {
                                        <button
                                            type="button"
                                            class="row-span-2 min-h-14 rounded-xl border border-amber-200 bg-amber-50 px-3 py-2 text-sm font-semibold text-amber-900 transition hover:bg-amber-100 focus:outline-none focus:ring-2 focus:ring-amber-500"
                                            aria-label=backspace_label
                                            on:click={
                                                let on_key = on_key.clone();
                                                move |_| on_key.call(KeyboardKey::Backspace)
                                            }
                                        >
                                            "⌫"
                                        </button>
                                    }
                                    .into_view(),
                                    None => view! {
                                        <button
                                            type="button"
                                            class="row-span-2 min-h-14 rounded-xl border border-rose-200 bg-rose-50 px-3 py-2 text-sm font-semibold text-rose-900 transition hover:bg-rose-100 focus:outline-none focus:ring-2 focus:ring-rose-500"
                                            aria-label=clear_label
                                            on:click={
                                                let on_key = on_key.clone();
                                                move |_| on_key.call(KeyboardKey::Clear)
                                            }
                                        >
                                            {clear_label}
                                        </button>
                                    }
                                    .into_view(),
                                })
                                .collect_view()
                        }}

                        <button
                            type="button"
                            class="min-h-14 rounded-xl border border-slate-200 bg-white text-lg font-semibold text-slate-900 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            aria-label="0"
                            on:click={
                                let on_key = on_key.clone();
                                move |_| on_key.call(KeyboardKey::Digit(0))
                            }
                        >
                            "0"
                        </button>
                        <button
                            type="button"
                            class="min-h-14 rounded-xl border border-slate-200 bg-white text-lg font-semibold text-slate-900 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            aria-label=decimal_label
                            on:click={
                                let on_key = on_key.clone();
                                move |_| on_key.call(KeyboardKey::Decimal)
                            }
                        >
                            {decimal.clone()}
                        </button>
                        <button
                            type="button"
                            class="min-h-14 rounded-xl border border-slate-200 bg-white text-lg font-semibold text-slate-900 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            aria-label=double_zero_label
                            on:click={
                                let on_key = on_key.clone();
                                move |_| on_key.call(KeyboardKey::DoubleZero)
                            }
                        >
                            "00"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
