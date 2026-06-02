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

### Requirement: Concise summary layout

The dialog SHALL present information in two tiers to prevent cognitive overload and reduce the chance of dismissal without reading. The summary tier MUST be readable in under 10 seconds.

#### Scenario: Dialog opens for the first time

- **WHEN** the dialog is displayed
- **THEN** only the summary tier SHALL be visible: a short headline and at most 3 one-line bullet points conveying the critical risk
- **THEN** a "Show details" toggle or section MUST be present but collapsed

#### Scenario: User expands details

- **WHEN** the user activates the "Show details" control
- **THEN** the details tier MUST expand in-place beneath the summary, showing storage quota benchmarks, a fuller explanation of browser storage mechanics, and (if Safari) the elevated eviction risk explanation

#### Scenario: Details section defaults to collapsed

- **WHEN** the dialog is displayed on any subsequent appearance (recurrence after 30 days)
- **THEN** the details tier MUST default to collapsed, not expanded

### Requirement: Storage quota benchmark display

The storage quota figures (used bytes, total quota from `navigator.storage.estimate()`) SHALL be shown inside the collapsible details tier only — not in the summary.

#### Scenario: Storage estimate resolves and details are expanded

- **WHEN** the user expands the details tier and `navigator.storage.estimate()` has resolved
- **THEN** the details MUST show used storage and total quota in a human-readable format (e.g., "4.2 MB used of 500 MB available")

#### Scenario: Storage estimate not yet resolved when details are expanded

- **WHEN** the user expands the details tier before `navigator.storage.estimate()` resolves
- **THEN** a loading indicator MUST appear in place of the quota figures until the promise settles

#### Scenario: Storage estimate fails or is unavailable

- **WHEN** `navigator.storage.estimate()` rejects or is unavailable
- **THEN** the details tier MUST still render without the quota figures; no error SHALL be surfaced to the user

### Requirement: Safari / iOS elevated warning variant

When the current browser is detected as Safari or iOS WebKit (via `detect_browser()`), the dialog SHALL include a Safari-specific warning in both tiers.

#### Scenario: Safari detected — summary tier

- **WHEN** the dialog is shown and `detect_browser()` returns `"Safari"`
- **THEN** one of the summary bullet points MUST flag the 7-day eviction risk in a single, scannable line

#### Scenario: Safari detected — details tier

- **WHEN** the user expands the details tier on a Safari browser
- **THEN** the details MUST include a fuller explanation of Safari's ITP-driven eviction policy and its concrete implications for data loss

#### Scenario: Non-Safari browser detected

- **WHEN** the dialog is shown and `detect_browser()` does not return `"Safari"`
- **THEN** no Safari-specific content SHALL appear in either tier

### Requirement: Resilience when localStorage is unavailable
The system SHALL handle environments where `localStorage` is inaccessible (e.g., private/incognito mode) without crashing.

#### Scenario: localStorage read fails at startup
- **WHEN** the app starts and reading `ez-booth-storage-warning-dismissed-at` from localStorage throws an exception
- **THEN** the system MUST treat the state as "never dismissed" and show the dialog

#### Scenario: localStorage write fails on dismissal
- **WHEN** the user confirms the dialog and writing `ez-booth-storage-warning-dismissed-at` throws an exception
- **THEN** the dialog MUST still close and the app MUST continue normally; the failed write MUST be silently swallowed
