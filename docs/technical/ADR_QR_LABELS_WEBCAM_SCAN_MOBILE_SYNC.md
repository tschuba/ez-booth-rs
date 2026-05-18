---
title: ADR QR Labels, Webcam Scan & Mobile Sync
nav_order: 6
parent: Technical Docs
---

# ADR: QR Labels, Webcam Scan & Mobile Sync (Phases 1–5)

Date: 2026-05-13
Status: Accepted
Decision Maker: ez-booth-rs maintainers

## Context

The current checkout flow requires the cashier to manually enter a vendor ID and amount for every item. At flea markets this slows down the queue and introduces transcription errors.

Three improvements address this:

1. **Vendors pre-label their items** with QR-code stickers before the event. Each QR code encodes vendor ID and price.
2. **Cashiers scan labels** via a USB webcam instead of typing.
3. **Helpers scan items on a phone** in the room and merge data with the register at the end of the event.

An additional phase adds camera-based recognition of handwritten price labels (OCR) for vendors who do not use QR stickers.

A fifth phase introduces a dedicated organizer app for pre-event vendor management.

All phases must maintain the offline-first guarantee: the core checkout flow must work without internet access.

---

## Deployment Scenarios

All features must function across three deployment models:

| Scenario | Description |
| --- | --- |
| Local only | Veranstalter runs the app locally via launcher or browser. No internet dependency at the event. |
| Static hosting | WASM bundle hosted on Netlify / GitHub Pages. Vendors open the label link on their own device. |
| With server (Coolify) | Optional self-hosted backend. Enables seamless server sync for mobile devices. |

---

## Key Architectural Decisions

### 1. Separate WASM bundles per audience

**Decision:** Each target audience gets its own independent Leptos WASM crate with its own deployment URL.

```text
crates/ez-booth-app/       → Kassen-App    (cashier, stationary)
crates/ez-booth-labels/    → Label-App     (vendor, prints stickers)
crates/ez-booth-mobile/    → Mobile-App    (helper, scans with phone)
crates/ez-booth-organizer/ → Organizer-App (organizer, Phase 5)
crates/domain/             → shared (models, services, QrLabelPayload)
crates/ez-booth-ui/        → UI components (Kassen-App only)
```

`ez-booth-mobile` depends **only** on `crates/domain`, not on `ez-booth-ui`. The mobile app reimplements UI components minimally to avoid dragging in checkout-specific code.

**Rationale:** Separate bundles keep download size small for each audience, prevent accidental cross-boundary dependencies, and allow independent deployment per feature phase.

---

### 2. QR payload format

**Decision:** `v={vendor_id}&p={price_cents}` — URL query parameter format, percent-encoded.

Example: `v=42&p=300` for vendor 42 at €3.00.

**Rationale:** Percent-encoding handles arbitrary vendor IDs safely. The format is extensible without a breaking change. Rust parses it via `url::form_urlencoded`.

The QR code printed on stickers always contains this plain-text format.

---

### 3. Label link URL format

**Decision:** Label links use a plain URL format. No obfuscation is applied.

Format: `{labels-url}/?v={vendor_id}&e={event_code}`

The label app reads both parameters client-side and displays the vendor ID. On error (missing parameters), a user-friendly message is shown.

**Rationale:** The previous design used XOR obfuscation with a key that was distributed openly in onboarding QR codes and booth exports, making the obfuscation cosmetic. Removing it eliminates complexity with no real security loss.

> **Note — label link vs. sticker QR:** These are two distinct formats. The label link (`?v=&e=`) is what vendors open in a browser to configure and print their stickers. The QR payload (`v=&p=`, Section 2) is what the Label-App encodes *onto* the printed sticker. One navigates to the Label-App; the other is scanned at the register during checkout.

---

### 4. `event_code` as cross-device event identifier

**Decision:** The `Booth` model gains a mandatory `event_code: String` field. This is the cross-device event identifier — shared across all registers and mobile devices for the same event. `booth.id` (UUID) remains device-local and is used for per-register accounting only.

**Derivation:** Auto-generated at event creation from name + date as a starting suggestion.

Format: `{initials}-{MMYY}` — initials of significant words + dash + zero-padded month + 2-digit year. **The MMYY component is derived from the event's start date as entered by the cashier, not the current system date.**

Rules:

- Transliterate umlauts: ä→A, ö→O, ü→U, ß→S
- Strip non-alphanumeric characters
- Skip words whose normalized form is ≤ 3 chars (language-agnostic; catches articles and prepositions without a hardcoded list)
- Take the first letter (uppercase) of each remaining significant word, up to 4 initials
- If only one significant word remains, take its first 2 chars instead of 1
- Date: zero-padded month + 2-digit year (MMYY)
- Separator: `-` between initials and date (prevents letter/number ambiguity for months 01–09)
- **Fallback:** If no significant words remain after filtering (every word normalizes to ≤ 3 chars), use the first 2 characters of the first word (normalized) as the initial component

Examples: `"Flohmarkt Mai 2026"` → `FM-0526`, `"Großer Herbstmarkt"` → `GH-1026`, `"Markt am Rathaus"` → `MR-0526`

The derived code is displayed prominently during event creation and is editable. The cashier confirms or adjusts it before saving.

**Leading register:** The first Kassen-App to create the event generates the `event_code`. All other registers adopt it — either by scanning a QR from the leading register (camera-enabled devices) or by typing it manually (camera-less devices). "Leading register" is an organizational convention, not a technical enforcement.

The QR that non-leading registers scan is the same mobile onboarding QR (see "Usage in sync" below). When a Kassen-App scans this QR it pre-fills the `event_code` field in the event creation form; the cashier reviews and confirms.

**Event creation UI flow (same on every register):**

1. Cashier enters event name + date
2. System suggests `event_code` (derived)
3. Code is displayed and editable — cashier adjusts to match the leading register if needed
4. Cashier confirms → event is created

**Lock after first sync.** After the first sync payload is received (file import or server GET), the Kassen-App locks `event_code` editing. Before any sync, a strong warning is shown when the code is changed: *"Changing the event code breaks any mobile device that has already configured this event."* The system cannot detect whether a remote device has already onboarded while offline — the lock triggers only on confirmed sync activity.

**Usage in sync:**

- The Kassen-App shows a QR code for mobile onboarding. Encoding: `ez-booth://onboard?e={event_code}&n={url_encoded_event_name}`. This same QR is scanned by other registers to pre-fill their `event_code`.
- The Mobile-App stores `event_code` per configured event in IndexedDB.
- Purchases synced from mobile carry `event_code` in their JSON payload.
- Any Kassen-App with matching `event_code` can accept the sync — helpers are not locked to one register.
- On import/sync: the Kassen-App resolves the local booth via `event_code`.

**Server side:** `events.booth_id` is set to `NULL` for mobile-synced purchases (the server has no booth registry). The `event_code` is stored in the `payload` JSONB field.

**Phase 5 note:** The Organizer-App generates `event_code` centrally. On import into a Kassen-App, `event_code` is preserved.

---

### 5. QR decoding in WASM — `rxing` crate

**Decision:** Use the `rxing` crate (pure Rust, no JS/npm dependency) for QR code decoding in the browser.

**Risk:** WASM compilation and in-browser performance must be verified via a spike before Phase 2 implementation. Fallback: `ZXing-wasm` via `wasm-bindgen` if `rxing` is not viable.

**Frame loop:** ~10 fps (every 100ms) to limit CPU usage. Frame extraction crops to the viewfinder area only, reducing false positives from background patterns.

---

### 6. Dedup timing in WASM

**Decision:** Use `web_sys::Performance::now()` (returns `f64` milliseconds) instead of `std::time::Instant`.

**Rationale:** `std::time::Instant` is not available in WASM environments.

```rust
last_scanned: HashMap<String, f64>
// Same QR code within 2000ms → ignore
```

---

### 7. Item source tracking

**Decision:** Every `CheckoutItem` and `StoredCheckoutItem` carries a `source: ItemSource` field.

```rust
enum ItemSource { Manual, Scanned }
```

Scanned items show a 📷 icon in the cart. The `StoredCheckoutItem` serialization includes `source` so the icon persists across browser reloads.

---

### 8. Input modes

**Decision:** The checkout page tracks an `InputMode` enum:

```rust
enum InputMode { Manual, Scan }  // Phase 2
// OcrScan added in Phase 4
```

Phase 4 extends this with `OcrScan` for handwritten label recognition.

---

### 9. Storage strategy per app

| App | Storage | Reason |
| --- | --- | --- |
| Kassen-App | `localStorage` | Small dataset, synchronous access sufficient |
| Mobile-App | `IndexedDB` | Larger dataset, async API, survives longer sessions |

The Kassen-App uses two `localStorage` keys: `StoredCheckoutItem` for the active cart, and `completed_purchases` for UUID deduplication of imported items (distinct from the active cart).

---

### 10. Purchase deduplication

**Decision:** Every purchase carries a UUID v4 generated locally at creation time.

- Collision probability across devices: practically zero (122 random bits)
- Server deduplication: partial unique index on the `events` table:

```sql
CREATE UNIQUE INDEX idx_events_purchase_dedup
  ON events (entity_id) WHERE event_type = 'purchase_upserted';
```

- Server insert: `INSERT ... ON CONFLICT (entity_id) WHERE event_type = 'purchase_upserted' DO NOTHING`
- File import on Kassen-App: UUID check against `completed_purchases` in localStorage; duplicates skipped silently. `StoredCheckoutItem` is the active cart — deduplication uses a separate completed-purchases key.
- Vendor auto-creation on import: if an imported purchase references a `vendor_id` not in the local database, the Kassen-App auto-creates the vendor via `get_or_create` using the booth resolved from `event_code`.
- The dedup index is global across all events on the server (no `event_code` column). This is intentional: UUID v4 collision probability is effectively zero, and the system targets single-organizer deployments where cross-event dedup is unnecessary.

---

### 11. Mobile sync: API endpoints and file export

**POST /api/sync** (upload — Coolify only):

```http
POST /api/sync
Body: { purchases: [...], client_id: "device-uuid" }

Each purchase object includes:
{ id, vendor_id, price_cents, occurred_at, event_code, ... }
```

The `client_id` is a UUID v4 generated on first app start, stored in IndexedDB as a stable device ID.

**GET /api/sync** (download — Coolify only):

```text
GET /api/sync?since={last_sequence}
Response: { purchases: [...], next_sequence: 42 }
```

Returns all `purchase_upserted` events with `sequence > since`, up to 500 per request. The Kassen-App stores `last_sync_sequence` in localStorage as a cursor. The "Sync" button triggers POST (upload) + GET (download) in one step.

**File export/import** (always available, all deployment scenarios):

The Mobile-App provides an "Export" button that downloads pending purchases as a `.json` file. Transfer via USB, AirDrop, email, or shared folder. The Kassen-App imports it via a file picker. Both paths use the same UUID deduplication as server sync.

---

### 12. Mobile: Multi-Event support

**Decision:** The Mobile-App supports multiple configured events simultaneously.

- Events are added via QR code scan from the Kassen-App (which shows `event_code` + event name).
- The helper selects the active event from an event list in the Mobile-App.
- Each event has its own purchase list in IndexedDB.
- Events can be removed individually; a warning is shown if there are unsynchronized purchases.

Purchases are grouped into sync batches at upload time. Each batch has a UUID and a displayed status (pending / synced). Batches already synced via server or file export are not re-uploaded.

---

### 13. Mobile PWA + offline support

- `manifest.json` for PWA install prompt — enables Add to Home Screen on Android and iOS (`name`, `icons`, `display: standalone`).
- Service Worker (manually written `sw.js`) with Cache-First strategy for all WASM/JS/CSS/HTML assets.
- Cache name is versioned — updating the cache name activates new releases immediately via `skipWaiting()` + `clients.claim()`.
- The OCR model (~15MB, Phase 4) is **not** included in the SW precache. Tesseract.js caches it independently via the browser Cache API or IndexedDB after the first explicit user-initiated download.
- The Scan loop pauses via the Page Visibility API when the app is backgrounded to conserve battery.

**Known iOS limitation:** Safari purges SW cache and IndexedDB after ~7 days of inactivity. Helpers should open the app shortly before the event to refresh the cache.

---

### 14. Phase 4 — Handwritten label OCR

**Decision:** Tesseract.js (JavaScript library, ~15MB model) for client-side OCR.

**Risk:** Tesseract.js requires JS interop from Leptos via `wasm-bindgen`/`js-sys`. A spike is required to verify that this works in the Leptos WASM environment without blocking the UI thread (a Web Worker may be necessary). Unlike Phase 2 which has a `ZXing-wasm` fallback, there is no alternative client-side library for OCR: if Tesseract.js interop is not feasible, Phase 4 is deferred until a pure-Rust OCR crate is viable, or limited to server-only OCR (Coolify only).

Recognition flow (confidence-based):

- Confidence ≥ 85% + validation OK → direct acceptance, no dialog
- Confidence < 85% or validation error → confirmation dialog shown to cashier

An optional server-side OCR endpoint (`POST /api/ocr`, Phase 4, Coolify only) offloads processing to the server when activated in settings.

---

### 15. Phase 5 — Organizer-App `event_code` generation

When the Organizer-App creates an event, it derives the `event_code` client-side using the same name+date derivation rules as the Kassen-App. The `event_code` is part of the exported JSON. On import into a Kassen-App, the `event_code` is preserved so label links and mobile onboarding remain consistent.

---

### 16. Label-App: minimum label size enforcement

**Decision:** The Label-App validates custom label dimensions against two thresholds:

- **Absolute minimum — enforced, printing blocked:** The QR code region must be ≥ 25×25 mm. Below this threshold, reliable decoding by a typical smartphone camera is not guaranteed. If the label is too small to fit a 25×25 mm QR code plus minimal text, the app blocks printing and shows an error.
- **Suggested minimum — warning only:** Labels below the suggested safe size (approx. 40×25 mm total) show a warning but are not blocked. The vendor may proceed and accept the scanning risk.

Custom dimensions entered by the vendor trigger real-time validation on every change. The three preset sizes (Klein 48×30 mm, Mittel 64×34 mm, Groß 70×50 mm) all exceed the suggested minimum and require no warning.

**Rationale:** Silent scan failures at the event are worse than a rejected print job. Enforcing the absolute minimum eliminates a class of silent errors; the suggested-minimum warning covers borderline cases where vendor judgement is sufficient.

---

## Consequences

### Positive

- Checkout throughput improves significantly with QR scan support.
- Offline-first guarantee is maintained across all phases and deployment scenarios.
- No vendor-side app required — vendors print labels from any browser.
- Pure Rust QR decoding avoids npm dependencies (subject to `rxing` spike).
- UUID-based deduplication is collision-safe across devices without a coordination server.
- Multi-event support on Mobile gives helpers full flexibility without data loss on event switching.

### Negative / Trade-offs

- `ez-booth-mobile` must reimplement UI components that already exist in `ez-booth-ui`, accepting code duplication in exchange for clean separation.
- `rxing` WASM compatibility is unverified — Phase 2 must begin with a spike.
- Tesseract.js JS interop is a risk for Phase 4 — may require a Web Worker workaround.
- Two registers independently creating the same event may derive different `event_code` values if event names differ slightly — cashiers must compare and manually align codes before the event starts.
- Changing `event_code` after mobile devices have onboarded breaks sync for those devices — the UI must warn or prevent this.
- iOS Safari's cache eviction policy requires helpers to refresh the PWA before each event.
- Camera-less registers can join an event by manually typing the `event_code` — no file import or QR scanner required.
- Any register with matching `event_code` can accept a mobile sync — helpers are not locked to one register.
- `POST /api/sync` is intentionally unauthenticated. The `event_code` provides minimal access control, appropriate for low-stakes, single-organizer, private deployments. Shared multi-tenant server deployments require additional authentication.
- `event_code` uniqueness is not enforced server-side. Two organizers on the same Coolify instance with similar event names in the same month could derive identical codes, interleaving their synced purchases. Separate organizers must use dedicated server instances.

---

## Not in Scope (all phases)

- Bluetooth sync (insufficient browser support)
- Server-side PDF generation
- Vendor login / authentication
- Event code / ID embedded in **item sticker** QR codes — per-item stickers encode only `vendor_id` and `price_cents`; the event is not bound to the sticker (the mobile onboarding QR does carry `event_code`, but that is a separate QR used only for device setup)
- Real-time relay between phone and register (replaced by batch sync)
- Digital payment (cash-only remains)
- **Remote shopping cart ("Send to register"):** Physically insecure — there is no way to guarantee that items scanned in a side room actually reach the register without being pocketed. The system cannot close this gap; it is therefore excluded entirely.

---

## Affected Files

| File | Change |
| --- | --- |
| `crates/domain/src/models/qr_label.rs` | New — `QrLabelPayload` encode/decode (plain, no XOR) |
| `crates/domain/src/models/mod.rs` | Add `qr_label` module |
| `crates/domain/src/models/booth.rs` | Add `event_code: String` (derived from name+date, editable); requires migration for existing booths |
| `crates/ez-booth-server/migrations/0003_booth_event_code.sql` | New — add `event_code` column; backfill existing rows or leave nullable with UI prompt |
| `crates/domain/src/services/vendor_service.rs` | Add `get_or_create(vendor_id, booth_id)` |
| `crates/ez-booth-labels/` | New — standalone Label-App crate |
| `crates/ez-booth-mobile/` | New — standalone Mobile-Scan crate (includes `sw.js`, `manifest.json`) |
| `crates/ez-booth-ui/src/pages/vendor_list.rs` | "Label-Link generieren" UI |
| `crates/ez-booth-ui/src/pages/checkout.rs` | `InputMode`, `ItemSource`, `StoredCheckoutItem+source`, dedup, "QR-Code für Mobile" button |
| `crates/ez-booth-ui/src/pages/settings.rs` | Public app URL setting |
| `crates/ez-booth-ui/src/audio.rs` | `play_scan_success_sound()` |
| `crates/ez-booth-server/src/routes/sync.rs` | New — `POST` + `GET /api/sync` |
| `crates/ez-booth-server/src/routes/mod.rs` | Register sync route |
| `crates/ez-booth-server/migrations/0002_purchase_dedup_index.sql` | New — partial unique index |
| `Cargo.toml` (workspace) | Add `ez-booth-labels`, `ez-booth-mobile` as members |
