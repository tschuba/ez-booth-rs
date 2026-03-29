# Migration Strategy: ez-booth to ez-booth-rs

## Overview

This document outlines the strategy for migrating data from the existing ez-booth (Java/Spring Boot) application to ez-booth-rs (Rust/Leptos). The migration allows users to preserve their existing booth data, vendors, purchases, and reports when transitioning to the new application.

## Background

### ez-booth Data Storage

The current ez-booth application uses:
- **Backend**: Java Spring Boot with SQLite database
- **Location**: `~/Documents/tschuba/ez-booth/booth.db`
- **ORM**: Spring Data JPA with Hibernate
- **Schema Management**: Hibernate DDL auto-update
- **Sync**: gRPC-based synchronization (for multi-device scenarios)

### ez-booth-rs Data Storage

The new ez-booth-rs application will use:
- **Backend**: Rust with browser IndexedDB
- **Location**: Browser-specific IndexedDB storage
- **ORM**: Custom domain models with serde serialization
- **Schema Management**: Versioned migrations
- **Sync**: JSON import/export for cross-browser portability

## Migration Approach

### Strategy: Direct SQLite Database Access

The ez-booth-rs application will be **extended with migration functionality** that directly accesses the ez-booth SQLite database, extracts the data, and imports it into ez-booth-rs's IndexedDB storage.

#### Why This Approach?

1. **Integrated Experience**: Migration built into the main application
2. **Direct Access**: Reads directly from ez-booth's SQLite database
3. **No External Tools**: Users don't need to install or run separate utilities
4. **Seamless Import**: Data goes directly from SQLite to IndexedDB
5. **User-Friendly**: Single-step process within the application UI

#### Implementation Details

The migration will be implemented as:
- **WASM Component**: Rust code compiled to WebAssembly for browser execution
- **SQLite Access**: Using `sql.js` (SQLite compiled to WASM) to read ez-booth database
- **File Upload**: User uploads their `booth.db` file via file picker
- **In-Browser Processing**: All processing happens locally in the browser
- **Direct Import**: Transformed data written directly to IndexedDB

#### Alternative Considered: Separate CLI Utility

A standalone CLI tool (`ez-booth-migrate`) was initially considered but rejected because:
- Requires separate installation and execution
- Two-step process (export then import) is less user-friendly
- Additional maintenance burden for separate tool
- Integrated approach provides better UX

## Database Schema Analysis

### ez-booth SQLite Schema

Based on the protobuf model and Spring JPA configuration:

```sql
-- Booth table
CREATE TABLE booth (
    booth_id TEXT PRIMARY KEY,
    description TEXT,
    date DATE,
    participation_fee REAL,
    sales_fee REAL,
    fees_rounding_step REAL,
    closed BOOLEAN,
    closed_on TIMESTAMP
);

-- Vendor table
CREATE TABLE vendor (
    booth_id TEXT,
    vendor_id TEXT,
    PRIMARY KEY (booth_id, vendor_id),
    FOREIGN KEY (booth_id) REFERENCES booth(booth_id)
);

-- Purchase table
CREATE TABLE purchase (
    booth_id TEXT,
    purchase_id TEXT,
    value REAL,
    purchased_on TIMESTAMP,
    PRIMARY KEY (booth_id, purchase_id),
    FOREIGN KEY (booth_id) REFERENCES booth(booth_id)
);

-- Purchase Item table
CREATE TABLE purchase_item (
    booth_id TEXT,
    purchase_id TEXT,
    item_id TEXT,
    vendor_id TEXT,
    price REAL,
    purchased_on TIMESTAMP,
    PRIMARY KEY (booth_id, purchase_id, item_id),
    FOREIGN KEY (booth_id, purchase_id) REFERENCES purchase(booth_id, purchase_id),
    FOREIGN KEY (booth_id, vendor_id) REFERENCES vendor(booth_id, vendor_id)
);
```

### ez-booth-rs JSON Export Format

The target format for ez-booth-rs import:

```json
{
  "version": "1.0",
  "exported_at": "2024-03-20T14:30:00Z",
  "booths": [
    {
      "id": "booth-2024-spring",
      "name": "Frühjahrsbasar 2024",
      "date": "2024-03-15",
      "participation_fee": 5.0,
      "sales_fee_percent": 10.0,
      "fee_rounding_step": 0.5,
      "status": "closed",
      "closed_at": "2024-03-15T18:00:00Z",
      "created_at": "2024-02-01T10:00:00Z",
      "updated_at": "2024-03-15T18:00:00Z"
    }
  ],
  "vendors": [
    {
      "id": "vendor-123",
      "booth_id": "booth-2024-spring",
      "created_at": "2024-03-15T09:00:00Z"
    }
  ],
  "transactions": [
    {
      "id": "tx-001",
      "booth_id": "booth-2024-spring",
      "timestamp": "2024-03-15T10:30:00Z",
      "items": [
        {
          "id": "item-1",
          "vendor_id": "vendor-123",
          "price": 12.50
        }
      ],
      "total": 12.50
    }
  ]
}
```

## Implementation Plan

### Phase 1: Migration Module in ez-booth-rs

**Location**: `crates/ez-booth-migration/`

**Dependencies**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["File", "FileReader"] }
# Note: sql.js will be used via JS interop for SQLite access
```

**Features**:
- File upload handling for booth.db
- SQLite database parsing using sql.js
- Data validation and transformation
- Progress reporting
- Direct IndexedDB import
- Detailed error handling

### Phase 2: User Interface Integration

**UI Flow**:
```
Welcome Screen (if no booths exist)
├─ "Start Fresh" → Create new booth
└─ "Import from ez-booth" → Migration wizard
   ├─ Step 1: Instructions
   │  └─ "Locate your ez-booth database at ~/Documents/tschuba/ez-booth/booth.db"
   ├─ Step 2: Upload Database
   │  └─ File picker for booth.db
   ├─ Step 3: Processing
   │  ├─ Parse SQLite database
   │  ├─ Validate data
   │  └─ Show progress bar
   ├─ Step 4: Preview
   │  ├─ Show booth count, vendor count, transaction count
   │  └─ Display any warnings or issues
   └─ Step 5: Confirm Import
      ├─ Success → Show imported booths
      └─ Error → Show detailed error, offer export for support
```

**Import Process**:
1. User opens ez-booth-rs in browser
2. User clicks "Import from ez-booth"
3. User uploads booth.db file via file picker
4. Browser reads file and passes to WASM migration module
5. Migration module parses SQLite data using sql.js
6. Data is validated and transformed
7. Transformed data is written directly to IndexedDB
8. User can now use all their historical data in ez-booth-rs

### Phase 3: Data Transformation Rules

#### Booth Mapping
| ez-booth | ez-booth-rs | Transformation |
|----------|-------------|----------------|
| `booth_id` | `id` | Direct mapping |
| `description` | `name` | Direct mapping |
| `date` | `date` | Parse date string |
| `participation_fee` | `participation_fee` | Direct mapping |
| `sales_fee` | `sales_fee_percent` | Direct mapping (already percentage) |
| `fees_rounding_step` | `fee_rounding_step` | Direct mapping |
| `closed` | `status` | Map: `true` → `"closed"`, `false` → `"open"` |
| `closed_on` | `closed_at` | Parse timestamp |
| - | `created_at` | Use earliest `purchased_on` or `date` |
| - | `updated_at` | Use `closed_on` or latest `purchased_on` |

#### Vendor Mapping
| ez-booth | ez-booth-rs | Transformation |
|----------|-------------|----------------|
| `vendor_id` | `id` | Direct mapping |
| `booth_id` | `booth_id` | Direct mapping |
| - | `created_at` | Use earliest purchase timestamp for this vendor |

#### Purchase/Transaction Mapping
| ez-booth | ez-booth-rs | Transformation |
|----------|-------------|----------------|
| `purchase_id` | `id` | Direct mapping |
| `booth_id` | `booth_id` | Direct mapping |
| `purchased_on` | `timestamp` | Parse timestamp |
| `value` | `total` | Direct mapping |
| `purchase_item[]` | `items[]` | Map each item |

#### Purchase Item Mapping
| ez-booth | ez-booth-rs | Transformation |
|----------|-------------|----------------|
| `item_id` | `id` | Direct mapping |
| `vendor_id` | `vendor_id` | Direct mapping |
| `price` | `price` | Direct mapping |

### Phase 4: Validation & Error Handling

**Pre-Migration Validation**:
- Check database file exists and is readable
- Verify SQLite database format
- Check schema matches expected structure
- Validate referential integrity

**Data Validation**:
- Booth IDs are unique
- Vendor IDs are unique within booth
- All foreign keys resolve correctly
- Numeric values are valid (non-negative fees, prices)
- Dates and timestamps are parseable
- Transaction totals match sum of item prices

**Error Handling**:
- **Missing Database**: Clear error message with default location
- **Schema Mismatch**: Detect version and offer guidance
- **Corrupt Data**: Skip invalid records with warnings
- **Partial Migration**: Allow partial import if some data is valid
- **Export Errors**: Generate error report with details

### Phase 5: User Documentation

#### For End Users

**Migration Guide** (`docs/MIGRATION_GUIDE.md`):
```markdown
# Migrating from ez-booth to ez-booth-rs

## Prerequisites
- Your ez-booth database file: `~/Documents/tschuba/ez-booth/booth.db`
- Modern web browser (Chrome, Firefox, Safari, or Edge)

## Steps
1. Open ez-booth-rs in your browser
2. On the welcome screen, click "Import from ez-booth"
3. Click "Choose File" and navigate to your booth.db file
   - Default location: `~/Documents/tschuba/ez-booth/booth.db`
4. Wait for the migration to process (usually takes a few seconds)
5. Review the import preview showing your booths, vendors, and transactions
6. Click "Complete Import" to finish
7. Your data is now available in ez-booth-rs!

## What Gets Migrated
- All booth information (dates, fees, settings)
- All vendor registrations
- All purchase transactions and items
- Booth status (open/closed)

## After Migration
- Your original ez-booth database remains unchanged
- You can continue using ez-booth if needed
- Data in ez-booth-rs is stored in your browser
- Use export/import to transfer data between browsers

## Troubleshooting
[Common issues and solutions]
```

#### For Developers

**Technical Documentation** in IMPLEMENTATION.md:
- Migration utility architecture
- Database schema mapping
- Testing strategy
- Error scenarios

## Testing Strategy

### Unit Tests
- SQLite query functions
- Data transformation functions
- JSON serialization
- Validation logic

### Integration Tests
- End-to-end migration with sample database
- Import into ez-booth-rs
- Verify data integrity

### Test Data
Create sample databases:
- Empty database
- Single booth with vendors and transactions
- Multiple booths (open and closed)
- Edge cases: special characters, large numbers, boundary values

### Manual Testing
- Real migration from production ez-booth database
- Import into different browsers
- Verify reports match original ez-booth reports

## Security & Privacy

### Local-Only Processing
- Migration utility runs entirely locally
- No data sent to external servers
- JSON file contains sensitive business data - user must handle securely

### Data Sanitization
- Option to anonymize vendor IDs in migration (for testing/demos)
- Clear warnings about JSON file containing business data

## Timeline

1. **Phase 1**: Implement migration module structure (Week 1-2)
2. **Phase 2**: Integrate sql.js and SQLite parsing (Week 2-3)
3. **Phase 3**: Implement data transformation and validation (Week 3-4)
4. **Phase 4**: Build migration UI wizard (Week 4-5)
5. **Phase 5**: Testing and documentation (Week 5-6)
6. **Phase 6**: User acceptance testing with real data (Week 6-7)

## Future Enhancements

### Possible Additions
- **Incremental Updates**: Merge new ez-booth data into existing ez-booth-rs data
- **Automated Detection**: Detect if user previously used ez-booth and offer migration
- **Backup Creation**: Automatically create backup of original database before migration

### Not Planned
- **Separate CLI Tool**: Migration utility as standalone application
  - Reason: Integrated in-browser experience is more user-friendly
- **Live Sync**: Real-time synchronization between ez-booth and ez-booth-rs
  - Reason: ez-booth-rs is intended as a replacement, not a parallel system
- **gRPC Migration**: Using ez-booth's gRPC API for migration
  - Reason: Unnecessary complexity; direct database access is simpler

## Success Criteria

- ✅ 100% of booth data migrated correctly
- ✅ 100% of vendor data migrated correctly
- ✅ 100% of transaction data migrated correctly
- ✅ Referential integrity maintained
- ✅ Reports in ez-booth-rs match ez-booth reports
- ✅ Migration completes in < 5 seconds for typical database (10 booths, 100 vendors, 1000 transactions)
- ✅ Clear error messages for all failure scenarios
- ✅ User documentation covers all common scenarios

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Schema changes in ez-booth | High | Version detection, schema validation |
| Data corruption in SQLite | Medium | Validation step, skip corrupt records |
| Large database performance | Low | Stream processing, progress reporting |
| User forgets to run migration utility | Low | Clear UI prompts, documentation |
| JSON file too large for browser | Low | Chunked import, compression |

## Open Questions

- [X] Should we support merging migrated data with existing ez-booth-rs data?
  - Recommendation: No, require clean import (simpler, less error-prone)
  - Decision: No merging, user must choose to import into empty ez-booth-rs instance
- [X] Should migration be available only on first use or always accessible?
  - Recommendation: Always accessible via import entry points such as the booth list
  - Decision: Always accessible, but prominently offered on first use
- [X] Should we handle very large database files (>100MB)?
  - Recommendation: Show file size warning and use chunked processing
  - Decision: Implement chunked processing for files >50MB, show warning for >100MB
- [X] How to handle database file format changes in future ez-booth versions?
  - Recommendation: Implement version detection and schema validation
  - Decision: Implement version detection, support migration from last 2 versions of ez-booth

## References

- ez-booth database location: `~/Documents/tschuba/ez-booth/booth.db`
- ez-booth protobuf models: `/Users/thomas/Projects/ez-booth/core/src/main/protobuf/model.proto`
- ez-booth SQLite configuration: `/Users/thomas/Projects/ez-booth/server/src/main/resources/application.yaml`
- ez-booth-rs import/export spec: See ARCHITECTURE.md Section 4.3
