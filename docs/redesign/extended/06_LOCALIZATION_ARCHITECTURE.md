# Localization (i18n) Architecture

**Date:** March 19, 2026  
**Status:** Critical Addition - NOT Previously Considered  
**Priority:** P0 - Must be in MVP

---

## Overview

The Java version has **full German localization** as primary language. This was completely missing from the Rust architecture documents and must be addressed immediately.

**Requirements:**
- ✅ Primary language: **German** (de)
- ✅ Fallback language: **English** (en)
- ✅ Future-proof for additional languages
- ✅ Browser language detection
- ✅ User can manually switch languages

---

## Current Java Implementation

### File Structure
```
vaadin-ui/src/main/resources/i18n/
├── i18n.json           # Config: default locale + supported locales
└── locale.json         # German translations (172 keys)
```

### Configuration
```json
{
  "defaultLocale": "de",
  "supportedLocales": {
    "de": "locale.json"
  }
}
```

### Locale File Structure
```json
{
  "format": {
    "currencyCode": "EUR",
    "dateTime": "dd.MM.yyyy HH:mm",
    "date": "dd.MM.yyyy",
    "decimal": "###,###,##0.00"
  },
  "texts": {
    "app.title": "Basar",
    "CheckoutView.title": "Kassieren",
    "BoothSelectionView.title": "Ereignis auswählen oder anlegen",
    // ... 172 translation keys total
  }
}
```

### Key Categories (172 keys total)
- **App Layout:** 3 keys (title, tooltips)
- **Forms:** 45 keys (labels, placeholders, validation errors)
- **Booth Management:** 35 keys (create, edit, delete, status)
- **Checkout:** 25 keys (keypad, summary, confirmation)
- **Vendor Reports:** 20 keys (list, print, statistics)
- **Data Exchange:** 30 keys (export, import, P2P sync)
- **Generic:** 14 keys (notifications, errors, buttons)

---

## Rust/WASM Architecture

### 1. Technology Stack

#### Chosen Library: **leptos_i18n 0.3+**

**Rationale:**
- Native Leptos integration (type-safe, reactive)
- Compile-time key checking (no runtime errors)
- JSON/YAML/TOML support
- Pluralization support
- Namespace support
- Small footprint (~20KB)
- Browser locale detection built-in

**Alternatives Considered:**
- **fluent-rs:** More powerful but overkill for our needs (~200KB)
- **rust-i18n:** Macro-based but less Leptos-friendly
- **Custom solution:** Not worth the effort for 172 keys

### 2. File Structure

```
ez-booth-wasm/
├── Cargo.toml
├── locales/
│   ├── de.json          # German (primary)
│   ├── en.json          # English (fallback)
│   └── translations.json # Config (default locale, fallback chain)
├── src/
│   ├── i18n/
│   │   ├── mod.rs       # I18n setup
│   │   ├── keys.rs      # Type-safe key constants (generated)
│   │   └── format.rs    # Formatters (date, currency, decimal)
│   └── main.rs
```

### 3. Configuration File

**`locales/translations.json`:**
```json
{
  "default": "de",
  "fallback": "en",
  "locales": {
    "de": {
      "name": "Deutsch",
      "format": {
        "currencyCode": "EUR",
        "dateTime": "dd.MM.yyyy HH:mm",
        "date": "dd.MM.yyyy",
        "decimal": "###,###,##0.00"
      }
    },
    "en": {
      "name": "English",
      "format": {
        "currencyCode": "USD",
        "dateTime": "MM/dd/yyyy hh:mm a",
        "date": "MM/dd/yyyy",
        "decimal": "#,###,##0.00"
      }
    }
  }
}
```

### 4. Translation Files

#### German (`locales/de.json`) - 172 keys
```json
{
  "app": {
    "title": "Basar",
    "toggleTheme": "Umschalten zwischen hellem und dunklem Modus"
  },
  "booth": {
    "title": "Ereignis",
    "create": "Neu",
    "edit": "Bearbeiten",
    "select": "Auswählen",
    "delete": "Endgültig löschen",
    "close": "Schließen",
    "reopen": "Erneut öffnen",
    "selectOrCreate": "Ereignis auswählen oder anlegen",
    "description": {
      "label": "Beschreibung",
      "placeholder": "Beschreibung eingeben..."
    },
    "date": {
      "label": "Datum"
    },
    "participationFee": {
      "label": "Teilnahmegebühr",
      "validation": {
        "required": "Teilnahmegebühr ist ein Pflichtfeld",
        "min": "Teilnahmegebühr darf nicht kleiner als 0 sein"
      }
    },
    "salesFee": {
      "label": "Umsatzbeteiligung",
      "validation": {
        "required": "Umsatzbeteiligung ist ein Pflichtfeld",
        "min": "Umsatzbeteiligung darf nicht kleiner als 0 sein"
      }
    },
    "roundingStep": {
      "label": "Gebühren aufrunden auf Betrag",
      "validation": {
        "required": "'Gebühren aufrunden auf Betrag' ist ein Pflichtfeld",
        "min": "Wert darf nicht kleiner als 0 sein"
      }
    },
    "status": {
      "open": "Ereignis ist offen",
      "closed": "Ereignis ist geschlossen"
    },
    "notifications": {
      "saveFailed": "Ereignis '%s' konnte nicht gespeichert werden!",
      "closeFailed": "Es gab einen Fehler beim Schließen!",
      "reopenFailed": "Es gab einen Fehler bei der Wiedereröffnung!",
      "deleteFailed": "Es gab einen Fehler beim Löschen!",
      "noSelection": "Bitte wählen ein bestehendes Ereignis aus oder lege ein neues Ereignis an."
    }
  },
  "vendor": {
    "title": "Verkäufer",
    "idFormat": "Verkäufer #%s",
    "idShort": "#%s"
  },
  "checkout": {
    "title": "Kassieren",
    "menuItem": "Kassieren",
    "itemCount": "%s Stück",
    "checkoutButton": "Kassieren",
    "checkoutButtonWithValue": "Kassieren: %s",
    "vendorNumber": {
      "label": "Verkäufernummer",
      "placeholder": "Verkäufernummer..."
    },
    "price": {
      "label": "Kaufpreis",
      "placeholder": "Kaufpreis..."
    },
    "clear": "Eingabefeld leeren",
    "finish": "Eingabe fertig",
    "confirmation": {
      "title": "Abschluss des Einkaufs bestätigen",
      "text": "Summe des Einkaufs",
      "confirm": "Bestätigen",
      "cancel": "Abbrechen",
      "printCheckbox": "Belegdruck"
    },
    "notifications": {
      "success": "Einkauf erfolgreich abgeschlossen",
      "failed": "Der Einkauf konnte nicht abgeschlossen werden!",
      "unsavedChanges": "Es liegen ungespeicherte Änderungen vor!%nBitte speichern oder löschen Sie diese zuerst.",
      "printReceipt": "Beleg wird in einem neuen Fenster geöffnet."
    }
  },
  "reports": {
    "title": "Abrechnung",
    "menuItem": "Abrechnung",
    "filterPlaceholder": "Filter...",
    "printAll": "Alle Belege drucken",
    "printReceipt": "Beleg drucken",
    "revenue": "Umsatz",
    "itemCount": "Anzahl verkaufter Gegenstände"
  },
  "export": {
    "title": "Daten exportieren / importieren",
    "menuItem": "Export/Import",
    "description": "💡 _Es werden nur fehlende Einträge von **Ereignissen mit identischem Namen und Datum** abgeglichen._",
    "save": "Als Datei speichern",
    "clickHere": "Hier klicken",
    "import": "Aus Datei übernehmen",
    "addFile": "Datei hochladen",
    "notifications": {
      "exportFailed": "Speichern der Datei fehlgeschlagen!",
      "importSuccess": "Datei wurde erfolgreich verarbeitet!",
      "importFailed": "Hochladen der Datei fehlgeschlagen!",
      "processing": "Datei wird verarbeitet...",
      "incorrectFileType": "Dateityp nicht unterstützt!"
    }
  },
  "common": {
    "save": "Speichern",
    "cancel": "Abbrechen",
    "delete": "Löschen",
    "edit": "Bearbeiten",
    "close": "Schließen",
    "or": "oder",
    "error": "Es ist ein Fehler aufgetreten!",
    "copiedToClipboard": "Wert wurde in die Zwischenablage kopiert"
  }
}
```

#### English (`locales/en.json`) - Fallback
```json
{
  "app": {
    "title": "Bazaar",
    "toggleTheme": "Toggle between light and dark mode"
  },
  "booth": {
    "title": "Event",
    "create": "New",
    "edit": "Edit",
    "select": "Select",
    "delete": "Delete permanently",
    "close": "Close",
    "reopen": "Reopen",
    "selectOrCreate": "Select or create event",
    "description": {
      "label": "Description",
      "placeholder": "Enter description..."
    },
    "date": {
      "label": "Date"
    },
    "participationFee": {
      "label": "Participation Fee",
      "validation": {
        "required": "Participation fee is required",
        "min": "Participation fee must not be negative"
      }
    },
    "salesFee": {
      "label": "Sales Commission",
      "validation": {
        "required": "Sales commission is required",
        "min": "Sales commission must not be negative"
      }
    },
    "roundingStep": {
      "label": "Round fees to",
      "validation": {
        "required": "Rounding step is required",
        "min": "Value must not be negative"
      }
    },
    "status": {
      "open": "Event is open",
      "closed": "Event is closed"
    },
    "notifications": {
      "saveFailed": "Failed to save event '%s'!",
      "closeFailed": "Error closing event!",
      "reopenFailed": "Error reopening event!",
      "deleteFailed": "Error deleting event!",
      "noSelection": "Please select an existing event or create a new one."
    }
  },
  "vendor": {
    "title": "Vendor",
    "idFormat": "Vendor #%s",
    "idShort": "#%s"
  },
  "checkout": {
    "title": "Checkout",
    "menuItem": "Checkout",
    "itemCount": "%s items",
    "checkoutButton": "Checkout",
    "checkoutButtonWithValue": "Checkout: %s",
    "vendorNumber": {
      "label": "Vendor Number",
      "placeholder": "Vendor number..."
    },
    "price": {
      "label": "Price",
      "placeholder": "Price..."
    },
    "clear": "Clear input",
    "finish": "Finish",
    "confirmation": {
      "title": "Confirm checkout",
      "text": "Purchase total",
      "confirm": "Confirm",
      "cancel": "Cancel",
      "printCheckbox": "Print receipt"
    },
    "notifications": {
      "success": "Purchase completed successfully",
      "failed": "Failed to complete purchase!",
      "unsavedChanges": "Unsaved changes! Please save or clear first.",
      "printReceipt": "Receipt will open in new window."
    }
  },
  "reports": {
    "title": "Reports",
    "menuItem": "Reports",
    "filterPlaceholder": "Filter...",
    "printAll": "Print all receipts",
    "printReceipt": "Print receipt",
    "revenue": "Revenue",
    "itemCount": "Items sold"
  },
  "export": {
    "title": "Export / Import Data",
    "menuItem": "Export/Import",
    "description": "💡 _Only missing entries from **events with identical name and date** will be synchronized._",
    "save": "Save as file",
    "clickHere": "Click here",
    "import": "Import from file",
    "addFile": "Upload file",
    "notifications": {
      "exportFailed": "Failed to save file!",
      "importSuccess": "File processed successfully!",
      "importFailed": "Failed to upload file!",
      "processing": "Processing file...",
      "incorrectFileType": "File type not supported!"
    }
  },
  "common": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete",
    "edit": "Edit",
    "close": "Close",
    "or": "or",
    "error": "An error occurred!",
    "copiedToClipboard": "Copied to clipboard"
  }
}
```

---

## 5. Implementation

### 5.1 Cargo.toml
```toml
[dependencies]
leptos = "0.6"
leptos_i18n = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
leptos_i18n = { version = "0.3", features = ["build"] }
```

### 5.2 Build Script (`build.rs`)
```rust
fn main() {
    leptos_i18n::build::generate_translations(
        leptos_i18n::build::Config {
            locales_dir: "locales",
            default_locale: "de",
            fallback_locale: "en",
        }
    );
}
```

### 5.3 I18n Setup (`src/i18n/mod.rs`)
```rust
use leptos::*;
use leptos_i18n::*;

// Include generated translations at compile time
include_i18n!();

/// Initialize i18n with browser locale detection
pub fn init_i18n() -> Locale {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    
    // Try browser language
    let browser_lang = navigator.language().unwrap_or_default();
    
    // Parse to locale (e.g., "de-DE" -> "de")
    let locale_code = browser_lang
        .split('-')
        .next()
        .unwrap_or("de");
    
    // Try to match supported locale, fallback to default
    match locale_code {
        "de" => Locale::De,
        "en" => Locale::En,
        _ => Locale::De, // Default to German
    }
}

/// Context provider for i18n
#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    let locale = create_rw_signal(init_i18n());
    
    provide_i18n_context(locale);
    
    view! {
        {children()}
    }
}

/// Get current locale signal
pub fn use_locale() -> RwSignal<Locale> {
    use_i18n_context().locale()
}

/// Switch to different locale
pub fn switch_locale(new_locale: Locale) {
    let i18n = use_i18n_context();
    i18n.set_locale(new_locale);
    
    // Persist in localStorage
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("ez_booth_locale", &new_locale.to_string());
    }
}
```

### 5.4 Format Helpers (`src/i18n/format.rs`)
```rust
use js_sys::Date;

pub struct LocaleFormatter {
    pub currency_code: &'static str,
    pub decimal_format: &'static str,
}

impl LocaleFormatter {
    pub fn for_locale(locale: Locale) -> Self {
        match locale {
            Locale::De => Self {
                currency_code: "EUR",
                decimal_format: "###,###,##0.00",
            },
            Locale::En => Self {
                currency_code: "USD",
                decimal_format: "#,###,##0.00",
            },
        }
    }
    
    /// Format currency (e.g., "42,50 €" for de, "$42.50" for en)
    pub fn format_currency(&self, amount: f64) -> String {
        match self.currency_code {
            "EUR" => format!("{:.2} €", amount).replace('.', ","),
            "USD" => format!("${:.2}", amount),
            _ => format!("{:.2}", amount),
        }
    }
    
    /// Format date (e.g., "19.03.2026" for de, "03/19/2026" for en)
    pub fn format_date(&self, timestamp: f64) -> String {
        let date = Date::new(&timestamp.into());
        let day = date.get_date();
        let month = date.get_month() + 1;
        let year = date.get_full_year();
        
        match self.currency_code {
            "EUR" => format!("{:02}.{:02}.{}", day, month, year),
            _ => format!("{:02}/{:02}/{}", month, day, year),
        }
    }
    
    /// Format datetime (e.g., "19.03.2026 15:13" for de)
    pub fn format_datetime(&self, timestamp: f64) -> String {
        let date = Date::new(&timestamp.into());
        let hours = date.get_hours();
        let minutes = date.get_minutes();
        
        let date_str = self.format_date(timestamp);
        
        match self.currency_code {
            "EUR" => format!("{} {:02}:{:02}", date_str, hours, minutes),
            _ => {
                let am_pm = if hours >= 12 { "PM" } else { "AM" };
                let hour_12 = if hours == 0 { 12 } else if hours > 12 { hours - 12 } else { hours };
                format!("{} {:02}:{:02} {}", date_str, hour_12, minutes, am_pm)
            }
        }
    }
}
```

### 5.5 Usage in Components
```rust
use crate::i18n::*;

#[component]
pub fn BoothForm() -> impl IntoView {
    let i18n = use_i18n();
    let formatter = LocaleFormatter::for_locale(use_locale().get());
    
    view! {
        <form>
            <h2>{t!(i18n, booth.title)}</h2>
            
            <label>
                {t!(i18n, booth.description.label)}
                <input 
                    type="text" 
                    placeholder={t!(i18n, booth.description.placeholder)} 
                />
            </label>
            
            <label>
                {t!(i18n, booth.date.label)}
                <input type="date" />
            </label>
            
            <label>
                {t!(i18n, booth.participationFee.label)}
                <input type="number" step="0.50" />
            </label>
            
            <div class="actions">
                <button type="submit">{t!(i18n, common.save)}</button>
                <button type="button">{t!(i18n, common.cancel)}</button>
            </div>
        </form>
    }
}

#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let locale = use_locale();
    
    view! {
        <select 
            on:change=move |ev| {
                let value = event_target_value(&ev);
                let new_locale = match value.as_str() {
                    "de" => Locale::De,
                    "en" => Locale::En,
                    _ => Locale::De,
                };
                switch_locale(new_locale);
            }
            prop:value=move || locale.get().to_string()
        >
            <option value="de">"Deutsch"</option>
            <option value="en">"English"</option>
        </select>
    }
}
```

---

## 6. Integration into Architecture

### Updated Cargo.toml (Section 4.4)
```toml
[dependencies]
leptos = "0.6"                  # UI framework
leptos_i18n = "0.3"            # Localization (NEW)
rexie = "0.6"                   # IndexedDB wrapper
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"             # JSON serialization
wasm-bindgen = "0.2"           # WASM-JS bridge
web-sys = "0.3"                # Browser APIs
js-sys = "0.1"                 # JavaScript types
```

### Module Structure Update (Section 5.1)
```
src/
├── i18n/
│   ├── mod.rs          # I18n context setup
│   └── format.rs       # Locale-aware formatters
├── domain/
├── services/
├── ui/
└── main.rs
```

---

## 7. Implementation Plan

### Phase 1: Foundation (Week 1 of Phase 1)
**Effort:** 8 hours

| Task | Effort | Priority |
|------|--------|----------|
| Add leptos_i18n dependency | 30 min | P0 |
| Create translation files (de.json, en.json) | 3 hours | P0 |
| Setup build.rs | 30 min | P0 |
| Create i18n module | 2 hours | P0 |
| Create format helpers | 1 hour | P0 |
| Browser locale detection | 1 hour | P0 |

### Phase 2: Integration (Week 2-5 of Phase 2)
**Effort:** 4 hours (spread across feature development)

- Replace hardcoded strings in components (ongoing)
- Add LanguageSwitcher component to AppLayout
- Test all 172 translation keys
- Validate formatting (currency, dates)

### Phase 3: Testing (Week 1 of Phase 4)
**Effort:** 2 hours

- Test browser locale detection (Chrome, Firefox, Safari)
- Test language switching
- Test fallback behavior
- Verify formatting in both locales

---

## 8. Success Metrics

| Metric | Target |
|--------|--------|
| Translation coverage | 100% (172 keys) |
| Browser locale detection | 95%+ accuracy |
| Language switch latency | <50ms |
| Bundle size impact | <20KB |
| Build time impact | <5 seconds |

---

## 9. Future Enhancements (Post-MVP)

### Phase 7+: Additional Languages
- Spanish (es)
- French (fr)
- Italian (it)
- Polish (pl)

### Phase 8+: Advanced Features
- Pluralization rules
- Date/time formatting with Intl API
- Number formatting with Intl API
- RTL support (Arabic, Hebrew)
- Translation management UI

---

## 10. Migration from Java

### Translation Key Mapping

| Java Key | Rust Key |
|----------|----------|
| `app.title` | `app.title` |
| `CheckoutView.title` | `checkout.title` |
| `UpsertBoothForm.descriptionField.label` | `booth.description.label` |
| `UpsertBoothForm.notification.invalidDate` | `booth.date.validation.required` |

**Differences:**
- Java uses flat structure with `.` separators
- Rust uses nested JSON for better organization
- Same semantic meaning, better hierarchy

### Complete Translation Matrix

See `/extended/07_TRANSLATION_MATRIX.md` for full 172-key mapping.

---

## 11. Critical Action Items

### Immediate (Before Phase 1 starts)
- [ ] Add leptos_i18n to dependencies
- [ ] Create locales/ directory
- [ ] Port 172 German translations from Java
- [ ] Create English translations
- [ ] Setup build.rs

### Phase 1 (Week 1)
- [ ] Implement i18n module
- [ ] Add format helpers
- [ ] Test browser locale detection

### Phase 2 (Ongoing)
- [ ] Replace hardcoded strings in components
- [ ] Add LanguageSwitcher to AppLayout

---

## 12. Impact on Timeline

**Additional Effort:**
- Phase 1: +8 hours (i18n setup)
- Phase 2: +4 hours (component integration)
- Phase 4: +2 hours (testing)

**Total:** +14 hours (~1.75 days)

**Updated Timeline:**
- Phase 1: 2 weeks → 2.5 weeks
- Total: 10 weeks → 10.5 weeks

**Still acceptable** - i18n is critical and cannot be skipped.

---

## 13. Trade-offs

### Accepted for MVP
- ✅ Only 2 languages (de, en) initially
- ✅ Manual translation management (JSON files)
- ✅ No translation management UI
- ✅ No RTL support

### Post-MVP Enhancements
- 🔄 Additional languages as needed
- 🔄 Translation management tool
- 🔄 Automated translation suggestions
- 🔄 Community translation contributions

---

**Status:** Ready for Implementation  
**Priority:** P0 - Must be in MVP  
**Effort:** 14 hours total  
**Impact:** +0.5 weeks to timeline
