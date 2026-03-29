use crate::error::{DomainError, DomainResult};
use crate::error_code::ValidationError;
use crate::models::{Booth, BoothId, FeeConfig, VendorIdValidation};
use crate::repositories::BoothRepository;
use crate::validation::validate_regex_pattern;
use chrono::NaiveDate;

/// Service for booth management operations
pub struct BoothService<R: BoothRepository> {
    repository: R,
}

impl<R: BoothRepository> BoothService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Create a new booth with validation
    pub async fn create_booth(
        &self,
        description: String,
        date: NaiveDate,
        fees: FeeConfig,
    ) -> DomainResult<Booth> {
        let booth = Booth::new(description, date, fees)?;
        self.save_booth(&booth).await?;
        Ok(booth)
    }

    /// Create a new booth from a fully configured booth value.
    pub async fn create_configured_booth(&self, booth: Booth) -> DomainResult<Booth> {
        self.save_booth(&booth).await?;
        Ok(booth)
    }

    /// Get a booth by ID
    pub async fn get_booth(&self, id: BoothId) -> DomainResult<Booth> {
        self.repository
            .find_by_id(&id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Booth {} not found", id.as_str())))
    }

    /// List all booths
    pub async fn list_booths(&self) -> DomainResult<Vec<Booth>> {
        self.repository.find_all().await
    }

    /// Update an existing booth with validation
    pub async fn update_booth(&self, booth: Booth) -> DomainResult<()> {
        self.validate_booth(&booth)?;
        self.ensure_unique_name_and_date(Some(booth.id), &booth.description, &booth.date)
            .await?;
        self.repository.save(&booth).await
    }

    /// Copy an existing booth to a new name and date.
    pub async fn copy_booth(
        &self,
        source_id: BoothId,
        new_description: String,
        new_date: NaiveDate,
    ) -> DomainResult<Booth> {
        let source = self.get_booth(source_id).await?;
        let mut booth = Booth::new(new_description, new_date, source.fees.clone())?;
        booth.vendor_id_validation = source.vendor_id_validation.clone();
        booth.vendor_id_omission_rules = source.vendor_id_omission_rules.clone();
        booth.keyboard_config = source.keyboard_config.clone();

        self.save_booth(&booth).await?;
        Ok(booth)
    }

    /// Delete a booth
    pub async fn delete_booth(&self, id: BoothId) -> DomainResult<()> {
        self.repository.delete(&id).await
    }

    async fn save_booth(&self, booth: &Booth) -> DomainResult<()> {
        self.validate_booth(booth)?;
        self.ensure_unique_name_and_date(None, &booth.description, &booth.date)
            .await?;
        self.repository.save(booth).await
    }

    fn validate_booth(&self, booth: &Booth) -> DomainResult<()> {
        booth.fees.validate_ranges()?;

        if let VendorIdValidation::Regex(pattern) = &booth.vendor_id_validation {
            validate_regex_pattern(pattern)?;
        }

        Ok(())
    }

    async fn ensure_unique_name_and_date(
        &self,
        current_id: Option<BoothId>,
        description: &str,
        date: &NaiveDate,
    ) -> DomainResult<()> {
        if let Some(existing) = self
            .repository
            .find_by_description_and_date(description, date)
            .await?
        {
            if current_id != Some(existing.id) {
                return Err(DomainError::Validation(
                    ValidationError::BoothDuplicateNameAndDate {
                        description: description.trim().to_string(),
                        date: date.format("%Y-%m-%d").to_string(),
                    },
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FeeConfig, VendorIdValidation};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
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

        async fn find_by_description_and_date(
            &self,
            description: &str,
            date: &NaiveDate,
        ) -> DomainResult<Option<Booth>> {
            Ok(self
                .booths
                .lock()
                .unwrap()
                .values()
                .find(|booth| booth.date == *date && booth.description.trim() == description.trim())
                .cloned())
        }

        async fn delete(&self, id: &BoothId) -> DomainResult<()> {
            self.booths.lock().unwrap().remove(id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_booth() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        let result = service
            .create_booth(
                "Test Booth".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees,
            )
            .await;

        assert!(result.is_ok());
        let booth = result.unwrap();
        assert_eq!(booth.description, "Test Booth");
    }

    #[tokio::test]
    async fn test_update_booth_with_valid_regex() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        let mut booth = service
            .create_booth(
                "Test Booth".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees,
            )
            .await
            .unwrap();

        // Update with valid regex pattern
        booth.vendor_id_validation = VendorIdValidation::Regex(r"^V\d{3}$".to_string());

        let result = service.update_booth(booth).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_booth_with_invalid_regex() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        let mut booth = service
            .create_booth(
                "Test Booth".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees,
            )
            .await
            .unwrap();

        // Update with invalid regex pattern
        booth.vendor_id_validation = VendorIdValidation::Regex("[invalid".to_string());

        let result = service.update_booth(booth).await;
        assert!(result.is_err());
        match result {
            Err(DomainError::Validation(
                crate::error_code::ValidationError::RegexPatternInvalid,
            )) => {}
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_booth_rejects_duplicate_name_and_date() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };
        let date = NaiveDate::from_ymd_opt(2026, 3, 22).unwrap();

        service
            .create_booth("Spring Fair".to_string(), date, fees.clone())
            .await
            .unwrap();

        let result = service
            .create_booth("Spring Fair".to_string(), date, fees)
            .await;

        assert!(matches!(
            result,
            Err(DomainError::Validation(
                ValidationError::BoothDuplicateNameAndDate { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_update_booth_allows_same_name_and_date_for_same_booth() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        let mut booth = service
            .create_booth(
                "Spring Fair".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees,
            )
            .await
            .unwrap();

        booth.update_fees(FeeConfig {
            participation_fee: dec!(6.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        });

        assert!(service.update_booth(booth).await.is_ok());
    }

    #[tokio::test]
    async fn test_copy_booth_copies_configuration_only() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let mut source = service
            .create_booth(
                "Spring Fair".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                FeeConfig {
                    participation_fee: dec!(5.0),
                    sales_fee_percent: dec!(10.0),
                    rounding_step: dec!(0.50),
                },
            )
            .await
            .unwrap();
        source.vendor_id_validation = VendorIdValidation::Regex(r"^V\d{3}$".to_string());
        source.vendor_id_omission_rules = crate::models::VendorIdOmissionRules::recommended();
        source.keyboard_config.quick_amounts = vec![dec!(1.0), dec!(2.5), dec!(5.0)];
        service.update_booth(source.clone()).await.unwrap();

        let copied = service
            .copy_booth(
                source.id,
                "Spring Fair Copy".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(copied.id, source.id);
        assert_eq!(copied.description, "Spring Fair Copy");
        assert_eq!(copied.date, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
        assert_eq!(copied.fees, source.fees);
        assert_eq!(copied.vendor_id_validation, source.vendor_id_validation);
        assert_eq!(
            copied.vendor_id_omission_rules,
            source.vendor_id_omission_rules
        );
        assert_eq!(copied.keyboard_config, source.keyboard_config);
    }
}
