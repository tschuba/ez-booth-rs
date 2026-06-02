## 0. Feasibility Spike (Phase Gate — complete before Phase 2)

- [ ] 0.1 Add `crates/ez-booth-prototype` to workspace `Cargo.toml`
- [ ] 0.2 Implement `qr_gen.rs` — encode `v={vendor_id}&p={price_cents}` via `qrcode` crate, render to `<canvas>` via `CanvasRenderingContext2d`
- [ ] 0.3 Implement `qr_scan.rs` — webcam frame loop (100 ms via `gloo_timers`), luma buffer to `rxing` decoder, dedup via `Performance::now()`
- [ ] 0.4 Implement `ocr_scan.rs` — `#[wasm_bindgen]` extern block for Tesseract.js, lazy load via `spawn_local`, confidence threshold display
- [ ] 0.5 Run spike: verify all success criteria from `specs/webcam-qr-scan/spec.md` (rxing requirement) and `specs/handwriting-ocr/spec.md` (Tesseract.js requirement)
- [ ] 0.6 Record spike outcomes: update design.md §5 (rxing result) and §14 (Tesseract.js result)
- [ ] 0.7 Gate decision: if `rxing` decode > 200ms/frame → switch to `zxing-wasm` and update design.md §5

## 1. Domain Model

- [ ] 1.1 Add `crates/domain/src/models/qr_label.rs` — `QrLabelPayload { vendor_id: String, price_cents: u32 }` with `encode() -> String` (`v={}&p={}`) and `decode(s: &str) -> Result<Self>` via `url::form_urlencoded`
- [ ] 1.2 Export `qr_label` module from `crates/domain/src/models/mod.rs`
- [ ] 1.3 Add `event_code: String` field to `Booth` struct in `crates/domain/src/models/booth.rs`
- [ ] 1.4 Implement `event_code` derivation function in `crates/domain/src/services/booth_service.rs` — algorithm from design.md §4 (umlaut transliteration, short-word skip, initials, MMYY suffix)
- [ ] 1.5 Add local collision check in `booth_service.rs` — check derived code against existing booth `event_code` values; append `-2` suffix if collision
- [ ] 1.6 Add `get_or_create(vendor_id: &str, booth_id: BoothId) -> Vendor` to `crates/domain/src/services/vendor_service.rs` — uniqueness key `(vendor_id, booth_id)`
- [ ] 1.7 Add `ItemSource { Manual, Scanned, ScannedEdited }` enum to `crates/domain` (or `ez-booth-ui`) — derive `Serialize`, `Deserialize`, `Clone`, `PartialEq`

## 2. Migrations

- [ ] 2.1 Create `crates/ez-booth-server/migrations/0002_purchase_dedup_index.sql` — `CREATE UNIQUE INDEX idx_events_purchase_dedup ON events (entity_id) WHERE event_type = 'purchase_upserted'`
- [ ] 2.2 Create `crates/ez-booth-server/migrations/0003_booth_event_code.sql` — `ALTER TABLE booths ADD COLUMN event_code TEXT`; backfill using derivation logic; `ALTER TABLE booths ALTER COLUMN event_code SET NOT NULL`

## 3. Phase 1 — Label-App (`ez-booth-labels`)

- [ ] 3.1 Add `crates/ez-booth-labels` to workspace `Cargo.toml` as a `cdylib` Leptos WASM crate
- [ ] 3.2 Create `crates/ez-booth-labels/index.html` — Trunk entry point; add `<script>` for browser capability check (WebAssembly, CanvasRenderingContext2D, window.print)
- [ ] 3.3 Implement capability check component — show plain-language error if any API missing; no WASM init attempted
- [ ] 3.4 Implement URL parameter reader — parse `?v=` (vendor_id) and `?e=` (event_code); show error if missing
- [ ] 3.5 Implement preset size selector — Klein (48×30), Mittel (64×34), Groß (70×50); all presets pass validation without warning
- [ ] 3.6 Implement custom dimension input with real-time validation — compute actual QR version from payload; hard block < 25×25 mm; soft warning < 40×25 mm; error messages per design.md §16
- [ ] 3.7 Implement QR sticker preview — render `v={vendor_id}&p={price_cents}` to `<canvas>` using `qrcode` crate; live update on dimension change
- [ ] 3.8 Implement print button — `window.print()` via `web_sys`; disabled when hard block is active; CSS `@media print` styles for label layout
- [ ] 3.9 Add vendor list "Label-Link" column to `crates/ez-booth-ui/src/pages/vendor_list.rs` — per-vendor QR modal (link as scannable QR) + "Link kopieren" clipboard button
- [ ] 3.10 Add "Alle Vendor-Links exportieren" button — download single-page HTML with all vendor QR codes
- [ ] 3.11 Add "Public app URL" setting to `crates/ez-booth-ui/src/pages/settings.rs` (used to build label link URLs)

## 4. Phase 2 — Webcam QR Scan in Kassen-App

- [ ] 4.1 Create `crates/ez-booth-ui/src/audio.rs` — `play_scan_success_sound()` via Web Audio API
- [ ] 4.2 Add `InputMode { Manual, Scan }` enum to `crates/ez-booth-ui/src/pages/checkout.rs`
- [ ] 4.3 Add `source: ItemSource` to `CheckoutItem` and `StoredCheckoutItem` in `checkout.rs` — include in JSON serialisation
- [ ] 4.4 Add `ItemSource::ScannedEdited` transition — when price of a `Scanned` item is manually edited, update `source` to `ScannedEdited`
- [ ] 4.5 Add toolbar toggle "Eingabe | QR-Scan" to checkout page — keyboard shortcut `S`; mode persists in localStorage key `input_mode`
- [ ] 4.6 Implement QR scan component in checkout — webcam via `getUserMedia`, `<video>` + hidden `<canvas>`, 100 ms frame loop via `gloo_timers`, `rxing` decode
- [ ] 4.7 Implement dedup logic — `HashMap<String, f64>` with `Performance::now()`; 2000 ms window; on suppression: highlight matching cart row (~300 ms CSS pulse), no sound
- [ ] 4.8 Add 📷 icon to `Scanned` cart rows; ✎ icon to `ScannedEdited` rows
- [ ] 4.9 Add "QR-Code für Mobile" button to checkout — renders onboarding QR (`ez-booth://onboard?e={event_code}&n={name}`); sets `onboarding_qr_shown_at`; locks `event_code` editing
- [ ] 4.10 Add event_code lock logic to event creation/edit UI — disabled when `onboarding_qr_shown_at` is set; warning message on edit attempt before lock
- [ ] 4.11 Show `event_code` prominently on event creation confirmation screen with sharing instruction and onboarding QR

## 5. Phase 3 — Mobile-App (`ez-booth-mobile`)

- [ ] 5.1 Add `crates/ez-booth-mobile` to workspace `Cargo.toml` as a `cdylib` Leptos WASM crate (depends only on `crates/domain`)
- [ ] 5.2 Create `manifest.json` — `name`, `icons`, `display: standalone`
- [ ] 5.3 Create manually written `sw.js` — Cache-First strategy, versioned cache name, `skipWaiting()` + `clients.claim()`
- [ ] 5.4 Embed build-version constant; implement startup version check against `version.json` endpoint; show staleness banner if offline and > 6 days since last open
- [ ] 5.5 Implement event onboarding screen — QR scan (`ez-booth://onboard?e=…&n=…`) with "Enter code manually" fallback (text fields for `event_code` + name)
- [ ] 5.6 Implement event list — IndexedDB; active-event selection; event detail with batch status bar ("N Batches ausstehend · zuletzt synchronisiert HH:MM")
- [ ] 5.7 Implement scan screen per event — webcam + `rxing` QR decode; decoded items stored in IndexedDB under active event
- [ ] 5.8 Implement batch state machine — `pending → server_uploaded | file_exported`; `file_exported` suppresses removal warning
- [ ] 5.9 Implement file export — filename `ez-booth_{event_code}_{client_id8}_{YYYY-MM-DD}.json`; transitions batch to `file_exported`
- [ ] 5.10 Implement event removal with safeguard — warn if any batch is `pending`; proceed without warning if all are `server_uploaded` or `file_exported`
- [ ] 5.11 Implement "Verlauf anzeigen" disclosure — full batch list with individual statuses
- [ ] 5.12 Implement `client_id` — UUID v4, generated on first start, stored in IndexedDB

## 6. Phase 3 — Server Sync (`ez-booth-server`)

- [ ] 6.1 Create `crates/ez-booth-server` workspace crate (if not yet exists) — add to workspace `Cargo.toml`
- [ ] 6.2 Implement `POST /api/sync` in `crates/ez-booth-server/src/routes/sync.rs` — accept `{ purchases, client_id }`; insert via `ON CONFLICT DO NOTHING` using dedup index; log warning when two `client_id` values use same `event_code` in same month
- [ ] 6.3 Implement `GET /api/sync?since={seq}` — return `purchase_upserted` events with `sequence > since`, up to 500; include `next_sequence`
- [ ] 6.4 Register sync routes in `crates/ez-booth-server/src/routes/mod.rs`
- [ ] 6.5 Implement Kassen-App Sync button — `POST /api/sync` (upload) + paginated `GET /api/sync` loop (max 20 iterations); split status UI "Upload: OK / Download: fehlgeschlagen"; store `last_upload_ok` and `last_sync_sequence` in localStorage
- [ ] 6.6 Implement Kassen-App file import — file picker, JSON parse, UUID dedup against `completed_purchases`, vendor auto-creation via `get_or_create`
- [ ] 6.7 Implement Kassen-App retry logic — skip POST on next Sync attempt if `last_upload_ok` is true from current session

## 7. Phase 4 — Handwriting OCR (DEFERRED — requires spike outcome)

- [ ] 7.1 Extend `InputMode` with `OcrScan` variant
- [ ] 7.2 Extend toolbar to three-way toggle "Eingabe | QR-Scan | Handschrift"
- [ ] 7.3 Implement `ocr_scan` component — Tesseract.js lazy load, `spawn_local` interop (Web Worker wrapper if required by spike)
- [ ] 7.4 Implement confidence threshold logic — ≥ 85% auto-accept; < 85% show confirmation dialog
- [ ] 7.5 Implement confirmation dialog — cropped image frame + editable price field + confidence % + Übernehmen/Abbrechen
- [ ] 7.6 Implement OCR failure state — toast "Beschriftung nicht lesbar" + focus to manual price field
- [ ] 7.7 Implement optional server OCR endpoint `POST /api/ocr` (Coolify only) — accept image frame, return `{ text, confidence }`; wire to settings toggle

## 8. Phase 5 — Organiser-App (DEFERRED)

- [ ] 8.1 Add `crates/ez-booth-organizer` to workspace `Cargo.toml`
- [ ] 8.2 Implement event creation with central `event_code` derivation (same algorithm as Kassen-App)
- [ ] 8.3 Implement vendor list management — create, edit, archive vendors
- [ ] 8.4 Implement export to Kassen-App import format — preserving `event_code`
