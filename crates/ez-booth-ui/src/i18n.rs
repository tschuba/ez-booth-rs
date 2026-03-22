use leptos::*;
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    De,
    En,
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::De => "de",
            Locale::En => "en",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "en" | "en-US" | "en-GB" => Locale::En,
            _ => Locale::De, // German is default
        }
    }
}

/// Translation data structure
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Translations {
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

impl Translations {
    /// Get a translation by key (supports nested keys with dots)
    pub fn get(&self, key: &str) -> String {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &serde_json::Value::Object(
            self.data.clone().into_iter().map(|(k, v)| (k, v)).collect(),
        );

        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    if let Some(value) = map.get(part) {
                        current = value;
                    } else {
                        return format!("[missing: {}]", key);
                    }
                }
                _ => return format!("[missing: {}]", key),
            }
        }

        match current {
            serde_json::Value::String(s) => s.clone(),
            _ => format!("[invalid: {}]", key),
        }
    }
}

/// Detect browser locale
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["navigator"], js_name = language, thread_local_v2)]
    static LANGUAGE: String;
}

/// Detect the user's browser locale
pub fn detect_locale() -> Locale {
    LANGUAGE.with(|lang| Locale::from_str(lang))
}

/// Load translations for a given locale
pub fn load_translations(locale: Locale) -> Translations {
    let json_str = match locale {
        Locale::De => include_str!("../locales/de.json"),
        Locale::En => include_str!("../locales/en.json"),
    };

    serde_json::from_str(json_str).expect("Failed to parse translations")
}

/// Initialize i18n context
pub fn provide_i18n() {
    let locale = create_rw_signal(detect_locale());
    let translations = create_memo(move |_| load_translations(locale.get()));

    provide_context(locale);
    provide_context(translations);
}

/// Get current locale from context
pub fn use_locale() -> RwSignal<Locale> {
    use_context::<RwSignal<Locale>>().expect("Locale context not found. Did you call provide_i18n?")
}

/// Get translations from context
pub fn use_translations() -> Memo<Translations> {
    use_context::<Memo<Translations>>()
        .expect("Translations context not found. Did you call provide_i18n?")
}

/// Macro for easy translation access
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        let translations = $crate::i18n::use_translations();
        move || translations.with(|t| t.get($key))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_from_str() {
        assert_eq!(Locale::from_str("de"), Locale::De);
        assert_eq!(Locale::from_str("de-DE"), Locale::De);
        assert_eq!(Locale::from_str("en"), Locale::En);
        assert_eq!(Locale::from_str("en-US"), Locale::En);
        assert_eq!(Locale::from_str("fr"), Locale::De); // Default
    }

    #[test]
    fn test_load_translations() {
        let de = load_translations(Locale::De);
        assert_eq!(de.get("common.save"), "Speichern");
        assert_eq!(de.get("booth.title"), "Stand");

        let en = load_translations(Locale::En);
        assert_eq!(en.get("common.save"), "Save");
        assert_eq!(en.get("booth.title"), "Booth");
    }

    #[test]
    fn test_missing_key() {
        let de = load_translations(Locale::De);
        assert!(de.get("nonexistent.key").contains("[missing:"));
    }
}
