# ez-booth-rs Architecture Design

**Document Version:** 1.0  
**Date:** March 19, 2026  
**Status:** Design Phase  
**Related Documents:** [SPEC.md](SPEC.md), [ANALYSIS.md](ANALYSIS.md)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Design Principles](#design-principles)
3. [System Architecture](#system-architecture)
4. [Technology Stack](#technology-stack)
5. [Module Structure](#module-structure)
6. [Data Architecture](#data-architecture)
7. [User Interface Design](#user-interface-design)
8. [Deployment Models](#deployment-models)
9. [Cross-Browser Data Portability](#cross-browser-data-portability)
10. [Security & Privacy](#security--privacy)
11. [Performance Targets](#performance-targets)

---

## Executive Summary

`ez-booth-rs` adopts a **WASM-first, offline-capable architecture** built entirely in Rust. The design eliminates the need for a traditional server process by running the complete application in the browser via WebAssembly, with optional backend services for advanced deployment scenarios.

### Key Architectural Decisions

1. **WASM-Native Application** - Full application logic runs in browser
2. **IndexedDB Storage** - Browser-based persistent storage
3. **Optional Backend** - Rust server for client-server deployments
4. **Modular Design** - Clean separation between core, UI, and storage layers
5. **Progressive Web App** - Installable, offline-capable web application

---

## Design Principles

### 1. Offline-First
- **Primary Goal:** Application must work without network connectivity
- **Strategy:** All data stored locally in browser (IndexedDB)
- **Sync:** Optional synchronization when network available

### 2. Minimal Resource Footprint
- **Target Binary Size:** <3MB WASM + <500KB JavaScript glue
- **Target Memory:** <50MB runtime memory usage
- **Target Startup:** <500ms time-to-interactive

### 3. Zero Installation Friction
- **Access Method:** Open URL in browser
- **PWA Installation:** Optional one-click install for offline access
- **No Prerequisites:** Works on any modern browser (2020+)

### 4. Cross-Platform Consistency
- **Single Codebase:** Same WASM binary for all platforms
- **Responsive UI:** Adapts to desktop, tablet, mobile
- **Browser Compatibility:** Chrome 90+, Firefox 88+, Safari 14+, Edge 90+

### 5. Type Safety & Performance
- **Compile-Time Guarantees:** Rust's type system prevents common errors
- **Zero-Cost Abstractions:** No runtime overhead for abstractions
- **Memory Safety:** No garbage collection pauses

### 6. Extensibility
- **Plugin Architecture:** Support for custom reports, sync protocols
- **Data Export:** Multiple formats (JSON, CSV, custom)
- **API Surface:** Clear boundaries for feature extensions

---

## System Architecture

### 3.1 Deployment Model 1: Browser-Only (Primary)

```
┌─────────────────────────────────────────────────────┐
│                    Browser                          │
│  ┌───────────────────────────────────────────────┐  │
│  │         ez-booth-rs WASM Application          │  │
│  │                                               │  │
│  │  ┌─────────────────────────────────────────┐ │  │
│  │  │         Presentation Layer              │ │  │
│  │  │  ┌────────────┐  ┌─────────────────┐   │ │  │
│  │  │  │  UI (Leptos)│  │  Components     │   │ │  │
│  │  │  │  - Views   │  │  - Forms        │   │ │  │
│  │  │  │  - Routing │  │  - Tables       │   │ │  │
│  │  │  │  - State   │  │  - Reports      │   │ │  │
│  │  │  └────────────┘  └─────────────────┘   │ │  │
│  │  └─────────────────────────────────────────┘ │  │
│  │                                               │  │
│  │  ┌─────────────────────────────────────────┐ │  │
│  │  │         Business Logic Layer            │ │  │
│  │  │  ┌────────────┐  ┌─────────────────┐   │ │  │
│  │  │  │  Services  │  │  Domain Model   │   │ │  │
│  │  │  │  - Booth   │  │  - Entities     │   │ │  │
│  │  │  │  - Vendor  │  │  - Value Objs   │   │ │  │
│  │  │  │  - Purchase│  │  - Calculations │   │ │  │
│  │  │  │  - Reports │  │  - Validation   │   │ │  │
│  │  │  └────────────┘  └─────────────────┘   │ │  │
│  │  └─────────────────────────────────────────┘ │  │
│  │                                               │  │
│  │  ┌─────────────────────────────────────────┐ │  │
│  │  │         Data Access Layer               │ │  │
│  │  │  ┌────────────┐  ┌─────────────────┐   │ │  │
│  │  │  │ Repository │  │  IndexedDB      │   │ │  │
│  │  │  │ Pattern    │  │  Wrapper        │   │ │  │
│  │  │  │ - CRUD     │  │  - Async API    │   │ │  │
│  │  │  │ - Queries  │  │  - Transactions │   │ │  │
│  │  │  └────────────┘  └─────────────────┘   │ │  │
│  │  └─────────────────────────────────────────┘ │  │
│  │                                               │  │
│  │  ┌─────────────────────────────────────────┐ │  │
│  │  │         Storage Layer                   │ │  │
│  │  │    ┌──────────┐    ┌────────────┐      │ │  │
│  │  │    │IndexedDB │    │LocalStorage│      │ │  │
│  │  │    │(Primary) │    │(Settings)  │      │ │  │
│  │  │    └──────────┘    └────────────┘      │ │  │
│  │  └─────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

**Characteristics:**
- ✅ No server required
- ✅ Complete offline operation
- ✅ Instant deployment (static files)
- ✅ Minimal hosting cost (static CDN)
- ⚠️ Data sync requires file export/import or peer-to-peer

### 3.2 Deployment Model 2: Client-Server (Optional)

```
┌──────────────────────┐         ┌─────────────────────┐
│   Browser Client     │         │   Rust Server       │
│  ┌────────────────┐  │         │  ┌──────────────┐   │
│  │ WASM UI        │  │◄───────►│  │ Axum/Actix   │   │
│  │ - Components   │  │  HTTP/  │  │ Web Server   │   │
│  │ - Local Cache  │  │  WS     │  └──────────────┘   │
│  └────────────────┘  │         │  ┌──────────────┐   │
│  ┌────────────────┐  │         │  │ Services     │   │
│  │ IndexedDB      │  │         │  │ - Sync       │   │
│  │ (Offline Cache)│  │         │  │ - Reports    │   │
│  └────────────────┘  │         │  │ - Export     │   │
└──────────────────────┘         │  └──────────────┘   │
                                 │  ┌──────────────┐   │
                                 │  │ Storage      │   │
                                 │  │ - SQLite     │   │
                                 │  │ - PostgreSQL │   │
                                 │  └──────────────┘   │
                                 └─────────────────────┘
```

**Characteristics:**
- ✅ Centralized data management
- ✅ Multi-user collaboration
- ✅ Automatic synchronization
- ✅ Backend report generation (PDF)
- ⚠️ Requires server infrastructure
- ⚠️ Needs network connectivity

### 3.3 Deployment Model 3: Hybrid (Best of Both)

```
┌──────────────────────┐         ┌─────────────────────┐
│   Browser Client     │         │  Optional Server    │
│  ┌────────────────┐  │         │  (When Available)   │
│  │ Full WASM App  │  │   ╔════►│  ┌──────────────┐   │
│  │ (Standalone)   │  │   ║     │  │ Sync Service │   │
│  └────────────────┘  │   ║     │  └──────────────┘   │
│  ┌────────────────┐  │   ║     │  ┌──────────────┐   │
│  │ IndexedDB      │  │───╝     │  │ Backup Store │   │
│  │ (Primary)      │  │ Sync    │  └──────────────┘   │
│  └────────────────┘  │ Agent   └─────────────────────┘
└──────────────────────┘
     │
     └──► Works offline, syncs when online
```

**Characteristics:**
- ✅ Works completely offline
- ✅ Optional background sync when server available
- ✅ Degrades gracefully without server
- ✅ Best user experience
- 🎯 **Recommended for ez-booth-rs**

---

## Technology Stack

### 4.1 Frontend (WASM)

#### UI Framework: **Leptos 0.6+**
**Rationale:**
- Fine-grained reactivity with signals (minimal re-renders)
- Smallest WASM bundle size (~150KB after optimization)
- Server-side rendering support for future enhancements
- Excellent TypeScript-like type inference
- Active community and development

**Alternatives Considered:**
- **Yew:** More mature but larger bundle size (~200KB), virtual DOM overhead
- **Dioxus:** React-like API, good for React developers, slightly larger bundle
- **Sycamore:** Similar to Leptos but less adoption

#### CSS Framework: **Tailwind CSS**
**Rationale:**
- Utility-first approach for rapid development
- Excellent purging (only used classes in output)
- Small production bundle (~10-30KB)
- Great responsive design primitives

#### Component Library: **Custom + Leptos UI Primitives**
**Rationale:**
- Full control over bundle size
- Tailored to ez-booth use cases
- No unnecessary dependencies

### 4.2 Data Layer (WASM)

#### Storage: **IndexedDB via rexie 0.6+**
**Rationale:**
- Native browser storage (persistent across sessions)
- Structured data with indexes (fast queries)
- Async API (non-blocking)
- Transactions for consistency
- ~100MB+ storage quota (browser-dependent)

**Schema Design:**
```rust
// Database: ez_booth_db
// Stores:
//   - booths (key: booth_id)
//   - vendors (key: [booth_id, vendor_id])
//   - purchases (key: [booth_id, purchase_id])
//   - purchase_items (key: [booth_id, purchase_id, item_id])
```

#### Serialization: **Serde + serde_json/bincode**
**Rationale:**
- Zero-copy deserialization where possible
- JSON for export/import (human-readable)
- Bincode for efficient storage (smaller size)

### 4.3 Backend (Optional Server)

#### Web Framework: **Axum 0.7+**
**Rationale:**
- Tokio-based async runtime (excellent performance)
- Type-safe routing and extractors
- Modular middleware system
- Small binary size (~5-10MB)
- Active development by Tokio team

**Alternatives:**
- **Actix-Web:** Faster benchmarks but more complex API
- **Rocket:** Good DX but slower compile times

#### Database: **SQLite (rusqlite) or PostgreSQL (sqlx)**
**Rationale:**
- **SQLite:** Zero-config, file-based, perfect for single-server deployments
- **PostgreSQL:** Production-grade, multi-user, better concurrency

#### ORM/Query Builder: **sqlx 0.7+**
**Rationale:**
- Compile-time query validation
- Async/await support
- No runtime overhead
- Direct SQL control

### 4.4 Shared Libraries

#### Core Dependencies (All Phases)

```toml
[dependencies]
leptos = "0.6"                  # UI framework
leptos_i18n = "0.3"            # Internationalization (DE/EN)
rexie = "0.6"                   # IndexedDB wrapper
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"             # JSON serialization
wasm-bindgen = "0.2"           # WASM-JS bridge
web-sys = "0.3"                # Browser APIs
js-sys = "0.1"                 # JavaScript types
rust_decimal = "1.34"          # Currency precision
chrono = "0.4"                 # Date/time handling
uuid = "1.7"                   # Unique identifiers
```

**Rationale:**
- Essential dependencies for production quality
- rust_decimal prevents floating-point errors in financial calculations
- chrono provides consistent date handling and timezone support
- i18n support built-in from start for German/English localization

#### Optional Dependencies (Server Features)

- **reqwest** → Only if cloud sync implemented
- **tokio** → Server runtime
- **axum** → Server framework
- **sqlx** → Database driver

### 4.5 Development Tools

#### Build Tool: **Trunk 0.19+**
**Rationale:**
- WASM-specific build tool
- Hot reload during development
- Asset bundling (CSS, images)
- Optimized production builds

#### Testing: **wasm-bindgen-test**
**Rationale:**
- Browser-based test runner
- Tests run in actual browser environment

#### Formatting: **rustfmt**
**Rationale:**
- Official Rust formatter
- Consistent code style

#### Linting: **clippy**
**Rationale:**
- Official Rust linter
- Catches common mistakes and anti-patterns

---

## Module Structure

### 5.1 Workspace Layout

```
ez-booth-rs/
├── Cargo.toml              # Workspace root
├── locales/                # i18n translations (NEW)
│   ├── de.json            # German (primary)
│   ├── en.json            # English (fallback)
│   └── translations.json  # Config
├── crates/
│   ├── core/               # Domain model & business logic (shared)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/   # Domain entities
│   │       ├── services/   # Business logic
│   │       ├── validation/ # Validation rules
│   │       └── error.rs    # Error types
│   │
│   ├── storage/            # Storage abstraction layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository/ # Repository traits
│   │       ├── indexeddb/  # IndexedDB implementation
│   │       └── sql/        # SQL implementation (optional)
│   │
│   ├── frontend/           # WASM UI application
│   │   ├── Cargo.toml
│   │   ├── index.html
│   │   ├── Trunk.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── components/ # UI components
│   │       ├── pages/      # Page views
│   │       ├── state/      # Global state management
│   │       ├── i18n/       # i18n setup & formatters (NEW)
│   │       └── api/        # Backend API client
│   │
│   ├── server/             # Optional backend (feature-gated)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers/   # HTTP handlers
│   │       ├── middleware/ # Auth, logging, etc.
│   │       └── sync/       # Sync protocol
│   │
│   └── shared/             # Shared types for client-server
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── dto/        # Data transfer objects
│           └── protocol/   # Sync protocol definitions
│
├── docs/                   # Documentation
├── tests/                  # Integration tests
└── scripts/                # Build/deploy scripts
```

### 5.2 Dependency Graph

```
┌─────────┐
│  core   │◄─────────┐
└─────────┘          │
     ▲               │
     │               │
┌────┴────┐     ┌────┴────┐
│ storage │     │ shared  │
└─────────┘     └─────────┘
     ▲               ▲
     │               │
┌────┴────┐     ┌────┴────┐
│frontend │     │ server  │
└─────────┘     └─────────┘
```

**Dependency Rules:**
- `core` has no dependencies (pure business logic)
- `storage` depends on `core` (implements storage for entities)
- `shared` depends on `core` (serializable DTOs)
- `frontend` depends on `core`, `storage`, `shared`
- `server` depends on `core`, `storage`, `shared`

---

## Data Architecture

### 6.1 Domain Model

#### Entities

```rust
// crates/core/src/entities/booth.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Booth {
    pub id: BoothId,
    pub description: String,
    pub date: NaiveDate,
    pub fees: FeeConfig,
    pub status: BoothStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeConfig {
    pub participation_fee: Decimal,  // Use rust_decimal for currency
    pub sales_fee_percent: Decimal,
    pub rounding_step: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BoothStatus {
    Open,
    Closed { closed_at: DateTime<Utc> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vendor {
    pub id: VendorId,
    pub booth_id: BoothId,
    // Extensible design - can add fields later
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Purchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    pub items: Vec<PurchaseItem>,
    pub total: Decimal,
    pub purchased_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub id: ItemId,
    pub vendor_id: VendorId,
    pub price: Decimal,
    pub purchased_at: DateTime<Utc>,
}

// Value objects for type safety
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoothId(Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorId(Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseId(Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(Uuid);
```

#### Services

```rust
// crates/core/src/services/booth_service.rs

pub trait BoothService {
    async fn create_booth(&self, description: String, date: NaiveDate, fees: FeeConfig) 
        -> Result<Booth, BoothError>;
    
    async fn get_booth(&self, id: BoothId) -> Result<Booth, BoothError>;
    
    async fn list_booths(&self) -> Result<Vec<Booth>, BoothError>;
    
    async fn update_booth(&self, id: BoothId, booth: Booth) -> Result<(), BoothError>;
    
    async fn close_booth(&self, id: BoothId) -> Result<Booth, BoothError>;
    
    async fn delete_booth(&self, id: BoothId) -> Result<(), BoothError>;
}

// crates/core/src/services/charging_service.rs

pub struct ChargingService;

impl ChargingService {
    pub fn calculate_fees(
        &self, 
        total_sales: Decimal, 
        config: &FeeConfig
    ) -> FeeCalculation {
        let participation = config.participation_fee;
        let sales_fee = total_sales * config.sales_fee_percent / Decimal::from(100);
        
        // Apply rounding
        let sales_fee_rounded = Self::round_to_step(sales_fee, config.rounding_step);
        
        FeeCalculation {
            participation_fee: participation,
            sales_fee: sales_fee_rounded,
            total_fees: participation + sales_fee_rounded,
            net_revenue: total_sales - participation - sales_fee_rounded,
        }
    }
    
    fn round_to_step(value: Decimal, step: Decimal) -> Decimal {
        if step.is_zero() {
            value
        } else {
            (value / step).round() * step
        }
    }
}
```

### 6.2 Storage Schema

#### IndexedDB Schema (Browser)

```
Database: ez_booth_v1

Object Stores:

1. booths
   - keyPath: "id"
   - indexes: ["date", "status"]

2. vendors
   - keyPath: ["booth_id", "id"]
   - indexes: ["booth_id"]

3. purchases
   - keyPath: ["booth_id", "id"]
   - indexes: ["booth_id", "purchased_at"]

4. purchase_items
   - keyPath: ["booth_id", "purchase_id", "id"]
   - indexes: ["booth_id", "vendor_id", "purchased_at"]

5. settings
   - keyPath: "key"
   - stores: UI preferences, sync config
```

#### SQLite Schema (Server - Optional)

```sql
-- Enable foreign keys
PRAGMA foreign_keys = ON;

CREATE TABLE booths (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    date TEXT NOT NULL,  -- ISO 8601 date
    participation_fee TEXT NOT NULL,  -- Decimal as string
    sales_fee_percent TEXT NOT NULL,
    rounding_step TEXT NOT NULL,
    status TEXT NOT NULL,  -- 'open' or 'closed'
    closed_at TEXT,  -- ISO 8601 timestamp
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE vendors (
    id TEXT NOT NULL,
    booth_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (booth_id, id),
    FOREIGN KEY (booth_id) REFERENCES booths(id) ON DELETE CASCADE
);

CREATE TABLE purchases (
    id TEXT NOT NULL,
    booth_id TEXT NOT NULL,
    total TEXT NOT NULL,  -- Decimal as string
    purchased_at TEXT NOT NULL,
    PRIMARY KEY (booth_id, id),
    FOREIGN KEY (booth_id) REFERENCES booths(id) ON DELETE CASCADE
);

CREATE TABLE purchase_items (
    id TEXT NOT NULL,
    booth_id TEXT NOT NULL,
    purchase_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    price TEXT NOT NULL,  -- Decimal as string
    purchased_at TEXT NOT NULL,
    PRIMARY KEY (booth_id, purchase_id, id),
    FOREIGN KEY (booth_id, purchase_id) REFERENCES purchases(booth_id, id) ON DELETE CASCADE,
    FOREIGN KEY (booth_id, vendor_id) REFERENCES vendors(booth_id, id)
);

-- Indexes for common queries
CREATE INDEX idx_vendors_booth ON vendors(booth_id);
CREATE INDEX idx_purchases_booth ON purchases(booth_id);
CREATE INDEX idx_purchases_date ON purchases(purchased_at);
CREATE INDEX idx_items_vendor ON purchase_items(booth_id, vendor_id);
CREATE INDEX idx_items_date ON purchase_items(purchased_at);
```

### 6.3 Data Flow

#### Write Operation (Browser-Only)
```
User Action → Component Event Handler → Service Call → Repository
→ IndexedDB Transaction → Storage Write → State Update → UI Re-render
```

#### Read Operation (Browser-Only)
```
Page Load → Service Call → Repository → IndexedDB Query
→ Deserialize → Return Entities → Update State → Render UI
```

#### Sync Operation (Hybrid Mode)
```
Browser                           Server
   │                                │
   ├─► Export data (JSON)           │
   │   with last_sync timestamp     │
   │                                │
   ├─────────── POST /sync ─────────►
   │                                │
   │                          Merge conflicts
   │                          Generate response
   │                                │
   │◄────── Response (changes) ─────┤
   │                                │
   ├─► Import changes               │
   │   Update local data            │
   │   Update last_sync             │
   │                                │
```

---

## User Interface Design

### 7.1 UI Framework: Leptos Component Architecture

#### Component Hierarchy

```
App
├── Router
│   ├── Layout
│   │   ├── Navigation
│   │   │   ├── Logo
│   │   │   ├── MenuItems
│   │   │   └── UserMenu
│   │   │
│   │   └── Content (Routes)
│   │       ├── HomePage
│   │       ├── BoothListPage
│   │       ├── BoothDetailPage
│   │       │   ├── BoothInfo
│   │       │   ├── VendorList
│   │       │   └── PurchaseHistory
│   │       │
│   │       ├── CheckoutPage
│   │       │   ├── VendorSelector
│   │       │   ├── ItemEntry
│   │       │   ├── Cart
│   │       │   └── CheckoutButton
│   │       │
│   │       ├── ReportsPage
│   │       │   ├── VendorReportForm
│   │       │   └── ReportPreview
│   │       │
│   │       └── SyncPage
│   │           ├── ExportButton
│   │           ├── ImportButton
│   │           └── ServerSyncPanel
│   │
│   └── Toasts (Global notifications)
```

### 7.2 Page Designs

#### Home Page
- Dashboard with quick stats
- Recent activity
- Quick actions (New Booth, Checkout, Reports)

#### Booth List Page
- Filterable/sortable table of booths
- Status badges (Open/Closed)
- Quick actions per booth

#### Booth Detail Page
- Booth information (editable)
- Vendor management (add/remove)
- Purchase history table
- Financial summary

#### Checkout Page
- Vendor selection (grid or dropdown)
- Quick price entry (numeric keypad)
- Shopping cart with items
- Print receipt option

#### Reports Page
- Vendor selection (multi-select)
- Report preview
- Print/export options

#### Sync Page
- Export data to file
- Import data from file
- Server sync status (if configured)
- Conflict resolution UI

### 7.3 Responsive Design

#### Breakpoints (Tailwind)
- **Mobile:** <640px (1 column)
- **Tablet:** 640px-1024px (2 columns)
- **Desktop:** >1024px (3+ columns)

#### Mobile-First Approach
- Touch-friendly controls (min 44x44px targets)
- Bottom navigation on mobile
- Sidebar navigation on desktop
- Collapsible sections for space efficiency

### 7.4 Accessibility

- **WCAG 2.1 AA Compliance**
- Semantic HTML5 elements
- ARIA labels for interactive elements
- Keyboard navigation support
- Focus indicators
- Screen reader friendly

---

## Deployment Models

### 8.1 Static Hosting (Browser-Only)

**Deployment Steps:**
1. Build WASM with `trunk build --release`
2. Upload `dist/` folder to static host
3. Configure MIME types (`.wasm` → `application/wasm`)

**Hosting Options:**
- **GitHub Pages** - Free, CDN, custom domains
- **Netlify** - Free tier, automatic deployments
- **Vercel** - Free tier, edge network
- **Cloudflare Pages** - Free, global CDN
- **AWS S3 + CloudFront** - Scalable, custom domain

**Requirements:**
- HTTPS required for WASM and service workers
- Proper CORS headers for assets
- Gzip/Brotli compression enabled

### 8.2 Self-Hosted Server (Client-Server)

**Deployment Steps:**
1. Build WASM frontend: `trunk build --release`
2. Build Rust server: `cargo build --release --bin server`
3. Copy frontend assets to server `static/` directory
4. Configure reverse proxy (nginx/caddy)
5. Run server binary

**System Requirements:**
- Linux/Windows/macOS server
- 512MB RAM minimum
- 100MB disk space
- HTTPS certificate (Let's Encrypt)

**Docker Option:**
```dockerfile
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
COPY --from=builder /app/frontend/dist /usr/local/share/ez-booth/static
CMD ["server"]
```

### 8.3 Progressive Web App (PWA)

**Manifest (`manifest.json`):**
```json
{
  "name": "ez-booth",
  "short_name": "ez-booth",
  "description": "Portable booth management system",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#4F46E5",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}
```

**Service Worker:**
- Cache WASM and assets for offline use
- Background sync when connection restored
- Push notifications for updates (optional)

---

## 9. Cross-Browser Data Portability

### 9.1 Challenge: Browser-Specific Storage

**Problem:** IndexedDB data is isolated per browser and per profile, creating data silos:

- ❌ Chrome data ≠ Firefox data
- ❌ Desktop Chrome ≠ Mobile Chrome  
- ❌ Chrome Profile A ≠ Chrome Profile B
- ❌ No automatic data transfer between browsers

**Impact:** Users cannot easily:
- Switch browsers without losing data
- Use multiple devices with the same data
- Recover data after browser reinstall
- Work across different browser profiles

### 9.2 Solution: Multi-Layer Portability Strategy

```
┌─────────────────────────────────────────────────────────┐
│              User's Device Ecosystem                     │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Chrome     │  │   Firefox    │  │    Safari    │ │
│  │  IndexedDB   │  │  IndexedDB   │  │  IndexedDB   │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                  │                  │          │
│         └──────────────────┼──────────────────┘          │
│                            ▼                             │
│              ┌─────────────────────────┐                │
│              │  Portability Solutions  │                │
│              │                         │                │
│              │  1. Export/Import (P0) │                │
│              │  2. File System (P1)   │                │
│              │  3. Cloud Sync (P2)    │                │
│              └─────────────────────────┘                │
└─────────────────────────────────────────────────────────┘
```

### 9.3 Layer 1: Manual Export/Import (P0 - Core Feature)

**Priority:** Must-have for MVP  
**Complexity:** Low  
**Implementation:** Phase 6 (Sync)

#### Export Functionality

```rust
// JSON export with integrity verification
pub struct ExportData {
    pub version: String,              // Schema version
    pub exported_at: DateTime<Utc>,   // Export timestamp
    pub client_id: String,            // Browser identifier
    pub booths: Vec<Booth>,
    pub vendors: Vec<Vendor>,
    pub purchases: Vec<Purchase>,
    pub checksum: String,             // SHA-256 integrity hash
}

impl ExportService {
    pub async fn export_as_json(&self) -> Result<String, Error> {
        let data = ExportData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: Utc::now(),
            client_id: self.get_client_id(),
            booths: self.fetch_all_booths().await?,
            vendors: self.fetch_all_vendors().await?,
            purchases: self.fetch_all_purchases().await?,
            checksum: String::new(), // Calculated after serialization
        };
        
        let json = serde_json::to_string_pretty(&data)?;
        Ok(json)
    }
    
    pub async fn download_export(&self) -> Result<(), JsValue> {
        let json = self.export_as_json().await?;
        let filename = format!(
            "ez-booth-export-{}.json",
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        
        // Trigger browser download
        self.trigger_download(&json, &filename).await
    }
}
```

#### Import Functionality

```rust
pub enum MergeStrategy {
    Replace,  // Clear existing, import all
    Merge,    // Merge by timestamp (newer wins)
    Preview,  // Show changes without applying
}

impl ImportService {
    pub async fn import_from_json(
        &self,
        json: &str,
        strategy: MergeStrategy,
    ) -> Result<ImportResult, Error> {
        // 1. Parse and validate
        let data: ExportData = serde_json::from_str(json)?;
        self.verify_checksum(&data)?;
        self.validate_schema_version(&data.version)?;
        
        // 2. Apply strategy
        match strategy {
            MergeStrategy::Replace => self.import_replace(data).await?,
            MergeStrategy::Merge => self.import_merge(data).await?,
            MergeStrategy::Preview => return self.preview_changes(data).await,
        }
        
        Ok(ImportResult {
            booths_imported: data.booths.len(),
            vendors_imported: data.vendors.len(),
            purchases_imported: data.purchases.len(),
        })
    }
    
    async fn import_merge(&self, data: ExportData) -> Result<(), Error> {
        // Merge logic: newer timestamp wins
        for booth in data.booths {
            match self.db.get_booth(&booth.id).await? {
                Some(existing) if existing.updated_at > booth.updated_at => {
                    // Keep existing (newer)
                    continue;
                }
                _ => {
                    // Import (newer or new)
                    self.db.save_booth(&booth).await?;
                }
            }
        }
        // Repeat for vendors and purchases...
        Ok(())
    }
}
```

#### UI Integration

**Sync Page Component:**
```rust
#[component]
pub fn SyncPage() -> impl IntoView {
    view! {
        <div class="sync-page">
            <section class="export-section">
                <h2>"Export Data"</h2>
                <p>"Download all data as JSON file for:"</p>
                <ul>
                    <li>"✓ Switching to another browser"</li>
                    <li>"✓ Backing up your data"</li>
                    <li>"✓ Transferring to another device"</li>
                </ul>
                <button on:click=handle_export>
                    "📥 Download Export"
                </button>
            </section>
            
            <section class="import-section">
                <h2>"Import Data"</h2>
                <input 
                    type="file" 
                    accept=".json"
                    on:change=handle_import
                />
                <select name="strategy">
                    <option value="merge">"Merge with existing"</option>
                    <option value="replace">"Replace all data"</option>
                    <option value="preview">"Preview changes"</option>
                </select>
            </section>
        </div>
    }
}
```

**User Flow:**
1. User opens Chrome → "Export Data" → Downloads `ez-booth-export-20260319.json`
2. User opens Firefox → "Import Data" → Selects file → "Merge" → Done
3. Data now available in both browsers

### 9.4 Layer 2: File System Access API (P1 - Enhanced)

**Priority:** Nice-to-have for better UX  
**Complexity:** Medium  
**Implementation:** Phase 7 (Polish)

#### Automatic Cloud Folder Sync

**Feature:** Save export directly to synced folder (Dropbox, Google Drive, iCloud)

```rust
#[cfg(feature = "file-system-access")]
impl ExportService {
    pub async fn export_to_file_system(&self) -> Result<(), JsValue> {
        // Use native File System Access API
        let file_handle = self.show_save_file_picker(
            "ez-booth-data.json",
            "application/json"
        ).await?;
        
        let writable = file_handle.create_writable().await?;
        let json = self.export_as_json().await?;
        
        writable.write(&JsValue::from_str(&json)).await?;
        writable.close().await?;
        
        Ok(())
    }
}
```

**Benefits:**
- User selects save location (e.g., `~/Dropbox/ez-booth-data.json`)
- OS/cloud provider handles sync automatically
- Other devices can import from same synced file
- No manual download/upload needed

**Browser Support:**
- Chrome 86+ ✅
- Edge 86+ ✅
- Firefox: In development ⚠️
- Safari: Not supported ❌

**Fallback:** Use standard download for unsupported browsers

### 9.5 Layer 3: Optional Cloud Sync (P2 - Advanced)

**Priority:** Future enhancement  
**Complexity:** High  
**Implementation:** Phase 6+ (Optional feature)

#### Cloud Sync Architecture

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│  Browser A   │         │  Cloud Sync  │         │  Browser B   │
│  (Chrome)    │◄───────►│   Service    │◄───────►│  (Firefox)   │
│  IndexedDB   │  HTTPS  │              │  HTTPS  │  IndexedDB   │
└──────────────┘         └──────────────┘         └──────────────┘
                                │
                                ▼
                         ┌──────────────┐
                         │   Storage    │
                         │ (PostgreSQL  │
                         │  or Supabase)│
                         └──────────────┘
```

#### Cloud Sync Service

```rust
pub struct CloudSyncService {
    backend_url: String,
    user_token: Option<String>,
    auto_sync_enabled: bool,
}

impl CloudSyncService {
    pub async fn sync_to_cloud(&self) -> Result<(), Error> {
        let export = ExportService::new().export_all_data().await?;
        
        // Upload to backend
        let response = reqwest::Client::new()
            .post(&format!("{}/api/sync", self.backend_url))
            .bearer_auth(self.user_token.as_ref().unwrap())
            .json(&export)
            .send()
            .await?;
        
        if response.status().is_success() {
            self.update_last_sync_timestamp().await?;
            Ok(())
        } else {
            Err(Error::SyncFailed)
        }
    }
    
    pub fn enable_auto_sync(&mut self, interval_minutes: u32) {
        self.auto_sync_enabled = true;
        
        // Background sync every N minutes
        spawn_local(async move {
            loop {
                sleep(Duration::from_secs(interval_minutes as u64 * 60)).await;
                let _ = self.sync_to_cloud().await;
            }
        });
    }
}
```

### 9.6 Data Format Specification

#### Export JSON Schema

```json
{
  "version": "0.1.0",
  "exported_at": "2026-03-19T14:31:00Z",
  "client_id": "chrome-desktop-abc123",
  "booths": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "description": "Spring Fair 2026",
      "date": "2026-03-25",
      "fees": {
        "participation_fee": "10.00",
        "sales_fee_percent": "5.0",
        "rounding_step": "0.50"
      },
      "status": { "type": "Open" },
      "created_at": "2026-03-19T10:00:00Z",
      "updated_at": "2026-03-19T14:00:00Z"
    }
  ],
  "vendors": [...],
  "purchases": [...],
  "checksum": "a3f5b8c9d2e1f4a7b6c5d8e9f2a1b4c7"
}
```

**Schema Features:**
- **Versioned:** `version` field for backward compatibility
- **Timestamped:** `exported_at` for audit trail  
- **Checksummed:** Integrity verification
- **Human-readable:** JSON for easy debugging

### 9.7 Implementation Priorities

| Priority | Feature | Phase | Effort |
|----------|---------|-------|--------|
| **P0** | Manual Export/Import | Phase 6 | 3 days |
| **P0** | JSON format with checksum | Phase 6 | 1 day |
| **P0** | Merge strategies | Phase 6 | 2 days |
| **P1** | File System Access API | Phase 7 | 2 days |
| **P2** | Cloud sync service | Phase 8+ | 2 weeks |

### 9.8 Success Criteria

**Phase 6 (MVP):**
- ✅ User can export all data as JSON
- ✅ User can import JSON in another browser
- ✅ Merge strategy preserves newer data
- ✅ Checksum verification prevents corruption
- ✅ <5 seconds for typical export/import

**Phase 7 (Enhanced):**
- ✅ File System Access API works in Chrome/Edge
- ✅ User can save to synced folder
- ✅ Graceful fallback for unsupported browsers
- ✅ Welcome screen detects empty state and prompts for import

### 9.9 User Onboarding & Browser Switch Detection

#### Problem: Silent Data Loss

When users open ez-booth in a new browser, they see an empty application with no indication that:
- They might have data in another browser
- They need to export/import to transfer data
- The application is working correctly (empty state vs. error)

#### Solution: Smart Welcome Screen

**Detection Logic:**
```rust
pub struct OnboardingState {
    pub is_first_visit: bool,
    pub has_data: bool,
    pub browser_info: BrowserInfo,
}

impl OnboardingState {
    pub async fn detect() -> Self {
        let has_data = Self::check_database_populated().await;
        let is_first_visit = Self::check_first_visit_flag().await;
        
        Self {
            is_first_visit,
            has_data,
            browser_info: Self::get_browser_info(),
        }
    }
    
    async fn check_database_populated() -> bool {
        // Check if any booths exist
        let booth_count = Database::new().count_booths().await.unwrap_or(0);
        booth_count > 0
    }
    
    async fn check_first_visit_flag() -> bool {
        // Check localStorage for first visit flag
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        
        storage.get_item("ez_booth_visited").unwrap().is_none()
    }
    
    fn get_browser_info() -> BrowserInfo {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        
        BrowserInfo {
            user_agent: navigator.user_agent().unwrap_or_default(),
            browser_name: Self::detect_browser_name(&navigator),
        }
    }
    
    fn detect_browser_name(navigator: &web_sys::Navigator) -> String {
        let ua = navigator.user_agent().unwrap_or_default();
        
        if ua.contains("Firefox") {
            "Firefox".to_string()
        } else if ua.contains("Edg") {
            "Edge".to_string()
        } else if ua.contains("Chrome") {
            "Chrome".to_string()
        } else if ua.contains("Safari") {
            "Safari".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}
```

**Welcome Screen Component:**
```rust
#[component]
pub fn WelcomeScreen() -> impl IntoView {
    let onboarding = create_resource(|| (), |_| OnboardingState::detect());
    
    view! {
        <Suspense fallback=|| view! { <div>"Loading..."</div> }>
            {move || onboarding.get().map(|state| {
                if state.is_first_visit && !state.has_data {
                    view! { <FirstTimeWelcome browser=state.browser_info.clone() /> }
                } else if state.has_data {
                    view! { <Navigate to="/booths" /> }
                } else {
                    view! { <EmptyState /> }
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn FirstTimeWelcome(browser: BrowserInfo) -> impl IntoView {
    view! {
        <div class="welcome-screen max-w-2xl mx-auto p-8 text-center">
            <h1 class="text-4xl font-bold mb-4">"Welcome to ez-booth!"</h1>
            <p class="text-xl mb-8">
                "You're using " {browser.browser_name.clone()} " for the first time"
            </p>
            
            <div class="mb-8 p-6 bg-blue-50 border-2 border-blue-200 rounded-lg">
                <h2 class="text-2xl font-semibold mb-4">"🔍 Have data in another browser?"</h2>
                <p class="mb-4">
                    "If you've used ez-booth in Chrome, Firefox, or another browser before, 
                     you'll need to transfer your data:"
                </p>
                <ol class="text-left list-decimal list-inside space-y-2 mb-4">
                    <li>"Open ez-booth in your other browser"</li>
                    <li>"Go to Settings → Export Data"</li>
                    <li>"Download the JSON file"</li>
                    <li>"Come back here and import it"</li>
                </ol>
                <button 
                    class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                    on:click=|_| {
                        // Navigate to import page
                        use leptos_router::*;
                        let navigate = use_navigate();
                        navigate("/sync", Default::default());
                    }
                >
                    "📥 Import Existing Data"
                </button>
            </div>
            
            <div class="p-6 bg-green-50 border-2 border-green-200 rounded-lg">
                <h2 class="text-2xl font-semibold mb-4">"🆕 First time using ez-booth?"</h2>
                <p class="mb-4">"Start fresh and create your first booth"</p>
                <button 
                    class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700"
                    on:click=move |_| {
                        // Mark as visited and create first booth
                        let window = web_sys::window().unwrap();
                        let storage = window.local_storage().unwrap().unwrap();
                        let _ = storage.set_item("ez_booth_visited", "true");
                        
                        let navigate = use_navigate();
                        navigate("/booths/new", Default::default());
                    }
                >
                    "🚀 Create First Booth"
                </button>
            </div>
            
            <div class="mt-8 text-sm text-gray-600">
                <a href="/help/getting-started" class="underline">"Learn more about getting started"</a>
            </div>
        </div>
    }
}
```

#### Additional Signals & Prompts

**1. Browser Tab Title Signal**
```rust
// Change tab title when empty
if !has_data {
    document.set_title("ez-booth (Empty - Import Data?)");
} else {
    document.set_title("ez-booth");
}
```

**2. Navigation Bar Hint**
```rust
#[component]
pub fn NavBar() -> impl IntoView {
    let booth_count = create_resource(|| (), |_| async {
        Database::new().count_booths().await.unwrap_or(0)
    });
    
    view! {
        <nav class="navbar">
            {move || booth_count.get().map(|count| {
                if count == 0 {
                    view! {
                        <div class="import-hint bg-yellow-100 border-yellow-400 p-2 text-sm">
                            "💡 No data yet. "
                            <a href="/sync" class="underline">"Import from another browser?"</a>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            })}
            // ... rest of nav
        </nav>
    }
}
```

**3. Empty State with Import CTA**
```rust
#[component]
pub fn EmptyBoothList() -> impl IntoView {
    view! {
        <div class="empty-state text-center p-12">
            <svg class="w-24 h-24 mx-auto mb-4 text-gray-300">"..."</svg>
            <h2 class="text-2xl font-semibold mb-2">"No booths yet"</h2>
            <p class="text-gray-600 mb-6">
                "Get started by creating a new booth or importing existing data"
            </p>
            
            <div class="flex gap-4 justify-center">
                <a 
                    href="/booths/new"
                    class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                    "➕ Create New Booth"
                </a>
                
                <a 
                    href="/sync"
                    class="px-6 py-3 bg-white border-2 border-gray-300 rounded-lg hover:bg-gray-50"
                >
                    "📥 Import Data"
                </a>
            </div>
            
            <div class="mt-8 p-4 bg-blue-50 rounded-lg inline-block">
                <p class="text-sm text-blue-800">
                    "💡 <strong>Switching browsers?</strong> "
                    "Export your data from the other browser, then import it here. "
                    <a href="/help/switching-browsers" class="underline">"Learn how →"</a>
                </p>
            </div>
        </div>
    }
}
```

**4. Persistent Helper in Settings**
```rust
#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="settings-page">
            <h1>"Settings"</h1>
            
            // Always show import/export prominently
            <section class="data-portability bg-gradient-to-r from-blue-50 to-purple-50 p-6 rounded-lg mb-8">
                <h2 class="text-xl font-semibold mb-2">"📦 Data Portability"</h2>
                <p class="mb-4">"Switch browsers or devices easily"</p>
                <div class="flex gap-4">
                    <a href="/sync" class="px-4 py-2 bg-blue-600 text-white rounded">
                        "Export / Import"
                    </a>
                    <a href="/help/switching-browsers" class="px-4 py-2 border rounded">
                        "Learn More"
                    </a>
                </div>
            </section>
            
            // ... other settings
        </div>
    }
}
```

**5. Browser Detection in Footer**
```rust
#[component]
pub fn Footer() -> impl IntoView {
    let browser = BrowserInfo::detect();
    
    view! {
        <footer class="text-center text-sm text-gray-500 p-4">
            <p>"Running in " {browser.browser_name} " | "
            <a href="/sync" class="underline">"Switch browsers?"</a>
            "</p>
        </footer>
    }
}
```

#### Help Documentation

**New Help Page:** `/help/switching-browsers`

```markdown
# Switching Browsers

## Why do I need to export/import?

ez-booth stores all data in your browser's local storage. Each browser 
(Chrome, Firefox, Safari, etc.) has its own separate storage that other 
browsers cannot access.

## Step-by-Step Guide

### From Your Old Browser:
1. Open ez-booth
2. Go to Settings → Export Data
3. Click "Download Export"
4. Save the `ez-booth-export-YYYYMMDD.json` file

### In Your New Browser:
1. Open ez-booth
2. Click "Import Data" on the welcome screen (or go to Settings → Import)
3. Select the JSON file you downloaded
4. Choose "Merge with existing" (recommended)
5. Click Import

### Pro Tip: Use Cloud Sync
Save your export to Dropbox or Google Drive for automatic sync across 
devices!

## Troubleshooting

**Q: I don't see the welcome screen**
A: Go directly to Settings → Export/Import

**Q: Import failed with "Invalid checksum"**
A: The file may be corrupted. Try exporting again.

**Q: Some data is missing after import**
A: Check that you selected the correct JSON file and used "Merge" strategy.
```

#### Implementation Priority

| Feature | Priority | Phase | Effort |
|---------|----------|-------|--------|
| Empty state detection | P0 | Phase 6 | 1 hour |
| Welcome screen | P0 | Phase 6 | 4 hours |
| Browser detection | P1 | Phase 6 | 2 hours |
| Help documentation | P0 | Phase 6 | 2 hours |
| Navigation hints | P1 | Phase 7 | 2 hours |
| Footer browser info | P2 | Phase 7 | 1 hour |

#### Success Metrics

**User Awareness:**
- 90%+ of new browser users see welcome screen
- 80%+ understand they need to import data
- <5% support tickets about "lost data"

**Conversion Rate:**
- 70%+ of users with empty state click "Import Data"
- 60%+ successfully complete import process
- 40%+ set up cloud folder sync (Phase 7)

---

## 10. Internationalization (i18n)

### 10.1 Overview

**Primary Language:** German (de)  
**Fallback Language:** English (en)  
**Future:** Extensible to additional languages

The Java version has full German localization (172 translation keys). This must be preserved in the Rust version.

### 10.2 Technology

**Library:** `leptos_i18n 0.3+`

**Features:**
- Compile-time key validation
- Type-safe translations
- Browser locale detection
- Reactive language switching
- Small footprint (~20KB)

### 10.3 File Structure

```
locales/
├── de.json          # German (primary, 172 keys)
├── en.json          # English (fallback)
└── translations.json # Config
```

### 10.4 Implementation

```rust
use leptos_i18n::*;

// Browser locale detection
pub fn init_i18n() -> Locale {
    let browser_lang = window()
        .navigator()
        .language()
        .unwrap_or_default();
    
    match browser_lang.split('-').next() {
        Some("de") => Locale::De,
        Some("en") => Locale::En,
        _ => Locale::De, // Default to German
    }
}

// Usage in components
#[component]
pub fn BoothForm() -> impl IntoView {
    let i18n = use_i18n();
    
    view! {
        <h2>{t!(i18n, booth.title)}</h2>
        <label>{t!(i18n, booth.description.label)}</label>
        <button>{t!(i18n, common.save)}</button>
    }
}
```

### 10.5 Format Helpers

Locale-aware formatting for:
- **Currency:** EUR vs USD formatting
- **Dates:** dd.MM.yyyy (de) vs MM/dd/yyyy (en)
- **Decimals:** Comma vs period separators

### 10.6 Translation Categories

| Category | Keys | Examples |
|----------|------|----------|
| App Layout | 3 | Title, tooltips |
| Booth Management | 35 | Forms, validation, status |
| Checkout | 25 | Keypad, confirmation, receipts |
| Vendor Reports | 20 | Lists, statistics, printing |
| Report Templates | 15 | Print headers, totals, labels |
| Export/Import | 30 | File operations, notifications |
| Generic | 14 | Buttons, errors, common text |

**Total:** ~142 translation keys (expandable to match Java's 172)

### 10.7 Report Template Localization

#### Current Problem
Java version uses hardcoded German strings in Thymeleaf templates:
- `VendorReport.template.html`: "Verkäufer-Quittung", "Gesamtsumme", "Zeitraum"
- Reports cannot be generated in other languages
- No locale-aware date/number formatting in templates

#### Proposed Solution
**Template Rendering with i18n:**
```rust
pub fn render_vendor_report(
    vendor: &Vendor,
    items: &[PurchaseItem],
    locale: Locale,
) -> String {
    let i18n = get_translations(locale);
    
    format!(r#"
        <html>
        <head>
            <title>{}</title>
            <style>/* Print-friendly CSS */</style>
        </head>
        <body>
            <h1>{}</h1>
            <table>
                <tr>
                    <th>{}</th>
                    <th>{}</th>
                    <th>{}</th>
                </tr>
                {items_html}
            </table>
            <div class="total">
                {}: {total}
            </div>
        </body>
        </html>
    "#,
        i18n.report.vendor_receipt,
        i18n.report.vendor_receipt,
        i18n.report.date,
        i18n.report.item,
        i18n.report.amount,
        items_html = render_items(items, locale),
        i18n.report.total,
        total = format_currency(calculate_total(items), locale)
    )
}
```

**Key Translation Keys for Reports:**
```json
{
  "report": {
    "vendor_receipt": {
      "de": "Verkäufer-Quittung",
      "en": "Vendor Receipt"
    },
    "total": {
      "de": "Gesamtsumme",
      "en": "Total"
    },
    "period": {
      "de": "Zeitraum",
      "en": "Period"
    },
    "date": {
      "de": "Datum",
      "en": "Date"
    },
    "item": {
      "de": "Artikel",
      "en": "Item"
    },
    "amount": {
      "de": "Betrag",
      "en": "Amount"
    },
    "quantity": {
      "de": "Anzahl",
      "en": "Quantity"
    }
  }
}
```

**Locale-Aware Formatting:**
- **Currency:** `12,50 €` (DE) vs `€12.50` (EN)
- **Date:** `19.03.2026` (DE) vs `03/19/2026` (EN)
- **Numbers:** `1.234,56` (DE) vs `1,234.56` (EN)

**Print CSS remains language-agnostic** - page breaks, margins, fonts work for all languages.

### 10.8 Implementation Timeline

**Phase 1 (Week 1):** Setup i18n infrastructure + report templates (10 hours)  
**Phase 2 (Ongoing):** Replace hardcoded strings (4 hours)  
**Phase 4 (Week 1):** Testing and validation (2 hours)

**Total Additional Effort:** 16 hours (~2 days)

### 10.9 Success Metrics

| Metric | Target |
|--------|--------|
| Translation coverage | 100% (UI + reports) |
| Browser locale detection | 95%+ accuracy |
| Language switch latency | <50ms |
| Bundle size impact | <20KB |
| Report language accuracy | 100% |

**For detailed implementation, see:** `/changelog/06_LOCALIZATION_ARCHITECTURE.md`

---

## 11. Security & Privacy

### 10.1 Data Protection

#### Browser-Only Mode
- **Encryption at Rest:** Not required (local-only data)
- **Privacy:** All data stays in browser, never transmitted
- **Clear Data:** User can clear browser storage to delete all data

#### Client-Server Mode
- **HTTPS Only:** Enforce TLS for all communication
- **Authentication:** Optional JWT-based auth
- **Authorization:** Role-based access control (RBAC)
- **Data Encryption:** Encrypt sensitive fields in database

### 10.2 Input Validation

- **Frontend:** Immediate validation feedback
- **Backend:** Re-validate all inputs (never trust client)
- **Type Safety:** Rust types prevent injection attacks
- **Sanitization:** Escape user input in reports/exports

### 10.3 CSRF Protection

- **SameSite Cookies:** Use `SameSite=Strict` for session cookies
- **CSRF Tokens:** For state-changing operations
- **Origin Validation:** Check `Origin` header on server

---

## 11. Performance Targets

### 11.1 Bundle Size

| Asset | Target | Rationale |
|-------|--------|-----------|
| WASM | <3MB | Gzipped to ~800KB over network |
| JavaScript | <500KB | Wasm-bindgen glue + Leptos runtime |
| CSS | <50KB | Tailwind purged of unused classes |
| **Total** | **<3.5MB** | Initial load, cached thereafter |

### 11.2 Runtime Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to Interactive | <500ms | Lighthouse audit |
| First Contentful Paint | <300ms | Lighthouse audit |
| Largest Contentful Paint | <1s | Lighthouse audit |
| Memory Usage | <50MB | Browser DevTools |
| Checkout Transaction | <100ms | Lighthouse audit |

### 11.3 Storage Limits

| Storage | Limit | Notes |
|---------|-------|-------|
| IndexedDB | ~50% of disk space | Browser-dependent quota |
| LocalStorage | ~10MB | Settings only |
| Cache API | ~50% of disk space | Service worker cache |

**Estimated Capacity:**
- ~10,000 purchases per booth (10MB typical)
- ~100 booths per installation
- ~1 million purchase items total

---

## Appendices

### A. Technology Comparison Matrix

| Feature | Leptos | Yew | Dioxus | Svelte* |
|---------|--------|-----|--------|---------|
| Bundle Size | ~150KB | ~200KB | ~250KB | ~50KB |
| Reactivity | Signals | Virtual DOM | Virtual DOM | Compiler |
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Maturity | Medium | High | Medium | High |
| Rust Native | ✅ | ✅ | ✅ | ❌ |
| SSR Support | ✅ | ❌ | ✅ | ✅ |

*Svelte requires JavaScript toolchain

**Decision:** Leptos for optimal balance of size, performance, and DX

### B. Browser Compatibility Matrix

| Browser | Minimum Version | WASM | IndexedDB | Service Worker |
|---------|----------------|------|-----------|----------------|
| Chrome | 90+ (2021) | ✅ | ✅ | ✅ |
| Firefox | 88+ (2021) | ✅ | ✅ | ✅ |
| Safari | 14+ (2020) | ✅ | ✅ | ✅ |
| Edge | 90+ (2021) | ✅ | ✅ | ✅ |

**Coverage:** ~95% of users (2024+ data)

### C. Development Timeline Estimate

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Core MVP | 4 weeks | Entities, services, storage, basic UI, booths, vendors, checkout |
| Phase 2: Reports & Export | 3 weeks | Report generation, printing, **export/import, cross-browser portability** |
| Phase 3: Polish & i18n | 3 weeks | Responsive design, DE/EN localization, PWA, accessibility, File System API |
| Phase 4: Testing & Launch | 2 weeks | E2E tests, browser testing, performance optimization |
| **Total** | **12 weeks** | **MVP Release** |

**Accelerated Timeline:** Consolidated phases reduce MVP delivery from 18 to 12 weeks by combining related work and deferring advanced features (CRDT sync, plugin system, cloud features) to post-MVP.

---

**Document Status:** Ready for Review  
**Next Steps:** 
1. Review and approval of architecture design
2. Proceed to Implementation Details specification
3. Set up initial project structure
