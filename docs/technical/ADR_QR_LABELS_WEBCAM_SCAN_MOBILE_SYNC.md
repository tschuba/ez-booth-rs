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
|----------|-------------|
| Local only | Veranstalter runs the app locally via launcher or browser. No internet dependency at the event. |
| Static hosting | WASM bundle hosted on Netlify / GitHub Pages. Vendors open the label link on their own device. |
| With server (Coolify) | Optional self-hosted backend. Enables seamless server sync for mobile devices. |

---

## Key Architectural Decisions

### 1. Separate WASM bundles per audience

**Decision:** Each target audience gets its own independent Leptos WASM crate with its own deployment URL.

```
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

The QR code printed on stickers always contains this plain-text format regardless of URL obfuscation.

---

### 3. Label link URL obfuscation

**Decision:** Label links use XOR obfuscation to prevent unregistered vendors from printing labels.

Format: `{labels-url}/?t={token}#k={event_key}`

- `event_key` = auto-generated UUID/16-char random string, stored as `label_key` on the `Booth` model, created at booth creation time
- `token` = `base64url(utf8(vendor_id) XOR repeat(utf8(event_key)))`
- `#k=...` is a URL hash fragment — never sent to a server, not visible in access logs

The label app reads the hash fragment client-side, decodes the token, and displays the vendor ID. On error (missing `t`, missing `k`, or decode failure), a user-friendly message is shown.

**Security scope:** Protects against casual manipulation by non-registered vendors. Not cryptographically strong — sufficient for the use case.

**Warning shown in UI:** Deleting and recreating an event generates a new `label_key`. All existing label links become invalid.

---

### 4. `label_key` as cross-device booth identifier

**Problem:** Booth UUIDs are generated locally on each device and are not shareable across devices.

**Decision:** The `Booth` model gains a mandatory `label_key: String` field (auto-generated at creation). This is the stable cross-device identifier for an event.

Usage:
- The Kassen-App shows a QR code containing `booth.label_key` for mobile onboarding.
- The Mobile-App stores `booth_label_key` per configured event in IndexedDB.
- Purchases synced from mobile carry `booth_label_key` in their JSON payload.
- On import/sync: the Kassen-App resolves the local booth via `label_key`.
- Multiple Kassen-Apps for the same event: export booth settings from one register and import on the other so `label_key` is identical across devices.

**Server side:** `events.booth_id` is set to `NULL` for mobile-synced purchases (the server has no booth registry). The `booth_label_key` is stored in the `payload` JSONB field.

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
|-----|---------|--------|
| Kassen-App | `localStorage` for `StoredCheckoutItem` | Small dataset, synchronous access sufficient |
| Mobile-App | `IndexedDB` for purchases and event config | Larger dataset, async API, survives longer sessions |

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
- File import on Kassen-App: UUID check against locally known purchases, duplicates skipped silently

---

### 11. Mobile sync: `POST /api/sync` body

```
POST /api/sync
Body: { purchases: [...], client_id: "device-uuid" }

Each purchase object includes:
{ id, vendor_id, price_cents, occurred_at, booth_label_key, ... }
```

The `client_id` is a UUID v4 generated on first app start, stored in IndexedDB as a stable device ID.

---

### 12. Mobile: Multi-Event support

**Decision:** The Mobile-App supports multiple configured events simultaneously.

- Events are added via QR code scan from the Kassen-App (which shows `booth.label_key`).
- The helper selects the active event from an event list in the Mobile-App.
- Each event has its own purchase list in IndexedDB.
- Events can be removed individually; a warning is shown if there are unsynchronized purchases.

---

### 13. Mobile PWA + offline support

- Service Worker (manually written `sw.js`) with Cache-First strategy for all WASM/JS/CSS/HTML assets.
- Cache name is versioned — updating the cache name activates new releases immediately via `skipWaiting()` + `clients.claim()`.
- The OCR model (~15MB, Phase 4) is **not** included in the SW precache. Tesseract.js caches it independently via the browser Cache API or IndexedDB after the first explicit user-initiated download.
- The Scan loop pauses via the Page Visibility API when the app is backgrounded to conserve battery.

**Known iOS limitation:** Safari purges SW cache and IndexedDB after ~7 days of inactivity. Helpers should open the app shortly before the event to refresh the cache.

---

### 14. Phase 4 — Handwritten label OCR

**Decision:** Tesseract.js (JavaScript library, ~15MB model) for client-side OCR.

**Risk:** Tesseract.js requires JS interop from Leptos via `wasm-bindgen`/`js-sys`. A spike is required to verify that this works in the Leptos WASM environment without blocking the UI thread (a Web Worker may be necessary).

Recognition flow (confidence-based):
- Confidence > 85% + validation OK → direct acceptance, no dialog
- Confidence < 85% or validation error → confirmation dialog shown to cashier

An optional server-side OCR endpoint (`POST /api/ocr`, Phase 4, Coolify only) offloads processing to the server when activated in settings.

---

### 15. Phase 5 — Organizer-App `label_key` generation

When the Organizer-App creates an event, it generates the `label_key` client-side (UUID v4 or 16-char random string), identical to the Kassen-App. The `label_key` is part of the exported JSON. On import into a Kassen-App, the `label_key` is preserved so label links and mobile onboarding remain consistent.

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
- XOR obfuscation for label links is not cryptographically strong. Determined attackers could forge tokens. This is accepted as the threat model only covers casual manipulation.
- Deleting and recreating an event invalidates all label links — this must be prominently communicated in the UI.
- iOS Safari's cache eviction policy requires helpers to refresh the PWA before each event.

---

## Not in Scope (all phases)

- Bluetooth sync (insufficient browser support)
- Server-side PDF generation
- Vendor login / authentication
- Event ID embedded in QR code
- Real-time relay between phone and register (replaced by batch sync)
- Digital payment (cash-only remains)
- **Remote shopping cart ("Send to register"):** Physically insecure — there is no way to guarantee that items scanned in a side room actually reach the register without being pocketed. The system cannot close this gap; it is therefore excluded entirely.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/domain/src/models/qr_label.rs` | New — `QrLabelPayload` encode/decode + XOR token |
| `crates/domain/src/models/mod.rs` | Add `qr_label` module |
| `crates/domain/src/models/booth.rs` | Add `label_key: String` (auto-generated on creation) |
| `crates/domain/src/services/vendor_service.rs` | Add `get_or_create(vendor_id, booth_id)` |
| `crates/ez-booth-labels/` | New — standalone Label-App crate |
| `crates/ez-booth-mobile/` | New — standalone Mobile-Scan crate |
| `crates/ez-booth-ui/src/pages/vendor_list.rs` | "Label-Link generieren" UI |
| `crates/ez-booth-ui/src/pages/checkout.rs` | `InputMode`, `ItemSource`, `StoredCheckoutItem+source`, dedup, "QR-Code für Mobile" button |
| `crates/ez-booth-ui/src/pages/settings.rs` | Public app URL setting |
| `crates/ez-booth-ui/src/audio.rs` | `play_scan_success_sound()` |
| `crates/ez-booth-server/src/routes/sync.rs` | New — `POST` + `GET /api/sync` |
| `crates/ez-booth-server/src/routes/mod.rs` | Register sync route |
| `crates/ez-booth-server/migrations/0002_purchase_dedup_index.sql` | New — partial unique index |
| `Cargo.toml` (workspace) | Add `ez-booth-labels`, `ez-booth-mobile` as members |
