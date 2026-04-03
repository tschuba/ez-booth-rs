# AGENTS.md

This file gives coding agents the repo-specific rules and commands needed to work safely in `ez-booth-rs`.

## Repository Snapshot

- Rust 2021 Cargo workspace with domain, storage, UI, app, and launcher crates.
- Primary app flow: `domain` -> `ez-booth-storage` -> `ez-booth-ui` -> `ez-booth-app`.
- UI is a client-side Leptos app compiled to WASM and bundled with Trunk.
- Storage uses IndexedDB in the browser.
- Launcher is a native Rust binary that serves the built WASM bundle locally.
- Money and fee calculations are business-critical; preserve exact decimal behavior.

## Repository Instruction Files

- No `.cursor/rules/` directory was found.
- No `.cursorrules` file was found.
- No `.github/copilot-instructions.md` file was found.
- Use this `AGENTS.md` plus existing repo docs as the active instruction set.

## Important Docs

- `README.md` for product scope, build flow, and delivery expectations.
- `TESTING.md` for unit, browser, and manual validation details.
- `docs/VALIDATION_WORKFLOW.md` for when manual validation is required.
- `docs/BRANCH_STRATEGY.md` for branch naming, PR expectations, and squash-merge policy.
- `docs/SAFARI_VALIDATION_CHECKLIST.md` and `docs/UAT_Ausfuehrungsplan_DE_EN.html` for manual operator validation.

## Setup Prerequisites

- Install Rust via `rustup`.
- Install Trunk: `cargo install trunk`.
- Install `wasm-pack` for browser tests: `cargo install wasm-pack`.
- Add the WASM target: `rustup target add wasm32-unknown-unknown`.
- In `crates/ez-booth-app`, install frontend tooling before Trunk builds: `npm ci`.
- Safari browser tests require one-time enablement: `sudo safaridriver --enable`.

## Build Commands

- Full workspace build: `cargo build --workspace --locked`
- WASM target build: `cargo build -p ez-booth-app --target wasm32-unknown-unknown --locked`
- Standalone launcher build: `cargo build --release -p ez-booth-launcher --locked`
- App dev server: `trunk serve` from `crates/ez-booth-app`
- Production bundle: `trunk build --release` from `crates/ez-booth-app`
- Tailwind CSS only: `npm run build:css` from `crates/ez-booth-app`

## Lint And Format Commands

- Format check: `cargo fmt --all --check`
- Apply formatting: `cargo fmt --all`
- Clippy: `cargo clippy --workspace --all-targets --locked`
- There is no repo-specific `rustfmt.toml` or `clippy.toml`; default tool behavior applies.

## Test Commands

- Fast local suite: `./run-tests.sh`
- Unit tests only: `cargo test --workspace --lib --locked`
- Chrome browser suite: `./run-tests.sh --chrome`
- Safari browser suite: `./run-tests.sh --safari`
- Full automated suite: `./run-tests.sh --chrome --safari`

## Single-Crate And Single-Test Commands

- Domain crate tests: `cargo test -p domain`
- UI unit tests: `cargo test -p ez-booth-ui --lib`
- Storage browser tests in Chrome: `wasm-pack test --headless --chrome crates/storage`
- Storage browser tests in Safari: `wasm-pack test --headless --safari crates/storage`
- UI browser tests in Chrome: `wasm-pack test --headless --chrome crates/ez-booth-ui`
- UI browser tests in Safari: `wasm-pack test --headless --safari crates/ez-booth-ui`
- Single Rust test by exact name: `cargo test -p <crate> <test_name> -- --exact`
- Single browser test by exact name: `wasm-pack test --headless --chrome crates/<crate> <test_name> -- --exact`
- Example single UI unit test: `cargo test -p ez-booth-ui test_to_booth_valid_data -- --exact`
- Example single browser test: `wasm-pack test --headless --chrome crates/storage test_save_booth -- --exact`
- Watch mode example: `cargo watch -x "test -p ez-booth-ui --lib"`

## Manual Validation Expectations

- If you change operator-facing flows, reporting, recovery behavior, print output, or Safari-sensitive behavior, do more than unit tests.
- Start the app with `trunk serve` from `crates/ez-booth-app` for manual validation.
- Use the smallest validation set that proves safety, then note what you ran.
- Update validation docs in `docs/` when workflows or acceptance coverage change.

## Architecture Guidance

- Keep domain rules in `crates/domain`; do not bury core business validation in UI-only code.
- Keep persistence concerns in `crates/storage`.
- Keep rendering, interaction, and translation wiring in `crates/ez-booth-ui`.
- Keep `crates/ez-booth-app` thin; it should mostly initialize logging, panic hooks, and mount the app.
- Keep `crates/ez-booth-launcher` focused on packaging and local serving concerns.

## Code Style: General

- Follow existing Rust 2021 idioms and let `rustfmt` drive layout.
- Prefer small helper functions over deeply nested event handlers or validation branches.
- Match surrounding style in a file before introducing a new pattern.
- Use 4-space indentation; do not manually align columns.
- Prefer expressive names over abbreviations.
- Keep comments sparse and only for non-obvious business or browser constraints.

## Code Style: Imports

- Keep imports explicit and local to the file.
- Preserve the existing grouping/order style of the file you touch; import ordering is not perfectly uniform across crates.
- Avoid wildcard imports except where the file already uses them heavily, such as some Leptos view modules.
- Remove unused imports rather than leaving them for later cleanup.

## Code Style: Types And Data Modeling

- Use `rust_decimal::Decimal` for money, fees, payout math, and amount stepping.
- Use `chrono::DateTime<Utc>` for timestamps and `chrono::NaiveDate` for booth dates.
- Reuse domain model types such as `BoothId`, `VendorId`, and `PurchaseId` instead of raw strings when possible.
- Return `DomainResult<T>` or crate-specific result types instead of ad hoc `Result<T, String>` in core logic.
- Use `Arc` for shared repositories and services in UI/storage state where the existing code does.
- Prefer typed enums for mode/state (`ConflictStrategy`, `DraftLoadOutcome`, `Locale`) over free-form strings.

## Code Style: Naming

- Types, enums, and traits use `PascalCase`.
- Functions, methods, modules, and variables use `snake_case`.
- Constants use `SCREAMING_SNAKE_CASE`.
- Test names should describe the scenario and expected outcome, usually in full snake_case phrases.
- Translation keys remain dot-separated strings and should follow existing naming families like `checkout.errors.*`.

## Code Style: Validation And Business Rules

- Validate at the domain boundary first.
- Trim and normalize user input before persisting or validating when the existing flow does so.
- Preserve exact current fee and payout semantics; even small rounding changes are high risk.
- Treat amount stepping, vendor ID rules, omission rules, and reporting totals as regression-sensitive.
- Prefer adding targeted regression tests when changing money handling or recovery logic.

## Code Style: Error Handling

- Use `thiserror` enums for structured errors.
- Convert infrastructure errors into domain-facing errors at crate boundaries.
- In UI code, translate domain errors for operator-facing messages with `translate_domain_error` or i18n helpers.
- Prefer returning rich errors over panicking.
- Limit `unwrap()` and `expect()` to tests or truly impossible invariants; avoid introducing them in normal runtime paths.
- Log actionable failures with `log::{error, warn, info}` when they help diagnose browser/storage issues.

## Code Style: UI And Leptos

- Use signals, memos, and effects in the established Leptos style already present in `crates/ez-booth-ui`.
- Use `spawn_local` for async browser-side tasks.
- Keep persistent UI preferences in local storage through focused helper functions.
- Route visible strings through translations; do not hard-code new operator-facing copy if an i18n key is appropriate.
- Keep accessibility attributes when editing reusable components like buttons and dialogs.
- Preserve current Tailwind utility style rather than introducing a second styling system.

## Testing Conventions

- Inline unit tests commonly live under `#[cfg(test)]` in the same Rust module.
- Browser integration tests live in crate-level `tests/` directories and use `wasm-bindgen-test`.
- Async domain/service tests often use `#[tokio::test]` with lightweight mock repositories.
- When fixing a bug, add or update the narrowest regression test that proves the fix.
- For browser-storage or recovery fixes, prefer both automated coverage and a note about manual validation needs.

## Workflow Expectations For Agents

- Start from a focused branch, usually `feature/...` or `fix/...`, when branch work is part of the task.
- Keep changes scoped to one problem or milestone.
- Before finishing, run the smallest relevant command set from this file.
- In PR summaries, explain why the change exists, what validation ran, and any deferred follow-up.
- Do not plan on direct merges to `main`; the repo expects pull requests and prefers squash merges.

## Safe Defaults For Agentic Changes

- If a change touches money math, reports, import/export, recovery, or deletion flows, be conservative and add tests.
- If a change affects operator workflows, review the relevant docs in `docs/` and update them when behavior changes.
- If a file already has local conventions, follow them instead of normalizing unrelated style.
- If browser behavior is uncertain, prefer `./run-tests.sh --chrome` and call out whether Safari validation is still needed.
