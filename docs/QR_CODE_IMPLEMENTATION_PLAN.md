# QR Code Export/Import Implementation Plan

**Last Updated**: 2026-03-29  
**Status**: Planning Phase  
**Target**: Single booth export/import via QR codes for device-to-device transfers

---

## Table of Contents

1. [Overview](#overview)
2. [Design Decisions](#design-decisions)
3. [Capacity Analysis](#capacity-analysis)
4. [UX Flow](#ux-flow)
5. [Technical Implementation](#technical-implementation)
6. [Implementation Sequence](#implementation-sequence)
7. [Testing Strategy](#testing-strategy)
8. [Translations](#translations)

---

## Overview

### Purpose

Add QR code-based export/import functionality for **single booth transfers** between devices. This complements the existing JSON file-based backup system by providing a camera-friendly alternative for quick device-to-device data transfers.

### Scope

**In Scope:**
- ✅ Single booth export as QR codes
- ✅ Time-filtered exports (Last 24h, 7d, 30d, Full)
- ✅ Multi-QR chunking with progress tracking
- ✅ Camera-based QR scanning with live feedback
- ✅ Import preview with conflict resolution
- ✅ Binary serialization for optimal compression

**Out of Scope:**
- ❌ Full database export via QR (use JSON file export)
- ❌ Multi-booth QR exports
- ❌ QR code encryption
- ❌ Cloud/server-based QR generation

### Use Cases

**Primary:**
- Transfer booth from organizer's laptop to vendor's phone
- Quick backup snapshots (daily/weekly) via QR
- Share booth data without file transfer complexity

**Not Suitable For:**
- Long-term archival (use JSON export)
- Very large booths with thousands of purchases (use JSON export)
- Bulk imports (use JSON export)

---

## Design Decisions

### 1. Default Export Scope: Last 7 Days

**Rationale**: Balances data coverage with QR code count. Typical booths generate 2-9 QR codes with 7-day filter.

**Alternatives Considered:**
- Last 24 hours: Too limited, misses data if exported infrequently
- Last 30 days: Often generates too many codes (10-30+)
- Full history: Frequently exceeds practical limits

### 2. Hard Limit: 10 QR Codes Maximum

**Rationale**: Enforces good UX by preventing tedious scanning sessions. Scanning 10 codes takes ~3-5 minutes; beyond that becomes impractical.

**Implementation**: 
- Export button disabled if estimate exceeds 10 codes
- User shown clear explanation and alternatives (shorter range or JSON export)

### 3. Binary Serialization (bincode)

**Rationale**: 60% smaller than JSON, reducing QR codes needed.

**Comparison:**
- JSON: Human-readable if decoded, but larger
- Binary (bincode): 60% size reduction, not human-readable
- **Decision**: Use binary to maximize capacity (QR not meant for human reading anyway)

### 4. Import Conflict Resolution: Show Preview Modal

**Rationale**: Reuses existing import preview UI, gives user control over merge strategy.

**Alternatives Considered:**
- Auto-merge: Less control, may surprise users
- Always skip: Too conservative, limits usefulness

---

## Capacity Analysis

### Data Structure Sizes (Binary Serialization)

**Per Record (bincode):**
- Booth: ~400 bytes (metadata, fees, validation rules, keyboard config)
- Vendor: ~80 bytes (vendor_id, booth_id, optional name, timestamp)
- Purchase: ~200 bytes (id, booth_id, timestamp, 2-3 items with amounts/vendor_ids)

**Compression:**
- Gzip compression: ~70% reduction on binary data
- Combined optimization: JSON → Binary (60%) → Gzip (70%) = **~88% total reduction**

### QR Code Technical Limits

- **QR Version 40, Low error correction**: ~2,953 bytes maximum
- **Practical target** (reliable scanning): keep the final QR payload around ~2.4 KB or below
- **Important overhead**: each chunk is base64-encoded and wrapped in JSON metadata before rendering as a QR code
- **Chunk size used in implementation**: 1,800 raw compressed bytes per chunk, which expands to roughly ~2.4 KB once base64 + JSON overhead is included

### Realistic Booth Scenarios

| Scenario | Vendors | Purchases | Scope | Final Size | QR Codes | Assessment |
|----------|---------|-----------|-------|------------|----------|------------|
| **Weekend Market** | 20 | 100 | Full | 1.5 KB | **1 code** | ✅ Perfect |
| **3-Day Event** | 50 | 300 | Full | 4 KB | **3 codes** | ✅ Excellent |
| **Weekly Market** | 50 | 500 | Last 7d | 3 KB | **2 codes** | ✅ Excellent |
| **Monthly Market** | 100 | 2,000 | Last 7d | 9 KB | **5 codes** | ✅ Good |
| **Monthly Market** | 100 | 2,000 | Full | 32 KB | **18 codes** | ❌ Use JSON |
| **Large Booth** | 100 | 2,000 | Last 7d (~467) | 9 KB | **5 codes** | ✅ Good |
| **Permanent Shop** | 200 | 10,000 | Last 7d (~385) | 16 KB | **9 codes** | ✅ Acceptable |
| **Permanent Shop** | 200 | 10,000 | Full | 270 KB | **150 codes** | ❌ Use JSON |

Note: QR code counts are based on compressed binary size divided by the 1,800-byte raw chunk size. The final QR payload is larger because `chunk.d` is base64-encoded and the full chunk is serialized to JSON, so chunk sizing must stay comfortably below the theoretical QR limit.

### Key Insights

1. **Sweet Spot**: 7-day exports for active booths (2-9 codes typical)
2. **Time Filtering Critical**: Makes large booths practical
3. **10-Code Limit**: Covers vast majority of realistic use cases
4. **Guidance Needed**: UI must guide users toward appropriate time ranges

---

## UX Flow

### Export Flow: Device A (Source)

#### Step 1: Initiate Export

**Location**: Booth list page, individual booth card

**User Action**: 
1. Click dropdown menu on booth card
2. Select "Export as QR Code"

```
┌─────────────────────────────┐
│ View Details                │
│ Export Booth (JSON)         │
│ Export as QR Code          │← NEW
│ Delete Booth                │
└─────────────────────────────┘
```

#### Step 2: Configure Export

**Modal Opens**: QR Export Configuration

**Default State:**
```
┌────────────────────────────────────────────────────┐
│ Export as QR Codes                      [×]        │
│ Spring Market 2026                                 │
├────────────────────────────────────────────────────┤
│                                                    │
│ Time Range:                                        │
│ ┌──────────────────────────────────────────────┐  │
│ │ ○ Last 24 hours         (~2 codes)          │  │
│ │ ● Last 7 days           (~5 codes)          │← Default
│ │ ○ Last 30 days          (~12 codes)         │  │
│ │ ○ Full history          (~18 codes)         │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ 📊 Estimated: 5 QR codes                          │
│    Data size: ~8.2 KB                             │
│    287 purchases (63 in last 7 days)              │
│                                                    │
│                                  [Generate Codes] │
└────────────────────────────────────────────────────┘
```

**Features:**
- Live estimate updates as user changes time range
- Shows filtered vs total purchase count
- Generate button enabled if ≤10 codes

**If Exceeds Limit (>10 codes):**
```
┌────────────────────────────────────────────────────┐
│ Export as QR Codes                      [×]        │
│ Spring Market 2026                                 │
├────────────────────────────────────────────────────┤
│                                                    │
│ Time Range:                                        │
│ ┌──────────────────────────────────────────────┐  │
│ │ ● Full history          (~18 codes)         │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ ⚠️ Warning: 18 QR codes required                   │
│                                                    │
│ This exceeds the 10-code limit. QR codes work     │
│ best for quick transfers of recent data.          │
│                                                    │
│ Suggestions:                                       │
│ • Select a shorter time range (Last 7 days)       │
│ • Use "Export Booth (JSON)" for full backups      │
│                                                    │
│                          [Generate Codes] ← DISABLED
└────────────────────────────────────────────────────┘
```

**User Guidance:**
- Clear explanation of why limit exists
- Actionable suggestions provided
- Easy to adjust without closing modal

#### Step 3: Generate QR Codes

**User Action**: Click [Generate Codes]

**Loading State** (2-3 seconds):
```
┌────────────────────────────────────────────────────┐
│ Export as QR Codes                      [×]        │
│ Spring Market 2026                                 │
├────────────────────────────────────────────────────┤
│                                                    │
│             ⌛ Generating QR codes...               │
│                                                    │
│    [████████████░░░░░░░░] 67%                     │
│                                                    │
│    Compressing data...                            │
│                                                    │
└────────────────────────────────────────────────────┘
```

**Processing Steps:**
1. Fetch booth data (booth, vendors, purchases)
2. Apply time filter
3. Serialize to binary (bincode)
4. Compress (gzip)
5. Calculate SHA-256 hash
6. Split into chunks (~1.8KB each)
7. Generate QR codes (SVG format)

#### Step 4: Display QR Codes (Pagination)

**First QR Code:**
```
┌────────────────────────────────────────────────────┐
│ Export as QR Codes                      [×]        │
│ Spring Market 2026                                 │
├────────────────────────────────────────────────────┤
│                                                    │
│            ┌─────────────────────┐                 │
│            │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                 │
│            │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                 │
│            │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │← QR Code (SVG) │
│            │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │   Sharp render │
│            │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │                 │
│            └─────────────────────┘                 │
│                                                    │
│                Code 1 of 5                         │
│            ● ○ ○ ○ ○  ← Progress dots             │
│                                                    │
│    [< Previous]              [Next >]              │
│    └─ Disabled               Enabled ─┘            │
│                                                    │
│ 📱 Scan each code with your other device           │
│                                                    │
│                                        [Close]     │
└────────────────────────────────────────────────────┘
```

**Navigation:**
- Previous button: Disabled on first code, enabled otherwise
- Next button: Enabled until last code, then disabled
- Progress dots: Visual indicator of position
- QR changes instantly (no loading between codes)

**User completes export by scanning all codes on Device B...**

---

### Import Flow: Device B (Destination)

#### Step 1: Initiate Import

**Location**: Booth list page

**User Action**: 
1. Click [Import] button
2. Select "Import from QR Code"

```
┌─────────────────────────────┐
│ Import from File (JSON)     │
│ Import from QR Code        │← NEW
└─────────────────────────────┘
```

#### Step 2: Camera Permission

**Browser Native Prompt:**
```
┌────────────────────────────────────────────────────┐
│ 📷 Camera Access Required                          │
│                                                    │
│ ez-booth-rs wants to access your camera to scan   │
│ QR codes for importing booth data.                │
│                                                    │
│                   [Block]      [Allow]             │
└────────────────────────────────────────────────────┘
```

**If User Blocks:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ❌ Camera access denied                            │
│                                                    │
│ To scan QR codes, please enable camera            │
│ permissions in your browser settings.              │
│                                                    │
│ Alternative:                                       │
│ • Use "Import from File" for JSON files           │
│                                                    │
│                    [Close]   [Try Again]           │
└────────────────────────────────────────────────────┘
```

#### Step 3: Scanner Interface

**After Permission Granted:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ┌────────────────────────────────────────────┐    │
│ │                                            │    │
│ │         [Live Camera Feed]                 │    │
│ │                                            │    │
│ │     ┌─────────────────────┐               │    │
│ │     │                     │← Overlay       │    │
│ │     │   Center QR code    │   guide        │    │
│ │     │      here           │                │    │
│ │     └─────────────────────┘               │    │
│ │                                            │    │
│ └────────────────────────────────────────────┘    │
│                                                    │
│ 📱 Point camera at first QR code                   │
│                                                    │
│ Scanned: 0 of ? codes                             │
│ ○ ○ ○ ○ ○  ← Will fill as scanned                │
│                                                    │
│                                       [Cancel]     │
└────────────────────────────────────────────────────┘
```

**Features:**
- Live video feed from device camera
- Overlay guide helps user aim
- Automatic QR detection (no manual scan button)
- Real-time scanning loop (checks every 200ms)
- Progress tracker starts at unknown total

#### Step 4: Scanning Progress

**First Code Detected:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ┌────────────────────────────────────────────┐    │
│ │         [Live Camera Feed]                 │    │
│ │                                            │    │
│ │     ┌─────────────────────┐               │    │
│ │     │  ✓ Code detected!   │← Green flash  │    │
│ │     └─────────────────────┘               │    │
│ └────────────────────────────────────────────┘    │
│                                                    │
│ ✓ Code 1 scanned successfully!                    │
│                                                    │
│ Scanned: 1 of 5 codes                             │
│ ● ○ ○ ○ ○  ← First dot filled, now knows total   │
│                                                    │
│ 📱 Scan next code...                               │
│                                                    │
│                                       [Cancel]     │
└────────────────────────────────────────────────────┘
```

**Feedback:**
- Instant visual confirmation (green flash, checkmark)
- Success message with code number
- Progress updated (1 of 5)
- Total now known from first chunk metadata
- Instructions updated ("Scan next code...")
- Camera remains active for next scan

**Subsequent Codes:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ┌────────────────────────────────────────────┐    │
│ │         [Live Camera Feed]                 │    │
│ └────────────────────────────────────────────┘    │
│                                                    │
│ ✓ Code 3 scanned successfully!                    │
│                                                    │
│ Scanned: 3 of 5 codes                             │
│ ● ● ● ○ ○  ← Progress continues                   │
│                                                    │
│ 📱 Scan next code...                               │
│                                                    │
│                                       [Cancel]     │
└────────────────────────────────────────────────────┘
```

**Duplicate Detection:**

If user accidentally scans same code twice:
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ┌────────────────────────────────────────────┐    │
│ │         [Live Camera Feed]                 │    │
│ └────────────────────────────────────────────┘    │
│                                                    │
│ ℹ️ Code 3 already scanned (skipped)                │
│                                                    │
│ Scanned: 3 of 5 codes                             │
│ ● ● ● ○ ○  ← Still 3 of 5 (not an error)         │
│                                                    │
│ 📱 Scan next code...                               │
│                                                    │
│                                       [Cancel]     │
└────────────────────────────────────────────────────┘
```

**Error Prevention:**
- Duplicate detection prevents confusion
- Informative message (not error, just skipped)
- Progress unchanged
- User continues without interruption

#### Step 5: Completion

**All Codes Scanned:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ┌────────────────────────────────────────────┐    │
│ │                                            │    │
│ │              ✓                             │    │
│ │      All codes scanned!                    │    │
│ │                                            │    │
│ └────────────────────────────────────────────┘    │
│    └─ Camera automatically stops                   │
│                                                    │
│ ✓ Successfully scanned 5 of 5 codes               │
│ ● ● ● ● ●  ← All filled!                          │
│                                                    │
│ Booth: Spring Market 2026                          │
│ 45 vendors • 63 purchases                          │
│                                                    │
│                              [Preview Import]      │
└────────────────────────────────────────────────────┘
```

**Automatic Processing:**
1. Camera stops (battery-friendly)
2. Chunks reassembled in order
3. Hash verified (integrity check)
4. Data decompressed
5. Binary deserialized to BoothBackupData
6. Preview prepared

#### Step 6: Import Preview

**Reuses Existing Import Preview Modal:**
```
┌────────────────────────────────────────────────────┐
│ Preview Import                          [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ Booth: Spring Market 2026                          │
│ Date: March 29, 2026                               │
│                                                    │
│ Contents:                                          │
│ • 1 booth                                          │
│ • 45 vendors                                       │
│ • 63 purchases (last 7 days)                       │
│                                                    │
│ ⚠️ Conflicts detected:                             │
│ • Booth "Spring Market 2026" already exists        │
│                                                    │
│ How should we handle conflicts?                    │
│ ┌──────────────────────────────────────────────┐  │
│ │ ● Merge (recommended)                        │  │
│ │   Add new records, update existing           │  │
│ │                                              │  │
│ │ ○ Skip conflicts                             │  │
│ │   Keep existing, add only new                │  │
│ │                                              │  │
│ │ ○ Replace                                    │  │
│ │   Overwrite all existing records             │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│                    [Cancel]      [Import]          │
└────────────────────────────────────────────────────┘
```

**User chooses strategy and clicks [Import]**

#### Step 7: Import Execution

**Progress Indicator:**
```
┌────────────────────────────────────────────────────┐
│ Importing...                                       │
├────────────────────────────────────────────────────┤
│                                                    │
│               ⌛ Importing booth data...            │
│                                                    │
│    [████████████████████░░] 80%                    │
│                                                    │
│    Adding vendors...                              │
│                                                    │
└────────────────────────────────────────────────────┘
```

#### Step 8: Success

**Modal closes, returns to booth list:**
```
┌─────────────────────────────────────────────────┐
│ My Booths                    [Import] [+ New]   │
├─────────────────────────────────────────────────┤
│                                                 │
│ ┌─────────────────────────────────────────┐    │
│ │  ✓ Import successful!                   │← Toast
│ │  Added 23 vendors, 63 purchases         │    │
│ └─────────────────────────────────────────┘    │
│                                                 │
│ ┌───────────────────────────────────────────┐  │
│ │ Spring Market 2026          March 29      │  │
│ │ 45 vendors • 219 purchases ← Updated      │  │
│ └───────────────────────────────────────────┘  │
│                                                 │
│ ┌───────────────────────────────────────────┐  │
│ │ Autumn Fair 2026           Sept 20        │  │
│ │ 23 vendors • 156 purchases                │  │
│ └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

**Confirmation:**
- Toast notification with import summary
- Booth appears in list (or counts updated if existed)
- Visual feedback of successful transfer

---

### Error Handling

#### Invalid QR Code Format

**During Scanning:**
```
┌────────────────────────────────────────────────────┐
│ Import from QR Codes                    [×]        │
├────────────────────────────────────────────────────┤
│ [Live Camera Feed]                                 │
│                                                    │
│ ⚠️ Invalid QR code format                          │
│ This doesn't appear to be an ez-booth export.     │
│                                                    │
│ Scanned: 2 of 5 codes                             │
│ ● ● ○ ○ ○                                         │
│                                                    │
│ 📱 Scan the correct QR code...                     │
│                                                    │
│                                       [Cancel]     │
└────────────────────────────────────────────────────┘
```

#### Hash Mismatch (Data Corruption)

**After Scanning All Codes:**
```
┌────────────────────────────────────────────────────┐
│ Import Error                            [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ❌ Data verification failed                        │
│                                                    │
│ The scanned QR codes don't match. This could mean:│
│ • One or more codes were from a different export  │
│ • QR codes were damaged or misread                │
│                                                    │
│ Please try again:                                  │
│ • Ensure all codes are from the same export       │
│ • Scan codes in good lighting conditions          │
│ • Hold camera steady while scanning               │
│                                                    │
│                    [Close]   [Scan Again]          │
└────────────────────────────────────────────────────┘
```

#### Export Size Exceeds Limit

**During Configuration:**
```
┌────────────────────────────────────────────────────┐
│ Export as QR Codes                      [×]        │
├────────────────────────────────────────────────────┤
│                                                    │
│ ⚠️ This export requires 23 QR codes                │
│                                                    │
│ The 10-code limit ensures a good scanning         │
│ experience. For larger exports:                   │
│                                                    │
│ Options:                                           │
│ 1. Select shorter time range (recommended)        │
│    • "Last 7 days" → ~5 codes                     │
│    • "Last 24 hours" → ~2 codes                   │
│                                                    │
│ 2. Use "Export Booth (JSON)" instead              │
│    Better for complete backups and large datasets │
│                                                    │
│                [Select 7 Days]        [Use JSON]  │
└────────────────────────────────────────────────────┘
```

---

## Technical Implementation

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                  UI Components                      │
│  ┌─────────────────┐      ┌────────────────────┐   │
│  │ QrExportModal   │      │ QrImportScanner    │   │
│  │ - Config UI     │      │ - Camera access    │   │
│  │ - QR display    │      │ - Live scanning    │   │
│  │ - Pagination    │      │ - Progress track   │   │
│  └────────┬────────┘      └──────────┬─────────┘   │
│           │                          │             │
└───────────┼──────────────────────────┼─────────────┘
            │                          │
┌───────────┼──────────────────────────┼─────────────┐
│           │     Export/Import        │             │
│  ┌────────▼────────┐      ┌──────────▼─────────┐   │
│  │ qr_export.rs    │      │ qr_import.rs       │   │
│  │ - Time filter   │      │ - Chunk collector  │   │
│  │ - Binary serial │      │ - Validation       │   │
│  │ - Compression   │      │ - Decompression    │   │
│  │ - Chunking      │      │ - Deserialization  │   │
│  │ - QR generation │      │ - QR parsing       │   │
│  └─────────────────┘      └────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
            │                          │
┌───────────┼──────────────────────────┼─────────────┐
│           │     Existing Services    │             │
│  ┌────────▼────────┐      ┌──────────▼─────────┐   │
│  │ ExportService   │      │ ImportService      │   │
│  │ (reuse)         │      │ (reuse)            │   │
│  └─────────────────┘      └────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Module Structure

```
crates/
├── storage/
│   └── src/
│       └── export/
│           ├── backup_format.rs       (existing)
│           ├── export_service.rs      (existing)
│           ├── import_service.rs      (existing)
│           ├── import_validator.rs    (existing)
│           ├── qr_export.rs          (NEW)
│           ├── qr_import.rs          (NEW)
│           └── error.rs              (extend)
│
└── ez-booth-ui/
    └── src/
        └── components/
            ├── export_button.rs       (update)
            ├── import_button.rs       (update)
            ├── qr_export_modal.rs    (NEW)
            └── qr_import_scanner.rs  (NEW)
```

### Dependencies

```toml
[workspace.dependencies]
# QR code generation (SVG output)
qrcode = { version = "0.14", default-features = false }

# Compression
flate2 = "1.0"

# Binary serialization (already in workspace)
bincode = "1.3"

# SHA-256 hashing for chunk verification
sha2 = "0.10"

# Base64 encoding for chunk data
base64 = "0.22"

# QR scanning (WASM-compatible)
rqrr = "0.7"
```

### Data Structures

#### QR Chunk Format

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrChunk {
    pub v: u32,           // Format version (always 1)
    pub i: usize,         // Chunk index (0-based)
    pub t: usize,         // Total chunks
    pub h: String,        // SHA-256 hash (hex, 64 chars)
    pub d: String,        // Base64-encoded chunk data
}
```

**Example JSON (what's in QR code):**
```json
{
  "v": 1,
  "i": 0,
  "t": 5,
  "h": "a3f2b9c...",
  "d": "H4sIAAAAAAAA/+2..."
}
```

#### Export Scope Enum

```rust
#[derive(Debug, Clone, Copy)]
pub enum ExportScope {
    Today,    // Last 24 hours
    Week,     // Last 7 days (default)
    Month,    // Last 30 days
    Full,     // All purchases
}

impl ExportScope {
    pub fn days(&self) -> Option<i64> {
        match self {
            Self::Today => Some(1),
            Self::Week => Some(7),
            Self::Month => Some(30),
            Self::Full => None,
        }
    }
    
    pub fn filter_purchases(&self, purchases: &[Purchase]) -> Vec<Purchase> {
        let Some(days) = self.days() else {
            return purchases.to_vec();
        };
        
        let cutoff = Utc::now() - chrono::Duration::days(days);
        purchases
            .iter()
            .filter(|p| p.timestamp >= cutoff)
            .cloned()
            .collect()
    }
}
```

#### Chunk Collector State Machine

```rust
pub struct QrChunkCollector {
    expected_total: Option<usize>,
    expected_hash: Option<String>,
    received_chunks: HashMap<usize, QrChunk>,
}

pub enum CollectorStatus {
    ChunkAdded,      // New chunk accepted
    Duplicate,       // Already have this chunk
    Complete,        // All chunks received
}

impl QrChunkCollector {
    pub fn add_chunk(&mut self, chunk: QrChunk) -> Result<CollectorStatus, ImportError>;
    pub fn is_complete(&self) -> bool;
    pub fn progress(&self) -> (usize, usize);
    pub fn reassemble(&self) -> Result<BoothBackupData, ImportError>;
}
```

### Key Algorithms

#### Export Pipeline

```rust
pub async fn export_booth_as_qr(
    booth_id: &BoothId,
    scope: ExportScope,
    repos: &Repositories,
) -> Result<Vec<QrChunk>, ExportError> {
    // 1. Fetch booth data
    let booth = booth_repo.get(booth_id).await?;
    let vendors = vendor_repo.list_by_booth(booth_id).await?;
    let all_purchases = purchase_repo.list_by_booth(booth_id).await?;
    
    // 2. Apply time filter
    let purchases = scope.filter_purchases(&all_purchases);
    
    // 3. Create backup structure (reuse existing)
    let backup = BoothBackupData {
        version: BACKUP_FORMAT_VERSION,
        created_at: Utc::now(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        booth,
        vendors,
        purchases,
    };
    
    // 4. Serialize to binary
    let binary = bincode::serialize(&backup)?;
    
    // 5. Compress with gzip
    let compressed = compress_data(&binary)?;
    
    // 6. Verify size limit
    let chunk_count = (compressed.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    if chunk_count > MAX_QR_CODES {
        return Err(ExportError::TooManyQrCodes {
            required: chunk_count,
            maximum: MAX_QR_CODES,
        });
    }
    
    // 7. Create chunks with hash
    let chunks = create_chunks(&compressed)?;
    
    Ok(chunks)
}
```

#### Chunking Algorithm

```rust
const CHUNK_SIZE: usize = 1_800; // Raw compressed bytes per chunk before base64 + JSON overhead

fn create_chunks(data: &[u8]) -> Result<Vec<QrChunk>, ExportError> {
    // Calculate hash of complete compressed data
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = format!("{:x}", hasher.finalize());
    
    // Split into chunks
    let total_chunks = (data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut chunks = Vec::new();
    
    for i in 0..total_chunks {
        let start = i * CHUNK_SIZE;
        let end = (start + CHUNK_SIZE).min(data.len());
        let chunk_data = &data[start..end];
        
        chunks.push(QrChunk {
            v: 1,
            i,
            t: total_chunks,
            h: hash.clone(),
            d: base64::encode(chunk_data),
        });
    }
    
    Ok(chunks)
}

// Note: `chunk.d` is base64-encoded and the full `QrChunk` struct is serialized as JSON.
// The QR code therefore carries more than `CHUNK_SIZE` bytes on the wire, which is why
// the raw chunk size must remain below the practical QR payload ceiling.
```

#### Import Pipeline

```rust
pub async fn import_from_qr(
    chunks: Vec<QrChunk>,
    strategy: ConflictStrategy,
    repos: &Repositories,
) -> Result<ImportResult, ImportError> {
    // 1. Reassemble chunks
    let mut collector = QrChunkCollector::new();
    for chunk in chunks {
        collector.add_chunk(chunk)?;
    }
    
    if !collector.is_complete() {
        return Err(ImportError::IncompleteChunks);
    }
    
    // 2. Reassemble, verify, decompress, deserialize
    let backup = collector.reassemble()?;
    
    // 3. Validate (reuse existing validator)
    ImportValidator::validate_booth_backup(&backup)?;
    
    // 4. Import (reuse existing import service)
    ImportService::import_booth_backup(backup, strategy, repos).await
}
```

#### QR Scanning Loop

```rust
// In qr_import_scanner.rs component

fn start_qr_scanning(
    video: HtmlVideoElement,
    canvas_ref: NodeRef<Canvas>,
    collector: Signal<QrChunkCollector>,
) {
    let interval_handle = set_interval(
        move || {
            if collector().is_complete() {
                return;
            }
            
            // Capture video frame to canvas
            let canvas = canvas_ref.get().unwrap();
            let ctx = canvas.get_context("2d").unwrap();
            ctx.draw_image_with_html_video_element(&video, 0.0, 0.0);
            
            // Get image data
            let image_data = ctx.get_image_data(
                0.0, 0.0, 
                canvas.width() as f64, 
                canvas.height() as f64
            ).ok();
            
            // Decode QR code
            if let Some(qr_content) = decode_qr_from_image_data(&image_data) {
                // Parse and validate chunk
                if let Ok(chunk) = parse_qr_chunk(&qr_content) {
                    // Add to collector
                    match collector().add_chunk(chunk) {
                        Ok(CollectorStatus::ChunkAdded) => {
                            show_success_toast("Code scanned!");
                        }
                        Ok(CollectorStatus::Duplicate) => {
                            show_info_toast("Already scanned");
                        }
                        Ok(CollectorStatus::Complete) => {
                            clear_interval(interval_handle);
                            stop_media_stream(video.src_object());
                            on_import_complete();
                        }
                        Err(e) => {
                            show_error_toast(&e.to_string());
                        }
                    }
                }
            }
        },
        Duration::from_millis(200), // Check every 200ms
    );
}
```

Returning from a single interval tick is not enough to stop scanning. On completion, the implementation must clear the interval and shut down the camera stream explicitly.

### Size Estimation

**Client-side estimation** (before generating):

```rust
pub fn estimate_qr_count(
    booth: &Booth,
    vendor_count: usize,
    purchase_count: usize,
    scope: ExportScope,
) -> usize {
    // Filter purchase count based on scope
    let filtered_purchases = match scope {
        ExportScope::Today => (purchase_count as f32 * 0.03) as usize,   // ~3% daily
        ExportScope::Week => (purchase_count as f32 * 0.21) as usize,    // ~21% weekly
        ExportScope::Month => (purchase_count as f32 * 0.9) as usize,    // ~90% monthly
        ExportScope::Full => purchase_count,
    };
    
    // Size estimates (empirically derived from bincode)
    let booth_size = 400;                           // bytes
    let vendor_size = 80 * vendor_count;            // 80 bytes per vendor
    let purchase_size = 200 * filtered_purchases;   // 200 bytes per purchase
    
    let total_binary = booth_size + vendor_size + purchase_size;
    let compressed = (total_binary as f32 * 0.3) as usize;  // 70% compression
    
    // Calculate chunks needed
    let chunk_count = (compressed + CHUNK_SIZE - 1) / CHUNK_SIZE;
    chunk_count.max(1)
}
```

---

## Implementation Sequence

### Phase 1: Backend Foundation (PR #1)

**Branch**: `feature/qr-export-backend`

**Deliverables:**
1. Add dependencies to `Cargo.toml`
2. Implement `crates/storage/src/export/qr_export.rs`:
   - `ExportScope` enum with time filtering
   - `export_booth_as_qr()` function
   - Binary serialization with `bincode`
   - Gzip compression
   - Chunking algorithm
   - SHA-256 hashing
   - QR code SVG generation
3. Implement `crates/storage/src/export/qr_import.rs`:
   - `QrChunk` struct
   - `QrChunkCollector` state machine
   - Chunk validation
   - Reassembly with hash verification
   - Decompression and deserialization
4. Extend `error.rs` with QR-specific errors
5. Unit tests:
   - Compression/decompression roundtrip
   - Chunking with various data sizes
   - Collector state machine (add, duplicate, complete)
   - Hash verification
   - Size limit enforcement

**Acceptance Criteria:**
- ✅ Can export booth as binary chunks
- ✅ Can reassemble and import from chunks
- ✅ All unit tests pass
- ✅ No changes to existing export/import logic

**Estimated Effort**: 6-8 hours

---

### Phase 2: Export UI (PR #2)

**Branch**: `feature/qr-export-ui`

**Deliverables:**
1. Create `crates/ez-booth-ui/src/components/qr_export_modal.rs`:
   - Export scope selector (Today/Week/Month/Full)
   - Live QR count estimation
   - Size limit warning (>10 codes)
   - Generate button with loading state
   - QR code pagination display
   - Previous/Next navigation
   - Progress dots
2. Update `crates/ez-booth-ui/src/components/export_button.rs`:
   - Add "Export as QR Code" option to dropdown
   - Integrate QrExportModal
3. Add translations (EN/DE) for export flow
4. CSS styling for modal and QR display

**Acceptance Criteria:**
- ✅ Users can access QR export from booth dropdown
- ✅ Modal shows accurate estimates
- ✅ QR codes generate and display correctly
- ✅ Navigation works smoothly
- ✅ Hard limit enforced at 10 codes
- ✅ Translations complete

**Estimated Effort**: 5-7 hours

---

### Phase 3: Import UI - Scanner (PR #3)

**Branch**: `feature/qr-import-scanner`

**Deliverables:**
1. Create `crates/ez-booth-ui/src/components/qr_import_scanner.rs`:
   - Camera permission request
   - Live video feed rendering
   - Overlay guide
   - Real-time QR detection loop
   - Chunk collector integration
   - Progress tracking UI
   - Visual feedback (flash, checkmark, toasts)
   - Duplicate detection feedback
2. Implement QR decoding:
   - Use `rqrr` crate for WASM-compatible decoding
   - Convert video frame to grayscale
   - Decode and parse chunk JSON
3. Handle camera errors gracefully

**Acceptance Criteria:**
- ✅ Camera access works on Chrome, Safari, Firefox
- ✅ QR codes detected automatically
- ✅ Progress updates in real-time
- ✅ Duplicate detection works
- ✅ Completion triggers next step
- ✅ Error messages clear and actionable

**Estimated Effort**: 8-10 hours (most complex component)

---

### Phase 4: Import Integration (PR #4)

**Branch**: `feature/qr-import-integration`

**Deliverables:**
1. Update `crates/ez-booth-ui/src/components/import_button.rs`:
   - Add "Import from QR Code" option
   - Integrate QrImportScanner
2. Connect scanner completion to existing import preview modal
3. Pass reassembled BoothBackupData to existing import flow
4. Reuse existing conflict resolution UI
5. Add translations (EN/DE) for import flow
6. Handle all error cases with user-friendly messages

**Acceptance Criteria:**
- ✅ Complete import flow works end-to-end
- ✅ Preview modal shows correct data
- ✅ Conflict resolution works as expected
- ✅ Success/error feedback clear
- ✅ Translations complete

**Estimated Effort**: 4-5 hours

---

### Phase 5: Polish & Validation (PR #5)

**Branch**: `feature/qr-polish`

**Deliverables:**
1. Loading states and spinners
2. Accessibility improvements:
   - Keyboard navigation
   - ARIA labels
   - Screen reader support
3. Error message refinement
4. Performance optimization (QR generation caching)
5. Update `docs/SAFARI_VALIDATION_CHECKLIST.md`
6. Update `docs/DATA_BACKUP_GUIDE.md`
7. Browser testing (Chrome, Safari, Firefox)
8. Mobile testing (iOS Safari, Android Chrome)

**Acceptance Criteria:**
- ✅ All loading states present
- ✅ Accessible via keyboard
- ✅ Works on Chrome (desktop/mobile)
- ✅ Works on Safari (desktop/iOS)
- ✅ Works on Firefox
- ✅ Documentation updated

**Estimated Effort**: 4-6 hours

---

### Phase 6: Documentation (PR #6)

**Branch**: `feature/qr-documentation`

**Deliverables:**
1. Update `README.md` with QR code mention
2. Update `docs/DATA_BACKUP_GUIDE.md`:
   - QR code export instructions
   - QR code import instructions
   - When to use QR vs JSON
3. Update `docs/VALIDATION_WORKFLOW.md`
4. Add validation results template
5. Record browser validation evidence

**Acceptance Criteria:**
- ✅ User documentation complete (EN/DE)
- ✅ Technical documentation updated
- ✅ Validation checklist has QR section
- ✅ Validation evidence captured

**Estimated Effort**: 3-4 hours

---

### Total Estimated Effort

**Development**: 30-40 hours  
**Testing**: 5-7 hours  
**Documentation**: 3-4 hours  
**Total**: 38-51 hours (approximately 1-1.5 weeks full-time)

---

## Testing Strategy

### Unit Tests

**Location**: `crates/storage/src/export/` tests

**Coverage:**

```rust
// Compression
#[test]
fn test_compress_decompress_roundtrip() {
    let data = vec![0u8; 10000];
    let compressed = compress_data(&data).unwrap();
    let decompressed = decompress_data(&compressed).unwrap();
    assert_eq!(data, decompressed);
}

// Chunking
#[test]
fn test_create_chunks_metadata() {
    let data = vec![0u8; 5000]; // ~3 chunks
    let chunks = create_chunks(&data).unwrap();
    
    assert_eq!(chunks.len(), 3);
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.i, i);
        assert_eq!(chunk.t, 3);
        assert_eq!(chunk.v, 1);
        assert_eq!(chunk.h.len(), 64); // SHA-256 hex
    }
}

// Collector
#[test]
fn test_collector_add_and_complete() {
    let mut collector = QrChunkCollector::new();
    
    let chunk1 = create_test_chunk(0, 2, "abc123");
    let chunk2 = create_test_chunk(1, 2, "abc123");
    
    assert_eq!(collector.add_chunk(chunk1).unwrap(), CollectorStatus::ChunkAdded);
    assert!(!collector.is_complete());
    
    assert_eq!(collector.add_chunk(chunk2).unwrap(), CollectorStatus::Complete);
    assert!(collector.is_complete());
}

#[test]
fn test_collector_duplicate_detection() {
    let mut collector = QrChunkCollector::new();
    let chunk = create_test_chunk(0, 2, "abc123");
    
    collector.add_chunk(chunk.clone()).unwrap();
    let status = collector.add_chunk(chunk).unwrap();
    
    assert_eq!(status, CollectorStatus::Duplicate);
}

#[test]
fn test_collector_hash_mismatch() {
    let mut collector = QrChunkCollector::new();
    
    let chunk1 = create_test_chunk(0, 2, "abc123");
    let chunk2 = create_test_chunk(1, 2, "xyz789"); // Different hash!
    
    collector.add_chunk(chunk1).unwrap();
    assert!(collector.add_chunk(chunk2).is_err());
}

// Size limit
#[test]
fn test_export_size_limit_enforced() {
    let large_booth = create_booth_with_purchases(5000); // Too large
    
    let result = export_booth_as_qr(&large_booth.id, ExportScope::Full, &repos).await;
    
    assert!(matches!(result, Err(ExportError::TooManyQrCodes { .. })));
}
```

### Integration Tests

**Location**: `crates/storage/tests/qr_export_import_tests.rs`

```rust
#[wasm_bindgen_test]
async fn test_qr_export_import_roundtrip() {
    let db = create_test_db().await;
    let booth_repo = Arc::new(IndexedDbBoothRepository::new(db.clone()));
    let vendor_repo = Arc::new(IndexedDbVendorRepository::new(db.clone()));
    let purchase_repo = Arc::new(IndexedDbPurchaseRepository::new(db.clone()));
    
    // Create test booth with data
    let booth = create_test_booth("Test Booth");
    let vendors = create_test_vendors(&booth, 20);
    let purchases = create_test_purchases(&booth, &vendors, 100);
    
    booth_repo.save(&booth).await.unwrap();
    for vendor in &vendors {
        vendor_repo.save(vendor).await.unwrap();
    }
    for purchase in &purchases {
        purchase_repo.save(purchase).await.unwrap();
    }
    
    // Export as QR
    let chunks = export_booth_as_qr(
        &booth.id,
        ExportScope::Full,
        &booth_repo,
        &vendor_repo,
        &purchase_repo,
    ).await.unwrap();
    
    assert!(!chunks.is_empty());
    assert!(chunks.len() <= MAX_QR_CODES);
    
    // Import from QR into a fresh target database so the roundtrip verifies real inserts
    let target_db = create_test_db().await;
    let target_booth_repo = Arc::new(IndexedDbBoothRepository::new(target_db.clone()));
    let target_vendor_repo = Arc::new(IndexedDbVendorRepository::new(target_db.clone()));
    let target_purchase_repo = Arc::new(IndexedDbPurchaseRepository::new(target_db.clone()));

    let result = import_from_qr(
        chunks,
        ConflictStrategy::Skip,
        &target_booth_repo,
        &target_vendor_repo,
        &target_purchase_repo,
    ).await.unwrap();
    
    // Verify
    assert_eq!(result.booths_imported, 1);
    assert_eq!(result.vendors_imported, 20);
    assert_eq!(result.purchases_imported, 100);

    // If this import ran back into the original repositories instead, `ConflictStrategy::Skip`
    // should produce zero new inserts and only skipped/conflicting records.
}

#[wasm_bindgen_test]
async fn test_qr_export_with_time_filter() {
    // ... setup ...
    
    let chunks_full = export_booth_as_qr(&booth.id, ExportScope::Full, &repos).await.unwrap();
    let chunks_week = export_booth_as_qr(&booth.id, ExportScope::Week, &repos).await.unwrap();
    
    // Week export should be smaller
    assert!(chunks_week.len() < chunks_full.len());
}
```

### Manual Browser Validation

**Add to `docs/SAFARI_VALIDATION_CHECKLIST.md`:**

```markdown
## QR Code Export/Import Tests

### Export Tests

**Small Booth (Weekend Market):**
- [ ] Chrome Desktop: Export 20 vendors, 100 purchases (Last 7 days)
  - Expected: 2-3 QR codes
  - QR codes render clearly (sharp SVG)
  - Navigation works (Previous/Next)
  - Progress dots update correctly
- [ ] Safari Desktop: Same test
- [ ] Chrome Mobile: Same test
- [ ] Safari iOS: Same test

**Medium Booth (Weekly Market):**
- [ ] Export 50 vendors, 500 purchases (Last 7 days)
  - Expected: 5-8 QR codes
  - Pagination smooth
  - All codes display correctly

**Size Limit Tests:**
- [ ] Large booth with Full history (>10 codes required)
  - Generate button disabled
  - Warning message shown
  - "Last 7 days" suggestion works
- [ ] Edge case: Exactly 10 codes
  - Generate button enabled
  - Export succeeds

### Import Tests

**Camera Permission:**
- [ ] Chrome: Permission prompt appears
- [ ] Safari: Permission prompt appears
- [ ] Allow: Camera feed starts
- [ ] Deny: Error message shown with alternatives

**Scanning:**
- [ ] Chrome Desktop: Scan 5 QR codes
  - Auto-detection works
  - Visual feedback (green flash, checkmark)
  - Progress updates (1 of 5, 2 of 5, ...)
  - Progress dots fill
  - Completes successfully
- [ ] Safari iOS: Same test
  - Camera focus works
  - QR detection in various lighting
  - Steady for clear read
- [ ] Chrome Android: Same test

**Duplicate Detection:**
- [ ] Scan same code twice
  - "Already scanned" message shown
  - Progress unchanged
  - Can continue scanning

**Error Handling:**
- [ ] Scan unrelated QR code
  - "Invalid QR code format" message
  - Prompts to scan correct code
  - Can continue after error
- [ ] Scan codes from different exports
  - "Hash mismatch" error after completion
  - Clear explanation shown
  - Can retry

**Import Completion:**
- [ ] Preview modal shows correct data
  - Booth name, date, counts
  - Conflict detection works
  - Can choose merge strategy
- [ ] Import succeeds
  - Toast notification appears
  - Booth appears in list
  - Counts updated correctly

### Performance Tests

**Large Dataset:**
- [ ] 100 vendors, 2000 purchases, Last 7 days (~467 purchases)
  - Export: ~5 codes, completes in <5 seconds
  - Import: Scanning takes <3 minutes
  - No browser freezing

**Edge Cases:**
- [ ] Tiny booth (5 vendors, 10 purchases)
  - Export: 1 code
  - Import: Instant
- [ ] Export interruption (close modal mid-generate)
  - No errors, can retry
- [ ] Import interruption (cancel mid-scan)
  - Camera stops, no memory leak

### Cross-Device Transfer

**Real-World Scenario:**
- [ ] Laptop (Chrome) → Phone (Safari iOS)
  - Export on laptop: 5 codes
  - Display on laptop screen
  - Scan with phone camera
  - Import succeeds on phone
  - Data matches source
- [ ] Tablet (Safari) → Laptop (Chrome)
  - Reverse direction works

### Validation Evidence

Record results in:
`docs/QR_CODE_VALIDATION_RESULTS_YYYY-MM-DD.md`

Include:
- Browser/device combinations tested
- Dataset sizes used
- QR code counts generated
- Time to complete export/import
- Any issues encountered
- Screenshots of successful transfers
```

---

## Translations

### English (`locales/en.json`)

```json
{
  "export": {
    "qr_button": "Export as QR Code",
    "qr_title": "Export as QR Codes",
    "time_range": "Time Range",
    "today": "Last 24 hours",
    "week": "Last 7 days",
    "month": "Last 30 days",
    "full": "Full history",
    "estimated_codes": "Estimated QR codes",
    "data_size": "Data size",
    "purchases_filtered": "{filtered} of {total} purchases",
    "generate": "Generate QR Codes",
    "generating": "Generating...",
    "qr_progress": "Code {current} of {total}",
    "qr_scan_instructions": "Scan each code with your other device",
    "qr_limit_exceeded": "This would generate {count} QR codes, exceeding the limit of {max}.",
    "qr_limit_explanation": "QR codes work best for quick transfers of recent data.",
    "qr_suggest_shorter": "Select a shorter time range (recommended)",
    "qr_suggest_json": "Use 'Export Booth (JSON)' for full backups",
    "qr_use_json": "Use JSON Export"
  },
  "import": {
    "qr_button": "Import from QR Code",
    "qr_title": "Import from QR Codes",
    "qr_instructions": "Point your camera at each QR code",
    "qr_permission_denied": "Camera access denied",
    "qr_permission_help": "To scan QR codes, please enable camera permissions in your browser settings.",
    "qr_permission_alternative": "Alternative: Use 'Import from File' for JSON files",
    "qr_try_again": "Try Again",
    "qr_scanned": "Scanned",
    "qr_of": "of",
    "qr_codes": "codes",
    "qr_code_detected": "Code detected!",
    "qr_code_success": "Code {number} scanned successfully!",
    "qr_code_duplicate": "Code {number} already scanned (skipped)",
    "qr_code_invalid": "Invalid QR code format",
    "qr_code_invalid_help": "This doesn't appear to be an ez-booth export.",
    "qr_scan_next": "Scan next code...",
    "qr_all_scanned": "All codes scanned!",
    "qr_complete_success": "Successfully scanned {count} of {total} codes",
    "qr_preview_import": "Preview Import",
    "qr_import_error": "Import Error",
    "qr_hash_mismatch": "Data verification failed",
    "qr_hash_mismatch_help": "The scanned QR codes don't match. This could mean:",
    "qr_hash_mismatch_reason1": "One or more codes were from a different export",
    "qr_hash_mismatch_reason2": "QR codes were damaged or misread",
    "qr_hash_mismatch_try_again": "Please try again:",
    "qr_hash_mismatch_tip1": "Ensure all codes are from the same export",
    "qr_hash_mismatch_tip2": "Scan codes in good lighting conditions",
    "qr_hash_mismatch_tip3": "Hold camera steady while scanning",
    "qr_scan_again": "Scan Again"
  },
  "common": {
    "close": "Close",
    "cancel": "Cancel",
    "previous": "Previous",
    "next": "Next",
    "loading": "Loading...",
    "of": "of"
  }
}
```

### German (`locales/de.json`)

```json
{
  "export": {
    "qr_button": "Als QR-Code exportieren",
    "qr_title": "Als QR-Codes exportieren",
    "time_range": "Zeitbereich",
    "today": "Letzte 24 Stunden",
    "week": "Letzte 7 Tage",
    "month": "Letzte 30 Tage",
    "full": "Gesamter Verlauf",
    "estimated_codes": "Geschätzte QR-Codes",
    "data_size": "Datengröße",
    "purchases_filtered": "{filtered} von {total} Verkäufen",
    "generate": "QR-Codes generieren",
    "generating": "Generiere...",
    "qr_progress": "Code {current} von {total}",
    "qr_scan_instructions": "Scannen Sie jeden Code mit Ihrem anderen Gerät",
    "qr_limit_exceeded": "Dies würde {count} QR-Codes generieren und das Limit von {max} überschreiten.",
    "qr_limit_explanation": "QR-Codes eignen sich am besten für schnelle Übertragungen aktueller Daten.",
    "qr_suggest_shorter": "Wählen Sie einen kürzeren Zeitbereich (empfohlen)",
    "qr_suggest_json": "Verwenden Sie 'Stand exportieren (JSON)' für vollständige Backups",
    "qr_use_json": "JSON-Export verwenden"
  },
  "import": {
    "qr_button": "Aus QR-Code importieren",
    "qr_title": "Aus QR-Codes importieren",
    "qr_instructions": "Richten Sie Ihre Kamera auf jeden QR-Code",
    "qr_permission_denied": "Kamerazugriff verweigert",
    "qr_permission_help": "Um QR-Codes zu scannen, aktivieren Sie bitte die Kameraberechtigungen in Ihren Browser-Einstellungen.",
    "qr_permission_alternative": "Alternative: Verwenden Sie 'Aus Datei importieren' für JSON-Dateien",
    "qr_try_again": "Erneut versuchen",
    "qr_scanned": "Gescannt",
    "qr_of": "von",
    "qr_codes": "Codes",
    "qr_code_detected": "Code erkannt!",
    "qr_code_success": "Code {number} erfolgreich gescannt!",
    "qr_code_duplicate": "Code {number} bereits gescannt (übersprungen)",
    "qr_code_invalid": "Ungültiges QR-Code-Format",
    "qr_code_invalid_help": "Dies scheint kein ez-booth-Export zu sein.",
    "qr_scan_next": "Nächsten Code scannen...",
    "qr_all_scanned": "Alle Codes gescannt!",
    "qr_complete_success": "Erfolgreich {count} von {total} Codes gescannt",
    "qr_preview_import": "Import-Vorschau",
    "qr_import_error": "Import-Fehler",
    "qr_hash_mismatch": "Datenüberprüfung fehlgeschlagen",
    "qr_hash_mismatch_help": "Die gescannten QR-Codes stimmen nicht überein. Dies könnte bedeuten:",
    "qr_hash_mismatch_reason1": "Ein oder mehrere Codes stammten von einem anderen Export",
    "qr_hash_mismatch_reason2": "QR-Codes wurden beschädigt oder falsch gelesen",
    "qr_hash_mismatch_try_again": "Bitte versuchen Sie es erneut:",
    "qr_hash_mismatch_tip1": "Stellen Sie sicher, dass alle Codes vom gleichen Export stammen",
    "qr_hash_mismatch_tip2": "Scannen Sie Codes bei guten Lichtverhältnissen",
    "qr_hash_mismatch_tip3": "Halten Sie die Kamera beim Scannen ruhig",
    "qr_scan_again": "Erneut scannen"
  },
  "common": {
    "close": "Schließen",
    "cancel": "Abbrechen",
    "previous": "Zurück",
    "next": "Weiter",
    "loading": "Lädt...",
    "of": "von"
  }
}
```

---

## Success Metrics

After implementation, measure:

1. **Usage Adoption**:
   - % of users who try QR export vs JSON export
   - % of successful QR imports vs total QR exports

2. **User Experience**:
   - Average QR code count per export
   - Average time to complete import (target: <3 minutes)
   - % of imports that hit 10-code limit warning

3. **Technical Performance**:
   - QR generation time (target: <5 seconds)
   - Scan success rate per code (target: >95%)
   - Error rate (hash mismatch, invalid codes)

4. **Device Distribution**:
   - Most common export/import device combinations
   - Browser distribution (Chrome/Safari/Firefox)
   - Mobile vs desktop usage

---

## Future Enhancements (Out of Scope)

**Not Planned for Initial Release:**

1. **QR Code Encryption**
   - Encrypt booth data before QR encoding
   - Password-protected exports
   - Use case: Sensitive vendor data

2. **Auto-Advance QR Display**
   - Automatically cycle through codes every N seconds
   - Hands-free scanning on receiving device
   - Configurable timing

3. **QR Code Download**
   - Save all QR codes as PDF
   - Print-friendly format
   - Use case: Paper backups

4. **Image Upload Fallback**
   - Upload QR code images instead of live camera
   - Use case: No camera access or screenshot-based transfer

5. **Multi-Booth QR Export**
   - Export multiple booths in one QR set
   - More complex UI/UX
   - Likely exceeds 10-code limit frequently

6. **Compression Algorithm Options**
   - Let users choose compression level
   - Trade-off: size vs generation time

7. **Custom QR Styling**
   - Colored QR codes
   - Logo embedding
   - Not critical for functionality

---

## References

- **Existing Documentation**:
  - `docs/DATA_BACKUP_IMPLEMENTATION_PLAN.md` - JSON export/import foundation
  - `docs/DATA_STORAGE_ARCHITECTURE.md` - Storage layer details
  - `docs/BRANCH_STRATEGY.md` - PR workflow

- **External Standards**:
  - QR Code Specification: ISO/IEC 18004:2015
  - Binary Serialization: [bincode documentation](https://docs.rs/bincode/)
  - Compression: [flate2 documentation](https://docs.rs/flate2/)
  - QR Decoding: [rqrr documentation](https://docs.rs/rqrr/)

- **Technical Decisions**:
  - Binary over JSON: 60% size reduction
  - 10-code hard limit: UX constraint
  - 7-day default: Balance coverage and practicality
  - SHA-256 hashing: Integrity verification

---

## Approval and Next Steps

**Before Implementation:**
- [ ] Review this plan with stakeholders
- [ ] Confirm design decisions (scope, limits, defaults)
- [ ] Approve UX flow and error handling
- [ ] Validate capacity estimates with real data samples
- [ ] Confirm translation requirements

**Ready to Implement:**
- [ ] Create feature branch: `feature/qr-export-backend`
- [ ] Begin Phase 1: Backend Foundation
- [ ] Follow PR sequence as outlined

---

**Document Version**: 1.0  
**Last Updated**: 2026-03-29  
**Status**: Draft - Awaiting Approval
