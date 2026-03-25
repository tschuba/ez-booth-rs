use crate::components::*;
use crate::formatting::{currency_symbol_for_label, format_decimal_for_input, parse_decimal_input};
use crate::i18n::{use_locale, Locale};
use crate::t;
use chrono::NaiveDate;
use domain::error::DomainError;
use domain::models::booth::{Booth, FeeConfig};
use leptos::*;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Form data for creating/editing a booth
#[derive(Clone, Debug)]
pub struct BoothFormData {
    pub description: String,
    pub date: String,
    pub participation_fee: String,
    pub sales_fee_percent: String,
    pub rounding_step: String,
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
            rounding_step: format_decimal_for_input(Decimal::from_str("0.50").unwrap(), locale, 2),
        }
    }

    /// Create form data from an existing Booth
    pub fn from_booth(booth: &Booth, locale: Locale) -> Self {
        Self {
            description: booth.description.clone(),
            date: booth.date.format("%Y-%m-%d").to_string(),
            participation_fee: format_decimal_for_input(booth.fees.participation_fee, locale, 2),
            sales_fee_percent: format_decimal_for_input(booth.fees.sales_fee_percent, locale, 2),
            rounding_step: format_decimal_for_input(booth.fees.rounding_step, locale, 2),
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
    pub fn to_booth(&self, locale: Locale) -> Result<Booth, DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| DomainError::Validation(format!("Invalid date format: {}", e)))?;

        // Parse fee values using flexible parsing (accepts both comma and dot)
        let participation_fee = parse_decimal_input(&self.participation_fee)
            .map_err(|e| DomainError::Validation(format!("Invalid participation fee: {}", e)))?;

        let sales_fee_percent = parse_decimal_input(&self.sales_fee_percent)
            .map_err(|e| DomainError::Validation(format!("Invalid sales fee percent: {}", e)))?;

        let rounding_step = parse_decimal_input(&self.rounding_step)
            .map_err(|e| DomainError::Validation(format!("Invalid rounding step: {}", e)))?;

        // Create FeeConfig
        let fees = FeeConfig {
            participation_fee,
            sales_fee_percent,
            rounding_step,
        };

        // Create and return Booth (this validates the fee ranges)
        Booth::new(self.description.clone(), date, fees)
    }

    /// Update an existing booth with form data
    ///
    /// # Errors
    ///
    /// Returns DomainError if fee configuration validation fails
    pub fn update_booth(&self, booth: &mut Booth, locale: Locale) -> Result<(), DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| DomainError::Validation(format!("Invalid date format: {}", e)))?;

        // Parse fee values using flexible parsing (accepts both comma and dot)
        let participation_fee = parse_decimal_input(&self.participation_fee)
            .map_err(|e| DomainError::Validation(format!("Invalid participation fee: {}", e)))?;

        let sales_fee_percent = parse_decimal_input(&self.sales_fee_percent)
            .map_err(|e| DomainError::Validation(format!("Invalid sales fee percent: {}", e)))?;

        let rounding_step = parse_decimal_input(&self.rounding_step)
            .map_err(|e| DomainError::Validation(format!("Invalid rounding step: {}", e)))?;

        // Create and validate FeeConfig
        let fees = FeeConfig {
            participation_fee,
            sales_fee_percent,
            rounding_step,
        };
        fees.validate_ranges()?;

        // Update booth fields
        booth.update_description(self.description.clone());
        booth.date = date;
        booth.update_fees(fees);

        Ok(())
    }
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

    // Validation errors
    let (description_error, set_description_error) = create_signal(None::<String>);
    let (date_error, set_date_error) = create_signal(None::<String>);
    let (participation_fee_error, set_participation_fee_error) = create_signal(None::<String>);
    let (sales_fee_percent_error, set_sales_fee_percent_error) = create_signal(None::<String>);
    let (rounding_step_error, set_rounding_step_error) = create_signal(None::<String>);

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

        let mut has_errors = false;

        // Validate description
        let desc = description.get();
        if desc.trim().is_empty() {
            set_description_error.set(Some(description_required_msg()));
            has_errors = true;
        } else if desc.len() > 200 {
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

        if !has_errors {
            let data = BoothFormData {
                description: description.get(),
                date: date.get(),
                participation_fee: participation_fee.get(),
                sales_fee_percent: sales_fee_percent.get(),
                rounding_step: rounding_step.get(),
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
                    // Participation Fee
                    <NumberInput
                        value=participation_fee
                        label={
                            let locale_val = locale.get();
                            let currency = currency_symbol_for_label(locale_val);
                            t!("booth.participation_fee")().replace("{currency}", currency)
                        }
                        placeholder="0.00".to_string()
                        required=true
                        error=participation_fee_error
                    />

                    // Sales Fee Percent
                    <NumberInput
                        value=sales_fee_percent
                        label=t!("booth.sales_fee_percent")()
                        placeholder="0.00".to_string()
                        required=true
                        error=sales_fee_percent_error
                    />

                    // Rounding Step
                    <NumberInput
                        value=rounding_step
                        label=t!("booth.rounding_step")()
                        placeholder="0.50".to_string()
                        required=true
                        error=rounding_step_error
                    />
                    <p class="text-sm text-gray-600 mt-1">
                        {t!("booth.rounding_step_help")()}
                    </p>
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
