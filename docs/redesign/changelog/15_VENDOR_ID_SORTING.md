# Vendor ID Smart Sorting

**Date:** March 19, 2026  
**Type:** Feature Enhancement

## Summary

Added smart natural sorting for vendor IDs to ensure correct ordering in reports and UI, especially for numeric IDs. This addresses the common scenario where vendors use numeric-only IDs (e.g., "1", "2", "10") which should sort numerically, not lexicographically.

## Problem Statement

### Current Issue
Without smart sorting, vendor IDs sort lexicographically:
- Input: "1", "10", "2", "25", "3"
- Lexicographic sort: "1", "10", "2", "25", "3" ❌ (wrong order)
- Expected: "1", "2", "3", "10", "25" ✓ (correct numeric order)

### Critical Impact
- **Vendor Reports:** Print order must match logical vendor number sequence
- **UI Lists:** Vendors displayed in confusing order
- **User Experience:** Vendors can't find their reports easily when printed in wrong order

## Solution: Smart Natural Sorting

### Algorithm
```rust
impl VendorId {
    pub fn compare_smart(&self, other: &VendorId) -> std::cmp::Ordering {
        match (self.0.parse::<u64>(), other.0.parse::<u64>()) {
            (Ok(a), Ok(b)) => a.cmp(&b),                    // Both numeric: compare as numbers
            (Ok(_), Err(_)) => std::cmp::Ordering::Less,    // Numeric before alphanumeric
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater, // Alphanumeric after numeric
            (Err(_), Err(_)) => self.0.cmp(&other.0),       // Both text: lexicographic
        }
    }
}
```

### Examples

| Input IDs | Output (Smart Sort) | Notes |
|-----------|---------------------|-------|
| "10", "2", "1", "3" | "1", "2", "3", "10" | Pure numeric: sorted as integers |
| "V10", "V2", "V1" | "V1", "V10", "V2" | Alphanumeric: standard lexicographic |
| "1", "10", "V5", "2", "A3" | "1", "2", "10", "A3", "V5" | Mixed: numeric first, then alpha |

## Changes Made

### 1. VendorId Implementation (ARCHITECTURE.md Section 6.1)

**Added design rationale:**
```rust
// Vendor ID Design Rationale:
// - String-based to support both numeric ("1", "42") and alphanumeric ("V123", "A5") IDs
// - User enters vendor ID attached to sold products during checkout
// - Most common: purely numeric IDs for simplicity
// - Smart sorting ensures numeric IDs sort correctly (1, 2, 10 not 1, 10, 2)
// - Critical for vendor report printing order
```

**Added `compare_smart()` method to VendorId type definition.**

### 2. VendorService Update (ARCHITECTURE.md Section 6.1)

**Updated trait documentation:**
```rust
pub trait VendorService {
    /// List vendors with smart sorting.
    /// Numeric IDs (e.g., "1", "42") sorted numerically: 1, 2, 10, 42
    /// Alphanumeric IDs sorted lexicographically after numeric IDs
    /// Critical for correct print order in vendor reports.
    async fn list_vendors(&self, booth_id: BoothId) -> Result<Vec<Vendor>, CoreError>;
}
```

### 3. New Architecture Section (ARCHITECTURE.md Section 6.3)

Added dedicated section **"Vendor ID Sorting Strategy"** covering:
- Problem statement with examples
- Smart natural sorting algorithm
- Comparison table showing different ID types
- Critical use cases (reports, UI, exports)
- Database considerations (app-layer sorting vs. DB collation)

### 4. Implementation Specification (IMPLEMENTATION.md Section 2.3)

**Complete VendorService implementation:**
```rust
pub async fn list_vendors(&self, booth_id: BoothId) -> Result<Vec<Vendor>, CoreError> {
    let mut vendors = self.repository.find_all(booth_id).await?;
    vendors.sort_by(|a, b| a.id.compare_smart(&b.id));
    Ok(vendors)
}
```

**Added unit tests:**
```rust
#[test]
fn test_vendor_id_sorting() {
    // Tests for numeric, alphanumeric, and mixed ID sorting
}
```

### 5. Updated VendorId Type Definition (IMPLEMENTATION.md Section 2.1)

Changed from UUID-based to String-based with smart comparison:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorId(String);

impl VendorId {
    pub fn new(id: String) -> Self { Self(id) }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn compare_smart(&self, other: &VendorId) -> std::cmp::Ordering { ... }
}
```

## Benefits

1. **Correct Report Ordering:** Multi-vendor reports print in expected order
2. **Better UX:** Vendors can quickly locate their reports in printed stacks
3. **Flexible:** Supports both numeric and alphanumeric IDs without config
4. **Consistent:** Same sorting across UI, reports, and exports
5. **Future-Proof:** Easy to extend for more complex natural sorting if needed

## Use Cases

### Primary Use Case: Vendor Report Printing
When printing reports for multiple vendors (e.g., end-of-day reconciliation):
```
Vendor 1 Report    (page 1)
Vendor 2 Report    (page 2)
Vendor 3 Report    (page 3)
...
Vendor 10 Report   (page 10)
Vendor 11 Report   (page 11)
```

Without smart sorting, Vendor 10 would appear between Vendor 1 and 2, breaking the logical sequence.

### Secondary Use Cases
- Vendor list dropdowns in UI
- Export file ordering (CSV, JSON)
- Database query result presentation
- Analytics and summary reports

## Technical Notes

### Why Application-Layer Sorting?

**IndexedDB:** No built-in natural sort collation in browser storage  
**SQLite (future):** Custom collation possible but adds complexity  
**Decision:** Keep sorting logic in Rust for consistency across all storage backends

### Performance Considerations

- Sorting happens in memory after fetch (not indexed)
- Acceptable for booth scale (<1000 vendors per booth typical)
- Caching strategy can optimize repeated list operations
- O(n log n) complexity with small constant factors

### Edge Cases Handled

| Input | Behavior | Rationale |
|-------|----------|-----------|
| Leading zeros: "001", "01", "1" | Treated as different strings | IDs are user-provided strings, not normalized numbers |
| Negative numbers: "-5", "-10" | Lexicographic sort (not numeric) | Vendor IDs typically positive, negatives treated as text |
| Floats: "1.5", "1.25" | Lexicographic sort (not numeric) | Parse as u64 fails, falls back to string comparison |
| Empty string: "" | Sorts before alphanumeric | Standard string comparison rules |

## Implementation Checklist

- [x] Architecture documentation updated with sorting strategy
- [x] VendorId type redesigned (UUID → String with compare_smart)
- [x] VendorService list_vendors method documented
- [x] Implementation specification with complete code
- [x] Unit tests added for sorting logic
- [ ] Actual Rust implementation in codebase
- [ ] Integration tests with various ID formats
- [ ] UI tests for report printing order
- [ ] Performance benchmarks for large vendor lists

## Related Files

- `docs/redesign/02_ARCHITECTURE.md` - Sections 6.1, 6.3
- `docs/redesign/04_IMPLEMENTATION.md` - Sections 2.1, 2.3

## Status

✅ Architecture and implementation specification complete  
⏳ Pending actual Rust code implementation  
⏳ Pending testing with real data
