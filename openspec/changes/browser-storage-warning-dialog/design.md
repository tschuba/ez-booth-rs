## Context

ez-booth stores all event data (booths, vendors, purchases) in IndexedDB via the browser's local storage. There is zero server-side backup — data loss is permanent and unrecoverable. The existing `StorageIndicator` footer hints at risk, but only surfaces after data already exists and requires deliberate attention to read. New users are never formally warned before they begin entering data.

The system runs as a WASM app in the browser. State persistence is done through IndexedDB and a small number of `localStorage` keys. The UI crate uses Dioxus (Rust/WASM reactive framework). All browser API calls go through `web-sys`.

## Goals / Non-Goals

**Goals:**
- Show a mandatory, full-screen blocking modal on first app launch (no existing `ez-booth-storage-warning-dismissed-at` key in localStorage)
- Re-show the dialog if more than 30 days have elapsed since last dismissal
- Display live storage quota figures (used bytes, total quota) from `navigator.storage.estimate()`
- Show an elevated Safari/iOS-specific variant using the same `detect_browser()` logic already in `storage_warning.rs`
- Record dismissal timestamp to localStorage on explicit user confirmation

**Non-Goals:**
- Replacing or modifying the footer `StorageIndicator`
- Server-side session tracking or analytics around dismissal
- Forcing users to take a backup before using the app (only informing them)
- Supporting a "never show again" opt-out beyond the 30-day recurrence

## Decisions

### 1. Persistence key: `localStorage` timestamp

**Decision:** Store `ez-booth-storage-warning-dismissed-at` as an ISO 8601 string in `localStorage`.

**Alternatives considered:**
- IndexedDB: Overkill for a single flag; adds async complexity at startup
- In-memory only (session): Dialog would reappear on every page load, defeating the purpose

**Rationale:** Consistent with existing app patterns (`ez-booth-selected-booth-id`, pagination preferences). Synchronous read at startup avoids async gating.

### 2. Show threshold: 30 days

**Decision:** Re-show if `now - dismissed_at >= 30 days`.

**Rationale:** Long enough not to annoy active users (who get frequent reminders via the footer), short enough to recapture returning users who may have forgotten the risk. Matches the non-Safari backup overdue threshold already used in the footer.

### 3. Storage quota via `navigator.storage.estimate()`

**Decision:** Call `navigator.storage.estimate()` asynchronously on dialog mount; show a loading state until resolved, then render the figures.

**Alternatives considered:**
- Skip quota display: Loses the "benchmark" requirement
- Hardcode known limits: Inaccurate across devices and OS versions

**Rationale:** The Storage API is available in all modern browsers. On Safari it still returns estimates (though subject to ITP restrictions). Async fetch keeps the dialog non-blocking for the rest of mount logic.

### 4. Safari detection: reuse `detect_browser()`

**Decision:** Import and call the existing `detect_browser()` from `storage_warning.rs` (or extract to a shared module) to determine the Safari/iOS variant.

**Rationale:** Single source of truth for browser detection; no duplicate UA parsing logic.

### 5. Dialog placement: root-level overlay

**Decision:** Render the dialog in the app shell / root component as an overlay above all content. The rest of the UI renders underneath but is covered by a backdrop that blocks interaction.

**Alternatives considered:**
- Render only in a specific route/page: Dialog would not appear if user navigates directly to a sub-route
- Separate route/page for onboarding: Requires navigation changes, more complex

**Rationale:** Root-level placement guarantees the dialog appears regardless of entry URL. Dioxus supports conditional rendering at the root level.

### 6. Dismiss action: single CTA button

**Decision:** One "I Understand" button. No secondary "Remind me later" or close-X affordance — the user must read and actively confirm.

**Rationale:** The dialog's purpose is to ensure awareness, not to be convenient to skip. Removing escape hatches (ESC key, backdrop click, X button) reinforces that this is a required acknowledgment.

### 7. Progressive disclosure: summary + collapsible details

**Decision:** The dialog presents a two-tier layout. The summary tier (always visible) contains a headline and at most 3 one-line bullets — readable in under 10 seconds. A "Show details" disclosure control reveals a second tier with storage quota benchmarks and deeper explanations.

**Alternatives considered:**

- Show everything at once: Users faced with a wall of text are more likely to dismiss without reading — the opposite of the intent.
- Hide all technical detail: Loses the benchmark requirement and gives power users no path to understand the concrete risk.

**Rationale:** Progressive disclosure lets casual users get the key message quickly while preserving access to the technical detail for users who want it. The Safari-specific bullet appears in both tiers: as a one-liner in the summary, with full ITP explanation in the details.

## Risks / Trade-offs

- **`navigator.storage.estimate()` reliability on Safari** → The Storage API is supported on Safari 15.2+ but quota figures may be lower or less precise due to ITP. Mitigation: show the numbers but add a note that Safari may report reduced capacity.
- **localStorage unavailable in private/incognito mode** → `localStorage` throws in some private-mode contexts. Mitigation: wrap read/write in try/catch; if unavailable, treat as "never dismissed" (show the dialog) and skip writing the timestamp.
- **WASM bundle size** → Adding `navigator.storage` web-sys bindings adds a small amount of generated glue. Mitigation: negligible impact; `web_sys` is already a dependency.
- **User annoyance for active users** → Users who launch the app daily will never see the dialog again after first dismissal (30-day threshold). Mitigation: this is the desired behavior; footer provides ongoing reminders.

## Migration Plan

No data migration required. The new localStorage key is absent for all existing users, so the dialog will appear once for everyone on the first build that includes this feature — intentional, as this is the desired re-education behavior.

Rollback: Remove the dialog component and its root-level render call. The localStorage key is harmless if left behind.

## Open Questions

- Should the dialog include a direct "Create backup now" CTA alongside "I Understand"? (Would link to the footer's export functionality.)
- Should the 30-day threshold be a compile-time constant or configurable at runtime (e.g., via a feature flag)?
