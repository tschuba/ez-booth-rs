use crate::components::*;
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
        // Default date to today
        let today = chrono::Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        Self {
            description: String::new(),
            date: date_str,
            participation_fee: "0.00".to_string(),
            sales_fee_percent: "0.00".to_string(),
            rounding_step: "0.50".to_string(),
        }
    }
}

impl BoothFormData {
    /// Create form data from an existing Booth
    pub fn from_booth(booth: &Booth) -> Self {
        Self {
            description: booth.description.clone(),
            date: booth.date.format("%Y-%m-%d").to_string(),
            participation_fee: booth.fees.participation_fee.to_string(),
            sales_fee_percent: booth.fees.sales_fee_percent.to_string(),
            rounding_step: booth.fees.rounding_step.to_string(),
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
    pub fn to_booth(&self) -> Result<Booth, DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| DomainError::Validation(format!("Invalid date format: {}", e)))?;

        // Parse fee values to Decimal
        let participation_fee = Decimal::from_str(&self.participation_fee)
            .map_err(|e| DomainError::Validation(format!("Invalid participation fee: {}", e)))?;

        let sales_fee_percent = Decimal::from_str(&self.sales_fee_percent)
            .map_err(|e| DomainError::Validation(format!("Invalid sales fee percent: {}", e)))?;

        let rounding_step = Decimal::from_str(&self.rounding_step)
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
    pub fn update_booth(&self, booth: &mut Booth) -> Result<(), DomainError> {
        // Parse date
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| DomainError::Validation(format!("Invalid date format: {}", e)))?;

        // Parse fee values
        let participation_fee = Decimal::from_str(&self.participation_fee)
            .map_err(|e| DomainError::Validation(format!("Invalid participation fee: {}", e)))?;

        let sales_fee_percent = Decimal::from_str(&self.sales_fee_percent)
            .map_err(|e| DomainError::Validation(format!("Invalid sales fee percent: {}", e)))?;

        let rounding_step = Decimal::from_str(&self.rounding_step)
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
            set_description_error.set(Some("Description is required".to_string()));
            has_errors = true;
        } else if desc.len() > 200 {
            set_description_error.set(Some(
                "Description must be 200 characters or less".to_string(),
            ));
            has_errors = true;
        }

        // Validate date
        let date_str = date.get();
        if date_str.trim().is_empty() {
            set_date_error.set(Some("Date is required".to_string()));
            has_errors = true;
        }

        // Validate participation fee
        let part_fee = participation_fee.get();
        if part_fee.trim().is_empty() {
            set_participation_fee_error.set(Some("Participation fee is required".to_string()));
            has_errors = true;
        } else if part_fee.parse::<f64>().is_err() {
            set_participation_fee_error.set(Some("Invalid number format".to_string()));
            has_errors = true;
        } else if part_fee.parse::<f64>().unwrap() < 0.0 {
            set_participation_fee_error.set(Some("Cannot be negative".to_string()));
            has_errors = true;
        }

        // Validate sales fee percent
        let sales_pct = sales_fee_percent.get();
        if sales_pct.trim().is_empty() {
            set_sales_fee_percent_error.set(Some("Sales fee percent is required".to_string()));
            has_errors = true;
        } else if sales_pct.parse::<f64>().is_err() {
            set_sales_fee_percent_error.set(Some("Invalid number format".to_string()));
            has_errors = true;
        } else {
            let val = sales_pct.parse::<f64>().unwrap();
            if val < 0.0 {
                set_sales_fee_percent_error.set(Some("Cannot be negative".to_string()));
                has_errors = true;
            } else if val > 100.0 {
                set_sales_fee_percent_error.set(Some("Cannot exceed 100%".to_string()));
                has_errors = true;
            }
        }

        // Validate rounding step
        let rounding = rounding_step.get();
        if rounding.trim().is_empty() {
            set_rounding_step_error.set(Some("Rounding step is required".to_string()));
            has_errors = true;
        } else if rounding.parse::<f64>().is_err() {
            set_rounding_step_error.set(Some("Invalid number format".to_string()));
            has_errors = true;
        } else if rounding.parse::<f64>().unwrap() < 0.0 {
            set_rounding_step_error.set(Some("Cannot be negative".to_string()));
            has_errors = true;
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
                        label=t!("booth.participation_fee")()
                        placeholder="0.00".to_string()
                        min=0.0
                        step=0.01
                        required=true
                        error=participation_fee_error
                    />

                    // Sales Fee Percent
                    <NumberInput
                        value=sales_fee_percent
                        label=t!("booth.sales_fee_percent")()
                        placeholder="0.00".to_string()
                        min=0.0
                        max=100.0
                        step=0.01
                        required=true
                        error=sales_fee_percent_error
                    />

                    // Rounding Step
                    <NumberInput
                        value=rounding_step
                        label=t!("booth.rounding_step")()
                        placeholder="0.50".to_string()
                        min=0.0
                        step=0.01
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
