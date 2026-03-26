use crate::error::{DomainError, DomainResult};
use crate::models::{BoothId, Vendor, VendorId};
use crate::repositories::{BoothRepository, VendorRepository};
use crate::validation::validate_vendor_id;

/// Service for vendor management operations
pub struct VendorService<VR: VendorRepository, BR: BoothRepository> {
    vendor_repository: VR,
    booth_repository: BR,
}

impl<VR: VendorRepository, BR: BoothRepository> VendorService<VR, BR> {
    pub fn new(vendor_repository: VR, booth_repository: BR) -> Self {
        Self {
            vendor_repository,
            booth_repository,
        }
    }

    /// Get or create vendor by ID (auto-created during checkout)
    ///
    /// Validates the vendor ID against the booth's validation rules before creation
    pub async fn get_or_create(
        &self,
        booth_id: BoothId,
        vendor_id_str: String,
    ) -> DomainResult<Vendor> {
        // Get booth to check validation rules
        let booth = self
            .booth_repository
            .find_by_id(&booth_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Booth {} not found", booth_id)))?;

        // Validate vendor ID
        validate_vendor_id(&vendor_id_str, &booth.vendor_id_validation)?;

        let vendor_id = VendorId::new(vendor_id_str.clone());

        if let Some(vendor) = self
            .vendor_repository
            .find_by_id(&booth_id, &vendor_id)
            .await?
        {
            Ok(vendor)
        } else {
            let vendor = Vendor::new(vendor_id, booth_id);
            self.vendor_repository.save(&vendor).await?;
            Ok(vendor)
        }
    }

    /// List all vendors with smart sorting.
    /// Numeric IDs (e.g., "1", "42") sorted numerically: 1, 2, 10, 42
    /// Alphanumeric IDs sorted lexicographically after numeric IDs.
    /// Critical for correct print order in vendor reports.
    pub async fn list_vendors(&self, booth_id: BoothId) -> DomainResult<Vec<Vendor>> {
        let mut vendors = self.vendor_repository.find_by_booth(&booth_id).await?;

        // VendorId already implements Ord with smart sorting
        vendors.sort_by_key(|v| v.vendor_id.clone());

        Ok(vendors)
    }

    /// Get a specific vendor
    pub async fn get_vendor(
        &self,
        booth_id: BoothId,
        vendor_id_str: String,
    ) -> DomainResult<Vendor> {
        let vendor_id = VendorId::new(vendor_id_str.clone());
        self.vendor_repository
            .find_by_id(&booth_id, &vendor_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!(
                    "Vendor {} not found in booth {}",
                    vendor_id_str,
                    booth_id.as_str()
                ))
            })
    }

    /// Delete a vendor
    pub async fn delete_vendor(
        &self,
        booth_id: BoothId,
        vendor_id_str: String,
    ) -> DomainResult<()> {
        let vendor_id = VendorId::new(vendor_id_str);
        self.vendor_repository.delete(&booth_id, &vendor_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Booth, FeeConfig, VendorIdValidation};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    #[derive(Clone)]
    struct MockVendorRepository {
        vendors: Arc<Mutex<HashMap<(BoothId, VendorId), Vendor>>>,
    }

    impl MockVendorRepository {
        fn new() -> Self {
            Self {
                vendors: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait(?Send)]
    impl VendorRepository for MockVendorRepository {
        async fn save(&self, vendor: &Vendor) -> DomainResult<()> {
            self.vendors
                .lock()
                .unwrap()
                .insert((vendor.booth_id, vendor.vendor_id.clone()), vendor.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            booth_id: &BoothId,
            vendor_id: &VendorId,
        ) -> DomainResult<Option<Vendor>> {
            Ok(self
                .vendors
                .lock()
                .unwrap()
                .get(&(*booth_id, vendor_id.clone()))
                .cloned())
        }

        async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Vendor>> {
            Ok(self
                .vendors
                .lock()
                .unwrap()
                .iter()
                .filter(|((bid, _), _)| bid == booth_id)
                .map(|(_, v)| v.clone())
                .collect())
        }

        async fn find_all(&self) -> DomainResult<Vec<Vendor>> {
            Ok(self.vendors.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, booth_id: &BoothId, vendor_id: &VendorId) -> DomainResult<()> {
            self.vendors
                .lock()
                .unwrap()
                .remove(&(*booth_id, vendor_id.clone()));
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockBoothRepository {
        booths: Arc<Mutex<HashMap<BoothId, Booth>>>,
    }

    impl MockBoothRepository {
        fn new() -> Self {
            Self {
                booths: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_booth(&self, booth: Booth) {
            self.booths.lock().unwrap().insert(booth.id, booth);
        }
    }

    #[async_trait(?Send)]
    impl BoothRepository for MockBoothRepository {
        async fn save(&self, booth: &Booth) -> DomainResult<()> {
            self.booths.lock().unwrap().insert(booth.id, booth.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &BoothId) -> DomainResult<Option<Booth>> {
            Ok(self.booths.lock().unwrap().get(id).cloned())
        }

        async fn find_all(&self) -> DomainResult<Vec<Booth>> {
            Ok(self.booths.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, id: &BoothId) -> DomainResult<()> {
            self.booths.lock().unwrap().remove(id);
            Ok(())
        }
    }

    fn create_test_booth_with_validation(validation: VendorIdValidation) -> Booth {
        let fees = FeeConfig {
            participation_fee: Decimal::new(5, 0),
            sales_fee_percent: Decimal::new(10, 0),
            rounding_step: Decimal::new(50, 2),
        };

        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

        let mut booth = Booth::new("Test Booth".to_string(), date, fees).unwrap();

        // Override the default validation with the one we want
        booth.vendor_id_validation = validation;
        booth
    }

    fn create_test_booth_id() -> BoothId {
        BoothId::new()
    }

    #[tokio::test]
    async fn test_get_or_create_new_vendor() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::Unrestricted);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        let vendor = service
            .get_or_create(booth_id, "V123".to_string())
            .await
            .unwrap();

        assert_eq!(vendor.vendor_id.as_str(), "V123");
        assert_eq!(vendor.booth_id, booth_id);
    }

    #[tokio::test]
    async fn test_get_or_create_existing_vendor() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::Unrestricted);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Create vendor first time
        let vendor1 = service
            .get_or_create(booth_id, "V123".to_string())
            .await
            .unwrap();

        // Get same vendor second time (should not create new)
        let vendor2 = service
            .get_or_create(booth_id, "V123".to_string())
            .await
            .unwrap();

        assert_eq!(vendor1.vendor_id, vendor2.vendor_id);
        assert_eq!(vendor1.created_at, vendor2.created_at);
    }

    #[tokio::test]
    async fn test_list_vendors_with_smart_sorting() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::Unrestricted);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Create vendors in random order
        service
            .get_or_create(booth_id, "10".to_string())
            .await
            .unwrap();
        service
            .get_or_create(booth_id, "2".to_string())
            .await
            .unwrap();
        service
            .get_or_create(booth_id, "1".to_string())
            .await
            .unwrap();
        service
            .get_or_create(booth_id, "V5".to_string())
            .await
            .unwrap();
        service
            .get_or_create(booth_id, "25".to_string())
            .await
            .unwrap();
        service
            .get_or_create(booth_id, "A3".to_string())
            .await
            .unwrap();

        // List vendors (should be sorted: 1, 2, 10, 25, A3, V5)
        let vendors = service.list_vendors(booth_id).await.unwrap();

        assert_eq!(vendors.len(), 6);
        assert_eq!(vendors[0].vendor_id.as_str(), "1");
        assert_eq!(vendors[1].vendor_id.as_str(), "2");
        assert_eq!(vendors[2].vendor_id.as_str(), "10");
        assert_eq!(vendors[3].vendor_id.as_str(), "25");
        assert_eq!(vendors[4].vendor_id.as_str(), "A3");
        assert_eq!(vendors[5].vendor_id.as_str(), "V5");
    }

    #[tokio::test]
    async fn test_delete_vendor() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::Unrestricted);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Create vendor
        service
            .get_or_create(booth_id, "V123".to_string())
            .await
            .unwrap();

        // Delete vendor
        service
            .delete_vendor(booth_id, "V123".to_string())
            .await
            .unwrap();

        // Verify deletion
        let result = service.get_vendor(booth_id, "V123".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_digits_only_success() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::DigitsOnly);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Should succeed with digits only
        let vendor = service
            .get_or_create(booth_id, "12345".to_string())
            .await
            .unwrap();

        assert_eq!(vendor.vendor_id.as_str(), "12345");
    }

    #[tokio::test]
    async fn test_validation_digits_only_failure() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth = create_test_booth_with_validation(VendorIdValidation::DigitsOnly);
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Should fail with non-digits
        let result = service.get_or_create(booth_id, "V123".to_string()).await;

        assert!(result.is_err());
        match result {
            Err(DomainError::Validation(msg)) => {
                assert!(msg.contains("only digits"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validation_regex_success() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth =
            create_test_booth_with_validation(VendorIdValidation::Regex(r"^V\d{3}$".to_string()));
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Should succeed with matching pattern
        let vendor = service
            .get_or_create(booth_id, "V123".to_string())
            .await
            .unwrap();

        assert_eq!(vendor.vendor_id.as_str(), "V123");
    }

    #[tokio::test]
    async fn test_validation_regex_failure() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let booth =
            create_test_booth_with_validation(VendorIdValidation::Regex(r"^V\d{3}$".to_string()));
        let booth_id = booth.id;
        booth_repo.add_booth(booth);

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());

        // Should fail with non-matching pattern
        let result = service.get_or_create(booth_id, "A123".to_string()).await;

        assert!(result.is_err());
        match result {
            Err(DomainError::Validation(msg)) => {
                assert!(msg.contains("does not match"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validation_booth_not_found() {
        let vendor_repo = MockVendorRepository::new();
        let booth_repo = MockBoothRepository::new();

        let service = VendorService::new(vendor_repo.clone(), booth_repo.clone());
        let booth_id = create_test_booth_id();

        // Should fail when booth doesn't exist
        let result = service.get_or_create(booth_id, "V123".to_string()).await;

        assert!(result.is_err());
        match result {
            Err(DomainError::NotFound(msg)) => {
                assert!(msg.contains("Booth"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
