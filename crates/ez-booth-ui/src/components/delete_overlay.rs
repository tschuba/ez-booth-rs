use leptos::*;

/// Shared red confirmation overlay used for two-click destructive actions.
#[component]
pub fn DeleteOverlay(
    #[prop(into)] prompt: String,
    #[prop(into, optional)] aria_label: Option<String>,
    #[prop(into)] on_click: Callback<ev::MouseEvent>,
) -> impl IntoView {
    let aria_prompt = aria_label.unwrap_or_else(|| prompt.clone());
    view! {
        <div
            class="absolute inset-0 rounded-lg cursor-pointer z-10 transition-all pointer-events-auto"
            style="background: rgba(220, 38, 38, 0.7); backdrop-filter: blur(2px);"
            role="alertdialog"
            aria-label=aria_prompt
            on:click=move |ev| {
                ev.stop_propagation();
                on_click.call(ev);
            }
        >
            <div class="flex items-center justify-center gap-3 h-full">
                <svg
                    class="w-8 h-8 text-white flex-shrink-0"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                >
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
                    <path d="M10 11v6" />
                    <path d="M14 11v6" />
                    <path d="M9 6V4a2 2 0 012-2h2a2 2 0 012 2v2" />
                </svg>

                <p class="text-white text-base font-semibold text-center">
                    {prompt}
                </p>
            </div>
        </div>
    }
}
