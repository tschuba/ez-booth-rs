use crate::error::{DomainError, DomainResult};
use crate::models::{Booth, BoothId, BoothStatus, FeeConfig};
use crate::repositories::BoothRepository;
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
        // Booth::new performs validation and returns Result
        let booth = Booth::new(description, date, fees)?;

        self.repository.save(&booth).await?;
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

    /// List booths filtered by status
    pub async fn list_booths_by_status(&self, status: BoothStatus) -> DomainResult<Vec<Booth>> {
        let all_booths = self.repository.find_all().await?;
        Ok(all_booths
            .into_iter()
            .filter(|b| b.status == status)
            .collect())
    }

    /// Update an existing booth with validation
    pub async fn update_booth(&self, booth: Booth) -> DomainResult<()> {
        // Validate fees configuration
        booth.fees.validate_ranges()?;

        self.repository.save(&booth).await
    }

    /// Close a booth (set status to Closed)
    pub async fn close_booth(&self, id: BoothId) -> DomainResult<Booth> {
        let mut booth = self.get_booth(id).await?;
        booth.close();
        self.repository.save(&booth).await?;
        Ok(booth)
    }

    /// Reopen a closed booth
    pub async fn reopen_booth(&self, id: BoothId) -> DomainResult<Booth> {
        let mut booth = self.get_booth(id).await?;
        booth.status = BoothStatus::Open;
        booth.updated_at = chrono::Utc::now();
        self.repository.save(&booth).await?;
        Ok(booth)
    }

    /// Delete a booth
    pub async fn delete_booth(&self, id: BoothId) -> DomainResult<()> {
        self.repository.delete(&id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FeeConfig;
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
        assert_eq!(booth.status, BoothStatus::Open);
    }

    #[tokio::test]
    async fn test_close_and_reopen_booth() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        let booth = service
            .create_booth(
                "Test Booth".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees,
            )
            .await
            .unwrap();

        // Close booth
        let closed = service.close_booth(booth.id).await.unwrap();
        assert!(matches!(closed.status, BoothStatus::Closed { .. }));

        // Reopen booth
        let reopened = service.reopen_booth(booth.id).await.unwrap();
        assert_eq!(reopened.status, BoothStatus::Open);
    }

    #[tokio::test]
    async fn test_list_booths_by_status() {
        let repo = MockBoothRepository::new();
        let service = BoothService::new(repo);

        let fees = FeeConfig {
            participation_fee: dec!(5.0),
            sales_fee_percent: dec!(10.0),
            rounding_step: dec!(0.50),
        };

        // Create two booths
        let booth1 = service
            .create_booth(
                "Booth 1".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
                fees.clone(),
            )
            .await
            .unwrap();

        let booth2 = service
            .create_booth(
                "Booth 2".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 23).unwrap(),
                fees,
            )
            .await
            .unwrap();

        // Close booth1
        service.close_booth(booth1.id).await.unwrap();

        // List open booths
        let open_booths = service
            .list_booths_by_status(BoothStatus::Open)
            .await
            .unwrap();
        assert_eq!(open_booths.len(), 1);
        assert_eq!(open_booths[0].id, booth2.id);
    }
}
