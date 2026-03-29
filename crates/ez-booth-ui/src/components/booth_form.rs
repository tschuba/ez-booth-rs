use crate::components::*;
use crate::error_translator::translate_domain_error;
use crate::formatting::{currency_symbol_for_label, format_decimal_for_input, parse_decimal_input};
use crate::i18n::{use_locale, Locale};
use crate::t;
use chrono::NaiveDate;
use domain::error::DomainError;
use domain::error_code::ValidationError;
use domain::models::booth::{
    Booth, FeeConfig, OmissionRule, VendorIdOmissionRules, VendorIdValidation,
};
use domain::validation::validate_regex_pattern;
use leptos::*;
use rust_decimal::Decimal;
use std::collections::HashSet;

/// Form data for creating/editing a booth
#[derive(Clone, Debug)]
pub struct BoothFormData {
    pub description: String,
    pub date: String,
    pub participation_fee: String,
    pub sales_fee_percent: String,
    pub rounding_step: String,
    pub vendor_validation_type: String, // "unrestricted", "digits_only", or "regex"
    pub vendor_validation_regex: String,
    pub vendor_omission_rules: VendorIdOmissionRules,
}

impl Default for BoothFormData {
    fn default() -> Self {
        // Use English locale for backward compatibility
        Self::default_with_locale(Locale::En)
    }
}

impl BoothFormData {
    /// Create default form data with locale-aware formatting
    pub fn default_with_locale(locale: Locale) -> Self {
        let today = chrono::Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        Self {
            description: String::new(),
            date: date_str,
            participation_fee: format_decimal_for_input(Decimal::ONE, locale, 2),
            sales_fee_percent: format_decimal_for_input(Decimal::from(15), locale, 2),
            rounding_step: format_decimal_for_input(Decimal::new(50, 2), locale, 2),
            vendor_validation_type: "digits_only".to_string(), // Default to digits only
            vendor_validation_regex: String::new(),
            vendor_omission_rules: VendorIdOmissionRules::recommended(),
        }
    }

    /// Create form data from an existing Booth
    pub fn from_booth(booth: &Booth, locale: Locale) -> Self {
        let (validation_type, validation_regex) = match &booth.vendor_id_validation {
            VendorIdValidation::Unrestricted => ("unrestricted".to_string(), String::new()),
            VendorIdValidation::DigitsOnly => ("digits_only".to_string(), String::new()),
            VendorIdValidation::Regex(pattern) => ("regex".to_string(), pattern.clone()),
        };

        Self {
            description: booth.description.clone(),
            date: booth.date.format("%Y-%m-%d").to_string(),
            participation_fee: format_decimal_for_input(booth.fees.participation_fee, locale, 2),
            sales_fee_percent: format_decimal_for_input(booth.fees.sales_fee_percent, locale, 2),
            rounding_step: format_decimal_for_input(booth.fees.rounding_step, locale, 2),
            vendor_validation_type: validation_type,
            vendor_validation_regex: validation_regex,
            vendor_omission_rules: booth.vendor_id_omission_rules.clone(),
        }
    }

    /// Convert form data to domain Booth model
    ///
    /// # Errors
    ///
    /// Returns DomainError if:
    /// - Date string cannot be parsed
    /// - Fee values cannot be parsed to Decimal
    /// - Fee configuration validation fails
    /// - Vendor ID validation configuration is invalid
    pub fn to_booth(&self, _locale: Locale) -> Result<Booth, DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|_| DomainError::Validation(ValidationError::DateInvalid))?;

        // Parse fee values using flexible parsing (accepts both comma and dot)
        let participation_fee = parse_decimal_input(&self.participation_fee)
            .map_err(|_| DomainError::Validation(ValidationError::ParticipationFeeInvalid))?;

        let sales_fee_percent = parse_decimal_input(&self.sales_fee_percent)
            .map_err(|_| DomainError::Validation(ValidationError::SalesFeePercentInvalid))?;

        let rounding_step = parse_decimal_input(&self.rounding_step)
            .map_err(|_| DomainError::Validation(ValidationError::RoundingStepInvalid))?;

        // Create FeeConfig
        let fees = FeeConfig {
            participation_fee,
            sales_fee_percent,
            rounding_step,
        };

        // Parse vendor validation rule
        let vendor_id_validation = match self.vendor_validation_type.as_str() {
            "unrestricted" => VendorIdValidation::Unrestricted,
            "digits_only" => VendorIdValidation::DigitsOnly,
            "regex" => {
                // Validate the regex pattern
                validate_regex_pattern(&self.vendor_validation_regex)?;
                VendorIdValidation::Regex(self.vendor_validation_regex.clone())
            }
            _ => VendorIdValidation::DigitsOnly, // Default fallback
        };

        // Create Booth (this validates the fee ranges)
        let mut booth = Booth::new(self.description.clone(), date, fees)?;
        self.vendor_omission_rules.validate()?;
        booth.vendor_id_validation = vendor_id_validation;
        booth.vendor_id_omission_rules = self.vendor_omission_rules.clone();

        Ok(booth)
    }

    /// Update an existing booth with form data
    ///
    /// # Errors
    ///
    /// Returns DomainError if fee configuration validation fails
    pub fn update_booth(&self, booth: &mut Booth, _locale: Locale) -> Result<(), DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|_| DomainError::Validation(ValidationError::DateInvalid))?;

        // Parse fee values using flexible parsing (accepts both comma and dot)
        let participation_fee = parse_decimal_input(&self.participation_fee)
            .map_err(|_| DomainError::Validation(ValidationError::ParticipationFeeInvalid))?;

        let sales_fee_percent = parse_decimal_input(&self.sales_fee_percent)
            .map_err(|_| DomainError::Validation(ValidationError::SalesFeePercentInvalid))?;

        let rounding_step = parse_decimal_input(&self.rounding_step)
            .map_err(|_| DomainError::Validation(ValidationError::RoundingStepInvalid))?;

        // Create and validate FeeConfig
        let fees = FeeConfig {
            participation_fee,
            sales_fee_percent,
            rounding_step,
        };
        fees.validate_ranges()?;

        // Parse vendor validation rule
        let vendor_id_validation = match self.vendor_validation_type.as_str() {
            "unrestricted" => VendorIdValidation::Unrestricted,
            "digits_only" => VendorIdValidation::DigitsOnly,
            "regex" => {
                // Validate the regex pattern
                validate_regex_pattern(&self.vendor_validation_regex)?;
                VendorIdValidation::Regex(self.vendor_validation_regex.clone())
            }
            _ => VendorIdValidation::DigitsOnly, // Default fallback
        };

        // Update booth fields
        self.vendor_omission_rules.validate()?;
        booth.update_description(self.description.clone());
        booth.date = date;
        booth.update_fees(fees);
        booth.vendor_id_validation = vendor_id_validation;
        booth.vendor_id_omission_rules = self.vendor_omission_rules.clone();

        Ok(())
    }
}

fn parse_u32_input(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok()
}

pub(crate) fn parse_exact_omission_values(input: &str) -> Vec<String> {
    let mut seen = HashSet::new();

    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert((*value).to_string()))
        .map(str::to_string)
        .collect()
}

fn omission_rule_type_label(rule: &OmissionRule) -> String {
    match rule {
        OmissionRule::Exact(_) => t!("booth.vendor_omission_type_exact")(),
        OmissionRule::Wildcard(_) => t!("booth.vendor_omission_type_wildcard")(),
        OmissionRule::Regex(_) => t!("booth.vendor_omission_type_regex")(),
        OmissionRule::Range { .. } | OmissionRule::RangeWithStep { .. } => {
            t!("booth.vendor_omission_type_range")()
        }
    }
}

fn omission_rule_value(rule: &OmissionRule) -> String {
    match rule {
        OmissionRule::Exact(value) => value.clone(),
        OmissionRule::Wildcard(pattern) => pattern.as_str().to_string(),
        OmissionRule::Regex(pattern) => pattern.as_str().to_string(),
        OmissionRule::Range { start, end } => format!("{start}-{end}"),
        OmissionRule::RangeWithStep { start, end, step } => format!("{start}-{end} (step {step})"),
    }
}

fn omission_rule_key(index: usize, rule: &OmissionRule) -> String {
    format!(
        "{index}:{}:{}",
        omission_rule_type_label(rule),
        omission_rule_value(rule)
    )
}

/// Booth form component for creating and editing booths
#[component]
pub fn BoothForm(
    /// Initial form data (for editing)
    #[prop(optional)]
    initial_data: Option<BoothFormData>,
    /// Callback when form is submitted
    on_submit: impl Fn(BoothFormData) + 'static,
    /// Callback when form is cancelled
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let form_data = create_rw_signal(initial_data.unwrap_or_default());

    // Individual field signals for Input components
    let description = create_rw_signal(form_data.get_untracked().description);
    let date = create_rw_signal(form_data.get_untracked().date);
    let participation_fee = create_rw_signal(form_data.get_untracked().participation_fee);
    let sales_fee_percent = create_rw_signal(form_data.get_untracked().sales_fee_percent);
    let rounding_step = create_rw_signal(form_data.get_untracked().rounding_step);
    let vendor_validation_type = create_rw_signal(form_data.get_untracked().vendor_validation_type);
    let vendor_validation_regex =
        create_rw_signal(form_data.get_untracked().vendor_validation_regex);
    let vendor_omission_rules = create_rw_signal(form_data.get_untracked().vendor_omission_rules);

    let new_omission_rule_type = create_rw_signal("exact".to_string());
    let new_omission_value = create_rw_signal(String::new());
    let new_omission_pattern = create_rw_signal(String::new());
    let new_omission_range_start = create_rw_signal(String::new());
    let new_omission_range_end = create_rw_signal(String::new());
    let new_omission_range_step = create_rw_signal(String::new());
    let omission_help_text = Signal::derive(move || match new_omission_rule_type.get().as_str() {
        "exact" => t!("booth.vendor_omission_help_exact")(),
        "wildcard" => t!("booth.vendor_omission_help_wildcard")(),
        "regex" => t!("booth.vendor_omission_help_regex")(),
        "range" => t!("booth.vendor_omission_help_range")(),
        _ => t!("booth.vendor_omission_help")(),
    });

    // Validation errors
    let (description_error, set_description_error) = create_signal(None::<String>);
    let (date_error, set_date_error) = create_signal(None::<String>);
    let (participation_fee_error, set_participation_fee_error) = create_signal(None::<String>);
    let (sales_fee_percent_error, set_sales_fee_percent_error) = create_signal(None::<String>);
    let (rounding_step_error, set_rounding_step_error) = create_signal(None::<String>);
    let (vendor_validation_regex_error, set_vendor_validation_regex_error) =
        create_signal(None::<String>);
    let (vendor_omission_error, set_vendor_omission_error) = create_signal(None::<String>);

    let locale = use_locale();

    // Localized validation messages
    let description_required_msg = t!("booth.form_errors.description_required");
    let description_length_msg = t!("booth.form_errors.description_length");
    let date_required_msg = t!("booth.form_errors.date_required");
    let participation_fee_required_msg = move || {
        let locale_val = locale.get();
        let currency = currency_symbol_for_label(locale_val);
        t!("booth.form_errors.participation_fee_required")().replace("{currency}", currency)
    };
    let sales_fee_required_msg = t!("booth.form_errors.sales_fee_required");
    let rounding_step_required_msg = t!("booth.form_errors.rounding_step_required");
    let cannot_be_negative_msg = t!("booth.form_errors.cannot_be_negative");
    let cannot_exceed_100_msg = t!("booth.form_errors.cannot_exceed_100");
    let invalid_number_format_msg = t!("booth.form_errors.invalid_number_format");
    let max_two_decimals_msg = t!("booth.form_errors.max_two_decimals");

    let validate_and_submit = move || {
        // Clear previous errors
        set_description_error.set(None);
        set_date_error.set(None);
        set_participation_fee_error.set(None);
        set_sales_fee_percent_error.set(None);
        set_rounding_step_error.set(None);
        set_vendor_validation_regex_error.set(None);
        set_vendor_omission_error.set(None);

        let mut has_errors = false;

        // Validate description
        let desc = description.get();
        if desc.trim().is_empty() {
            set_description_error.set(Some(description_required_msg()));
            has_errors = true;
        } else if desc.chars().count() > 200 {
            set_description_error.set(Some(description_length_msg()));
            has_errors = true;
        }

        // Validate date
        let date_str = date.get();
        if date_str.trim().is_empty() {
            set_date_error.set(Some(date_required_msg()));
            has_errors = true;
        }

        // Validate participation fee using flexible parsing (accepts both comma and dot)
        let part_fee = participation_fee.get();
        if part_fee.trim().is_empty() {
            set_participation_fee_error.set(Some(participation_fee_required_msg()));
            has_errors = true;
        } else {
            match parse_decimal_input(&part_fee) {
                Ok(val) => {
                    if val < Decimal::ZERO {
                        set_participation_fee_error.set(Some(cannot_be_negative_msg()));
                        has_errors = true;
                    }
                }
                Err(e) => {
                    let message = if e == "Maximum 2 decimal places allowed" {
                        max_two_decimals_msg()
                    } else {
                        invalid_number_format_msg()
                    };
                    set_participation_fee_error.set(Some(message));
                    has_errors = true;
                }
            }
        }

        // Validate revenue share percent using flexible parsing (accepts both comma and dot)
        let sales_pct = sales_fee_percent.get();
        if sales_pct.trim().is_empty() {
            set_sales_fee_percent_error.set(Some(sales_fee_required_msg()));
            has_errors = true;
        } else {
            match parse_decimal_input(&sales_pct) {
                Ok(val) => {
                    if val < Decimal::ZERO {
                        set_sales_fee_percent_error.set(Some(cannot_be_negative_msg()));
                        has_errors = true;
                    } else if val > Decimal::from(100) {
                        set_sales_fee_percent_error.set(Some(cannot_exceed_100_msg()));
                        has_errors = true;
                    }
                }
                Err(e) => {
                    let message = if e == "Maximum 2 decimal places allowed" {
                        max_two_decimals_msg()
                    } else {
                        invalid_number_format_msg()
                    };
                    set_sales_fee_percent_error.set(Some(message));
                    has_errors = true;
                }
            }
        }

        // Validate rounding step using flexible parsing (accepts both comma and dot)
        let rounding = rounding_step.get();
        if rounding.trim().is_empty() {
            set_rounding_step_error.set(Some(rounding_step_required_msg()));
            has_errors = true;
        } else {
            match parse_decimal_input(&rounding) {
                Ok(val) => {
                    if val < Decimal::ZERO {
                        set_rounding_step_error.set(Some(cannot_be_negative_msg()));
                        has_errors = true;
                    }
                }
                Err(e) => {
                    let message = if e == "Maximum 2 decimal places allowed" {
                        max_two_decimals_msg()
                    } else {
                        invalid_number_format_msg()
                    };
                    set_rounding_step_error.set(Some(message));
                    has_errors = true;
                }
            }
        }

        // Validate vendor ID validation regex pattern if type is "regex"
        let validation_type = vendor_validation_type.get();
        if validation_type == "regex" {
            let regex_pattern = vendor_validation_regex.get();
            if regex_pattern.trim().is_empty() {
                set_vendor_validation_regex_error
                    .set(Some(t!("booth.form_errors.regex_pattern_required")()));
                has_errors = true;
            } else {
                // Validate the regex pattern
                if let Err(e) = validate_regex_pattern(&regex_pattern) {
                    set_vendor_validation_regex_error.set(Some(translate_domain_error(&e)));
                    has_errors = true;
                }
            }
        }

        if let Err(err) = vendor_omission_rules.get().validate() {
            set_vendor_omission_error.set(Some(translate_domain_error(&err)));
            has_errors = true;
        }

        if !has_errors {
            let data = BoothFormData {
                description: description.get(),
                date: date.get(),
                participation_fee: participation_fee.get(),
                sales_fee_percent: sales_fee_percent.get(),
                rounding_step: rounding_step.get(),
                vendor_validation_type: vendor_validation_type.get(),
                vendor_validation_regex: vendor_validation_regex.get(),
                vendor_omission_rules: vendor_omission_rules.get(),
            };
            on_submit(data);
        }
    };

    view! {
        <form class="space-y-6" on:submit=|e| e.prevent_default()>
            // Description
            <div>
                <Input
                    value=description
                    label=t!("booth.description_label")()
                    placeholder=t!("booth.description_placeholder")()
                    required=true
                    error=description_error
                />
            </div>

            // Date
            <div>
                <Input
                    value=date
                    input_type=crate::components::InputType::Date
                    label=t!("booth.date_label")()
                    required=true
                    error=date_error
                />
            </div>

            // Fee Configuration Section
            <div class="border-t pt-6">
                <h3 class="text-lg font-semibold mb-4">{t!("booth.fee_configuration_title")()}</h3>

                <div class="space-y-4">
                    <div class="grid gap-4 md:grid-cols-2">
                        // Participation Fee
                        <NumberInput
                            value=participation_fee
                            label={
                                let locale_val = locale.get();
                                let currency = currency_symbol_for_label(locale_val);
                                t!("booth.participation_fee")().replace("{currency}", currency)
                            }
                            placeholder=t!("common.placeholders.decimal_zero")()
                            required=true
                            error=participation_fee_error
                        />

                        // Sales Fee Percent
                        <NumberInput
                            value=sales_fee_percent
                            label=t!("booth.sales_fee_percent")()
                            placeholder=t!("common.placeholders.decimal_zero")()
                            required=true
                            error=sales_fee_percent_error
                        />
                    </div>

                    // Rounding Step
                    <div>
                        <NumberInput
                            value=rounding_step
                            label=t!("booth.rounding_step")()
                            placeholder=t!("common.placeholders.decimal_half")()
                            required=true
                            error=rounding_step_error
                        />
                        <p class="mt-1 text-sm text-gray-600">
                            {t!("booth.rounding_step_help")()}
                        </p>
                    </div>
                </div>
            </div>

            // Vendor ID Validation Section
            <div class="border-t pt-6">
                <h3 class="text-lg font-semibold mb-4">{t!("booth.vendor_validation_title")()}</h3>
                <p class="text-sm text-gray-600 mb-4">{t!("booth.vendor_validation_description")()}</p>

                <div class="space-y-4">
                    // Validation Type Select
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            {t!("booth.vendor_validation_type_label")()}
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                vendor_validation_type.set(event_target_value(&ev));
                            }
                            prop:value=vendor_validation_type
                        >
                            <option value="unrestricted" selected=move || vendor_validation_type.get() == "unrestricted">{t!("booth.vendor_validation_unrestricted")()}</option>
                            <option value="digits_only" selected=move || vendor_validation_type.get() == "digits_only">{t!("booth.vendor_validation_digits_only")()}</option>
                            <option value="regex" selected=move || vendor_validation_type.get() == "regex">{t!("booth.vendor_validation_regex")()}</option>
                        </select>
                    </div>

                    // Regex Pattern Input (only shown when type is "regex")
                    {move || {
                        if vendor_validation_type.get() == "regex" {
                            view! {
                                <div class="space-y-2">
                                    <Input
                                        value=vendor_validation_regex
                                        label=t!("booth.vendor_validation_regex_pattern")()
                                        placeholder=r"^V\d{3}$".to_string()
                                        required=true
                                        error=vendor_validation_regex_error
                                    />
                                    <p class="text-sm text-gray-600">
                                        {t!("booth.vendor_validation_regex_help")()}
                                    </p>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}
                </div>
            </div>

            <div class="border-t pt-6">
                <h3 class="text-lg font-semibold mb-4">{t!("booth.vendor_omission_title")()}</h3>
                <p class="text-sm text-gray-600 mb-4">{t!("booth.vendor_omission_description")()}</p>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            {t!("booth.vendor_omission_type_label")()}
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                new_omission_rule_type.set(event_target_value(&ev));
                                set_vendor_omission_error.set(None);
                            }
                            prop:value=new_omission_rule_type
                        >
                            <option value="exact">{t!("booth.vendor_omission_type_exact")()}</option>
                            <option value="wildcard">{t!("booth.vendor_omission_type_wildcard")()}</option>
                            <option value="regex">{t!("booth.vendor_omission_type_regex")()}</option>
                            <option value="range">{t!("booth.vendor_omission_type_range")()}</option>
                        </select>
                    </div>

                    {move || {
                        match new_omission_rule_type.get().as_str() {
                            "exact" => view! {
                                <Input
                                    value=new_omission_value
                                    label=t!("booth.vendor_omission_value_label")()
                                    placeholder=t!("booth.vendor_omission_value_placeholder")()
                                />
                            }.into_view(),
                            "wildcard" => view! {
                                <Input
                                    value=new_omission_pattern
                                    label=t!("booth.vendor_omission_pattern_label")()
                                    placeholder=t!("booth.vendor_omission_pattern_wildcard_placeholder")()
                                />
                            }.into_view(),
                            "regex" => view! {
                                <Input
                                    value=new_omission_pattern
                                    label=t!("booth.vendor_omission_pattern_label")()
                                    placeholder=t!("booth.vendor_omission_pattern_regex_placeholder")()
                                />
                            }.into_view(),
                            "range" => view! {
                                <div class="grid grid-cols-3 gap-4">
                                    <Input
                                        value=new_omission_range_start
                                        label=t!("booth.vendor_omission_start_label")()
                                        placeholder="56".to_string()
                                    />
                                    <Input
                                        value=new_omission_range_end
                                        label=t!("booth.vendor_omission_end_label")()
                                        placeholder="182".to_string()
                                    />
                                    <Input
                                        value=new_omission_range_step
                                        label=t!("booth.vendor_omission_step_label")()
                                        placeholder="6".to_string()
                                    />
                                </div>
                            }.into_view(),
                            _ => view! {
                                <Input
                                    value=new_omission_value
                                    label=t!("booth.vendor_omission_value_label")()
                                    placeholder=t!("booth.vendor_omission_value_placeholder")()
                                />
                            }.into_view(),
                        }
                    }}

                    <div class="flex items-center justify-between gap-3">
                        <p class="text-sm text-gray-600">{move || omission_help_text.get()}</p>
                        <Button
                            on_click=Box::new(move || {
                                set_vendor_omission_error.set(None);

                                let next_rules = match new_omission_rule_type.get().as_str() {
                                    "exact" => {
                                        let values = parse_exact_omission_values(&new_omission_value.get());
                                        if values.is_empty() {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_value_required")(),
                                            ));
                                            return;
                                        }
                                        values
                                            .into_iter()
                                            .map(OmissionRule::Exact)
                                            .collect::<Vec<_>>()
                                    }
                                    "wildcard" => {
                                        let pattern = new_omission_pattern.get().trim().to_string();
                                        if pattern.is_empty() {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_pattern_required")(),
                                            ));
                                            return;
                                        }
                                        if pattern.len() > 100 {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_pattern_too_long")(),
                                            ));
                                            return;
                                        }
                                        vec![OmissionRule::Wildcard(pattern.into())]
                                    }
                                    "regex" => {
                                        let pattern = new_omission_pattern.get().trim().to_string();
                                        if let Err(err) = validate_regex_pattern(&pattern) {
                                            set_vendor_omission_error
                                                .set(Some(translate_domain_error(&err)));
                                            return;
                                        }
                                        vec![OmissionRule::Regex(pattern.into())]
                                    }
                                    "range" => {
                                        let Some(start) = parse_u32_input(&new_omission_range_start.get()) else {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_number_required")(),
                                            ));
                                            return;
                                        };
                                        let Some(end) = parse_u32_input(&new_omission_range_end.get()) else {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_number_required")(),
                                            ));
                                            return;
                                        };
                                        if start > end {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_range_invalid")(),
                                            ));
                                            return;
                                        }
                                        let step_input = new_omission_range_step.get();
                                        let step_input = step_input.trim();

                                        if step_input.is_empty() {
                                            vec![OmissionRule::Range { start, end }]
                                        } else {
                                            let Some(step) = parse_u32_input(step_input) else {
                                                set_vendor_omission_error.set(Some(
                                                    t!("booth.form_errors.vendor_omission_step_required")(),
                                                ));
                                                return;
                                            };
                                            if step == 0 {
                                                set_vendor_omission_error.set(Some(
                                                    t!("booth.form_errors.vendor_omission_step_invalid")(),
                                                ));
                                                return;
                                            }
                                            vec![OmissionRule::RangeWithStep { start, end, step }]
                                        }
                                    }
                                    _ => {
                                        let values = parse_exact_omission_values(&new_omission_value.get());
                                        if values.is_empty() {
                                            set_vendor_omission_error.set(Some(
                                                t!("booth.form_errors.vendor_omission_value_required")(),
                                            ));
                                            return;
                                        }
                                        values
                                            .into_iter()
                                            .map(OmissionRule::Exact)
                                            .collect::<Vec<_>>()
                                    }
                                };

                                let mut updated_rules = vendor_omission_rules.get();
                                updated_rules.rules.extend(next_rules);

                                if let Err(err) = updated_rules.validate() {
                                    set_vendor_omission_error.set(Some(translate_domain_error(&err)));
                                    return;
                                }

                                vendor_omission_rules.set(updated_rules);
                                new_omission_value.set(String::new());
                                new_omission_pattern.set(String::new());
                                new_omission_range_start.set(String::new());
                                new_omission_range_end.set(String::new());
                                new_omission_range_step.set(String::new());
                            })
                            variant=crate::components::ButtonVariant::Secondary
                        >
                            {t!("booth.vendor_omission_add_rule")()}
                        </Button>
                    </div>

                    <Show when=move || vendor_omission_error.get().is_some()>
                        <p class="text-sm text-red-600">{move || vendor_omission_error.get().unwrap_or_default()}</p>
                    </Show>

                    <div class="space-y-2">
                        <Show
                            when=move || !vendor_omission_rules.get().rules.is_empty()
                            fallback=move || view! {
                                <p class="text-sm text-gray-500">{t!("booth.vendor_omission_empty")()}</p>
                            }
                        >
                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                                <For
                                    each=move || {
                                        vendor_omission_rules
                                            .get()
                                            .rules
                                            .into_iter()
                                            .enumerate()
                                            .map(|(index, rule)| {
                                                let key = omission_rule_key(index, &rule);
                                                (index, key, rule)
                                            })
                                    }
                                    key=|(_, key, _)| key.clone()
                                    children=move |(_, rule_key, rule)| {
                                        let type_label = omission_rule_type_label(&rule);
                                        let value_label = omission_rule_value(&rule);

                                        view! {
                                            <div class="flex h-full flex-col justify-between gap-3 rounded-xl border border-gray-200 bg-white p-4 shadow-sm transition-shadow hover:shadow-md">
                                                <div class="min-h-[4.5rem] space-y-1">
                                                    <p
                                                        class="text-xs font-semibold uppercase tracking-[0.12em] text-gray-500"
                                                    >
                                                        {type_label}
                                                    </p>
                                                    <p
                                                        class="text-sm font-medium text-gray-800"
                                                        title=value_label.clone()
                                                        style="display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; overflow: hidden;"
                                                    >
                                                        {value_label}
                                                    </p>
                                                </div>

                                                <button
                                                    type="button"
                                                    class="w-full rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm font-medium text-red-700 transition-colors hover:border-red-300 hover:bg-red-100 hover:text-red-800"
                                                    on:click=move |_| {
                                                        vendor_omission_rules.update(|rules| {
                                                            if let Some(index) = rules
                                                                .rules
                                                                .iter()
                                                                .enumerate()
                                                                .find_map(|(index, existing_rule)| {
                                                                    (omission_rule_key(index, existing_rule) == rule_key)
                                                                        .then_some(index)
                                                                })
                                                            {
                                                                rules.rules.remove(index);
                                                            }
                                                        });
                                                    }
                                                >
                                                    {t!("common.delete")()}
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </Show>
                    </div>
                </div>
            </div>

            // Form Actions
            <div class="flex justify-end gap-3 pt-4 border-t">
                <Button
                    on_click=Box::new(move || on_cancel())
                    variant=crate::components::ButtonVariant::Secondary
                >
                    {t!("common.cancel")()}
                </Button>
                <Button
                    on_click=Box::new(validate_and_submit)
                    variant=crate::components::ButtonVariant::Primary
                >
                    {t!("booth.save_button")()}
                </Button>
            </div>
        </form>
    }
}
