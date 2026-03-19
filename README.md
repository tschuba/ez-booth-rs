# ez-booth-rs

A Rust-based flea market booth management system for tracking vendor sales and calculating commissions.

## Project Structure

This project uses a Cargo workspace with multiple crates:

- **`crates/core`** - Pure domain logic with entities and business rules
- **`crates/storage`** - Storage abstraction layer (to be implemented)
- **`crates/frontend`** - Web UI using Leptos and WASM (to be implemented)
- **`crates/server`** - Optional backend server (to be implemented)
- **`crates/shared`** - Shared utilities (to be implemented)

## Current Status

### ✅ Phase 1: Core Domain (Completed)

The core domain logic has been implemented with:

- **Type-safe identifiers**: `BoothId`, `VendorId`, `PurchaseId`
- **Entities**:
  - `Booth` - Represents a flea market booth with commission configuration
  - `Vendor` - Vendors selling items at a booth
  - `Purchase` - Purchase transactions with multiple items
  - `PurchaseItem` - Individual items in a purchase
- **Error handling**: Comprehensive error types for validation and business logic
- **Smart vendor sorting**: Numeric IDs sort numerically (1, 2, 10) while alphanumeric IDs sort lexicographically

All core entities include:
- Validation logic
- Immutability where appropriate
- Full test coverage

### 📋 Next Steps

- Phase 2: Storage layer with IndexedDB
- Phase 3: Frontend UI with Leptos
- Phase 4: Localization (German primary, English fallback)
- Phase 5: Cross-browser data portability (export/import)

## Building

```bash
# Build the core crate
cargo build -p ez-booth-core

# Run tests
cargo test -p ez-booth-core

# Build entire workspace
cargo build
```

## Documentation

See the `docs/redesign/` folder for detailed architecture and implementation documentation:

- `00_SPEC.md` - Project specification
- `01_ANALYSIS.md` - Analysis of current implementation
- `02_ARCHITECTURE.md` - System architecture
- `03_IMPROVEMENTS.md` - Planned improvements
- `04_IMPLEMENTATION.md` - Implementation guide

## License

PolyForm-Noncommercial-1.0.0
