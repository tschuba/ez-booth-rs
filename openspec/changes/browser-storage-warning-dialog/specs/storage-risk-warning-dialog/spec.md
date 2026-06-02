## ADDED Requirements

### Requirement: First-use warning dialog
The system SHALL display a full-screen, blocking modal dialog on every app launch when no dismissal timestamp has been recorded in localStorage (key: `ez-booth-storage-warning-dismissed-at`).

#### Scenario: First launch with no dismissal record
- **WHEN** the app starts and `ez-booth-storage-warning-dismissed-at` is absent from localStorage
- **THEN** the storage risk warning dialog MUST appear as a full-screen overlay before any app content is usable

#### Scenario: Subsequent launch within threshold
- **WHEN** the app starts and `ez-booth-storage-warning-dismissed-at` is present and fewer than 30 days have elapsed since that timestamp
- **THEN** the dialog MUST NOT appear and the app proceeds normally

### Requirement: Periodic re-display after 30 days
The system SHALL re-display the warning dialog if 30 or more days have elapsed since the last recorded dismissal timestamp.

#### Scenario: Launch after 30-day silence
- **WHEN** the app starts and `ez-booth-storage-warning-dismissed-at` is present and 30 or more days have elapsed since that timestamp
- **THEN** the storage risk warning dialog MUST appear as a full-screen overlay

#### Scenario: Launch at exactly 30 days
- **WHEN** exactly 30 days (to the millisecond) have elapsed since dismissal
- **THEN** the dialog MUST appear (boundary is inclusive)

### Requirement: Active dismissal required
The dialog SHALL require an explicit user action to dismiss. No passive dismiss path (ESC key, backdrop click, or close button) SHALL be available.

#### Scenario: User confirms understanding
- **WHEN** the user clicks the "I Understand" confirmation button
- **THEN** the dialog MUST close and the current UTC timestamp MUST be written to `ez-booth-storage-warning-dismissed-at` in localStorage

#### Scenario: ESC key pressed while dialog is open
- **WHEN** the dialog is visible and the user presses the ESC key
- **THEN** the dialog MUST remain open and no dismissal timestamp SHALL be recorded

#### Scenario: Backdrop click while dialog is open
- **WHEN** the dialog is visible and the user clicks outside the dialog panel
- **THEN** the dialog MUST remain open and no dismissal timestamp SHALL be recorded

### Requirement: Storage quota benchmark display
The dialog SHALL display live storage capacity figures retrieved from `navigator.storage.estimate()` so the user can see the concrete quota constraints of their browser.

#### Scenario: Storage estimate resolves successfully
- **WHEN** the dialog is displayed and `navigator.storage.estimate()` resolves
- **THEN** the dialog MUST show used storage bytes and total quota bytes in a human-readable format (e.g., "4.2 MB used of 500 MB available")

#### Scenario: Storage estimate not yet resolved
- **WHEN** the dialog is displayed and `navigator.storage.estimate()` has not yet resolved
- **THEN** the dialog MUST show a loading indicator in place of the quota figures until the promise resolves

#### Scenario: Storage estimate fails or is unavailable
- **WHEN** `navigator.storage.estimate()` rejects or is unavailable in the browser
- **THEN** the dialog MUST still be displayed without the quota section (graceful degradation), and no error SHALL be surfaced to the user

### Requirement: Safari / iOS elevated warning variant
The dialog SHALL display an elevated, Safari-specific warning section when the current browser is detected as Safari or iOS WebKit, using the same detection logic (`detect_browser()`) already used by the footer component.

#### Scenario: Safari browser detected
- **WHEN** the dialog is shown and `detect_browser()` returns `"Safari"`
- **THEN** the dialog MUST include a dedicated Safari warning section explaining the 7-day eviction policy and the elevated risk of data loss compared to other browsers

#### Scenario: Non-Safari browser detected
- **WHEN** the dialog is shown and `detect_browser()` does not return `"Safari"`
- **THEN** the dialog MUST NOT display the Safari-specific section

### Requirement: Resilience when localStorage is unavailable
The system SHALL handle environments where `localStorage` is inaccessible (e.g., private/incognito mode) without crashing.

#### Scenario: localStorage read fails at startup
- **WHEN** the app starts and reading `ez-booth-storage-warning-dismissed-at` from localStorage throws an exception
- **THEN** the system MUST treat the state as "never dismissed" and show the dialog

#### Scenario: localStorage write fails on dismissal
- **WHEN** the user confirms the dialog and writing `ez-booth-storage-warning-dismissed-at` throws an exception
- **THEN** the dialog MUST still close and the app MUST continue normally; the failed write MUST be silently swallowed
