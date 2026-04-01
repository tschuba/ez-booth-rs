# ez-booth-rs

A Rust-based flea market booth management system for tracking vendor sales and calculating commissions.

## Project Structure

This project uses a Cargo workspace with multiple crates:

- **`crates/domain`** - Pure domain logic with entities, business rules, and service layer
- **`crates/storage`** - IndexedDB storage implementation with repository pattern
- **`crates/ez-booth-ui`** - Web UI components and pages using Leptos
- **`crates/ez-booth-app`** - WASM application entry point
- **`crates/ez-booth-core`** - Legacy compatibility layer (deferred)

## Current Status

### Phase 1 Complete

Phase 1 focused on correctness, recovery, and validation of the critical money-handling paths.

Delivered:
- validated `Purchase` and `PurchaseItem` creation at the domain boundary
- consistent fee and payout handling across checkout, reports, and summaries
- corruption diagnostics and partial-recovery warnings for damaged purchase data
- centralized checkout draft recovery with explicit restored/corrupted outcomes
- regression coverage for purchase deletion and recalculated reports
- Chrome and Safari browser validation support
- reusable manual validation assets for Safari and UAT execution

### Phase 2 Progress

Phase 2 shifts the project toward product readiness, lower warning noise, and more confident operator workflows.

Delivered so far:
- Milestone 1: warning reduction and active-crate cleanup
- Milestone 2: clearer correction and deletion workflows
- Milestone 3: refreshed onboarding, branch strategy, and validation documentation

Still open:
- optional Milestone 4 UX and reporting refinements
- retroactive manual execution of the prepared Milestone 2 validation template, if formal sign-off is needed later

### Current Product Capabilities

- booth management with fee configuration and status handling
- vendor management with smart numeric/alphanumeric sorting
- checkout flow with validation, draft persistence, and recovery guidance
- checkout keypad with booth-configurable quick amounts and persistent amount entry mode
- correction and deletion flows with operator-facing recalculation and aftercare messaging
- vendor and booth reporting with consistent payout-derived totals
- German primary / English fallback translations
- IndexedDB persistence through repository abstractions
- offline-first architecture ready for PWA deployment planning; see `docs/ADR_PWA_IMPLEMENTATION.md`

### Current Validation Workflow

- fast local unit suite: `./run-tests.sh`
- Chrome browser validation: `./run-tests.sh --chrome`
- Safari browser validation: `./run-tests.sh --safari`
- full automated suite: `./run-tests.sh --chrome --safari`

Manual validation assets:
- `docs/SAFARI_VALIDATION_CHECKLIST.md`
- `docs/UAT_Ausfuehrungsplan_DE_EN.html`
- `docs/PHASE2_M2_VALIDATION_RESULTS.md` as an example milestone sign-off template

How to use them:
- use `docs/SAFARI_VALIDATION_CHECKLIST.md` for focused Safari and operator-flow regression checks
- use `docs/UAT_Ausfuehrungsplan_DE_EN.html` for guided bilingual acceptance sessions
- use a milestone result file when a change needs a durable manual sign-off record

See `docs/VALIDATION_WORKFLOW.md` for the full validation workflow and artifact guidance.

### Checkout Keyboard

- checkout includes a toggleable on-screen keypad in the top-right of the checkout card
- keypad quick amounts are configurable per event; defaults are `0.50`, `1.00`, `5.00`, `10.00`, and `15.00`
- amount entry supports `Cash Register` mode (right-to-left cents filling) and `Regular` mode
- the selected keypad visibility and amount entry mode persist in browser storage for the next session
- physical keyboard entry remains available and follows the selected amount entry mode

### Next Focus

The current cleanup track has completed Phase 2 Milestones 1 through 3.

Next up:
- optional Phase 2 Milestone 4 UX and reporting refinements
- any retroactive manual validation needed to formally close Milestone 2
- planned backup/recovery track captured in `docs/DATA_BACKUP_IMPLEMENTATION_PLAN.md`

See `docs/PHASE2_PROPOSAL.md` for the concrete follow-up plan.

## How to Run

### Prerequisites

- [Rust](https://rustup.rs/) (via rustup, not Homebrew)
- [Trunk](https://trunkrs.dev/) for WASM bundling: `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

**Important:** Ensure your shell has the correct PATH set up. Add to `~/.zshrc` (or `~/.bashrc`):

```bash
# Rust cargo bin
export PATH="$HOME/.cargo/bin:$PATH"
# Rustup toolchain binaries
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
```

Then reload: `source ~/.zshrc`

### Development Server

```bash
cd crates/ez-booth-app
trunk serve
```

The application will open in your browser at `http://127.0.0.1:8080`.

Trunk will watch for changes and automatically rebuild and reload.

### Production Build

```bash
cd crates/ez-booth-app
trunk build --release
```

The optimized build will be output to `dist/` and can be deployed to any static web server.

## Building

```bash
# Build the domain crate
cargo build -p domain

# Build the storage crate (IndexedDB)
cargo build -p ez-booth-storage

# Build the UI crate
cargo build -p ez-booth-ui

# Build for WASM target
cargo build -p ez-booth-app --target wasm32-unknown-unknown

# Run unit tests only
./run-tests.sh

# Run full automated validation
./run-tests.sh --chrome --safari

# Run tests for a specific crate
cargo test -p domain
```

## Documentation

### User Documentation

- **[Fee Calculation Guide](docs/FEE_CALCULATION.md)** - Detailed explanation of how vendor fees and payouts are calculated (Bilingual: DE/EN)
- **[Data Backup Implementation Plan](docs/DATA_BACKUP_IMPLEMENTATION_PLAN.md)** - Agreed execution plan for export/import, browser-storage warnings, and recovery guidance
- **[Safari Validation Checklist](docs/SAFARI_VALIDATION_CHECKLIST.md)** - Manual Safari validation for checkout, recovery, and reporting
- **[Bilingual UAT Execution Plan](docs/UAT_Ausfuehrungsplan_DE_EN.html)** - Reusable on-screen / printable UAT guide
- **[Validation Workflow](docs/VALIDATION_WORKFLOW.md)** - When to run automated checks, Safari validation, UAT, and milestone sign-off docs
- **[Branch Strategy](docs/BRANCH_STRATEGY.md)** - Branch naming, PR workflow, and squash-merge policy used in this repository

### Technical Documentation

See the `docs/redesign/` folder for detailed architecture and implementation documentation:

- `00_SPEC.md` - Project specification
- `01_ANALYSIS.md` - Analysis of current implementation
- `02_ARCHITECTURE.md` - System architecture
- `03_IMPROVEMENTS.md` - Planned improvements
- `04_IMPLEMENTATION.md` - Implementation guide with code examples
- `05_STATUS.md` - Current implementation and cleanup-track status
- `PHASE1_M2_PREPARATION.md` - Milestone 2 safety/recovery planning reference
- `PHASE2_PROPOSAL.md` - Proposed next phase for cleanup, maintainability, and operator polish
- **[Device-to-Device Transfer ADR](docs/ADR_DEVICE_TO_DEVICE_TRANSFER.md)** - Architecture decision record for offline single-booth device-to-device transfer options

## Contributing

- create short-lived branches from `main`; use `feature/...` for most work and `fix/...` for focused bug fixes
- open a pull request for all changes and summarize the why, validation run, and any deferred follow-up
- prefer squash merges so `main` keeps one reviewed commit per completed change
- update validation artifacts in `docs/` when operator-facing behavior or acceptance coverage changes

See `docs/BRANCH_STRATEGY.md` for the repository workflow and `docs/VALIDATION_WORKFLOW.md` for expected validation evidence.

## License

PolyForm-Noncommercial-1.0.0
