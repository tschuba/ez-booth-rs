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

### ✅ Phase 1: Foundation & Core Architecture (88% Complete)

**Domain Layer** - Comprehensive business logic implementation:
- **Type-safe identifiers**: `BoothId`, `VendorId`, `PurchaseId`
- **Entities**:
  - `Booth` - Flea market booth with fee configuration and status
  - `Vendor` - Vendors selling items at a booth
  - `Purchase` - Purchase transactions with multiple items
  - `PurchaseItem` - Individual items in a purchase
- **Services**: BoothService, VendorService, TransactionService with full CRUD operations
- **Smart vendor sorting**: Numeric IDs sort numerically (1, 2, 10) while alphanumeric IDs sort lexicographically
- **Comprehensive validation**: Using validator crate with custom validation logic
- **Error handling**: DomainError with 4 variants (Validation, NotFound, InvalidState, Storage)

**Storage Layer** - IndexedDB persistence:
- Repository pattern with async trait interfaces
- Three repositories: BoothRepository, VendorRepository, PurchaseRepository
- Efficient serialization with serde-wasm-bindgen
- Proper error propagation from StorageError to DomainError

### ✅ Phase 2: UI Foundation (67% Complete)

**Internationalization**:
- Custom JSON-based i18n system
- German (primary) and English (fallback) translations
- Browser locale detection
- Translation context with `use_translations()` hook and `t!` macro

**Component Library**:
- Button (variants, sizes, states)
- Input and NumberInput
- Card and Container layout components
- Tailwind CSS styling

**Application Structure**:
- Leptos 0.6 with client-side rendering
- Leptos Router for navigation
- WASM build pipeline with Trunk
- Auto-initialization with wasm_bindgen

### 📋 Next Steps

- Phase 2.2: Expand component library (Modal, Toast, Form helpers)
- Phase 2.3: Global error handling system
- Phase 3: Core features (Booth management, Checkout flow, Reports)
- Phase 4: Testing and refinement
- Phase 5: Production deployment

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

# Run tests
cargo test

# Run tests for specific crate
cargo test -p domain
```

## Documentation

See the `docs/redesign/` folder for detailed architecture and implementation documentation:

- `00_SPEC.md` - Project specification
- `01_ANALYSIS.md` - Analysis of current implementation
- `02_ARCHITECTURE.md` - System architecture
- `03_IMPROVEMENTS.md` - Planned improvements
- `04_IMPLEMENTATION.md` - Implementation guide with code examples
- `05_STATUS.md` - **Current implementation progress (33% complete)**

## License

PolyForm-Noncommercial-1.0.0
