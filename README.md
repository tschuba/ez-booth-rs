# ez-booth-rs

[![CI](https://github.com/tschuba/ez-booth-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tschuba/ez-booth-rs/actions/workflows/ci.yml)

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

The optimized build is output to `crates/ez-booth-app/dist/`.

To build the standalone launcher for your current platform:

```bash
cargo build --release -p ez-booth-launcher
```

To run the built app locally without Python or Node.js:

1. keep all files in `crates/ez-booth-app/dist/` together
2. copy `target/release/ez-booth-launcher` into `crates/ez-booth-app/dist/`
3. run `crates/ez-booth-app/dist/ez-booth-launcher`
4. use the same browser profile on the same device to keep IndexedDB data available

CI artifacts follow the same layout:

1. download the `wasm-dist` artifact from GitHub Actions
2. extract it fully
3. run the launcher for your platform: `ez-booth.exe`, `ez-booth-macos`, or `ez-booth-linux`

The launcher starts a local server, opens your browser, and enforces single-instance execution per device with a lock file in the user config directory.

Launcher notes:

- the lock file uses atomic creation plus stale-process cleanup; a narrow cleanup race still exists if two launchers start at the exact same time, which is an accepted desktop-app tradeoff for this local single-user workflow
- `crates/ez-booth-app/Trunk.toml` uses `public_url = "./"` so downloaded builds work from any extracted folder, but that relative asset layout is not suitable for deployments that serve the app from a URL subdirectory

## Releases

Download stable builds from the [GitHub Releases page](https://github.com/tschuba/ez-booth-rs/releases).

Each release publishes complete platform bundles:

- `ez-booth-windows-vX.Y.Z.zip`
- `ez-booth-macos-vX.Y.Z.tar.gz`
- `ez-booth-linux-vX.Y.Z.tar.gz`
- `checksums.txt`

Every platform archive includes the launcher binary, the full WASM app bundle, and a `README.txt` with usage guidance.

Verify a download with the published checksum file:

```bash
shasum -a 256 -c checksums.txt
```

For maintainers creating releases, see `docs/RELEASE_PROCESS.md`.

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

# Build the standalone launcher
cargo build -p ez-booth-launcher

# Run unit tests only
./run-tests.sh

# Run full automated validation
./run-tests.sh --chrome --safari

# Run tests for a specific crate
cargo test -p domain
```

## Troubleshooting

### Development

- `trunk build` fails: run `npm ci` in `crates/ez-booth-app`, then retry the build
- `cargo build -p ez-booth-launcher` fails: update your Rust toolchain with `rustup update`
- local launcher says another instance is running: remove the lock file from your OS config directory and retry
- browser changes do not appear: hard-refresh the page or clear the cache for `127.0.0.1`
- ports `8080-8089` are busy: stop the conflicting process or run your local checks after freeing one of those ports

### CI And Downloaded Artifacts

- GitHub Actions launcher build fails on one platform: inspect that job's artifact-build log for platform-specific linker or dependency issues
- downloaded app shows a blank page: launch it with the included binary instead of opening `index.html` directly
- macOS or Windows warns about the launcher binary: expected for unsigned builds; follow the steps in `crates/ez-booth-app/ARTIFACT_README.md`
- lock file seems stuck after a crash: delete `launcher.lock` from the user config directory listed in `crates/ez-booth-app/ARTIFACT_README.md`
- downloaded build is being hosted from a web server subdirectory: rebuild with a deployment-specific `public_url` instead of `./`

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
