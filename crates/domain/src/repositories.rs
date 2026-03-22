//! Repository trait definitions for data persistence
//!
//! These traits define the interface between the domain logic and storage implementations.
//! They use async_trait with ?Send to support WASM environments.

use crate::error::DomainResult;
use crate::models::{Booth, BoothId, Purchase, PurchaseId, Vendor, VendorId};
use async_trait::async_trait;

/// Repository trait for booth persistence operations
#[async_trait(?Send)]
pub trait BoothRepository {
    /// Save a booth (insert or update)
    async fn save(&self, booth: &Booth) -> DomainResult<()>;
    
    /// Find a booth by its ID
    async fn find_by_id(&self, id: &BoothId) -> DomainResult<Option<Booth>>;
    
    /// Find all booths
    async fn find_all(&self) -> DomainResult<Vec<Booth>>;
    
    /// Delete a booth by ID
    async fn delete(&self, id: &BoothId) -> DomainResult<()>;
}

/// Repository trait for vendor persistence operations
#[async_trait(?Send)]
pub trait VendorRepository {
    /// Save a vendor (insert or update)
    async fn save(&self, vendor: &Vendor) -> DomainResult<()>;
    
    /// Find a vendor by booth ID and vendor ID
    async fn find_by_id(&self, booth_id: &BoothId, vendor_id: &VendorId) -> DomainResult<Option<Vendor>>;
    
    /// Find all vendors for a specific booth
    async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Vendor>>;
    
    /// Find all vendors across all booths
    async fn find_all(&self) -> DomainResult<Vec<Vendor>>;
    
    /// Delete a vendor
    async fn delete(&self, booth_id: &BoothId, vendor_id: &VendorId) -> DomainResult<()>;
}

/// Repository trait for purchase persistence operations
#[async_trait(?Send)]
pub trait PurchaseRepository {
    /// Save a purchase (insert or update)
    async fn save(&self, purchase: &Purchase) -> DomainResult<()>;
    
    /// Find a purchase by its ID
    async fn find_by_id(&self, id: &PurchaseId) -> DomainResult<Option<Purchase>>;
    
    /// Find all purchases for a specific booth
    async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Purchase>>;
    
    /// Find all purchases for a specific vendor in a booth
    async fn find_by_vendor(&self, booth_id: &BoothId, vendor_id: &VendorId) -> DomainResult<Vec<Purchase>>;
    
    /// Find all purchases
    async fn find_all(&self) -> DomainResult<Vec<Purchase>>;
    
    /// Delete a purchase by ID
    async fn delete(&self, id: &PurchaseId) -> DomainResult<()>;
}
