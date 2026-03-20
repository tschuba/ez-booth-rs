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

### Preferred Strategy: Direct SQLite Access

We will implement a **one-time migration utility** that reads directly from the ez-booth SQLite database and generates a JSON export file compatible with ez-booth-rs import functionality.

#### Why This Approach?

1. **Simplicity**: Direct database access is straightforward and doesn't require running ez-booth
2. **Independence**: Works even if ez-booth server is not running or configured
3. **Control**: Full control over data transformation and validation
4. **Compatibility**: Leverages existing JSON import/export feature for cross-browser data portability

#### Alternative Considered: gRPC Sync

The ez-booth application already provides gRPC-based sync functionality. However, this approach was rejected because:
- Requires ez-booth server to be running
- Adds complexity with gRPC client implementation in Rust/WASM
- Not necessary for one-time migration
- gRPC is designed for ongoing sync, not one-time migration

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

### Phase 1: Migration Utility (Rust CLI Tool)

Create a standalone Rust CLI tool: `ez-booth-migrate`

**Location**: `crates/ez-booth-migrate/`

**Dependencies**:
```toml
[dependencies]
rusqlite = "0.31"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
```

**Features**:
- Read from ez-booth SQLite database
- Validate and transform data
- Generate JSON export file
- Detailed logging and error handling
- Progress reporting

**Usage**:
```bash
# Auto-detect default database location
ez-booth-migrate export

# Specify custom database path
ez-booth-migrate export --db ~/custom/path/booth.db --output migration.json

# Verify migration without creating output
ez-booth-migrate verify --db ~/Documents/tschuba/ez-booth/booth.db
```

### Phase 2: Integration with ez-booth-rs

**Import Process**:
1. User runs migration utility to generate JSON file
2. User opens ez-booth-rs in browser
3. User uses "Import Data" feature to load JSON file
4. ez-booth-rs validates and imports data into IndexedDB
5. User can now use all their historical data in ez-booth-rs

**UI Flow**:
```
Welcome Screen (if no booths exist)
├─ "Start Fresh" → Create new booth
└─ "Import from ez-booth" → File picker
   ├─ Instructions: "Run ez-booth-migrate first"
   ├─ Select migration.json file
   ├─ Preview import (booth count, vendor count, etc.)
   └─ Confirm Import
      ├─ Success → Show imported booths
      └─ Error → Show detailed error, offer export for support
```

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
- ez-booth database at: ~/Documents/tschuba/ez-booth/booth.db
- ez-booth-migrate utility installed

## Steps
1. Download ez-booth-migrate for your platform
2. Run: ez-booth-migrate export
3. Open ez-booth-rs in your browser
4. Click "Import from ez-booth"
5. Select the migration.json file
6. Verify the preview and confirm import

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

1. **Week 1**: Implement migration utility CLI
2. **Week 2**: Implement data transformation and validation
3. **Week 3**: Integrate with ez-booth-rs import feature
4. **Week 4**: Testing and documentation
5. **Week 5**: User acceptance testing with real data

## Future Enhancements

### Possible Additions
- **Incremental Sync**: Merge new ez-booth data into existing ez-booth-rs data
- **Automated Detection**: Auto-detect ez-booth database and offer migration
- **Direct Database Mode**: Run ez-booth-rs directly on SQLite (desktop only)
- **Reverse Migration**: Export from ez-booth-rs back to ez-booth format

### Not Planned
- **Live Sync**: Real-time synchronization between ez-booth and ez-booth-rs
  - Reason: ez-booth-rs is intended as a replacement, not a parallel system
- **gRPC Migration**: Using ez-booth's gRPC API for migration
  - Reason: Unnecessary complexity for one-time migration

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

- [ ] Should we support merging migrated data with existing ez-booth-rs data?
  - Recommendation: No, require clean import (simpler, less error-prone)
- [ ] Should migration utility be bundled with ez-booth-rs or separate?
  - Recommendation: Separate for now, bundle later if widely used
- [ ] Should we archive the original database automatically?
  - Recommendation: Yes, create backup copy with timestamp

## References

- ez-booth database location: `~/Documents/tschuba/ez-booth/booth.db`
- ez-booth protobuf models: `/Users/thomas/Projects/ez-booth/core/src/main/protobuf/model.proto`
- ez-booth SQLite configuration: `/Users/thomas/Projects/ez-booth/server/src/main/resources/application.yaml`
- ez-booth-rs import/export spec: See ARCHITECTURE.md Section 4.3
