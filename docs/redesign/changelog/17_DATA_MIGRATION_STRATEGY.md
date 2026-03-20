# Data Migration Strategy from ez-booth to ez-booth-rs

## Overview

This document outlines the strategy for migrating data from the existing ez-booth (Java/Vaadin) application to ez-booth-rs (Rust/Leptos).

## Current ez-booth Architecture

### Storage
- **Database**: SQLite (`${user.home}/Documents/tschuba/ez-booth/booth.db`)
- **Driver**: JDBC SQLite
- **Platform**: Hibernate with SQLite dialect

### Sync/Export Capabilities
ez-booth already provides gRPC-based data exchange services:

```protobuf
service DataExchangeService {
  rpc SyncData(ExchangeData) returns (ExchangeData);
  rpc ExportData(BoothKey) returns (ExchangeData);
  rpc MergeData(ExchangeData) returns (google.protobuf.Empty);
}

message ExchangeData {
  Booth booth = 1;
  repeated Vendor vendors = 2;
  repeated Purchase purchases = 3;
}
```

## Migration Options

### Option 1: gRPC-Based Migration (Recommended)
**Leverage existing DataExchangeService for clean, API-driven migration**

#### Advantages
- ✅ Uses official, tested ez-booth API
- ✅ Clean separation between applications
- ✅ No direct database access needed
- ✅ Handles data validation and consistency
- ✅ Future-proof for potential ez-booth updates
- ✅ Can support live migration while ez-booth is running

#### Implementation Approach
1. **Add gRPC client to ez-booth-rs**
   - Use `tonic` crate for Rust gRPC support
   - Generate Rust bindings from ez-booth proto files
   - Create migration service in ez-booth-rs

2. **Migration Workflow**
   ```
   ez-booth-rs (Migration Tool)
   ↓
   gRPC Client → ez-booth DataExchangeService
   ↓
   ExportData(BoothKey) → ExchangeData
   ↓
   Transform to ez-booth-rs models
   ↓
   Store in IndexedDB
   ```

3. **User Experience**
   - UI button: "Import from ez-booth"
   - Input: ez-booth server endpoint (default: localhost:9090)
   - Select booths to import
   - Progress indicator during import
   - Success/error reporting

#### Technical Requirements
- **Dependencies**:
  - `tonic` - gRPC client
  - `prost` - Protocol Buffers implementation
  - `tokio` - Async runtime
- **Build Process**: Add proto file compilation to build.rs
- **Error Handling**: Network errors, authentication, data validation

### Option 2: Direct SQLite Access
**Read directly from ez-booth's SQLite database**

#### Advantages
- ✅ Fast bulk import
- ✅ No need for ez-booth to be running
- ✅ Complete control over data extraction

#### Disadvantages
- ❌ Tightly coupled to ez-booth's database schema
- ❌ Breaks if ez-booth schema changes
- ❌ Requires understanding of Hibernate's table structure
- ❌ No validation through ez-booth's business logic
- ❌ Risk of data inconsistency

#### Implementation Approach
1. **Add SQLite reader**
   - Use `rusqlite` crate
   - Map ez-booth Hibernate entities to queries
   - Transform to ez-booth-rs models

2. **User Experience**
   - File picker to select booth.db
   - Default path: `~/Documents/tschuba/ez-booth/booth.db`
   - Parse and import data
   - Progress indicator

#### Technical Requirements
- **Dependencies**: `rusqlite`, `serde`
- **Schema Knowledge**: Reverse-engineer Hibernate table structure
- **Maintenance**: Update if ez-booth schema changes

### Option 3: JSON Export/Import
**Export from ez-booth to JSON, import to ez-booth-rs**

#### Advantages
- ✅ Simple, human-readable format
- ✅ Can be version-controlled
- ✅ Easy debugging and inspection

#### Disadvantages
- ❌ Requires ez-booth to add JSON export feature
- ❌ Manual two-step process
- ❌ Not suitable for large datasets

## Recommended Approach: Hybrid Strategy

### Phase 1: gRPC Migration (Priority 1)
Implement gRPC-based migration as the primary method:
- Best user experience
- Most maintainable
- Supports selective booth import

### Phase 2: SQLite Fallback (Priority 2)
Add direct SQLite access as fallback:
- For cases where ez-booth server isn't available
- Bulk migration of historical data
- Emergency recovery scenarios

### Phase 3: JSON Export (Priority 3)
Optional JSON import/export for:
- Data portability
- Backup/restore
- Debugging

## Implementation Plan

### Step 1: gRPC Integration
```rust
// Add to Cargo.toml
[dependencies]
tonic = "0.10"
prost = "0.12"
tokio = { version = "1.0", features = ["full"] }

[build-dependencies]
tonic-build = "0.10"
```

### Step 2: Proto File Integration
```rust
// build.rs
fn main() {
    tonic_build::configure()
        .build_server(false) // Client only
        .compile(
            &["proto/services.proto", "proto/model.proto"],
            &["proto/"],
        )
        .unwrap();
}
```

### Step 3: Migration Service
```rust
pub struct EzBoothMigration {
    client: DataExchangeServiceClient<Channel>,
}

impl EzBoothMigration {
    pub async fn connect(endpoint: String) -> Result<Self>;
    pub async fn list_booths() -> Result<Vec<Booth>>;
    pub async fn export_booth(booth_key: BoothKey) -> Result<ExchangeData>;
    pub async fn import_to_indexed_db(data: ExchangeData) -> Result<()>;
}
```

### Step 4: UI Integration
- Add migration page/modal
- Server endpoint configuration
- Booth selection interface
- Progress tracking
- Error handling and retry logic

## Data Mapping

### Booth Mapping
```
ez-booth.Booth → ez-booth-rs.Booth
├── boothId → id
├── description → name
├── date → date
├── participation_fee → default_participation_fee
├── sales_fee → default_sales_fee
├── fees_rounding_step → fees_rounding_step
└── closed → status (Open/Closed)
```

### Vendor Mapping
```
ez-booth.Vendor → ez-booth-rs.Vendor
├── vendorId → id (with smart sorting)
└── booth → booth_id
```

### Purchase Mapping
```
ez-booth.Purchase → ez-booth-rs.Transaction
├── purchaseId → id
├── items → items
├── value → total_amount
└── purchased_on → timestamp
```

### Purchase Item Mapping
```
ez-booth.PurchaseItem → ez-booth-rs.TransactionItem
├── itemId → id
├── vendor → vendor_id
├── price → amount
└── purchased_on → timestamp
```

## Testing Strategy

### Unit Tests
- Proto message conversion
- Data transformation logic
- Error handling

### Integration Tests
- Connect to test ez-booth instance
- Export sample booth data
- Verify import into IndexedDB
- Data integrity checks

### Manual Testing
- Test with real ez-booth database
- Verify report generation after import
- Cross-check calculations
- Print functionality

## Security Considerations

1. **gRPC Connection**
   - Support TLS for remote servers
   - Authentication if required
   - Timeout handling

2. **SQLite Access**
   - Read-only access
   - File permissions check
   - Handle corrupted databases

3. **Data Validation**
   - Verify data integrity after import
   - Check for required fields
   - Validate calculations

## User Documentation

### Migration Guide
1. Ensure ez-booth server is running (for gRPC method)
2. Open ez-booth-rs
3. Navigate to Settings → Data Migration
4. Choose migration method:
   - **From running ez-booth server**: Enter endpoint
   - **From database file**: Select booth.db
5. Select booths to import
6. Click "Import"
7. Verify imported data

### Troubleshooting
- **Cannot connect to ez-booth**: Check server is running, verify endpoint
- **Database file not found**: Check default path or select manually
- **Import fails**: Check logs, verify data integrity, contact support

## Future Enhancements

### Bi-directional Sync
- Keep ez-booth and ez-booth-rs in sync
- Real-time updates
- Conflict resolution

### Incremental Updates
- Only import new/changed data
- Track last sync timestamp
- Delta synchronization

### Batch Migration
- Import multiple booths at once
- Progress tracking per booth
- Parallel imports for performance

## Decision

**Recommended**: Start with **Option 1 (gRPC-based migration)** for the cleanest, most maintainable solution.

- Implement during **Phase 2** of the redesign
- Add as a separate feature after core functionality is stable
- Create dedicated migration UI component
- Document thoroughly for users

This approach provides the best balance of:
- User experience (automated, guided process)
- Maintainability (uses official API)
- Reliability (built-in validation)
- Future-proofing (independent of schema changes)
