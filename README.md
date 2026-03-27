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

### Current Product Capabilities

- booth management with fee configuration and status handling
- vendor management with smart numeric/alphanumeric sorting
- checkout flow with validation, draft persistence, and recovery guidance
- vendor and booth reporting with consistent payout-derived totals
- German primary / English fallback translations
- IndexedDB persistence through repository abstractions

### Current Validation Workflow

- fast local unit suite: `./run-tests.sh`
- Chrome browser validation: `./run-tests.sh --chrome`
- Safari browser validation: `./run-tests.sh --safari`
- full automated suite: `./run-tests.sh --chrome --safari`

Manual validation assets:
- `docs/SAFARI_VALIDATION_CHECKLIST.md`
- `docs/UAT_Ausfuehrungsplan_DE_EN.html`

### Next Focus

The next planned phase is product-readiness cleanup and maintainability work:
- reduce warning noise in active crates
- refresh onboarding and technical documentation
- polish operator-facing recovery and correction workflows

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
- **[Safari Validation Checklist](docs/SAFARI_VALIDATION_CHECKLIST.md)** - Manual Safari validation for checkout, recovery, and reporting
- **[Bilingual UAT Execution Plan](docs/UAT_Ausfuehrungsplan_DE_EN.html)** - Reusable on-screen / printable UAT guide

### Technical Documentation

See the `docs/redesign/` folder for detailed architecture and implementation documentation:

- `00_SPEC.md` - Project specification
- `01_ANALYSIS.md` - Analysis of current implementation
- `02_ARCHITECTURE.md` - System architecture
- `03_IMPROVEMENTS.md` - Planned improvements
- `04_IMPLEMENTATION.md` - Implementation guide with code examples
- `05_STATUS.md` - **Current implementation progress (33% complete)**
- `PHASE1_M2_PREPARATION.md` - Milestone 2 safety/recovery planning reference
- `PHASE2_PROPOSAL.md` - Proposed next phase for cleanup, maintainability, and operator polish

## License

PolyForm-Noncommercial-1.0.0
