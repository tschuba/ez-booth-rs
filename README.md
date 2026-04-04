# ez-booth-rs

[![CI](https://github.com/tschuba/ez-booth-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tschuba/ez-booth-rs/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://tschuba.github.io/ez-booth-rs/)

`ez-booth-rs` is a browser-based flea market booth management system built with Rust and WebAssembly.

It helps event teams track vendor sales, calculate fees and payouts, print reports, and protect booth data with export and import workflows. The app is designed to stay fast, portable, and usable in offline event environments.

## Why ez-booth-rs?

- runs as a browser-first app with no Java runtime or server setup required for normal use
- keeps booth data locally on the device with IndexedDB for offline-first operation
- preserves business-critical fee and payout logic in a dedicated Rust domain layer
- supports bilingual operator workflows in German and English
- adds practical backup and recovery workflows for browser-stored data

If you already know the original Java-based `ez-booth`, start with [What Changed from ez-booth to ez-booth-rs?](docs/COMPARISON_TO_ORIGINAL.md).

## Quick Start

### Just Want to Use It?

Download ready-to-run releases from the [GitHub Releases page](https://github.com/tschuba/ez-booth-rs/releases).

Each platform bundle for Windows, macOS, and Linux includes everything needed to run the app:

- extract the archive
- run the included launcher
- the app opens in your browser automatically

See the included `README.txt` in each download for platform-specific details.

### Want to Build or Contribute?

#### Prerequisites

- [Rust via rustup](https://rustup.rs/)
- `trunk`: `cargo install trunk`
- `wasm-pack`: `cargo install wasm-pack`
- WASM target: `rustup target add wasm32-unknown-unknown`
- frontend dependencies: run `npm ci` in `crates/ez-booth-app`

#### Start the App

```bash
cd crates/ez-booth-app
trunk serve
```

Then open `http://127.0.0.1:8080` in your browser.

For a more guided setup and first-run walkthrough, see [Getting Started](docs/GETTING_STARTED.md).

## Features at a Glance

### Booth Management

- create, edit, close, reopen, and delete booth events
- configure participation fees, sales fee percentages, and rounding rules
- keep multiple booths on one device with independent data

### Checkout and Sales

- keyboard-first checkout flow with localized validation
- optional on-screen keypad with configurable quick amounts
- persistent amount entry mode and keypad visibility preferences
- draft recovery for interrupted checkout sessions
- correction and deletion flows with operator guidance

### Vendors and Reporting

- vendor handling with smart numeric and alphanumeric sorting
- vendor payout reports with consistent fee breakdowns
- booth summaries with payout-derived totals
- print-ready reporting workflows from the browser

### Data Safety

- IndexedDB persistence for booths, vendors, and purchases
- JSON export and import for full-database and per-booth backups
- conflict handling with `Merge`, `Skip`, and `Replace`
- browser storage warnings and backup guidance
- corruption detection and partial-recovery messaging

## Current Product Shape

What works today:

- booth management with fee configuration and status handling
- vendor management and checkout persistence
- reporting and printing flows
- bilingual German and English UI
- backup and restore workflows
- Chrome and Safari validation support
- standalone launcher builds for local distribution

Not available yet:

- automatic migration from original `ez-booth` SQLite data
- cloud sync or server-backed remote storage
- full PWA installation flow for field deployment

See [docs/redesign/05_STATUS.md](docs/redesign/05_STATUS.md) for the broader implementation history and current status tracking.

## Documentation

### Start Here

- Published site: [EZ Booth Documentation](https://tschuba.github.io/ez-booth-rs/)
- [Getting Started](docs/GETTING_STARTED.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Testing Guide](TESTING.md)

### User Guides

- [Fee Calculation Guide](docs/user-guides/FEE_CALCULATION.md)
- [Data Backup Guide](docs/user-guides/DATA_BACKUP_GUIDE.md)
- [Multi-Device Booth Merge Guide](docs/user-guides/MULTI_DEVICE_MERGE_GUIDE.md)

### Validation and Workflow

- [Validation Workflow](docs/validation/VALIDATION_WORKFLOW.md)
- [Safari Validation Checklist](docs/validation/SAFARI_VALIDATION_CHECKLIST.md)
- [Bilingual UAT Execution Plan](docs/validation/UAT_Ausfuehrungsplan_DE_EN.html)
- [Branch Strategy](docs/BRANCH_STRATEGY.md)

### Technical and Planning Docs

- [Comparison to the Original App](docs/COMPARISON_TO_ORIGINAL.md)
- [Redesign Summary](docs/redesign/REDESIGN_SUMMARY.md)
- [Technical Notes and ADRs](docs/technical/)
- [Planning Documents and Roadmap](docs/planning/)

## Architecture Overview

The app is a Rust Cargo workspace with focused crates:

- `crates/domain` for business rules, validation, and fee logic
- `crates/storage` for IndexedDB persistence and export/import support
- `crates/ez-booth-ui` for Leptos UI, routing, and translation wiring
- `crates/ez-booth-app` for WASM startup and Trunk bundling
- `crates/ez-booth-launcher` for optional local serving and desktop-style distribution

See [ARCHITECTURE.md](ARCHITECTURE.md) for the high-level architecture and [docs/redesign/02_ARCHITECTURE.md](docs/redesign/02_ARCHITECTURE.md) for the deeper redesign document.

## Building and Running

### Development Server

```bash
cd crates/ez-booth-app
trunk serve
```

### Production Web Bundle

```bash
cd crates/ez-booth-app
trunk build --release
```

### Workspace Build

```bash
cargo build --workspace --locked
```

### Standalone Launcher

```bash
cargo build --release -p ez-booth-launcher --locked
```

The launcher serves the built app locally, opens the browser, and enforces single-instance execution per device.

## Validation

Recommended validation commands:

- fast local suite: `./run-tests.sh`
- Chrome browser validation: `./run-tests.sh --chrome`
- Safari browser validation: `./run-tests.sh --safari`
- full automated suite: `./run-tests.sh --chrome --safari`

Manual validation assets:

- `docs/validation/SAFARI_VALIDATION_CHECKLIST.md`
- `docs/validation/UAT_Ausfuehrungsplan_DE_EN.html`
- `docs/validation/PHASE2_M2_VALIDATION_RESULTS.md`

For when to use which artifact, see [docs/validation/VALIDATION_WORKFLOW.md](docs/validation/VALIDATION_WORKFLOW.md).

## Releases

Stable downloads are published on the [GitHub Releases page](https://github.com/tschuba/ez-booth-rs/releases).

Typical release artifacts:

- `ez-booth-windows-vX.Y.Z.zip`
- `ez-booth-macos-vX.Y.Z.tar.gz`
- `ez-booth-linux-vX.Y.Z.tar.gz`
- `checksums.txt`

Each archive contains the launcher binary, the built web bundle, and a small usage note for operators.

Verify a downloaded release with:

```bash
shasum -a 256 -c checksums.txt
```

Maintainers should follow [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md).

## Troubleshooting

### Local Development

- if `trunk build` fails, run `npm ci` in `crates/ez-booth-app`
- if the launcher build fails, update Rust with `rustup update`
- if browser changes do not appear, hard-refresh the page
- if ports `8080-8089` are busy, stop the conflicting process and retry

### Browser Data and Downloads

- if the app shows a blank page from a downloaded bundle, launch it with the included binary instead of opening `index.html` directly
- if the launcher says another instance is running, remove the stale lock file from the user config directory referenced in the packaged readme
- if you clear browser storage, locally stored booth data can be lost unless you exported a backup first

## Contributing

Contributions are welcome.

Start here:

1. read [docs/BRANCH_STRATEGY.md](docs/BRANCH_STRATEGY.md)
2. review [AGENTS.md](AGENTS.md) for repo-specific coding rules
3. use [TESTING.md](TESTING.md) to run the smallest relevant validation set
4. open a pull request with the why, what was validated, and any deferred follow-up

## License

PolyForm-Noncommercial-1.0.0
