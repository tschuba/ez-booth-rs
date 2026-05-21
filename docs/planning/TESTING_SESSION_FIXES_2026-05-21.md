# Testing Session Findings — Implementation Plan

> Session date: 2026-05-21

## Issue 1 — Vendor and position display order is swapped in two places

**Problem:** In both the pending checkout items list and the expanded last-checkout detail view, the item position number is shown as the primary (bold) label and vendor as secondary (small/muted). Both should show vendor first (primary) and position number second (small/muted).

### Location A — Pending checkout items list

**File:** `crates/ez-booth-ui/src/pages/checkout.rs:2332–2333`

Swap the two `<p>` elements:

```rust
// Before
<p class="font-medium">{format!("{} {}", t!("checkout.item_label")(), display_number)}</p>
<p class="text-xs text-gray-500">{format!("{} {}", t!("checkout.vendor_label")(), vendor_label)}</p>

// After
<p class="font-medium">{format!("{} {}", t!("checkout.vendor_label")(), vendor_label)}</p>
<p class="text-xs text-gray-500">{format!("{} {}", t!("checkout.item_label")(), display_number)}</p>
```

### Location B — Last-checkout expanded detail view

**File:** `crates/ez-booth-ui/src/pages/checkout.rs:2573–2589`

Swap the two sub-rows inside the per-item `<div>`:

```rust
// Before: item number first, vendor second
<div class="flex justify-between text-sm">
    <span class="font-medium text-gray-900">{/* item_number */}</span>
    <span class="font-medium text-gray-900">{format_currency(item.amount, locale)}</span>
</div>
<div class="text-xs text-gray-500 mt-0.5">{vendor_text}</div>

// After: vendor first (with amount), item number second
<div class="flex justify-between text-sm">
    <span class="font-medium text-gray-900">{vendor_text}</span>
    <span class="font-medium text-gray-900">{format_currency(item.amount, locale)}</span>
</div>
<div class="text-xs text-gray-500 mt-0.5">{/* item_number */}</div>
```

---

## Issue 2 — WASM app becomes inaccessible after a regular PR merge to main

**Root cause:** `deploy-pages.yml` has two triggers:

1. `workflow_run` on "Tag Release" → builds docs + WASM → deploys both
2. `push` to `main` (paths: `docs/**`, README, the workflow file itself) → builds docs **only**, WASM steps are gated by `if: github.event_name == 'workflow_run'`

When a PR touching `docs/` or `README.md` is merged, trigger 2 fires and deploys a Pages site with no `/pos/` directory, making the WASM app return 404.

**Context — one build must serve two environments:**

The WASM app runs in two contexts with different base paths: the launcher (`http://127.0.0.1:<port>/`, base `""`) and GitHub Pages (`/ez-booth-rs/pos/`). `Trunk.toml` already sets `public_url = "./"` so asset references are relative and portable. The only thing preventing a single shared build is `ROUTER_BASE`, currently a compile-time constant in `crates/ez-booth-ui/src/lib.rs:5–8`. The `build-wasm` job in `release.yml` already builds without `--public-url` — no change needed there.

**Fix — runtime router base + single WASM build downloaded from release:**

### Part A — App change: runtime router base

In `crates/ez-booth-ui/src/lib.rs`, replace the compile-time `BASE_PATH` constant with a function that reads `<meta name="router-base">` from the DOM at startup via `web_sys`, defaulting to `""`:

```rust
pub fn base_path() -> &'static str {
    use std::sync::OnceLock;
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("meta[name='router-base']").ok().flatten())
            .and_then(|el| el.get_attribute("content"))
            .unwrap_or_default()
    })
}
```

Update all `BASE_PATH` references to `base_path()` in three files:

- `crates/ez-booth-ui/src/lib.rs` — Router init (line 94), Routes init (line 185), navigation hrefs (lines 101, 114, 117, 120, 160), pathname stripping (line 38)
- `crates/ez-booth-ui/src/pages/home.rs` — lines 61, 64
- `crates/ez-booth-ui/src/components/storage_warning.rs` — line 146

No new `web_sys` features needed — `Document`, `Element`, and `Window` are already in `Cargo.toml`. `OnceLock<String>` returning `&'static str` is sound; the Leptos Router `base` prop accepts `&'static str` exactly.

### Part B — CI/CD changes

**`.github/workflows/release.yml`** — `package` job only; `build-wasm` needs no change.

In the `package` job, after the "Prepare release archives" step, add a step to zip the WASM dist. Must `cd` into the source directory to avoid embedding the path prefix:

```bash
(cd release-assets/base && zip -rq "$GITHUB_WORKSPACE/packaged/wasm-bundle.zip" .)
```

Add `packaged/wasm-bundle.zip` to the existing `upload-artifact@v7` path list so the `publish` job picks it up via the `release-packages/*` glob in `gh release create`.

**`.github/workflows/deploy-pages.yml`** — two changes:

Change `workflow_run` trigger from `"Tag Release"` to `"Release"` (fires after the full release workflow — and `wasm-bundle.zip` — is published):

```yaml
workflow_run:
  workflows: ["Release"]   # was "Tag Release"
  types: [completed]
```

Replace the three WASM build steps (currently gated `if: github.event_name == 'workflow_run'`) with one combined step running on **both** triggers. The `sed` injection is guarded so a missing download does not fail the job:

```yaml
- name: Restore WASM from latest release
  env:
    GH_TOKEN: ${{ github.token }}
  run: |
    mkdir -p _site/pos
    if gh release download --latest --pattern "wasm-bundle.zip" --dir /tmp/wasm-dl; then
      unzip -q /tmp/wasm-dl/wasm-bundle.zip -d _site/pos
      sed -i 's|</head>|<meta name="router-base" content="/ez-booth-rs/pos">\n</head>|' \
        _site/pos/index.html
    else
      echo "No wasm-bundle.zip found — /pos/ not included in this deploy."
    fi
```

### Result

| Trigger | Behavior |
| ------- | -------- |
| Release deploy (`workflow_run` on "Release") | Fires after release is fully published → downloads `wasm-bundle.zip` → injects router base → deploys docs + new WASM |
| Docs-only deploy (`push` to main) | Fires immediately on merge → downloads latest released WASM → injects router base → deploys updated docs + stable WASM |

Single WASM build per release serves both launcher and Pages. No race condition. WASM toolchain no longer needed in `deploy-pages.yml`.

> **Transition note:** The fix only takes effect after the first release containing `wasm-bundle.zip` is published. Doc pushes before that first release will hit the soft-fail fallback (no `/pos/`). Ship this change in a release before merging any docs-only PRs.

---

## Issue 3 — Amount stepping validation rule not shown in active-rules info panel

**Root cause:** `VendorRulesInfoModal` (`crates/ez-booth-ui/src/components/vendor_rules_info.rs`) has no `amount_stepping` prop. The Amount section renders a hardcoded static summary (line 122) and ignores the `amount_stepping` booth setting.

### Changes

**`crates/ez-booth-ui/src/components/vendor_rules_info.rs`:**

- Add prop: `#[prop(into)] amount_stepping: Signal<Option<Decimal>>`
- Add imports (none of these are currently present in this file):
  - `use rust_decimal::Decimal`
  - `use crate::formatting::format_decimal`
  - `use crate::i18n::use_locale`
- In the Amount section, keep the static `rules_amount_summary` paragraph (base "> 0" rule — always shown), and add a conditional block using `{move || amount_stepping.get().map(|step| view! { <p>...</p> })}` (no fallback branch needed). When `amount_stepping` is `None`, nothing extra is shown.
- Format `step` with `format_decimal(step, locale, 2)` — do not use `step.to_string()`, which always produces dot notation and would show `"0.50"` to German users instead of `"0,50"`.

**`crates/ez-booth-ui/src/pages/checkout.rs:2733–2738`:**

- Pass `amount_stepping` to the modal (`amount_stepping` memo already exists at line 859):

  ```rust
  amount_stepping=Signal::derive(move || amount_stepping.get())
  ```

**`crates/ez-booth-ui/locales/en.json` and `locales/de.json`:**

- Add key under `checkout`:
  - EN: `"rules_amount_stepping": "Amounts must be in increments of {step}."`
  - DE: `"rules_amount_stepping": "Beträge müssen in Schritten von {step} erfasst werden."`

---

## Issues 4–6 — Cross-device event merge

The import duplicate bug, pre-import merge analysis, and local duplicate cleanup are specified in a dedicated document:

**→ [CROSS_DEVICE_EVENT_MERGE.md](CROSS_DEVICE_EVENT_MERGE.md)**

---

## Files to modify (summary)

| File | Issues |
| ---- | ------ |
| `crates/ez-booth-ui/src/pages/checkout.rs` | 1, 3 |
| `crates/ez-booth-ui/src/components/vendor_rules_info.rs` | 3 |
| `crates/ez-booth-ui/locales/en.json` | 3 |
| `crates/ez-booth-ui/locales/de.json` | 3 |
| `crates/ez-booth-ui/src/lib.rs` | 2 |
| `crates/ez-booth-ui/src/pages/home.rs` | 2 |
| `crates/ez-booth-ui/src/components/storage_warning.rs` | 2 |
| `.github/workflows/release.yml` | 2 |
| `.github/workflows/deploy-pages.yml` | 2 |

See [CROSS_DEVICE_EVENT_MERGE.md](CROSS_DEVICE_EVENT_MERGE.md) for files affected by Issues 4–6.

---

## Verification

- **Issue 1:** Start dev server, open register view, add items — confirm vendor name is the primary (bold) label and position number is secondary in both the pending list and the last-checkout detail view.
- **Issue 2:** After the fix ships in a release, merge a docs-only PR and confirm the WASM app is still reachable at `/ez-booth-rs/pos/` after the Pages deploy completes.
- **Issue 3:** Open register view with a booth that has `amount_stepping` set; open the rules info panel — confirm the stepping rule appears. Test with no stepping set — confirm only the static base summary shows.

See [CROSS_DEVICE_EVENT_MERGE.md](CROSS_DEVICE_EVENT_MERGE.md) for verification of Issues 4–6.
