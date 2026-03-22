use leptos::*;

/// Input type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Text,
    Number,
    Email,
    Password,
    Date,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Number => "number",
            InputType::Email => "email",
            InputType::Password => "password",
            InputType::Date => "date",
        }
    }
}

/// Text input component
#[component]
pub fn Input(
    /// Input value signal
    value: RwSignal<String>,
    /// Input type
    #[prop(optional)]
    input_type: Option<InputType>,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<&'static str>,
    /// Label text
    #[prop(optional)]
    label: Option<&'static str>,
    /// Whether the input is disabled
    #[prop(optional)]
    disabled: Option<bool>,
    /// Whether the input is required
    #[prop(optional)]
    required: Option<bool>,
    /// Error message to display
    #[prop(optional)]
    error: Option<String>,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
    /// ARIA label for accessibility
    #[prop(optional)]
    aria_label: Option<String>,
) -> impl IntoView {
    let input_type = input_type.unwrap_or(InputType::Text);
    let disabled = disabled.unwrap_or(false);
    let required = required.unwrap_or(false);

    let input_classes = if error.is_some() {
        "w-full px-3 py-2 border border-red-500 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500"
    } else {
        "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
    };

    let additional_classes = class.unwrap_or("");
    let combined_classes = format!("{} {}", input_classes, additional_classes);
    
    // Generate unique ID for error message association
    let error_id = error.as_ref().map(|_| format!("input-error-{}", value.get_untracked().len()));

    view! {
        <div class="w-full">
            {label.map(|l| view! {
                <label class="block text-sm font-medium text-gray-700 mb-1">
                    {l}
                    {if required { " *" } else { "" }}
                </label>
            })}
            <input
                type=input_type.as_str()
                class=combined_classes
                placeholder=placeholder.unwrap_or("")
                disabled=disabled
                required=required
                aria-label=aria_label
                aria-invalid=if error.is_some() { Some("true") } else { None }
                aria-describedby=error_id.clone()
                aria-required=if required { Some("true") } else { None }
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
            {error.map(|err| view! {
                <p class="mt-1 text-sm text-red-600" id=error_id role="alert">{err}</p>
            })}
        </div>
    }
}

/// Number input component (specialized for decimals)
#[component]
pub fn NumberInput(
    /// Input value signal (as string for editing)
    value: RwSignal<String>,
    /// Label text
    #[prop(optional)]
    label: Option<&'static str>,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<&'static str>,
    /// Minimum value
    #[prop(optional)]
    min: Option<f64>,
    /// Maximum value
    #[prop(optional)]
    max: Option<f64>,
    /// Step value
    #[prop(optional)]
    step: Option<f64>,
    /// Whether the input is disabled
    #[prop(optional)]
    disabled: Option<bool>,
    /// Whether the input is required
    #[prop(optional)]
    required: Option<bool>,
    /// Error message to display
    #[prop(optional)]
    error: Option<String>,
    /// ARIA label for accessibility
    #[prop(optional)]
    aria_label: Option<String>,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let required = required.unwrap_or(false);

    let input_classes = if error.is_some() {
        "w-full px-3 py-2 border border-red-500 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500"
    } else {
        "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
    };
    
    // Generate unique ID for error message association
    let error_id = error.as_ref().map(|_| format!("number-input-error-{}", value.get_untracked().len()));

    view! {
        <div class="w-full">
            {label.map(|l| view! {
                <label class="block text-sm font-medium text-gray-700 mb-1">
                    {l}
                    {if required { " *" } else { "" }}
                </label>
            })}
            <input
                type="number"
                class=input_classes
                placeholder=placeholder.unwrap_or("")
                disabled=disabled
                required=required
                min=min.map(|m| m.to_string()).unwrap_or_default()
                max=max.map(|m| m.to_string()).unwrap_or_default()
                step=step.map(|s| s.to_string()).unwrap_or_else(|| "0.01".to_string())
                aria-label=aria_label
                aria-invalid=if error.is_some() { Some("true") } else { None }
                aria-describedby=error_id.clone()
                aria-required=if required { Some("true") } else { None }
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
            {error.map(|err| view! {
                <p class="mt-1 text-sm text-red-600" id=error_id role="alert">{err}</p>
            })}
        </div>
    }
}
