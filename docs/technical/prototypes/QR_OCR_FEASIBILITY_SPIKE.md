# QR & OCR Feasibility Spike

**Date:** 2026-05-21
**Status:** Design approved, implementation pending

## Context

Before Phase 2 (QR scanning) and Phase 4 (handwritten OCR) of the ez-booth-rs roadmap are implemented in the register app, two risks must be validated:

1. **`rxing` in WASM** — the ADR selects this pure-Rust crate for QR decoding, but its WASM compilation and in-browser decode performance are unverified.
2. **Tesseract.js JS interop from Leptos WASM** — the ADR classifies this as HIGH risk. If `wasm-bindgen`/`js-sys` interop with Tesseract.js is not feasible without blocking the UI thread, Phase 4 must be re-scoped.

A dedicated prototype crate validates both risks in the real target environment (Rust/Leptos/WASM), plus demonstrates QR generation and the confidence-based accept/reject UX.

The prototype is **not** a deliverable for end users — it is a developer spike that answers: *does this work, at what quality, and what does the UX look like?*

---

## Decisions

| Question | Decision |
|---|---|
| Platform | Rust + Leptos WASM (same stack as production) |
| Location | `crates/ez-booth-prototype/` in this workspace |
| UI style | Minimal spike — plain DOM output, no design polish |
| Image input | Live webcam via `getUserMedia` |
| QR payload format | `v={vendor_id}&p={price_cents}` per ADR §2 |
| OCR targets | Handwritten vendor ID (numeric, 1–999) + price (German decimal: `3,50`) |
| Confidence threshold | ≥ 85% → AUTO-ACCEPT (green); < 85% → NEEDS CONFIRMATION (red) |

---

## Architecture

### Crate structure

```
crates/ez-booth-prototype/
├── Cargo.toml         # cdylib; depends on leptos, rxing, qrcode, web-sys, js-sys, gloo-timers
├── index.html         # trunk entry point
└── src/
    ├── lib.rs         # Leptos app root; mounts all three sections; no router
    ├── qr_gen.rs      # QR Generator component
    ├── qr_scan.rs     # QR Scanner component (rxing + webcam frame loop)
    └── ocr_scan.rs    # OCR Scanner component (Tesseract.js interop + webcam)
```

### Page layout (single scrollable page, no router)

```
┌──────────────────────────────────────┐
│  ez-booth prototype — feasibility    │
├──────────────────────────────────────┤
│  § QR GENERATOR                      │
│  Vendor ID: [__]  Price (¢): [____]  │
│  [Generate QR]                       │
│  [canvas — QR image]                 │
├──────────────────────────────────────┤
│  § SCANNER                           │
│  [QR Scan] [OCR Scan]  ← toggle      │
│  [video — webcam feed]               │
│  [canvas — hidden, frame extraction] │
│  Result: …                           │
│  Confidence: …%                      │
│  → AUTO-ACCEPT / NEEDS CONFIRMATION  │
├──────────────────────────────────────┤
│  § LOG                               │
│  [scrolling text of decode attempts] │
└──────────────────────────────────────┘
```

### Shared webcam lifecycle

One `<video>` element is shared between QR Scan and OCR Scan modes. `getUserMedia` is called once when the Scanner section mounts. The frame loop (100ms / ~10fps via `gloo_timers::callback::Interval`) feeds the hidden `<canvas>` and dispatches to whichever mode is active. Camera is stopped (`MediaStream.getTracks()[0].stop()`) when the component unmounts.

---

## Component Details

### `qr_gen.rs` — QR Generator

- Inputs: `vendor_id: String`, `price_cents: u32`
- Encodes payload: `format!("v={vendor_id}&p={price_cents}")`
- Uses `qrcode` crate to produce a pixel matrix
- Renders matrix to `<canvas>` via `web_sys::CanvasRenderingContext2d`
- Validates inputs before generating (non-empty vendor_id, price > 0)

### `qr_scan.rs` — QR Scanner

- Shares the `<video>` element via a Leptos `NodeRef`
- Frame loop: `gloo_timers::callback::Interval::new(100, ...)`
- Each tick: copy video frame to canvas → get `ImageData` → pass luma buffer to `rxing`
- Dedup: `HashMap<String, f64>` keyed on decoded string; suppress re-emit within 2000ms using `web_sys::window().unwrap().performance().unwrap().now()`
- Displays: decoded payload string, parse result (`v=`, `p=`), decode latency (ms)
- On decode failure: logs to the LOG section

### `ocr_scan.rs` — OCR Scanner

- Same webcam feed and frame loop cadence as QR Scanner
- Tesseract.js interop via `#[wasm_bindgen]` extern block:

  ```rust
  #[wasm_bindgen]
  extern "C" {
      type TesseractWorker;
      // recognize(imageData, lang) -> Promise<{data: {text, confidence}}>
  }
  ```

- Tesseract.js loaded lazily on first OCR attempt (avoids 15MB download at startup)
- Tesseract.js injected via `<script>` tag in `index.html`; language hint: `"deu"` (German)
- OCR called via `wasm_bindgen_futures::spawn_local` to avoid blocking the UI thread
- Confidence threshold applied to result:
  - ≥ 85%: display in green, label **AUTO-ACCEPT**
  - < 85%: display in red, label **NEEDS CONFIRMATION**
- If Tesseract.js interop blocks UI despite `spawn_local`, the LOG section notes this — confirms Web Worker is required for Phase 4

### `lib.rs` — App root

- `ScanMode` signal: `enum ScanMode { Qr, Ocr }`
- `log_entries: RwSignal<Vec<String>>` — append-only log shared across components
- Mounts `QrGenerator`, `Scanner` (with mode toggle), `Log`

---

## Key Dependencies

| Crate | Purpose | Notes |
|---|---|---|
| `rxing` | QR decode (pure Rust) | Enable `wasm` feature; primary ADR risk |
| `qrcode` | QR encode | Pure Rust, no WASM feature needed |
| `gloo-timers` | 100ms frame loop | Check if already in workspace |
| `wasm-bindgen-futures` | Async Tesseract.js calls | Likely already present |
| `js-sys` | Tesseract.js JS interop | Likely already present |

Tesseract.js is loaded via `<script>` tag in `index.html` (CDN or local copy).

---

## Success Criteria

| Test | Pass condition |
|---|---|
| `rxing` WASM compilation | `trunk build` succeeds with `rxing` in Cargo.toml |
| QR decode (live) | Holding a QR sticker to webcam → decoded payload appears within 500ms |
| QR decode latency | Reported decode time < 100ms per frame |
| QR generate | Generated canvas QR is scannable by a smartphone camera app |
| Tesseract.js interop | `recognize()` callable from Rust without compile error |
| OCR quality ≥ 85% | Clearly handwritten `42` and `3,50` on white paper → confidence ≥ 85% |
| OCR quality < 85% | Messy/small handwriting → confidence shown in red, NEEDS CONFIRMATION |
| UI not blocked | OCR call completes without freezing the page |

---

## Failure Modes (also valuable output)

| Failure | Implication |
|---|---|
| `rxing` WASM binary too large or decode > 200ms/frame | Switch to ZXing-wasm fallback; update ADR §5 |
| Tesseract.js interop blocks UI thread | Web Worker required for Phase 4; update ADR §14 |
| Tesseract.js not callable via `wasm-bindgen` extern | Phase 4 deferred; server-side `/api/ocr` promoted to primary path |
| OCR confidence < 85% on clean handwriting | Threshold needs adjustment; consider 70% or manual-only fallback |

---

## Verification

```bash
cd crates/ez-booth-prototype
trunk serve

# Open http://localhost:8080 in Chrome
# 1. QR Generator: enter vendor_id=42, price_cents=300 → scan with phone → confirms "v=42&p=300"
# 2. QR Scan: hold printed/phone QR to webcam → decoded payload appears in DOM
# 3. OCR Scan: hold handwritten "42 / 3,50" to webcam → text + confidence % appear
# 4. Check LOG for decode latency and any interop errors
```

---

## Out of Scope

- Mobile-App crate, sync API, server endpoints
- PWA service worker, manifest.json
- Integration with the existing checkout page
- UI design polish, German translations
- Confidence calibration / model fine-tuning (evaluation only)
