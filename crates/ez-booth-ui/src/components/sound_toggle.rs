use crate::t;
use leptos::*;

#[component]
pub fn SoundToggle(enabled: Signal<bool>, on_toggle: Callback<()>) -> impl IntoView {
    view! {
        <button
            type="button"
            class=move || {
                if enabled.get() {
                    "inline-flex items-center rounded-full border border-amber-300 bg-amber-50 px-4 py-2 text-sm font-semibold text-amber-900 shadow-sm transition-colors hover:bg-amber-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                } else {
                    "inline-flex items-center rounded-full border border-slate-200 bg-white/80 px-4 py-2 text-sm font-semibold text-slate-700 shadow-sm backdrop-blur transition-colors hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                }
            }
            aria-label=move || {
                if enabled.get() {
                    t!("checkout.error_sound_disable")()
                } else {
                    t!("checkout.error_sound_enable")()
                }
            }
            aria-pressed=move || if enabled.get() { "true" } else { "false" }
            title=move || {
                if enabled.get() {
                    t!("checkout.error_sound_disable")()
                } else {
                    t!("checkout.error_sound_enable")()
                }
            }
            on:click=move |_| on_toggle.call(())
        >
            <svg
                class="h-5 w-5"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
                aria-hidden="true"
            >
                <path
                    d="M14 5L9 9H5V15H9L14 19V5Z"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                <Show
                    when=move || enabled.get()
                    fallback=move || view! {
                        <path
                            d="M4 4L20 20"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                        />
                    }
                >
                    <>
                        <path
                            d="M17 9.5C18.3333 10.6667 18.3333 13.3333 17 14.5"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                        />
                        <path
                            d="M19.5 7C22.1667 9.33333 22.1667 14.6667 19.5 17"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                        />
                    </>
                </Show>
            </svg>
        </button>
    }
}
