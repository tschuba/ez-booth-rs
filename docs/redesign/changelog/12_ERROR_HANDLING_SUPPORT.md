# Error Handling & User Support Strategy

**Document Version:** 1.0  
**Date:** March 19, 2026  
**Status:** Design Phase  
**Context:** Adding comprehensive error handling and support mechanisms

---

## Executive Summary

This document defines error handling and user support strategies for ez-booth-rs to ensure users can troubleshoot issues independently and support teams have necessary diagnostic tools.

### Key Additions

1. **User-Facing Error Messages** - Clear, actionable error messages in user's language
2. **Diagnostic Tools** - Built-in system health checks and diagnostic exports
3. **Self-Service Support** - In-app help, FAQs, and troubleshooting guides
4. **Error Recovery** - Automatic recovery mechanisms and manual fallbacks
5. **Support Bundle Export** - One-click export of diagnostic information

---

## 1. Error Handling Architecture

### 1.1 Error Hierarchy

```rust
// Core error types
pub enum AppError {
    // User-recoverable errors (show friendly message)
    Validation(ValidationError),
    NotFound(EntityType, String),
    Conflict(ConflictError),
    
    // System errors (show recovery options)
    Storage(StorageError),
    Network(NetworkError),
    Sync(SyncError),
    
    // Fatal errors (show diagnostic export)
    Corruption(CorruptionError),
    BrowserCompatibility(CompatError),
    OutOfMemory,
    
    // Unknown (fallback)
    Unknown(String),
}

pub struct ValidationError {
    field: String,
    message: TranslationKey,
    suggestion: Option<TranslationKey>,
}

pub struct StorageError {
    operation: StorageOp,
    cause: String,
    recoverable: bool,
    recovery_steps: Vec<TranslationKey>,
}

pub struct CorruptionError {
    entity_type: EntityType,
    corruption_type: CorruptionType,
    affected_records: usize,
    can_repair: bool,
}
```

### 1.2 Error Display Strategy

#### User-Recoverable Errors
Show friendly message with clear action:
```
┌─────────────────────────────────────────┐
│ ⚠️  Vendor name required                │
│                                         │
│ Please enter a name for this vendor.    │
│                                         │
│ [Go back] [Learn more]                  │
└─────────────────────────────────────────┘
```

#### System Errors
Show problem + recovery options:
```
┌─────────────────────────────────────────┐
│ ❌ Failed to save changes                │
│                                         │
│ Could not write to local storage.       │
│                                         │
│ Try these steps:                        │
│ • Refresh the page and try again        │
│ • Check browser storage settings        │
│ • Export your data as backup            │
│                                         │
│ [Retry] [Export Backup] [Get Help]      │
└─────────────────────────────────────────┘
```

#### Fatal Errors
Show diagnostic export option:
```
┌─────────────────────────────────────────┐
│ 🔴 Critical Error                        │
│                                         │
│ The application encountered a problem   │
│ it cannot recover from automatically.   │
│                                         │
│ Your data is safe. Download a support   │
│ bundle to share with our team.          │
│                                         │
│ [Download Support Bundle]               │
│ [Contact Support] [Reload App]          │
└─────────────────────────────────────────┘
```

---

## 2. Diagnostic Tools

### 2.1 System Health Check

Built-in diagnostic panel accessible from Settings:

```
System Health
─────────────────────────────────────────
✅ Browser: Chrome 120.0.6099 (supported)
✅ Storage: 45.2 MB / 500 MB available
✅ WASM: Loaded successfully
✅ Database: 3 tables, 156 records
⚠️  IndexedDB version: 2 (upgrade available)
✅ Last backup: 2026-03-18 14:32

[Run Full Diagnostic] [Export Diagnostic Report]
```

### 2.2 Diagnostic Report Export

**File:** `ez-booth-diagnostic-YYYYMMDD-HHMMSS.json`

```json
{
  "diagnostic_version": "1.0",
  "timestamp": "2026-03-19T16:15:00Z",
  "app_version": "0.1.0",
  "browser": {
    "name": "Chrome",
    "version": "120.0.6099",
    "user_agent": "Mozilla/5.0...",
    "language": "de-DE",
    "platform": "MacIntel"
  },
  "storage": {
    "quota_mb": 500,
    "used_mb": 45.2,
    "available_mb": 454.8,
    "database_version": 2,
    "indexeddb_supported": true,
    "localstorage_supported": true
  },
  "data_summary": {
    "booths": 1,
    "vendors": 12,
    "purchases": 144,
    "last_modified": "2026-03-19T14:23:11Z",
    "checksum": "a3f5b9c8..."
  },
  "errors": [
    {
      "timestamp": "2026-03-19T15:45:23Z",
      "type": "StorageError",
      "message": "Failed to update vendor",
      "stack_trace": "...",
      "user_action": "Editing vendor #7",
      "recovered": true
    }
  ],
  "performance": {
    "startup_time_ms": 342,
    "average_operation_ms": 15,
    "memory_usage_mb": 28
  },
  "features": {
    "wasm_enabled": true,
    "service_worker": false,
    "web_crypto": true,
    "file_system_access": true
  }
}
```

**Privacy Note:** Diagnostic report contains NO sensitive data:
- ✅ System information
- ✅ Error logs
- ✅ Performance metrics
- ❌ NO user data
- ❌ NO vendor names
- ❌ NO purchase details

### 2.3 Error Log Viewer

In-app error log with filtering:

```
Error Log (Last 7 Days)                [Export] [Clear]
─────────────────────────────────────────────────────
🔴 Critical: 0  🟡 Warning: 3  ℹ️ Info: 24

Filter: [All] [Critical] [Warning] [Info]   Search: ___

─────────────────────────────────────────────────────
2026-03-19 15:45:23  ⚠️  STORAGE
Failed to update vendor #7
→ Retried successfully after 150ms

2026-03-19 14:12:05  ℹ️  SYNC
Export completed: 156 records, 1.2 MB

2026-03-19 09:03:41  ⚠️  VALIDATION
Invalid commission rate (125%) corrected to 100%
```

---

## 3. Self-Service Support

### 3.1 In-App Help System

#### Help Panel (Slide-out)
```
┌─ Help ────────────────────────[x]
│
│ 🔍 Search help...
│ ─────────────────────────────
│
│ 📘 Getting Started
│    • Creating your first booth
│    • Adding vendors
│    • Recording purchases
│
│ 💾 Data Management
│    • Exporting data
│    • Importing data
│    • Switching browsers
│    • Backup strategies
│
│ 🖨️ Reports
│    • Vendor settlements
│    • Commission reports
│    • Printing reports
│
│ ⚙️ Settings
│    • Language preferences
│    • Commission rates
│    • Number formats
│
│ 🐛 Troubleshooting
│    • Common problems
│    • Error messages
│    • System diagnostics
│
│ [Contact Support] [View Changelog]
└──────────────────────────────
```

#### Context-Sensitive Help
Tooltips and help icons next to complex features:

```
Commission Rate (%)  [?]
┌────────────────────────────────┐
│ Default: 15%                   │
│                                │
│ This is the percentage you     │
│ charge vendors for each sale.  │
│                                │
│ Example: €100 sale @ 15%       │
│ = €15 commission               │
│                                │
│ [Learn more about commission]  │
└────────────────────────────────┘
```

### 3.2 FAQ Database

**Common Questions with Solutions:**

```typescript
const FAQ_DATABASE = [
  {
    id: "lost-data",
    question: "Where did my data go?",
    category: "data-management",
    answer: `If you're using a different browser or device, your data
             is still safe on your previous device. Follow these steps:
             
             1. On old device: Settings → Export Data
             2. Transfer file to new device
             3. On new device: Settings → Import Data
             
             Your data is stored locally in each browser.`,
    related: ["export-data", "import-data", "switching-browsers"]
  },
  {
    id: "storage-full",
    question: "Storage quota exceeded",
    category: "troubleshooting",
    answer: `Your browser's storage is full. Try these solutions:
             
             1. Clear browser cache (Settings → Privacy)
             2. Delete old booths (Booth Management → Archive)
             3. Use Chrome/Edge for larger storage limits
             
             Current usage: Check Settings → System Health`,
    related: ["browser-storage", "delete-booth"]
  },
  {
    id: "print-problems",
    question: "Reports not printing correctly",
    category: "reports",
    answer: `Ensure correct print settings:
             
             1. Browser print dialog: Settings
             2. Orientation: Portrait (for most reports)
             3. Margins: Default or Minimum
             4. Background graphics: Enabled
             
             Use Print Preview to verify before printing.`,
    related: ["vendor-reports", "print-settings"]
  }
];
```

### 3.3 Guided Tours

First-time user onboarding:

```
┌─────────────────────────────────────────┐
│ 👋 Welcome to ez-booth!              [x]│
│                                         │
│ Let's get you started with a quick     │
│ tour (2 minutes).                       │
│                                         │
│ [Start Tour] [Skip] [Remind me later]  │
└─────────────────────────────────────────┘

Step 1/5: Creating a Booth
────────────────────────────
│ A booth represents one event where     │
│ you manage vendor sales.               │
│                                        │
│    [+] ← Click here to create          │
│                                        │
│ [Previous] [Next] [Skip tour]          │
```

---

## 4. Error Recovery Mechanisms

### 4.1 Automatic Recovery

#### Storage Transaction Rollback
```rust
pub async fn safe_update<T>(
    &self,
    entity_id: &str,
    update_fn: impl FnOnce(&mut T),
) -> Result<T, StorageError> {
    let transaction = self.begin_transaction().await?;
    
    // Create savepoint
    let original = self.get(entity_id).await?.clone();
    
    match self.apply_update(entity_id, update_fn).await {
        Ok(updated) => {
            transaction.commit().await?;
            Ok(updated)
        }
        Err(e) => {
            // Automatic rollback
            self.restore(entity_id, original).await?;
            Err(e)
        }
    }
}
```

#### Optimistic Retry
```rust
pub async fn resilient_save<T>(
    &self,
    entity: &T,
    max_retries: u8,
) -> Result<(), StorageError> {
    let mut attempts = 0;
    let mut backoff_ms = 100;
    
    loop {
        match self.save(entity).await {
            Ok(()) => return Ok(()),
            Err(e) if attempts < max_retries && e.is_transient() => {
                attempts += 1;
                sleep(backoff_ms).await;
                backoff_ms *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 4.2 Manual Recovery

#### Data Repair Tool
```
Data Repair Wizard
─────────────────────────────────────────
We detected 3 issues with your data:

⚠️  Issue 1: Orphaned purchases (5 records)
    Purchases linked to deleted vendor #7
    
    [Reassign to vendor] [Delete purchases]

⚠️  Issue 2: Invalid commission rate
    Booth has commission rate of 125%
    
    [Reset to 15%] [Keep value]

⚠️  Issue 3: Missing booth reference
    Vendor "ABC Store" has no booth
    
    [Assign to current booth] [Delete vendor]

[Auto-Repair All] [Repair Selected] [Cancel]
```

#### Database Rebuild
```
Rebuild Database
─────────────────────────────────────────
⚠️  This will rebuild your database from
    scratch. Your data will be preserved.
    
Estimated time: 30 seconds

Steps:
1. Export all data as backup
2. Clear current database
3. Recreate tables
4. Re-import data
5. Verify integrity

[Start Rebuild] [Cancel]
```

---

## 5. Support Bundle Export

### 5.1 One-Click Export

**User Interface:**
```
Support & Diagnostics
─────────────────────────────────────────

Need help from our support team?

Download a support bundle that includes:
✅ System information
✅ Error logs
✅ Performance metrics
❌ No sensitive data (guaranteed)

This bundle helps us diagnose problems
quickly without needing your personal data.

[Download Support Bundle]

Then email the file to: support@ez-booth.com
or open a GitHub issue with the attachment.
```

### 5.2 Support Bundle Contents

**File:** `ez-booth-support-bundle-YYYYMMDD-HHMMSS.zip`

```
support-bundle/
├── diagnostic-report.json      # System health
├── error-log.json              # Recent errors
├── performance-metrics.json    # Performance data
├── system-info.json            # Browser & platform
├── storage-stats.json          # Storage usage
├── feature-flags.json          # Enabled features
└── README.txt                  # Instructions for support
```

**README.txt:**
```
ez-booth Support Bundle
Generated: 2026-03-19 16:18:00 UTC
Version: 0.1.0

This bundle contains diagnostic information to help
troubleshoot issues with ez-booth.

Privacy Notice:
NO sensitive data is included (no vendor names, sales
data, or personal information).

To get support:
1. Email this bundle to support@ez-booth.com
2. Or create a GitHub issue and attach this file
3. Include a description of your problem

Response time: Usually within 24 hours
```

---

## 6. Integration into Architecture

### 6.1 Update ARCHITECTURE.md

Add new section: **Error Handling & Diagnostics**

**Location:** After "Security & Privacy" section

**Content:**
- Error hierarchy diagram
- Error display UX mockups
- Diagnostic tools overview
- Recovery mechanism flowcharts

### 6.2 Update IMPROVEMENTS.md

Add to **User Experience** section:

**Current Problems (Java):**
- Generic Java stack traces shown to users
- No in-app help system
- No diagnostic tools
- Manual log file collection for support

**Proposed Solutions (Rust):**
- User-friendly error messages with recovery steps
- Built-in help system with search
- One-click diagnostic export
- Automatic error recovery

**Success Metrics:**
| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Support tickets (errors) | 10/month | 2/month | **80% reduction** |
| Time to diagnose issue | 30 min | 5 min | **6x faster** |
| Self-service resolution | 20% | 70% | **3.5x higher** |
| User error understanding | 30% | 90% | **3x better** |

### 6.3 Update IMPLEMENTATION.md

Add section: **10. Error Handling Implementation**

**Content:**
- Error type definitions (Rust code)
- Error display components (Leptos)
- Diagnostic API implementation
- Support bundle generation code
- Testing strategies for error scenarios

---

## 7. Priority & Effort

### 7.1 Feature Priority

| Feature | Priority | Phase | Effort | Rationale |
|---------|----------|-------|--------|-----------|
| User-friendly error messages | P0 | 2 | 8h | Critical UX |
| Error recovery (retry) | P0 | 2 | 4h | Reliability |
| In-app help system | P1 | 4 | 12h | Reduces support |
| Diagnostic report export | P1 | 4 | 6h | Support efficiency |
| Error log viewer | P2 | 5 | 4h | Power users |
| Data repair wizard | P2 | 6 | 8h | Advanced cases |
| Guided tours | P2 | 6 | 8h | Onboarding |
| Support bundle generation | P1 | 4 | 4h | Support essential |

**Total Effort:** 54 hours (~1.5 weeks)

### 7.2 Implementation Phases

**Phase 2 (MVP):**
- User-friendly error messages with translation
- Automatic retry for transient errors
- Basic error logging

**Phase 4 (Polish):**
- In-app help system with search
- Diagnostic report export
- Support bundle generation
- Context-sensitive help tooltips

**Phase 5 (Enhancement):**
- Error log viewer with filtering
- System health dashboard
- FAQ database

**Phase 6 (Advanced):**
- Data repair wizard
- Guided tours for new users
- Advanced diagnostics

---

## 8. Success Metrics

### 8.1 Support Efficiency

**Target Reductions:**
- Support tickets: 80% reduction (10/month → 2/month)
- Diagnostic time: 6x faster (30 min → 5 min)
- Support email exchanges: 50% reduction (3-4 → 1-2 per ticket)

**Measurement:**
- Track support ticket volume by category
- Measure time-to-resolution
- Survey user satisfaction after support interactions

### 8.2 User Self-Service

**Target Improvements:**
- Self-service resolution: 70% (vs. 20% current)
- Help system usage: 50% of users use help before contacting support
- Error recovery: 90% of transient errors recovered automatically

**Measurement:**
- Track help system searches
- Monitor automatic error recovery success rate
- Measure support ticket deflection rate

### 8.3 User Understanding

**Target Improvements:**
- Error message comprehension: 90% (vs. 30% current)
- Recovery action completion: 80% (users follow suggested steps)
- Support bundle usage: 60% of support tickets include bundle

**Measurement:**
- User surveys on error message clarity
- Track completion rate of suggested recovery actions
- Monitor support bundle attachment rate in tickets

---

## 9. Localization Considerations

All error messages, help content, and diagnostic reports must support German (primary) and English (fallback).

### 9.1 Error Message Keys

```json
// de.json
{
  "errors": {
    "validation": {
      "vendor_name_required": "Bitte geben Sie einen Händlernamen ein.",
      "commission_rate_invalid": "Provision muss zwischen 0% und 100% liegen.",
      "purchase_amount_required": "Bitte geben Sie einen Betrag ein."
    },
    "storage": {
      "save_failed": "Änderungen konnten nicht gespeichert werden.",
      "database_locked": "Datenbank ist gesperrt. Bitte versuchen Sie es erneut.",
      "quota_exceeded": "Speicherplatz voll. Bitte alte Daten archivieren."
    },
    "recovery": {
      "retry": "Erneut versuchen",
      "export_backup": "Backup exportieren",
      "contact_support": "Support kontaktieren"
    }
  },
  "help": {
    "getting_started": "Erste Schritte",
    "troubleshooting": "Problembehandlung",
    "faq": "Häufig gestellte Fragen"
  }
}
```

### 9.2 Help Content Translation

Help articles stored as separate markdown files per language:

```
docs/help/
├── de/
│   ├── getting-started.md
│   ├── switching-browsers.md
│   └── troubleshooting.md
└── en/
    ├── getting-started.md
    ├── switching-browsers.md
    └── troubleshooting.md
```

---

## 10. Testing Strategy

### 10.1 Error Scenario Tests

```rust
#[wasm_bindgen_test]
async fn test_storage_failure_recovery() {
    let storage = create_failing_storage();
    let result = resilient_save(&storage, &vendor, 3).await;
    
    assert!(result.is_ok(), "Should recover after retry");
    assert_eq!(storage.attempt_count(), 2, "Should retry once");
}

#[wasm_bindgen_test]
async fn test_user_error_message() {
    let error = AppError::Validation(ValidationError {
        field: "vendor_name",
        message: "errors.validation.vendor_name_required",
        suggestion: Some("errors.validation.enter_valid_name"),
    });
    
    let message = error.user_message(Locale::De);
    assert!(message.contains("Händlernamen"));
}
```

### 10.2 Diagnostic Export Tests

```rust
#[wasm_bindgen_test]
async fn test_diagnostic_export_no_sensitive_data() {
    let diagnostic = generate_diagnostic_report().await;
    
    // Verify no sensitive data
    assert!(!diagnostic.contains_vendor_names());
    assert!(!diagnostic.contains_purchase_data());
    assert!(!diagnostic.contains_personal_info());
    
    // Verify required data present
    assert!(diagnostic.has_browser_info());
    assert!(diagnostic.has_error_log());
    assert!(diagnostic.has_system_health());
}
```

---

## 11. Related Documents

- **ARCHITECTURE.md** - Add Error Handling & Diagnostics section
- **IMPROVEMENTS.md** - Update User Experience metrics
- **IMPLEMENTATION.md** - Add Error Handling Implementation section

---

## Appendix: Error Message Examples

### German (Primary)
```
✅ "Händler erfolgreich gespeichert."
⚠️ "Provision muss zwischen 0% und 100% liegen."
❌ "Änderungen konnten nicht gespeichert werden. Versuchen Sie es erneut."
🔴 "Kritischer Fehler. Bitte laden Sie einen Support-Bundle herunter."
```

### English (Fallback)
```
✅ "Vendor saved successfully."
⚠️ "Commission must be between 0% and 100%."
❌ "Failed to save changes. Please try again."
🔴 "Critical error. Please download a support bundle."
```

---

**End of Document**
