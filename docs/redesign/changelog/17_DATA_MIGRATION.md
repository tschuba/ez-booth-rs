# Data Migration from ez-booth to ez-booth-rs

## Overview

This document describes the optional data migration strategy from the original Java-based ez-booth application to the new Rust-based ez-booth-rs application.

## Source Data Structure (ez-booth)

The original ez-booth uses a PostgreSQL database with JPA entities:

### Tables
- `ez_booth.booths` - Booth/event definitions
- `ez_booth.vendors` - Vendor registrations per booth
- `ez_booth.purchases` - Purchase transactions
- `ez_booth.purchase_items` - Individual items in purchases

### Data Model Mapping

| ez-booth Field | ez-booth-rs Field | Notes |
|----------------|-------------------|-------|
| `Booth.boothId` | `Booth.id` | Direct mapping |
| `Booth.description` | `Booth.name` | Renamed for clarity |
| `Booth.date` | `Booth.date` | Direct mapping |
| `Booth.participationFee` | `Booth.participation_fee` | Direct mapping |
| `Booth.salesFee` | `Booth.commission_rate` | Renamed, same meaning |
| `Booth.feesRoundingStep` | `Booth.rounding_step` | Direct mapping |
| `Booth.closed` | `Booth.status` | Boolean → Enum (Active/Closed) |
| `Booth.closedOn` | `Booth.closed_at` | Direct mapping |
| `Vendor.vendorId` | `Vendor.id` | Direct mapping |
| `Vendor.booth` | `Vendor.booth_id` | Foreign key relationship |
| `Purchase.purchaseId` | `Transaction.id` | Renamed entity |
| `Purchase.value` | `Transaction.total` | Direct mapping |
| `Purchase.purchasedOn` | `Transaction.timestamp` | Direct mapping |
| `PurchaseItem` | `TransactionItem` | Renamed entity |
| `PurchaseItem.vendorId` | `TransactionItem.vendor_id` | Direct mapping |
| `PurchaseItem.price` | `TransactionItem.amount` | Direct mapping |

## Migration Strategy

### Phase 1: Export from ez-booth (Optional Tool)

Create a migration export tool that:

1. **Database Export**
   - Connect to PostgreSQL database
   - Query all booths, vendors, purchases, and items
   - Export to JSON format compatible with ez-booth-rs

2. **Export Format (JSON)**
```json
{
  "version": "1.0",
  "export_date": "2026-03-20T14:00:00Z",
  "booths": [
    {
      "id": "booth001",
      "name": "Spring Festival 2025",
      "date": "2025-05-15",
      "participation_fee": "50.00",
      "commission_rate": "0.15",
      "rounding_step": "0.50",
      "status": "Closed",
      "closed_at": "2025-05-15T18:30:00Z",
      "vendors": [
        {
          "id": "001",
          "name": "Vendor A",
          "sales_total": "450.00",
          "items_sold": 23
        }
      ],
      "transactions": [
        {
          "id": "tx001",
          "timestamp": "2025-05-15T10:15:00Z",
          "total": "25.50",
          "items": [
            {
              "vendor_id": "001",
              "amount": "15.00"
            },
            {
              "vendor_id": "002",
              "amount": "10.50"
            }
          ]
        }
      ]
    }
  ]
}
```

### Phase 2: Import into ez-booth-rs

1. **Import Tool in Web UI**
   - File upload interface for JSON export
   - Validation and preview before import
   - Conflict resolution (if booth IDs already exist)

2. **Migration Process**
   - Parse JSON file
   - Validate data integrity
   - Convert data types (dates, decimals, enums)
   - Import into IndexedDB
   - Generate migration report

3. **Data Validation**
   - Verify all booths imported
   - Verify vendor counts match
   - Verify transaction totals match
   - Verify item counts match

## Implementation Plan

### Step 1: Export Tool (Optional, Java)

Create standalone CLI tool in ez-booth repository:

```bash
# Location: ez-booth/migration-tool
java -jar ez-booth-migration-tool.jar \
  --db-url jdbc:postgresql://localhost:5432/ezboot \
  --db-user postgres \
  --output export.json
```

**Features:**
- Connect to PostgreSQL
- Export all data for selected booths
- Anonymize sensitive data option
- Validate export completeness

### Step 2: Import Component (Rust/Leptos)

Add to ez-booth-rs:

**File:** `crates/ez-booth-app/src/features/migration/mod.rs`

```rust
pub struct MigrationImport {
    pub version: String,
    pub export_date: DateTime<Utc>,
    pub booths: Vec<BoothExport>,
}

pub struct BoothExport {
    pub id: String,
    pub name: String,
    pub date: NaiveDate,
    // ... other fields
    pub vendors: Vec<VendorExport>,
    pub transactions: Vec<TransactionExport>,
}
```

**UI Location:** Settings → Data Management → Import Legacy Data

### Step 3: Validation & Testing

1. **Test Cases**
   - Empty export
   - Single booth with minimal data
   - Multiple booths with complex transactions
   - Invalid/corrupted data
   - Duplicate booth IDs

2. **Validation Rules**
   - All amounts must be positive
   - Transaction totals must match sum of items
   - Vendor IDs must exist
   - Dates must be valid
   - No orphaned records

## User Experience

### Migration Workflow

1. **In ez-booth (Old System)**
   - User downloads migration tool
   - Runs export command
   - Receives `export.json` file

2. **In ez-booth-rs (New System)**
   - User navigates to Settings → Data Management
   - Clicks "Import Legacy Data"
   - Selects `export.json` file
   - Reviews preview of data to import
   - Confirms import
   - Views migration report
   - Verifies data accuracy

### User Guidance

**First-time users see:**
- Welcome screen detecting no existing data
- Option to "Import from ez-booth" or "Start Fresh"
- Link to migration documentation
- Video tutorial (future)

**Migration page includes:**
- Clear instructions
- File format requirements
- Expected completion time
- Progress indicator during import
- Detailed error messages if validation fails

## Technical Considerations

### Data Type Conversions

1. **BigDecimal → Rust Decimal**
   - Parse using `rust_decimal` crate
   - Maintain precision (2 decimal places)
   - Validate range (0 to 999999.99)

2. **LocalDateTime → Chrono DateTime**
   - Parse ISO 8601 format
   - Convert to UTC
   - Store with timezone info

3. **Boolean Status → Enum**
   - `closed: true` → `Status::Closed`
   - `closed: false` → `Status::Active`

### Error Handling

1. **Recoverable Errors**
   - Invalid vendor reference → Skip item, log warning
   - Invalid date format → Request user clarification
   - Missing optional fields → Use defaults

2. **Fatal Errors**
   - Corrupted JSON structure
   - Incompatible version
   - Database constraint violations
   - Insufficient storage space

### Performance

- Stream large files (don't load entirely in memory)
- Batch IndexedDB writes (100 records at a time)
- Show progress for imports > 5 seconds
- Cancel operation option

## Security Considerations

1. **Data Sanitization**
   - Validate all inputs
   - Prevent script injection in text fields
   - Limit file size (max 50MB)

2. **Privacy**
   - All data stays in browser
   - No upload to external servers
   - Clear warning about data sensitivity

3. **Integrity**
   - Checksum validation (optional)
   - Verify transaction totals
   - Audit trail of imported data

## Future Enhancements

1. **Direct Database Migration**
   - Server-side tool that writes directly to IndexedDB format
   - Eliminates intermediate JSON step

2. **Incremental Migration**
   - Import specific booth by ID
   - Update existing booth with new transactions

3. **Migration Rollback**
   - Backup before import
   - Restore previous state if needed

4. **Data Transformation**
   - Custom field mapping
   - Data cleanup rules
   - Merge duplicate vendors

## Priority

**P1 (Optional Enhancement)**
- Not required for MVP
- Can be implemented after core features are stable
- Useful for existing ez-booth users transitioning
- Can be a separate module/plugin

## Implementation Status

- [ ] Design export tool architecture
- [ ] Implement export CLI tool (Java)
- [ ] Design import data structures (Rust)
- [ ] Implement JSON parser and validator
- [ ] Create import UI component
- [ ] Add preview functionality
- [ ] Implement data conversion logic
- [ ] Add error handling and recovery
- [ ] Create migration documentation
- [ ] Test with real ez-booth data
- [ ] Add migration report generation

## Notes

- Migration is **optional** - users can start fresh
- Export tool can be maintained separately from ez-booth-rs
- Consider creating a Docker container for easy export tool deployment
- Document known limitations and edge cases
