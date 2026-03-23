# EZ Booth Testing Guide

This document describes the testing strategy and how to run tests for the EZ Booth project.

## Test Structure

The project uses multiple levels of testing:

### 1. Unit Tests
**Location**: `crates/ez-booth-ui/src/components/booth_form_tests.rs`

Tests individual functions and logic in isolation, particularly:
- `BoothFormData` validation
- Form data conversion (`to_booth()`, `from_booth()`, `update_booth()`)
- Error handling for invalid inputs
- Edge cases (empty descriptions, negative fees, out-of-range values)

**Run with**:
```bash
cargo test -p ez-booth-ui --lib
```

**Coverage**:
- ✅ Valid booth creation from form data
- ✅ Invalid date formats
- ✅ Invalid numeric values  
- ✅ Negative fees
- ✅ Out-of-range percentages (> 100%)
- ✅ Empty descriptions
- ✅ Conversion from Booth to form data
- ✅ Updating existing booths
- ✅ Update validation failures

### 2. Integration Tests  
**Location**: `crates/storage/tests/booth_repository_tests.rs`

Tests the IndexedDB storage layer with real browser database operations:
- Save and retrieve booths
- Find by ID
- Find all booths
- Update existing booths
- Delete booths
- Concurrent operations
- Error handling

**Run with** (requires browser):
```bash
wasm-pack test --headless --chrome crates/storage
# or
wasm-pack test --headless --firefox crates/storage
```

**Coverage**:
- ✅ Save and find booth by ID
- ✅ Find all booths (empty and populated)
- ✅ Update booth data
- ✅ Delete booth
- ✅ Delete non-existent booth (idempotent)
- ✅ Concurrent save operations

### 3. Browser Component Tests (Future)
**Location**: `crates/ez-booth-ui/tests/` (to be created)

Tests Leptos components in a real browser environment:
- Component rendering
- User interactions
- Event handling
- Signal reactivity

**Run with**:
```bash
wasm-pack test --headless --chrome crates/ez-booth-ui
```

## Running Tests

### Prerequisites

1. **Rust toolchain**: Install from https://rustup.rs/
2. **wasm-pack**: Install for browser tests
   ```bash
   cargo install wasm-pack
   ```

### Run All Tests

```bash
# Unit tests (fast, no browser required)
cargo test --workspace --lib

# Integration tests (requires browser)
wasm-pack test --headless --chrome crates/storage
```

### Run Specific Test Suites

```bash
# Only UI unit tests
cargo test -p ez-booth-ui --lib

# Only storage integration tests  
wasm-pack test --headless --chrome crates/storage

# Run a specific test
cargo test -p ez-booth-ui test_to_booth_valid_data
```

### Watch Mode (Development)

```bash
# Rerun tests on file changes
cargo watch -x "test -p ez-booth-ui --lib"
```

## Test Configuration

### Cargo.toml Configuration

Unit tests are configured in each crate's `Cargo.toml`:

```toml
[dev-dependencies]
# For regular unit tests
# (uses existing workspace dependencies)

# For browser tests
wasm-bindgen-test = "0.3"
```

### Browser Test Configuration

Browser tests use `wasm-bindgen-test` which automatically:
- Compiles Rust to WebAssembly
- Starts a headless browser
- Runs tests in the browser environment
- Reports results back to the terminal

## Writing Tests

### Unit Tests Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let form = BoothFormData {
            description: "Test".to_string(),
            date: "2026-03-25".to_string(),
            // ...
        };
        
        let result = form.to_booth();
        assert!(result.is_ok());
    }
}
```

### Integration Tests Example

```rust
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_save_booth() {
    let db = create_test_db().await;
    let repo = IndexedDbBoothRepository::new(Arc::new(db));
    
    let booth = create_test_booth("Test");
    assert!(repo.save(&booth).await.is_ok());
}
```

## Continuous Integration

### GitHub Actions (Future)

Add to `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install wasm-pack
      - run: wasm-pack test --headless --chrome crates/storage
```

## Test Coverage

To generate coverage reports:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html
# Open tarpaulin-report.html
```

## Manual Testing Checklist

While automated tests cover logic and storage, manual browser testing is still needed for:

### Create Operation
- [ ] Valid form submission creates booth
- [ ] Empty fields show validation errors
- [ ] Invalid values show errors
- [ ] Booth appears immediately in list
- [ ] Success toast shows

### Read Operation  
- [ ] Booths display in grid
- [ ] All booth data shows correctly
- [ ] Page refresh preserves data
- [ ] Multiple booths display correctly

### Update Operation
- [ ] Edit button opens modal
- [ ] Modal pre-fills with existing data
- [ ] Valid changes save successfully
- [ ] Invalid changes show errors
- [ ] Changes appear immediately
- [ ] Cancel button discards changes

### Delete Operation
- [ ] Delete button opens confirmation
- [ ] Confirmation shows booth description
- [ ] Confirm deletes and removes from list immediately
- [ ] Cancel keeps booth
- [ ] Success toast shows

### UI/UX
- [ ] Language toggle works (EN/DE)
- [ ] Responsive layout (mobile/desktop)
- [ ] No console errors
- [ ] Loading states show appropriately

## Debugging Tests

### Enable Debug Logging

```rust
// In test
web_sys::console::log_1(&format!("Debug: {:?}", value).into());
```

### Run Tests with Backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

### Run Tests in Visible Browser

```bash
# Remove --headless to see browser
wasm-pack test --chrome crates/storage
```

## Known Issues

1. **Browser tests require Chrome/Firefox**: Safari support is limited
2. **IndexedDB cleanup**: Each test uses a unique database name to avoid conflicts
3. **Async timing**: Some tests may be flaky due to async operations

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Integration tests use unique DB names per test
3. **Descriptive names**: Test names should describe what they test
4. **AAA pattern**: Arrange, Act, Assert structure
5. **Fast feedback**: Unit tests should be fast (<1s each)
6. **Deterministic**: Tests should always produce the same result

## Resources

- [wasm-bindgen-test docs](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
- [Leptos testing](https://leptos-rs.github.io/leptos/testing.html)
- [Rust book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
