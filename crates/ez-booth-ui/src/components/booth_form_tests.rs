#[cfg(test)]
mod tests {
    use super::super::booth_form::BoothFormData;
    use crate::i18n::Locale;
    use chrono::NaiveDate;
    use domain::error::DomainError;
    use domain::models::booth::{Booth, FeeConfig, OmissionRule, VendorIdOmissionRules};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn default_rules() -> VendorIdOmissionRules {
        VendorIdOmissionRules::default()
    }

    #[test]
    fn test_default_form_data() {
        let form = BoothFormData::default();

        let today = chrono::Local::now().date_naive();
        let expected_date = today.format("%Y-%m-%d").to_string();

        assert_eq!(form.description, "");
        assert_eq!(form.date, expected_date);
        assert_eq!(form.participation_fee, "1.00");
        assert_eq!(form.sales_fee_percent, "15.00");
        assert_eq!(form.rounding_step, "0.50");
        assert_eq!(form.vendor_omission_rules, default_rules());
    }

    #[test]
    fn test_to_booth_valid_data() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let booth = form.to_booth(Locale::En);
        assert!(booth.is_ok());

        let booth = booth.unwrap();
        assert_eq!(booth.description, "Test Booth");
        assert_eq!(booth.date, NaiveDate::from_ymd_opt(2026, 3, 25).unwrap());
        assert_eq!(
            booth.fees.participation_fee,
            Decimal::from_str("10.00").unwrap()
        );
        assert_eq!(
            booth.fees.sales_fee_percent,
            Decimal::from_str("15.00").unwrap()
        );
        assert_eq!(booth.fees.rounding_step, Decimal::from_str("0.50").unwrap());
        assert_eq!(booth.vendor_id_omission_rules, default_rules());
    }

    #[test]
    fn test_to_booth_invalid_date() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "invalid-date".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_invalid_participation_fee() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "invalid".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_negative_participation_fee() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "-10.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_invalid_sales_fee_percent() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "invalid".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_sales_fee_percent_out_of_range() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "150.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_invalid_rounding_step() {
        let form = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "invalid".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_to_booth_empty_description() {
        let form = BoothFormData {
            description: "".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.to_booth(Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_from_booth() {
        let fees = FeeConfig {
            participation_fee: Decimal::from_str("25.50").unwrap(),
            sales_fee_percent: Decimal::from_str("12.50").unwrap(),
            rounding_step: Decimal::from_str("0.10").unwrap(),
        };

        let booth = Booth::new(
            "Original Booth".to_string(),
            NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
            fees,
        )
        .unwrap();

        let form = BoothFormData::from_booth(&booth, Locale::En);

        assert_eq!(form.description, "Original Booth");
        assert_eq!(form.date, "2026-04-15");
        assert_eq!(form.participation_fee, "25.50");
        assert_eq!(form.sales_fee_percent, "12.50");
        assert_eq!(form.rounding_step, "0.10");
        assert_eq!(form.vendor_omission_rules, booth.vendor_id_omission_rules);
    }

    #[test]
    fn test_from_booth_locale_formatting() {
        let fees = FeeConfig {
            participation_fee: Decimal::from_str("10.50").unwrap(),
            sales_fee_percent: Decimal::from_str("15.00").unwrap(),
            rounding_step: Decimal::from_str("0.50").unwrap(),
        };

        let booth = Booth::new(
            "Test Booth".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 25).unwrap(),
            fees,
        )
        .unwrap();

        let form_en = BoothFormData::from_booth(&booth, Locale::En);
        assert_eq!(form_en.participation_fee, "10.50");
        assert_eq!(form_en.sales_fee_percent, "15.00");
        assert_eq!(form_en.rounding_step, "0.50");

        let form_de = BoothFormData::from_booth(&booth, Locale::De);
        assert_eq!(form_de.participation_fee, "10,50");
        assert_eq!(form_de.sales_fee_percent, "15,00");
        assert_eq!(form_de.rounding_step, "0,50");
    }

    #[test]
    fn test_to_booth_flexible_parsing() {
        let form_dot = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10.50".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "0.50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let booth_dot = form_dot.to_booth(Locale::En).unwrap();
        assert_eq!(
            booth_dot.fees.participation_fee,
            Decimal::from_str("10.50").unwrap()
        );
        assert_eq!(
            booth_dot.fees.sales_fee_percent,
            Decimal::from_str("15.00").unwrap()
        );
        assert_eq!(
            booth_dot.fees.rounding_step,
            Decimal::from_str("0.50").unwrap()
        );

        let form_comma = BoothFormData {
            description: "Test Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "10,50".to_string(),
            sales_fee_percent: "15,00".to_string(),
            rounding_step: "0,50".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let booth_comma = form_comma.to_booth(Locale::De).unwrap();
        assert_eq!(
            booth_comma.fees.participation_fee,
            Decimal::from_str("10.50").unwrap()
        );
        assert_eq!(
            booth_comma.fees.sales_fee_percent,
            Decimal::from_str("15.00").unwrap()
        );
        assert_eq!(
            booth_comma.fees.rounding_step,
            Decimal::from_str("0.50").unwrap()
        );

        assert_eq!(
            booth_dot.fees.participation_fee,
            booth_comma.fees.participation_fee
        );
        assert_eq!(
            booth_dot.fees.sales_fee_percent,
            booth_comma.fees.sales_fee_percent
        );
        assert_eq!(booth_dot.fees.rounding_step, booth_comma.fees.rounding_step);
    }

    #[test]
    fn test_update_booth_valid() {
        let fees = FeeConfig {
            participation_fee: Decimal::from_str("10.00").unwrap(),
            sales_fee_percent: Decimal::from_str("10.00").unwrap(),
            rounding_step: Decimal::from_str("0.50").unwrap(),
        };

        let mut booth = Booth::new(
            "Original".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            fees,
        )
        .unwrap();

        let custom_rules = VendorIdOmissionRules {
            rules: vec![OmissionRule::Exact("999".to_string())],
        };

        let form = BoothFormData {
            description: "Updated Booth".to_string(),
            date: "2026-03-25".to_string(),
            participation_fee: "20.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "1.00".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: custom_rules.clone(),
        };

        let result = form.update_booth(&mut booth, Locale::En);
        assert!(result.is_ok());

        assert_eq!(booth.description, "Updated Booth");
        assert_eq!(booth.date, NaiveDate::from_ymd_opt(2026, 3, 25).unwrap());
        assert_eq!(
            booth.fees.participation_fee,
            Decimal::from_str("20.00").unwrap()
        );
        assert_eq!(
            booth.fees.sales_fee_percent,
            Decimal::from_str("15.00").unwrap()
        );
        assert_eq!(booth.fees.rounding_step, Decimal::from_str("1.00").unwrap());
        assert_eq!(booth.vendor_id_omission_rules, custom_rules);
    }

    #[test]
    fn test_update_booth_invalid_date() {
        let fees = FeeConfig {
            participation_fee: Decimal::from_str("10.00").unwrap(),
            sales_fee_percent: Decimal::from_str("10.00").unwrap(),
            rounding_step: Decimal::from_str("0.50").unwrap(),
        };

        let mut booth = Booth::new(
            "Original".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            fees,
        )
        .unwrap();

        let form = BoothFormData {
            description: "Updated".to_string(),
            date: "invalid-date".to_string(),
            participation_fee: "20.00".to_string(),
            sales_fee_percent: "15.00".to_string(),
            rounding_step: "1.00".to_string(),
            vendor_validation_type: "digits_only".to_string(),
            vendor_validation_regex: String::new(),
            vendor_omission_rules: default_rules(),
        };

        let result = form.update_booth(&mut booth, Locale::En);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }
}
