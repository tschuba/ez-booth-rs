# ez-booth-rs Implementation Specification

**Document Version:** 1.0  
**Date:** March 19, 2026  
**Status:** Design Phase  
**Related Documents:** [ARCHITECTURE.md](ARCHITECTURE.md), [IMPROVEMENTS.md](IMPROVEMENTS.md)

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Core Domain Implementation](#core-domain-implementation)
3. [Storage Layer Specification](#storage-layer-specification)
4. [Frontend Implementation](#frontend-implementation)
5. [Backend Implementation (Optional)](#backend-implementation-optional)
6. [Data Synchronization Protocol](#data-synchronization-protocol)
7. [Build & Deployment](#build--deployment)
8. [Testing Strategy](#testing-strategy)
9. [Performance Optimization](#performance-optimization)
10. [Security Implementation](#security-implementation)

---

## 1. Project Structure

### 1.1 Workspace Configuration

**File:** `Cargo.toml` (workspace root)

```toml
[workspace]
members = [
    "crates/core",
    "crates/storage",
    "crates/frontend",
    "crates/server",
    "crates/shared",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["ez-booth-rs contributors"]
license = "PolyForm-Noncommercial-1.0.0"
repository = "https://github.com/tschuba/ez-booth-rs"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.7", features = ["v4", "serde"] }

# Decimal (for currency)
rust_decimal = { version = "1.34", features = ["serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async runtime
tokio = { version = "1.36", features = ["full"] }
futures = "0.3"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Better optimization, slower compile
panic = "abort"     # Smaller binary
strip = true        # Remove symbols
```

### 1.2 Directory Structure

```
ez-booth-rs/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/
│   ├── core/                     # Domain logic (no dependencies)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── entities/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── booth.rs      # Booth entity
│   │   │   │   ├── vendor.rs     # Vendor entity
│   │   │   │   ├── purchase.rs   # Purchase + PurchaseItem
│   │   │   │   └── ids.rs        # Type-safe IDs
│   │   │   ├── services/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── booth_service.rs
│   │   │   │   ├── vendor_service.rs
│   │   │   │   ├── purchase_service.rs
│   │   │   │   ├── charging_service.rs
│   │   │   │   └── reporting_service.rs
│   │   │   ├── validation/
│   │   │   │   ├── mod.rs
│   │   │   │   └── rules.rs
│   │   │   └── error.rs
│   │   └── tests/
│   │
│   ├── storage/                  # Storage abstraction
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── repository/       # Repository traits
│   │   │   │   ├── mod.rs
│   │   │   │   ├── booth_repo.rs
│   │   │   │   ├── vendor_repo.rs
│   │   │   │   └── purchase_repo.rs
│   │   │   ├── indexeddb/        # IndexedDB implementation
│   │   │   │   ├── mod.rs
│   │   │   │   ├── database.rs
│   │   │   │   ├── booth_repo.rs
│   │   │   │   ├── vendor_repo.rs
│   │   │   │   └── purchase_repo.rs
│   │   │   ├── sql/              # SQL implementation (server)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── migrations/
│   │   │   │   └── repos.rs
│   │   │   └── memory/           # In-memory (testing)
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   ├── frontend/                 # WASM UI (Leptos)
│   │   ├── Cargo.toml
│   │   ├── Trunk.toml
│   │   ├── index.html
│   │   ├── style/
│   │   │   ├── main.css
│   │   │   └── tailwind.css
│   │   ├── assets/
│   │   │   ├── icons/
│   │   │   └── manifest.json
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs            # Root component
│   │   │   ├── router.rs         # Route definitions
│   │   │   ├── pages/            # Page components
│   │   │   │   ├── mod.rs
│   │   │   │   ├── home.rs
│   │   │   │   ├── booth_list.rs
│   │   │   │   ├── booth_detail.rs
│   │   │   │   ├── checkout.rs
│   │   │   │   ├── reports.rs
│   │   │   │   └── sync.rs
│   │   │   ├── components/       # Reusable components
│   │   │   │   ├── mod.rs
│   │   │   │   ├── button.rs
│   │   │   │   ├── table.rs
│   │   │   │   ├── form.rs
│   │   │   │   └── modal.rs
│   │   │   ├── state/            # Global state
│   │   │   │   ├── mod.rs
│   │   │   │   ├── app_state.rs
│   │   │   │   └── storage_state.rs
│   │   │   ├── hooks/            # Custom hooks
│   │   │   │   └── mod.rs
│   │   │   └── utils/
│   │   │       ├── mod.rs
│   │   │       └── format.rs
│   │   └── tests/
│   │
│   ├── server/                   # Optional backend
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── config.rs
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── booth.rs
│   │   │   │   ├── vendor.rs
│   │   │   │   ├── purchase.rs
│   │   │   │   └── sync.rs
│   │   │   ├── middleware/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── auth.rs
│   │   │   │   └── logging.rs
│   │   │   └── sync/
│   │   │       ├── mod.rs
│   │   │       └── protocol.rs
│   │   └── tests/
│   │
│   └── shared/                   # Client-server shared types
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── dto/              # Data transfer objects
│           │   └── mod.rs
│           └── protocol/         # Sync protocol
│               └── mod.rs
│
├── docs/                         # Documentation
│   ├── SPEC.md
│   ├── ANALYSIS.md
│   ├── ARCHITECTURE.md
│   ├── IMPROVEMENTS.md
│   ├── IMPLEMENTATION.md         # This file
│   └── api/                      # API documentation
│
├── tests/                        # Integration tests
│   ├── integration/
│   └── e2e/
│
├── scripts/                      # Build/deploy scripts
│   ├── build.sh
│   ├── test.sh
│   └── deploy.sh
│
└── .github/                      # CI/CD
    └── workflows/
        └── ci.yml
```

---

## 2. Core Domain Implementation

### 2.1 Entity Definitions

**File:** `crates/core/src/entities/ids.rs`

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn as_str(&self) -> String {
                self.0.to_string()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }
    };
}

define_id!(BoothId);
define_id!(VendorId);
define_id!(PurchaseId);
define_id!(ItemId);
```

**File:** `crates/core/src/entities/booth.rs`

```rust
use crate::entities::ids::BoothId;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Booth {
    pub id: BoothId,
    
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    
    pub date: NaiveDate,
    
    #[validate]
    pub fees: FeeConfig,
    
    pub status: BoothStatus,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct FeeConfig {
    #[validate(range(min = 0.0))]
    pub participation_fee: Decimal,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub sales_fee_percent: Decimal,
    
    #[validate(range(min = 0.0))]
    pub rounding_step: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BoothStatus {
    Open,
    Closed { closed_at: DateTime<Utc> },
}

impl Booth {
    pub fn new(description: String, date: NaiveDate, fees: FeeConfig) -> Self {
        let now = Utc::now();
        Self {
            id: BoothId::new(),
            description,
            date,
            fees,
            status: BoothStatus::Open,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn close(&mut self) {
        let now = Utc::now();
        self.status = BoothStatus::Closed { closed_at: now };
        self.updated_at = now;
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.status, BoothStatus::Closed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_booth_creation() {
        let fees = FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(5.0),
            rounding_step: dec!(0.50),
        };
        
        let booth = Booth::new(
            "Test Booth".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 19).unwrap(),
            fees,
        );
        
        assert!(!booth.is_closed());
        assert_eq!(booth.description, "Test Booth");
    }

    #[test]
    fn test_booth_closing() {
        let fees = FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(5.0),
            rounding_step: dec!(0.50),
        };
        
        let mut booth = Booth::new(
            "Test Booth".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 19).unwrap(),
            fees,
        );
        
        booth.close();
        assert!(booth.is_closed());
    }
}
```

**File:** `crates/core/src/entities/vendor.rs`

```rust
use crate::entities::ids::{BoothId, VendorId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vendor {
    pub id: VendorId,
    pub booth_id: BoothId,
    pub created_at: DateTime<Utc>,
}

impl Vendor {
    pub fn new(booth_id: BoothId) -> Self {
        Self {
            id: VendorId::new(),
            booth_id,
            created_at: Utc::now(),
        }
    }
}
```

**File:** `crates/core/src/entities/purchase.rs`

```rust
use crate::entities::ids::{BoothId, ItemId, PurchaseId, VendorId};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Purchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    
    #[validate]
    pub items: Vec<PurchaseItem>,
    
    pub total: Decimal,
    pub purchased_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct PurchaseItem {
    pub id: ItemId,
    pub vendor_id: VendorId,
    
    #[validate(range(min = 0.0))]
    pub price: Decimal,
    
    pub purchased_at: DateTime<Utc>,
}

impl Purchase {
    pub fn new(booth_id: BoothId, items: Vec<PurchaseItem>) -> Self {
        let total = items.iter().map(|item| item.price).sum();
        
        Self {
            id: PurchaseId::new(),
            booth_id,
            items,
            total,
            purchased_at: Utc::now(),
        }
    }

    pub fn recalculate_total(&mut self) {
        self.total = self.items.iter().map(|item| item.price).sum();
    }
}

impl PurchaseItem {
    pub fn new(vendor_id: VendorId, price: Decimal) -> Self {
        Self {
            id: ItemId::new(),
            vendor_id,
            price,
            purchased_at: Utc::now(),
        }
    }
}
```

### 2.2 Service Implementations

**File:** `crates/core/src/services/charging_service.rs`

```rust
use crate::entities::booth::FeeConfig;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeCalculation {
    pub total_sales: Decimal,
    pub participation_fee: Decimal,
    pub sales_fee: Decimal,
    pub total_fees: Decimal,
    pub net_revenue: Decimal,
}

pub struct ChargingService;

impl ChargingService {
    pub fn calculate_fees(
        total_sales: Decimal,
        config: &FeeConfig,
    ) -> FeeCalculation {
        let participation = config.participation_fee;
        
        // Calculate sales fee as percentage
        let sales_fee_raw = total_sales * config.sales_fee_percent 
            / Decimal::from(100);
        
        // Apply rounding
        let sales_fee = Self::round_to_step(sales_fee_raw, config.rounding_step);
        
        let total_fees = participation + sales_fee;
        let net_revenue = total_sales - total_fees;
        
        FeeCalculation {
            total_sales,
            participation_fee: participation,
            sales_fee,
            total_fees,
            net_revenue,
        }
    }
    
    fn round_to_step(value: Decimal, step: Decimal) -> Decimal {
        if step.is_zero() {
            value
        } else {
            // Round to nearest step
            let steps = (value / step).round();
            steps * step
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::booth::FeeConfig;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fee_calculation() {
        let config = FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(5.0),
            rounding_step: dec!(0.50),
        };
        
        let result = ChargingService::calculate_fees(dec!(100.00), &config);
        
        assert_eq!(result.total_sales, dec!(100.00));
        assert_eq!(result.participation_fee, dec!(10.00));
        assert_eq!(result.sales_fee, dec!(5.00)); // 5% of 100
        assert_eq!(result.total_fees, dec!(15.00));
        assert_eq!(result.net_revenue, dec!(85.00));
    }

    #[test]
    fn test_rounding() {
        let config = FeeConfig {
            participation_fee: dec!(0.00),
            sales_fee_percent: dec!(5.0),
            rounding_step: dec!(0.50),
        };
        
        // 5% of 107 = 5.35, rounded to 5.50
        let result = ChargingService::calculate_fees(dec!(107.00), &config);
        assert_eq!(result.sales_fee, dec!(5.50));
    }
}
```

### 2.3 Repository Traits

**File:** `crates/core/src/services/booth_service.rs`

```rust
use crate::entities::booth::{Booth, FeeConfig};
use crate::entities::ids::BoothId;
use crate::error::CoreError;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait(?Send)] // ?Send for WASM compatibility
pub trait BoothRepository {
    async fn save(&self, booth: &Booth) -> Result<(), CoreError>;
    async fn find_by_id(&self, id: BoothId) -> Result<Option<Booth>, CoreError>;
    async fn find_all(&self) -> Result<Vec<Booth>, CoreError>;
    async fn delete(&self, id: BoothId) -> Result<(), CoreError>;
}

pub struct BoothService<R: BoothRepository> {
    repository: R,
}

impl<R: BoothRepository> BoothService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_booth(
        &self,
        description: String,
        date: NaiveDate,
        fees: FeeConfig,
    ) -> Result<Booth, CoreError> {
        let booth = Booth::new(description, date, fees);
        booth.validate()?;
        
        self.repository.save(&booth).await?;
        Ok(booth)
    }

    pub async fn get_booth(&self, id: BoothId) -> Result<Booth, CoreError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound("Booth not found".to_string()))
    }

    pub async fn list_booths(&self) -> Result<Vec<Booth>, CoreError> {
        self.repository.find_all().await
    }

    pub async fn update_booth(&self, booth: Booth) -> Result<(), CoreError> {
        booth.validate()?;
        self.repository.save(&booth).await
    }

    pub async fn close_booth(&self, id: BoothId) -> Result<Booth, CoreError> {
        let mut booth = self.get_booth(id).await?;
        booth.close();
        self.repository.save(&booth).await?;
        Ok(booth)
    }

    pub async fn delete_booth(&self, id: BoothId) -> Result<(), CoreError> {
        self.repository.delete(id).await
    }
}
```

---

## 3. Storage Layer Specification

### 3.1 IndexedDB Implementation

**File:** `crates/storage/Cargo.toml`

```toml
[package]
name = "ez-booth-storage"
version.workspace = true
edition.workspace = true

[dependencies]
ez-booth-core = { path = "../core" }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
bincode = { workspace = true }

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window",
    "IdbFactory",
    "IdbDatabase",
    "IdbObjectStore",
    "IdbTransaction",
    "IdbRequest",
    "IdbOpenDbRequest",
    "IdbCursor",
    "IdbKeyRange",
] }

# IndexedDB wrapper
rexie = "0.6"

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Async
futures = { workspace = true }

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**File:** `crates/storage/src/indexeddb/database.rs`

```rust
use rexie::{Rexie, TransactionMode};
use wasm_bindgen::prelude::*;

const DB_NAME: &str = "ez_booth_v1";
const DB_VERSION: u32 = 1;

pub struct Database {
    db: Rexie,
}

impl Database {
    pub async fn new() -> Result<Self, JsValue> {
        let rexie = Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(
                rexie::ObjectStore::new("booths")
                    .key_path("id")
                    .add_index(rexie::Index::new("date", "date"))
                    .add_index(rexie::Index::new("status", "status.type"))
            )
            .add_object_store(
                rexie::ObjectStore::new("vendors")
                    .key_path_array(&["booth_id", "id"])
                    .add_index(rexie::Index::new("booth_id", "booth_id"))
            )
            .add_object_store(
                rexie::ObjectStore::new("purchases")
                    .key_path_array(&["booth_id", "id"])
                    .add_index(rexie::Index::new("booth_id", "booth_id"))
                    .add_index(rexie::Index::new("purchased_at", "purchased_at"))
            )
            .add_object_store(
                rexie::ObjectStore::new("purchase_items")
                    .key_path_array(&["booth_id", "purchase_id", "id"])
                    .add_index(rexie::Index::new("booth_id", "booth_id"))
                    .add_index(rexie::Index::new("vendor_id", "vendor_id"))
                    .add_index(rexie::Index::new("purchased_at", "purchased_at"))
            )
            .build()
            .await?;
        
        Ok(Self { db: rexie })
    }

    pub fn transaction(
        &self,
        stores: &[&str],
        mode: TransactionMode,
    ) -> Result<rexie::Transaction, JsValue> {
        self.db.transaction(stores, mode)
    }
}
```

**File:** `crates/storage/src/indexeddb/booth_repo.rs`

```rust
use crate::indexeddb::database::Database;
use async_trait::async_trait;
use ez_booth_core::entities::booth::Booth;
use ez_booth_core::entities::ids::BoothId;
use ez_booth_core::error::CoreError;
use ez_booth_core::services::booth_service::BoothRepository;
use rexie::TransactionMode;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

pub struct IndexedDbBoothRepository {
    db: Database,
}

impl IndexedDbBoothRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait(?Send)]
impl BoothRepository for IndexedDbBoothRepository {
    async fn save(&self, booth: &Booth) -> Result<(), CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| CoreError::Storage(format!("Transaction error: {:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::Storage(format!("Store error: {:?}", e)))?;
        
        let value = to_value(booth)
            .map_err(|e| CoreError::Serialization(e.to_string()))?;
        
        store
            .put(&value, None)
            .await
            .map_err(|e| CoreError::Storage(format!("Put error: {:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| CoreError::Storage(format!("Commit error: {:?}", e)))?;
        
        Ok(())
    }

    async fn find_by_id(&self, id: BoothId) -> Result<Option<Booth>, CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| CoreError::Storage(format!("Transaction error: {:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::Storage(format!("Store error: {:?}", e)))?;
        
        let key = to_value(&id.as_str())
            .map_err(|e| CoreError::Serialization(e.to_string()))?;
        
        let result = store
            .get(&key)
            .await
            .map_err(|e| CoreError::Storage(format!("Get error: {:?}", e)))?;
        
        match result {
            Some(value) => {
                let booth: Booth = from_value(value)
                    .map_err(|e| CoreError::Deserialization(e.to_string()))?;
                Ok(Some(booth))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<Booth>, CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadOnly)
            .map_err(|e| CoreError::Storage(format!("Transaction error: {:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::Storage(format!("Store error: {:?}", e)))?;
        
        let values = store
            .get_all(None, None)
            .await
            .map_err(|e| CoreError::Storage(format!("GetAll error: {:?}", e)))?;
        
        let booths: Vec<Booth> = values
            .into_iter()
            .map(|v| from_value(v).map_err(|e| CoreError::Deserialization(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(booths)
    }

    async fn delete(&self, id: BoothId) -> Result<(), CoreError> {
        let transaction = self
            .db
            .transaction(&["booths"], TransactionMode::ReadWrite)
            .map_err(|e| CoreError::Storage(format!("Transaction error: {:?}", e)))?;
        
        let store = transaction
            .store("booths")
            .map_err(|e| CoreError::Storage(format!("Store error: {:?}", e)))?;
        
        let key = to_value(&id.as_str())
            .map_err(|e| CoreError::Serialization(e.to_string()))?;
        
        store
            .delete(&key)
            .await
            .map_err(|e| CoreError::Storage(format!("Delete error: {:?}", e)))?;
        
        transaction
            .done()
            .await
            .map_err(|e| CoreError::Storage(format!("Commit error: {:?}", e)))?;
        
        Ok(())
    }
}
```

---

## 4. Frontend Implementation

### 4.1 Leptos Configuration

**File:** `crates/frontend/Cargo.toml`

```toml
[package]
name = "ez-booth-frontend"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ez-booth-core = { path = "../core" }
ez-booth-storage = { path = "../storage" }

# Leptos
leptos = { version = "0.6", features = ["csr", "nightly"] }
leptos_meta = { version = "0.6", features = ["csr"] }
leptos_router = { version = "0.6", features = ["csr"] }

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlElement"] }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Date/Time
chrono = { workspace = true, features = ["wasmbind"] }

# Logging
console_error_panic_hook = "0.1"
tracing = { workspace = true }
tracing-wasm = "0.2"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**File:** `crates/frontend/Trunk.toml`

```toml
[build]
target = "index.html"
release = true
dist = "dist"
public_url = "/"

[watch]
ignore = ["dist", "target"]

[serve]
address = "127.0.0.1"
port = 8080
open = false

[clean]
dist = "dist"
cargo = true
```

### 4.2 Main Application

**File:** `crates/frontend/src/main.rs`

```rust
use leptos::*;
use ez_booth_frontend::app::App;

fn main() {
    // Set up logging
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    // Mount the app
    mount_to_body(|| view! { <App/> })
}
```

**File:** `crates/frontend/src/app.rs`

```rust
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::{
    home::HomePage,
    booth_list::BoothListPage,
    booth_detail::BoothDetailPage,
    checkout::CheckoutPage,
    reports::ReportsPage,
    sync::SyncPage,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/ez-booth-frontend.css"/>
        <Title text="ez-booth"/>
        <Meta name="description" content="Portable booth management system"/>
        
        <Router>
            <nav class="navbar">
                <A href="/">"Home"</A>
                <A href="/booths">"Booths"</A>
                <A href="/checkout">"Checkout"</A>
                <A href="/reports">"Reports"</A>
                <A href="/sync">"Sync"</A>
            </nav>
            
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/booths" view=BoothListPage/>
                    <Route path="/booths/:id" view=BoothDetailPage/>
                    <Route path="/checkout" view=CheckoutPage/>
                    <Route path="/reports" view=ReportsPage/>
                    <Route path="/sync" view=SyncPage/>
                </Routes>
            </main>
        </Router>
    }
}
```

### 4.3 State Management

**File:** `crates/frontend/src/state/app_state.rs`

```rust
use ez_booth_core::entities::booth::Booth;
use ez_booth_core::entities::ids::BoothId;
use ez_booth_storage::indexeddb::database::Database;
use leptos::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub booths: RwSignal<HashMap<BoothId, Booth>>,
    pub selected_booth: RwSignal<Option<BoothId>>,
    pub database: Database,
}

impl AppState {
    pub async fn new() -> Result<Self, String> {
        let database = Database::new()
            .await
            .map_err(|e| format!("Database init error: {:?}", e))?;
        
        Ok(Self {
            booths: create_rw_signal(HashMap::new()),
            selected_booth: create_rw_signal(None),
            database,
        })
    }

    pub async fn load_booths(&self) -> Result<(), String> {
        // Implementation using repository
        Ok(())
    }

    pub fn select_booth(&self, id: BoothId) {
        self.selected_booth.set(Some(id));
    }

    pub fn get_selected_booth(&self) -> Option<Booth> {
        self.selected_booth.get()
            .and_then(|id| self.booths.get().get(&id).cloned())
    }
}
```

---

## 5. Backend Implementation (Optional)

### 5.1 Server Configuration

**File:** `crates/server/Cargo.toml`

```toml
[package]
name = "ez-booth-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "server"
path = "src/main.rs"

[dependencies]
ez-booth-core = { path = "../core" }
ez-booth-storage = { path = "../storage" }
ez-booth-shared = { path = "../shared" }

# Web framework
axum = { version = "0.7", features = ["macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "trace", "cors"] }

# Async runtime
tokio = { workspace = true }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono", "uuid"] }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Config
config = "0.14"
```

**File:** `crates/server/src/main.rs`

```rust
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod middleware;
mod sync;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::load()?;

    // Build application routes
    let api_routes = Router::new()
        .route("/booths", get(handlers::booth::list_booths))
        .route("/booths", post(handlers::booth::create_booth))
        .route("/sync", post(handlers::sync::sync_data));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## 6. Data Synchronization Protocol

### 6.1 Sync Protocol Definition

**File:** `crates/shared/src/protocol/mod.rs`

```rust
use chrono::{DateTime, Utc};
use ez_booth_core::entities::{booth::Booth, vendor::Vendor, purchase::Purchase};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub client_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub changes: ChangeSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub server_time: DateTime<Utc>,
    pub changes: ChangeSet,
    pub conflicts: Vec<Conflict>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeSet {
    pub booths: Vec<Change<Booth>>,
    pub vendors: Vec<Change<Vendor>>,
    pub purchases: Vec<Change<Purchase>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change<T> {
    pub operation: Operation,
    pub entity: T,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Operation {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conflict {
    pub entity_type: String,
    pub entity_id: String,
    pub client_version: u64,
    pub server_version: u64,
    pub resolution: ConflictResolution,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Manual,
}
```

---

## 7. Build & Deployment

### 7.1 Build Scripts

**File:** `scripts/build.sh`

```bash
#!/bin/bash
set -e

echo "Building ez-booth-rs..."

# Build frontend (WASM)
echo "Building frontend..."
cd crates/frontend
trunk build --release
cd ../..

# Optimize WASM
echo "Optimizing WASM..."
wasm-opt -Oz -o crates/frontend/dist/optimized.wasm \
    crates/frontend/dist/*.wasm

# Build server (optional)
if [ "$BUILD_SERVER" = "true" ]; then
    echo "Building server..."
    cargo build --release --bin server
fi

echo "Build complete!"
echo "Frontend: crates/frontend/dist/"
echo "Server: target/release/server"
```

### 7.2 GitHub Actions CI

**File:** `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run wasm tests
        run: wasm-pack test --headless --firefox crates/frontend

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Install Trunk
        run: wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.19.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-
      
      - name: Build frontend
        run: |
          cd crates/frontend
          ../../trunk build --release
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: frontend-dist
          path: crates/frontend/dist/
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
// Example: crates/core/src/services/charging_service.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_basic_fee_calculation() {
        let config = FeeConfig {
            participation_fee: dec!(10.00),
            sales_fee_percent: dec!(5.0),
            rounding_step: dec!(0.01),
        };
        
        let result = ChargingService::calculate_fees(dec!(100.00), &config);
        
        assert_eq!(result.participation_fee, dec!(10.00));
        assert_eq!(result.sales_fee, dec!(5.00));
        assert_eq!(result.total_fees, dec!(15.00));
    }
}
```

### 8.2 WASM Browser Tests

```rust
// Example: crates/frontend/tests/integration.rs

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use ez_booth_frontend::app::App;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_app_renders() {
    // Test that app component mounts without errors
    leptos::mount_to_body(App);
}
```

---

## 9. Performance Optimization

### 9.1 WASM Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit
panic = "abort"           # Smaller binary
strip = true              # Remove debug symbols

[profile.release.package."*"]
opt-level = "z"
```

### 9.2 Bundle Size Optimization

```bash
# Post-build optimization
wasm-opt -Oz input.wasm -o output.wasm
wasm-strip output.wasm
gzip -9 output.wasm
```

---

## 10. Security Implementation

### 10.1 Input Validation

```rust
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct CreateBoothRequest {
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    
    #[validate(custom = "validate_date")]
    pub date: NaiveDate,
    
    #[validate]
    pub fees: FeeConfig,
}

fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
    let min_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let max_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
    
    if date < &min_date || date > &max_date {
        return Err(ValidationError::new("date_out_of_range"));
    }
    
    Ok(())
}
```

### 10.2 Content Security Policy

```html
<!-- index.html -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               script-src 'self' 'wasm-unsafe-eval'; 
               style-src 'self' 'unsafe-inline';">
```

---

**Document Status:** Ready for Implementation  
**Next Steps:**
1. Set up project structure
2. Implement Phase 1 (Core domain)
3. Begin iterative development

**Estimated Implementation Timeline:** 18 weeks to MVP

