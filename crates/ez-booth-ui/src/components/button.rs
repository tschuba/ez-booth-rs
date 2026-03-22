use leptos::*;

/// Button variant styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

impl ButtonVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
            ButtonVariant::Secondary => "bg-gray-200 hover:bg-gray-300 text-gray-900",
            ButtonVariant::Danger => "bg-red-600 hover:bg-red-700 text-white",
            ButtonVariant::Ghost => "bg-transparent hover:bg-gray-100 text-gray-700",
        }
    }
}

/// Button size
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonSize::Small => "px-3 py-1.5 text-sm",
            ButtonSize::Medium => "px-4 py-2 text-base",
            ButtonSize::Large => "px-6 py-3 text-lg",
        }
    }
}

/// Reusable button component
#[component]
pub fn Button(
    /// Button content
    children: Children,
    /// Click handler
    #[prop(optional)]
    on_click: Option<Box<dyn Fn() + 'static>>,
    /// Button variant (default: Primary)
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    /// Button size (default: Medium)
    #[prop(optional)]
    size: Option<ButtonSize>,
    /// Whether button is disabled
    #[prop(optional)]
    disabled: Option<bool>,
    /// Whether button should take full width
    #[prop(optional)]
    full_width: Option<bool>,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let size = size.unwrap_or(ButtonSize::Medium);
    let disabled = disabled.unwrap_or(false);
    let full_width = full_width.unwrap_or(false);

    let base_classes = "inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed";
    let width_class = if full_width { "w-full" } else { "" };
    let additional_classes = class.unwrap_or("");

    let class_list = format!(
        "{} {} {} {} {}",
        base_classes,
        variant.classes(),
        size.classes(),
        width_class,
        additional_classes
    );

    view! {
        <button
            class=class_list
            disabled=disabled
            on:click=move |_| {
                if let Some(handler) = &on_click {
                    handler();
                }
            }
        >
            {children()}
        </button>
    }
}
