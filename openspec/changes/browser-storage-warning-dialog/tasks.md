# Tasks: browser-storage-warning-dialog

## 1. localStorage Utilities

- [ ] 1.1 Add `get_storage_warning_dismissed_at() -> Option<DateTime<Utc>>` helper that reads `ez-booth-storage-warning-dismissed-at` from localStorage, returning `None` if absent or unreadable (try/catch wrapper)
- [ ] 1.2 Add `set_storage_warning_dismissed_at(now: DateTime<Utc>)` helper that writes the ISO 8601 timestamp to `ez-booth-storage-warning-dismissed-at`, silently ignoring write failures
- [ ] 1.3 Add `should_show_storage_warning() -> bool` function that returns `true` when the key is absent or the elapsed time since dismissal is >= 30 days

## 2. Storage Quota API Binding

- [ ] 2.1 Wire `navigator.storage.estimate()` via `web-sys` — return a `(used: u64, quota: u64)` tuple from an async Rust function `estimate_storage_quota() -> Option<(u64, u64)>`
- [ ] 2.2 Add human-readable byte formatting utility (e.g., `4.2 MB`, `500 MB`) for display in the dialog

## 3. Dialog Component

- [ ] 3.1 Create `StorageRiskWarningDialog` Dioxus component in `crates/ez-booth-ui/src/components/storage_risk_warning_dialog.rs`
- [ ] 3.2 Implement full-screen backdrop overlay that blocks pointer events on the underlying UI
- [ ] 3.3 Implement the summary tier: headline + at most 3 one-line bullets conveying the critical risk (no quota figures here)
- [ ] 3.4 Implement the "Show details" disclosure toggle and collapsible details tier containing quota benchmarks and deeper browser storage explanation; defaults to collapsed
- [ ] 3.5 Add Safari/iOS variant: one-line bullet in the summary tier + expanded ITP eviction explanation in the details tier, both rendered conditionally via `detect_browser()`
- [ ] 3.5 Add "I Understand" CTA button that calls `set_storage_warning_dismissed_at()` and closes the dialog; ensure no other dismiss path exists (no ESC, no backdrop click, no X button)
- [ ] 3.6 On mount, fire the async `estimate_storage_quota()` call and update a local signal with the result; render loading spinner until resolved or failed

## 4. Root Integration

- [ ] 4.1 In the app shell / root component, evaluate `should_show_storage_warning()` on startup and store the result in a reactive signal
- [ ] 4.2 Conditionally render `<StorageRiskWarningDialog>` at the root level when the signal is `true`, overlaying all other content
- [ ] 4.3 Wire the dialog's on-dismiss callback to set the signal to `false` so the dialog disappears after confirmation

## 5. Browser Detection Refactor (if needed)

- [ ] 5.1 If `detect_browser()` / `is_safari()` are private to `storage_warning.rs`, extract them to a shared module (e.g., `crates/ez-booth-ui/src/browser.rs`) and update existing callers
- [ ] 5.2 Import the shared detection function in `StorageRiskWarningDialog`

## 6. Verification

- [ ] 6.1 Manually test: clear localStorage, launch app — dialog appears, blocks interaction, shows quota figures
- [ ] 6.2 Manually test: dismiss dialog — timestamp written, relaunch within 30 days — dialog does not appear
- [ ] 6.3 Manually test: artificially set `ez-booth-storage-warning-dismissed-at` to 31 days ago — dialog reappears on launch
- [ ] 6.4 Manually test Safari (or UA-spoof): Safari warning section is visible; on other browsers it is absent
- [ ] 6.5 Manually test: simulate localStorage unavailability (private mode or mock) — dialog appears, app does not crash
- [ ] 6.6 Run `cargo check` and WASM build — no new warnings or errors
