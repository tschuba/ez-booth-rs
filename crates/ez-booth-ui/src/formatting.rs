use crate::i18n::Locale;
use chrono::{DateTime, Local};
use rust_decimal::Decimal;

/// Format a Decimal as currency with locale-aware formatting
///
/// German (DE): €1.234,56
/// English (EN): €1,234.56
pub fn format_currency(amount: Decimal, locale: Locale) -> String {
    let symbol = currency_symbol(locale);
    let formatted = format_decimal_internal(amount, locale, 2, true);
    format!("{}{}", symbol, formatted)
}

/// Format a Decimal as a plain number with locale-aware separators
///
/// German (DE): 1.234,56
/// English (EN): 1,234.56
pub fn format_decimal(amount: Decimal, locale: Locale, decimals: u32) -> String {
    format_decimal_internal(amount, locale, decimals, true)
}

/// Format a Decimal as a percentage with locale-aware formatting
///
/// German (DE): 15,5%
/// English (EN): 15.5%
pub fn format_percentage(percent: Decimal, locale: Locale) -> String {
    let formatted = format_decimal_internal(percent, locale, 2, false);
    format!("{}%", formatted)
}

/// Format a chrono DateTime<Local> with locale-aware date and time
/// German: 25.03.2026 14:30
/// English: Mar 25, 2026 2:30 PM
pub fn format_datetime(date: DateTime<Local>, locale: Locale) -> String {
    match locale {
        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => {
            date.format("%d.%m.%Y %H:%M").to_string()
        }
        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => {
            date.format("%b %d, %Y %I:%M %p").to_string()
        }
    }
}

/// Format a decimal value for display in input fields (no thousand separators)
/// Used for displaying numbers in text input fields with locale-specific decimal separator
pub fn format_decimal_for_input(amount: Decimal, locale: Locale, decimals: u32) -> String {
    let rounded = amount.round_dp(decimals);
    let amount_str = format!("{:.prec$}", rounded, prec = decimals as usize);

    // Split into integer and fractional parts
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = parts[0];
    let fractional_part = parts.get(1).copied().unwrap_or("00");

    // Combine with locale-specific decimal separator (no thousand separators)
    if decimals > 0 {
        format!(
            "{}{}{}",
            integer_part,
            decimal_separator(locale),
            fractional_part
        )
    } else {
        integer_part.to_string()
    }
}

/// Get ISO currency code for locale
pub fn currency_code(locale: Locale) -> &'static str {
    match locale {
        Locale::DeDE | Locale::DeAT | Locale::De | Locale::EnEU => "EUR",
        Locale::DeCH => "CHF",
        Locale::EnUS => "USD",
        Locale::EnGB => "GBP",
        Locale::En => "EUR", // Default to EUR for unspecified English
    }
}

/// Get currency symbol without trailing space (for labels)
pub fn currency_symbol_for_label(locale: Locale) -> &'static str {
    match locale {
        Locale::DeDE | Locale::DeAT | Locale::De | Locale::EnEU => "€",
        Locale::DeCH => "CHF",
        Locale::EnUS => "$",
        Locale::EnGB => "£",
        Locale::En => "€",
    }
}

/// Get currency symbol for locale
pub fn currency_symbol(locale: Locale) -> &'static str {
    match locale {
        Locale::DeDE | Locale::DeAT | Locale::De | Locale::EnEU => "€ ",
        Locale::DeCH => "CHF ",
        Locale::EnUS => "$ ",
        Locale::EnGB => "£ ",
        Locale::En => "€ ",
    }
}

/// Get decimal separator for locale
pub fn decimal_separator(locale: Locale) -> char {
    match locale {
        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => ',',
        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => '.',
    }
}

/// Get thousands separator for locale
pub fn thousands_separator(locale: Locale) -> char {
    match locale {
        Locale::De | Locale::DeDE | Locale::DeAT | Locale::DeCH => '.',
        Locale::En | Locale::EnUS | Locale::EnGB | Locale::EnEU => ',',
    }
}

/// Internal function to format decimal numbers with locale support
fn format_decimal_internal(
    amount: Decimal,
    locale: Locale,
    decimals: u32,
    use_thousand_sep: bool,
) -> String {
    // Round to the specified number of decimals
    let rounded = amount.round_dp(decimals);

    // Convert to string using standard formatting
    let amount_str = format!("{:.prec$}", rounded, prec = decimals as usize);

    // Split into integer and fractional parts
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = parts[0];
    let fractional_part = parts.get(1).copied().unwrap_or("00");

    // Add thousand separators to integer part if requested
    let formatted_integer = if use_thousand_sep {
        add_thousand_separators(integer_part, locale)
    } else {
        integer_part.to_string()
    };

    // Combine with locale-specific decimal separator
    if decimals > 0 {
        format!(
            "{}{}{}",
            formatted_integer,
            decimal_separator(locale),
            fractional_part
        )
    } else {
        formatted_integer
    }
}

/// Add thousand separators to an integer string
fn add_thousand_separators(int_str: &str, locale: Locale) -> String {
    let sep = thousands_separator(locale);

    // Handle negative numbers
    let (is_negative, digits) = if int_str.starts_with('-') {
        (true, &int_str[1..])
    } else {
        (false, int_str)
    };

    // Add separators from right to left
    let mut result = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(sep);
        }
        result.push(ch);
    }

    // Reverse back and add negative sign if needed
    let reversed: String = result.chars().rev().collect();
    if is_negative {
        format!("-{}", reversed)
    } else {
        reversed
    }
}

/// Parse user input to Decimal flexibly
///
/// Accepts both comma and dot as decimal separators regardless of locale
/// Automatically detects which separator is used based on position
/// Validates that there are no more than 2 digits after the decimal separator
/// Returns error if the input is not a valid number or has more than 2 decimal places
pub fn parse_decimal_input(input: &str) -> Result<Decimal, String> {
    // Trim whitespace
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }

    // Determine which character appears last - that's the decimal separator
    let last_dot = trimmed.rfind('.');
    let last_comma = trimmed.rfind(',');

    // Find the position of the decimal separator and check decimal places
    let decimal_separator_pos = match (last_dot, last_comma) {
        (None, None) => None,
        (Some(pos), None) => Some(pos),
        (None, Some(pos)) => Some(pos),
        (Some(d_pos), Some(c_pos)) => Some(d_pos.max(c_pos)),
    };

    // Validate decimal places (max 2 digits after separator)
    if let Some(sep_pos) = decimal_separator_pos {
        let after_separator = &trimmed[sep_pos + 1..];
        // Remove any thousand separators that might appear after (though unlikely)
        let digits_after = after_separator
            .chars()
            .filter(|c| c.is_ascii_digit())
            .count();
        if digits_after > 2 {
            return Err("Maximum 2 decimal places allowed".to_string());
        }
    }

    let normalized = match (last_dot, last_comma) {
        (None, None) => trimmed.to_string(),          // No separators
        (Some(_), None) => trimmed.replace(',', ""), // Only dots (remove any commas that might exist)
        (None, Some(_)) => trimmed.replace(',', "."), // Only commas → replace with dot
        (Some(d_pos), Some(c_pos)) => {
            // Both present - last one is decimal separator
            if c_pos > d_pos {
                // Comma is decimal: "1.234,56" → "1234.56"
                trimmed.replace('.', "").replace(',', ".")
            } else {
                // Dot is decimal: "1,234.56" → "1234.56"
                trimmed.replace(',', "")
            }
        }
    };

    // Parse using rust_decimal
    Decimal::from_str_exact(&normalized).map_err(|e| format!("Invalid number format: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_format_currency_de() {
        assert_eq!(
            format_currency(Decimal::from_str("1234.56").unwrap(), Locale::De),
            "€ 1.234,56"
        );
        assert_eq!(
            format_currency(Decimal::from_str("10.50").unwrap(), Locale::De),
            "€ 10,50"
        );
        assert_eq!(
            format_currency(Decimal::from_str("0.99").unwrap(), Locale::De),
            "€ 0,99"
        );
        assert_eq!(
            format_currency(Decimal::from_str("1000000.00").unwrap(), Locale::De),
            "€ 1.000.000,00"
        );
    }

    #[test]
    fn test_format_currency_en() {
        assert_eq!(
            format_currency(Decimal::from_str("1234.56").unwrap(), Locale::En),
            "€ 1,234.56"
        );
        assert_eq!(
            format_currency(Decimal::from_str("10.50").unwrap(), Locale::En),
            "€ 10.50"
        );
        assert_eq!(
            format_currency(Decimal::from_str("0.99").unwrap(), Locale::En),
            "€ 0.99"
        );
        assert_eq!(
            format_currency(Decimal::from_str("1000000.00").unwrap(), Locale::En),
            "€ 1,000,000.00"
        );
    }

    #[test]
    fn test_format_currency_negative() {
        assert_eq!(
            format_currency(Decimal::from_str("-50.25").unwrap(), Locale::De),
            "€ -50,25"
        );
        assert_eq!(
            format_currency(Decimal::from_str("-1234.56").unwrap(), Locale::En),
            "€ -1,234.56"
        );
    }

    #[test]
    fn test_format_decimal_de() {
        assert_eq!(
            format_decimal(Decimal::from_str("1234.567").unwrap(), Locale::De, 2),
            "1.234,57"
        );
        assert_eq!(
            format_decimal(Decimal::from_str("1234.567").unwrap(), Locale::De, 3),
            "1.234,567"
        );
        assert_eq!(
            format_decimal(Decimal::from_str("99.9").unwrap(), Locale::De, 1),
            "99,9"
        );
    }

    #[test]
    fn test_format_decimal_en() {
        assert_eq!(
            format_decimal(Decimal::from_str("1234.567").unwrap(), Locale::En, 2),
            "1,234.57"
        );
        assert_eq!(
            format_decimal(Decimal::from_str("1234.567").unwrap(), Locale::En, 3),
            "1,234.567"
        );
        assert_eq!(
            format_decimal(Decimal::from_str("99.9").unwrap(), Locale::En, 1),
            "99.9"
        );
    }

    #[test]
    fn test_format_percentage_de() {
        assert_eq!(
            format_percentage(Decimal::from_str("15.5").unwrap(), Locale::De),
            "15,50%"
        );
        assert_eq!(
            format_percentage(Decimal::from_str("100.0").unwrap(), Locale::De),
            "100,00%"
        );
        assert_eq!(
            format_percentage(Decimal::from_str("0.5").unwrap(), Locale::De),
            "0,50%"
        );
    }

    #[test]
    fn test_format_percentage_en() {
        assert_eq!(
            format_percentage(Decimal::from_str("15.5").unwrap(), Locale::En),
            "15.50%"
        );
        assert_eq!(
            format_percentage(Decimal::from_str("100.0").unwrap(), Locale::En),
            "100.00%"
        );
        assert_eq!(
            format_percentage(Decimal::from_str("0.5").unwrap(), Locale::En),
            "0.50%"
        );
    }

    #[test]
    fn test_parse_decimal_input() {
        // Single comma as decimal
        assert_eq!(
            parse_decimal_input("15,00").unwrap(),
            Decimal::from_str("15.00").unwrap()
        );

        // Single dot as decimal
        assert_eq!(
            parse_decimal_input("15.00").unwrap(),
            Decimal::from_str("15.00").unwrap()
        );

        // No separator
        assert_eq!(
            parse_decimal_input("15").unwrap(),
            Decimal::from_str("15").unwrap()
        );

        // Both separators - comma last (decimal)
        assert_eq!(
            parse_decimal_input("1.234,56").unwrap(),
            Decimal::from_str("1234.56").unwrap()
        );

        // Both separators - dot last (decimal)
        assert_eq!(
            parse_decimal_input("1,234.56").unwrap(),
            Decimal::from_str("1234.56").unwrap()
        );

        // Single separator treated as decimal
        assert_eq!(
            parse_decimal_input("1.5").unwrap(),
            Decimal::from_str("1.5").unwrap()
        );
        assert_eq!(
            parse_decimal_input("1,5").unwrap(),
            Decimal::from_str("1.5").unwrap()
        );

        // Negative numbers
        assert_eq!(
            parse_decimal_input("-50,25").unwrap(),
            Decimal::from_str("-50.25").unwrap()
        );
        assert_eq!(
            parse_decimal_input("-50.25").unwrap(),
            Decimal::from_str("-50.25").unwrap()
        );

        // Invalid input
        assert!(parse_decimal_input("abc").is_err());
        assert!(parse_decimal_input("").is_err());
    }

    #[test]
    fn test_parse_decimal_input_max_two_decimals() {
        // Valid: 2 decimal places
        assert_eq!(
            parse_decimal_input("15.00").unwrap(),
            Decimal::from_str("15.00").unwrap()
        );
        assert_eq!(
            parse_decimal_input("15,99").unwrap(),
            Decimal::from_str("15.99").unwrap()
        );

        // Valid: 1 decimal place
        assert_eq!(
            parse_decimal_input("15.5").unwrap(),
            Decimal::from_str("15.5").unwrap()
        );

        // Valid: 0 decimal places
        assert_eq!(
            parse_decimal_input("15").unwrap(),
            Decimal::from_str("15").unwrap()
        );

        // Invalid: 3 decimal places
        assert!(parse_decimal_input("15.123").is_err());
        assert_eq!(
            parse_decimal_input("15.123").unwrap_err(),
            "Maximum 2 decimal places allowed"
        );

        // Invalid: 4 decimal places
        assert!(parse_decimal_input("15,1234").is_err());
        assert_eq!(
            parse_decimal_input("15,1234").unwrap_err(),
            "Maximum 2 decimal places allowed"
        );

        // Invalid: more than 2 decimals with thousand separator
        assert!(parse_decimal_input("1.234,567").is_err());
        assert_eq!(
            parse_decimal_input("1.234,567").unwrap_err(),
            "Maximum 2 decimal places allowed"
        );
    }

    #[test]
    fn test_format_decimal_for_input() {
        // German locale
        assert_eq!(
            format_decimal_for_input(Decimal::from_str("10.5").unwrap(), Locale::De, 2),
            "10,50"
        );

        // English locale
        assert_eq!(
            format_decimal_for_input(Decimal::from_str("10.5").unwrap(), Locale::En, 2),
            "10.50"
        );

        // Large numbers (no thousand separators in inputs)
        assert_eq!(
            format_decimal_for_input(Decimal::from_str("1234.56").unwrap(), Locale::De, 2),
            "1234,56"
        );

        assert_eq!(
            format_decimal_for_input(Decimal::from_str("1234.56").unwrap(), Locale::En, 2),
            "1234.56"
        );

        // Zero decimal places
        assert_eq!(
            format_decimal_for_input(Decimal::from_str("15").unwrap(), Locale::De, 0),
            "15"
        );

        // Negative numbers
        assert_eq!(
            format_decimal_for_input(Decimal::from_str("-10.5").unwrap(), Locale::De, 2),
            "-10,50"
        );
    }

    #[test]
    fn test_thousand_separators() {
        assert_eq!(add_thousand_separators("1234567", Locale::De), "1.234.567");
        assert_eq!(add_thousand_separators("1234567", Locale::En), "1,234,567");
        assert_eq!(add_thousand_separators("123", Locale::De), "123");
        assert_eq!(add_thousand_separators("1234", Locale::En), "1,234");
        assert_eq!(
            add_thousand_separators("-1234567", Locale::De),
            "-1.234.567"
        );
    }

    #[test]
    fn test_separators() {
        assert_eq!(decimal_separator(Locale::De), ',');
        assert_eq!(decimal_separator(Locale::En), '.');
        assert_eq!(thousands_separator(Locale::De), '.');
        assert_eq!(thousands_separator(Locale::En), ',');
    }

    #[test]
    fn test_currency_code() {
        // EUR locales
        assert_eq!(currency_code(Locale::DeDE), "EUR");
        assert_eq!(currency_code(Locale::DeAT), "EUR");
        assert_eq!(currency_code(Locale::De), "EUR");
        assert_eq!(currency_code(Locale::EnEU), "EUR");
        assert_eq!(currency_code(Locale::En), "EUR");

        // CHF locale
        assert_eq!(currency_code(Locale::DeCH), "CHF");

        // USD locale
        assert_eq!(currency_code(Locale::EnUS), "USD");

        // GBP locale
        assert_eq!(currency_code(Locale::EnGB), "GBP");
    }

    #[test]
    fn test_currency_symbol() {
        // EUR locales (with space)
        assert_eq!(currency_symbol(Locale::DeDE), "€ ");
        assert_eq!(currency_symbol(Locale::DeAT), "€ ");
        assert_eq!(currency_symbol(Locale::De), "€ ");
        assert_eq!(currency_symbol(Locale::EnEU), "€ ");
        assert_eq!(currency_symbol(Locale::En), "€ ");

        // Other currencies (with space)
        assert_eq!(currency_symbol(Locale::DeCH), "CHF ");
        assert_eq!(currency_symbol(Locale::EnUS), "$ ");
        assert_eq!(currency_symbol(Locale::EnGB), "£ ");
    }

    #[test]
    fn test_currency_symbol_for_label() {
        // EUR locales (no space)
        assert_eq!(currency_symbol_for_label(Locale::DeDE), "€");
        assert_eq!(currency_symbol_for_label(Locale::DeAT), "€");
        assert_eq!(currency_symbol_for_label(Locale::De), "€");
        assert_eq!(currency_symbol_for_label(Locale::EnEU), "€");
        assert_eq!(currency_symbol_for_label(Locale::En), "€");

        // Other currencies (no space)
        assert_eq!(currency_symbol_for_label(Locale::DeCH), "CHF");
        assert_eq!(currency_symbol_for_label(Locale::EnUS), "$");
        assert_eq!(currency_symbol_for_label(Locale::EnGB), "£");
    }

    #[test]
    fn test_format_currency_with_variants() {
        let amount = Decimal::from_str("1234.56").unwrap();

        // EUR variants (German number format)
        assert_eq!(format_currency(amount, Locale::DeDE), "€ 1.234,56");
        assert_eq!(format_currency(amount, Locale::DeAT), "€ 1.234,56");
        assert_eq!(format_currency(amount, Locale::De), "€ 1.234,56");

        // CHF (German number format)
        assert_eq!(format_currency(amount, Locale::DeCH), "CHF 1.234,56");

        // USD (English number format)
        assert_eq!(format_currency(amount, Locale::EnUS), "$ 1,234.56");

        // GBP (English number format)
        assert_eq!(format_currency(amount, Locale::EnGB), "£ 1,234.56");

        // EUR with English format
        assert_eq!(format_currency(amount, Locale::EnEU), "€ 1,234.56");
        assert_eq!(format_currency(amount, Locale::En), "€ 1,234.56");
    }
}
