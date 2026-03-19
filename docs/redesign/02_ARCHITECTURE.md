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
9. [Security & Privacy](#security--privacy)
10. [Performance Targets](#performance-targets)

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

#### Date/Time: **chrono 0.4+**
**Rationale:**
- Industry standard for Rust
- Timezone support
- WASM compatible

#### UUID Generation: **uuid 1.7+**
**Rationale:**
- Fast UUID v4 generation
- WASM compatible

#### Validation: **validator 0.18+**
**Rationale:**
- Derive macros for validation rules
- Works in WASM and native

#### Error Handling: **thiserror 1.0+**
**Rationale:**
- Ergonomic error types
- Good error messages

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

## Security & Privacy

### 9.1 Data Protection

#### Browser-Only Mode
- **Encryption at Rest:** Not required (local-only data)
- **Privacy:** All data stays in browser, never transmitted
- **Clear Data:** User can clear browser storage to delete all data

#### Client-Server Mode
- **HTTPS Only:** Enforce TLS for all communication
- **Authentication:** Optional JWT-based auth
- **Authorization:** Role-based access control (RBAC)
- **Data Encryption:** Encrypt sensitive fields in database

### 9.2 Input Validation

- **Frontend:** Immediate validation feedback
- **Backend:** Re-validate all inputs (never trust client)
- **Type Safety:** Rust types prevent injection attacks
- **Sanitization:** Escape user input in reports/exports

### 9.3 CSRF Protection

- **SameSite Cookies:** Use `SameSite=Strict` for session cookies
- **CSRF Tokens:** For state-changing operations
- **Origin Validation:** Check `Origin` header on server

---

## Performance Targets

### 10.1 Bundle Size

| Asset | Target | Rationale |
|-------|--------|-----------|
| WASM | <3MB | Gzipped to ~800KB over network |
| JavaScript | <500KB | Wasm-bindgen glue + Leptos runtime |
| CSS | <50KB | Tailwind purged of unused classes |
| **Total** | **<3.5MB** | Initial load, cached thereafter |

### 10.2 Runtime Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to Interactive | <500ms | Lighthouse audit |
| First Contentful Paint | <300ms | Lighthouse audit |
| Largest Contentful Paint | <1s | Lighthouse audit |
| Memory Usage | <50MB | Browser DevTools |
| Checkout Transaction | <100ms | Lighthouse audit |

### 10.3 Storage Limits

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
| Phase 1: Foundation | 2 weeks | Core entities, services, tests |
| Phase 2: Storage | 2 weeks | IndexedDB wrapper, repositories |
| Phase 3: UI Core | 3 weeks | Component library, routing, state |
| Phase 4: Features | 3 weeks | Booth mgmt, vendors, checkout |
| Phase 5: Reports | 2 weeks | Report generation, printing |
| Phase 6: Sync | 2 weeks | Export/import, server protocol |
| Phase 7: Polish | 2 weeks | Styling, PWA, accessibility |
| Phase 8: Testing | 2 weeks | E2E tests, browser testing |
| **Total** | **18 weeks** | **MVP Release** |

---

**Document Status:** Ready for Review  
**Next Steps:** 
1. Review and approval of architecture design
2. Proceed to Implementation Details specification
3. Set up initial project structure
