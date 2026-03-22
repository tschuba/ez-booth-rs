use leptos::*;

/// Card component for consistent layout
#[component]
pub fn Card(
    /// Card content
    children: Children,
    /// Card title
    #[prop(optional)]
    title: Option<&'static str>,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
) -> impl IntoView {
    let additional_classes = class.unwrap_or("");
    let card_classes = format!("bg-white rounded-lg shadow-md p-6 {}", additional_classes);

    view! {
        <div class=card_classes>
            {title.map(|t| view! {
                <h2 class="text-xl font-semibold mb-4">{t}</h2>
            })}
            {children()}
        </div>
    }
}

/// Container component for centering content
#[component]
pub fn Container(
    /// Container content
    children: Children,
    /// Maximum width (default: "max-w-7xl")
    #[prop(optional)]
    max_width: Option<&'static str>,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
) -> impl IntoView {
    let max_width = max_width.unwrap_or("max-w-7xl");
    let additional_classes = class.unwrap_or("");
    let container_classes = format!(
        "mx-auto px-4 sm:px-6 lg:px-8 {} {}",
        max_width, additional_classes
    );

    view! {
        <div class=container_classes>
            {children()}
        </div>
    }
}
