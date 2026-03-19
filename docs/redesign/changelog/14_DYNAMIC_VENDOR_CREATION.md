# Dynamic Vendor Creation During Checkout

**Date:** March 19, 2026  
**Type:** Architecture Clarification

## Summary

Updated architecture documentation to accurately reflect the dynamic vendor creation workflow used in the current ez-booth implementation, based on analysis of the Java codebase.

## Changes Made

### 1. Checkout Workflow Updated (Section 7.2)

**Previous:** Vendor selection from dropdown/grid  
**Now:** Dynamic vendor input during checkout

**New Workflow:**
1. User enters vendor ID (attached to product)
2. User enters item price
3. System auto-creates vendor if doesn't exist
4. Item added to cart
5. Repeat for additional items
6. Complete purchase with optional receipt

### 2. Vendor Data Model (Section 6.1)

**Updated `Vendor` struct:**
```rust
pub struct Vendor {
    pub id: VendorId,          // String-based ID (e.g., "V123")
    pub booth_id: BoothId,
    pub created_at: DateTime<Utc>,
    // Minimal data - vendors created dynamically during checkout
}
```

**Key Change:** `VendorId` changed from `Uuid` to `String` to support user-provided IDs like "V123", "42", etc.

### 3. Service Layer (Section 6.1)

**Added `PurchaseService`:**
```rust
pub trait PurchaseService {
    async fn checkout(
        &self,
        booth_id: BoothId,
        items: Vec<CheckoutItem>,
    ) -> Result<Purchase, PurchaseError>;
}

pub struct CheckoutItem {
    pub vendor_id: String,  // User-entered vendor ID
    pub price: Decimal,
    pub purchased_at: DateTime<Utc>,
}
```

**Added `VendorService`:**
```rust
pub trait VendorService {
    /// Get or create a vendor by ID
    async fn get_or_create(
        &self,
        booth_id: BoothId,
        vendor_id: String,
    ) -> Result<Vendor, VendorError>;
    
    async fn get_vendor_sales(...) -> Result<VendorSalesReport, ...>;
}
```

## Benefits

1. **Simplified UX:** No pre-registration of vendors needed
2. **Faster Checkout:** Direct vendor ID entry during purchase
3. **Flexibility:** Supports any vendor ID format (numeric, alphanumeric)
4. **Matches Current:** Aligns with existing Java implementation

## Implementation Notes

- Vendors table uses composite key `(booth_id, vendor_id)` where `vendor_id` is TEXT
- No validation on vendor ID format - accepts any string
- Vendor creation is idempotent (get-or-create pattern)
- Minimal vendor data stored initially, extensible for future fields

## Related Files

- `02_ARCHITECTURE.md` - Updated sections 6.1, 6.2, 7.2
- Original implementation: `/Users/thomas/Projects/ez-booth/core/src/main/java/tschuba/ez/booth/services/ServiceModel.java`

## Status

✅ Architecture documentation updated  
✅ Consistent with current implementation  
⏳ Pending implementation in Rust
