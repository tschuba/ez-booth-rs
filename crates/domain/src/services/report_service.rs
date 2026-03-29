use crate::error::{DomainError, DomainResult};
use crate::models::{BoothId, BoothSummary, Purchase, VendorBoothSummary, VendorId};
use crate::repositories::{BoothRepository, PurchaseRepository, VendorRepository};
use crate::services::dto::{ChargingConfig, VendorReportData, VendorReportItem};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// Service for generating reports and analytics
pub struct ReportService<PR: PurchaseRepository, BR: BoothRepository, VR: VendorRepository> {
    purchase_repository: PR,
    booth_repository: BR,
    vendor_repository: VR,
}

impl<PR: PurchaseRepository, BR: BoothRepository, VR: VendorRepository> ReportService<PR, BR, VR> {
    pub fn new(purchase_repository: PR, booth_repository: BR, vendor_repository: VR) -> Self {
        Self {
            purchase_repository,
            booth_repository,
            vendor_repository,
        }
    }

    /// Generate a comprehensive summary for a booth
    pub async fn generate_booth_summary(
        &self,
        booth_id: &BoothId,
        date_range: Option<DateRange>,
    ) -> DomainResult<BoothSummary> {
        // Get booth information
        let booth = self
            .booth_repository
            .find_by_id(booth_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Booth {} not found", booth_id)))?;

        // Get all purchases for the booth
        let mut purchases = self.purchase_repository.find_by_booth(booth_id).await?;

        // Apply date range filter if specified
        if let Some(range) = &date_range {
            purchases = Self::filter_by_date_range(purchases, range);
        }

        // Group purchase items by vendor
        let mut vendor_items: HashMap<VendorId, Vec<(&Purchase, &crate::models::PurchaseItem)>> =
            HashMap::new();
        for purchase in &purchases {
            for item in &purchase.items {
                vendor_items
                    .entry(item.vendor_id.clone())
                    .or_default()
                    .push((purchase, item));
            }
        }

        // Calculate charging config
        let charging_config = ChargingConfig::from_booth(&booth);

        // Generate vendor summaries
        let mut vendor_summaries: Vec<VendorBoothSummary> = Vec::new();
        for (vendor_id, vendor_item_list) in vendor_items.iter() {
            let gross_sales: Decimal = vendor_item_list.iter().map(|(_, item)| item.amount).sum();

            let payout = charging_config.calculate_payout(gross_sales);

            // Count total items for this vendor
            let item_count: usize = vendor_item_list.len();

            vendor_summaries.push(VendorBoothSummary {
                vendor_id: vendor_id.clone(),
                gross_sales: payout.gross_sales,
                fees_due: payout.fees_due,
                net_payout: payout.net_payout,
                item_count,
            });
        }

        // Sort vendor summaries by vendor_id (uses smart sorting)
        vendor_summaries.sort_by(|a, b| a.vendor_id.cmp(&b.vendor_id));

        // Calculate totals
        let total_revenue: Decimal = vendor_summaries.iter().map(|v| v.gross_sales).sum();
        let total_purchases = purchases.len();
        let total_items: usize = vendor_summaries.iter().map(|v| v.item_count).sum();
        let unique_vendors = vendor_items.len();

        // Calculate booth revenue metrics
        let total_participation_fees: Decimal = vendor_summaries
            .iter()
            .map(|_| charging_config.participation_fee)
            .sum();
        let total_sales_fees: Decimal = vendor_summaries
            .iter()
            .map(|v| v.fees_due - charging_config.participation_fee)
            .sum();
        let total_booth_revenue = total_participation_fees + total_sales_fees;

        debug_assert_eq!(
            vendor_summaries.iter().map(|v| v.fees_due).sum::<Decimal>(),
            total_booth_revenue,
            "sum of vendor fees must match booth revenue"
        );

        Ok(BoothSummary {
            booth_id: *booth_id,
            total_revenue,
            total_purchases,
            total_items,
            unique_vendors,
            vendor_summaries,
            participation_fee: booth.fees.participation_fee,
            sales_fee_percent: booth.fees.sales_fee_percent,
            total_participation_fees,
            total_sales_fees,
            total_booth_revenue,
        })
    }

    /// Generate a detailed report for a specific vendor in a booth
    pub async fn generate_vendor_report(
        &self,
        booth_id: &BoothId,
        vendor_id: &VendorId,
        date_range: Option<DateRange>,
    ) -> DomainResult<VendorReportData> {
        // Get booth information
        let booth = self
            .booth_repository
            .find_by_id(booth_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Booth {} not found", booth_id)))?;

        // Get vendor information
        let vendor = self
            .vendor_repository
            .find_by_id(booth_id, vendor_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!(
                    "Vendor {} not found in booth {}",
                    vendor_id, booth_id
                ))
            })?;

        // Get all purchases for the vendor
        let mut purchases = self
            .purchase_repository
            .find_by_vendor(booth_id, vendor_id)
            .await?;

        // Apply date range filter if specified
        if let Some(range) = &date_range {
            purchases = Self::filter_by_date_range(purchases, range);
        }

        // Collect all items from purchases with their transaction IDs
        // Filter to only include items for this specific vendor
        let items: Vec<VendorReportItem> = purchases
            .iter()
            .flat_map(|p| {
                p.items
                    .iter()
                    .filter(|item| &item.vendor_id == vendor_id)
                    .map(|item| VendorReportItem {
                        transaction_id: p.id,
                        item: item.clone(),
                        timestamp: p.timestamp,
                    })
            })
            .collect();

        // Calculate totals
        let sales_sum: Decimal = items
            .iter()
            .map(|report_item| report_item.item.amount)
            .sum();

        // Calculate payout with rounding applied to net payout
        let charging_config = ChargingConfig::from_booth(&booth);
        let payout = charging_config.calculate_payout(sales_sum);

        Ok(VendorReportData {
            vendor,
            booth,
            items,
            sales_sum: payout.gross_sales,
            participation_fee: charging_config.participation_fee,
            sales_fee: payout.fees_due - charging_config.participation_fee,
            total_revenue: payout.net_payout,
        })
    }

    /// Generate reports for multiple vendors in a booth
    pub async fn generate_vendor_reports(
        &self,
        booth_id: &BoothId,
        vendor_ids: Vec<VendorId>,
        date_range: Option<DateRange>,
    ) -> DomainResult<Vec<VendorReportData>> {
        let mut reports = Vec::new();

        for vendor_id in vendor_ids {
            let report = self
                .generate_vendor_report(booth_id, &vendor_id, date_range.clone())
                .await?;
            reports.push(report);
        }

        // Sort reports by vendor_id (uses smart sorting through Ord implementation)
        reports.sort();

        Ok(reports)
    }

    /// Get all vendors who have made purchases in a booth
    pub async fn get_active_vendors(
        &self,
        booth_id: &BoothId,
        date_range: Option<DateRange>,
    ) -> DomainResult<Vec<VendorId>> {
        let mut purchases = self.purchase_repository.find_by_booth(booth_id).await?;

        // Apply date range filter if specified
        if let Some(range) = &date_range {
            purchases = Self::filter_by_date_range(purchases, range);
        }

        // Collect unique vendor IDs from all items across all purchases
        let vendor_ids: HashSet<VendorId> = purchases
            .into_iter()
            .flat_map(|p| p.items.into_iter().map(|item| item.vendor_id))
            .collect();

        // Convert to sorted vector
        let mut vendor_ids: Vec<VendorId> = vendor_ids.into_iter().collect();
        vendor_ids.sort();

        Ok(vendor_ids)
    }

    /// Filter purchases by date range
    fn filter_by_date_range(purchases: Vec<Purchase>, range: &DateRange) -> Vec<Purchase> {
        purchases
            .into_iter()
            .filter(|p| {
                let timestamp = p.timestamp;
                if let Some(start) = range.start {
                    if timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = range.end {
                    if timestamp > end {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

/// Date range filter for reports
#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl DateRange {
    pub fn new(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        Self { start, end }
    }

    /// Create a date range for all time
    pub fn all_time() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Booth, BoothStatus, FeeConfig, PurchaseItem, Vendor};
    use crate::{BoothRunningTotals, PaginatedPurchases};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repositories for testing
    #[derive(Clone)]
    struct MockPurchaseRepository {
        purchases: Arc<Mutex<HashMap<BoothId, Vec<Purchase>>>>,
    }

    impl MockPurchaseRepository {
        fn new() -> Self {
            Self {
                purchases: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_purchase(&self, purchase: Purchase) {
            let mut purchases = self.purchases.lock().unwrap();
            purchases
                .entry(purchase.booth_id.clone())
                .or_default()
                .push(purchase);
        }
    }

    #[async_trait(?Send)]
    impl PurchaseRepository for MockPurchaseRepository {
        async fn save(&self, _purchase: &Purchase) -> DomainResult<()> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            _purchase_id: &crate::models::PurchaseId,
        ) -> DomainResult<Option<Purchase>> {
            Ok(None)
        }

        async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Purchase>> {
            let purchases = self.purchases.lock().unwrap();
            Ok(purchases.get(booth_id).cloned().unwrap_or_default())
        }

        async fn find_by_booth_paginated(
            &self,
            booth_id: &BoothId,
            offset: usize,
            limit: usize,
        ) -> DomainResult<PaginatedPurchases> {
            let mut purchases = self.find_by_booth(booth_id).await?;
            purchases.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            let total_count = purchases.len();
            let items: Vec<Purchase> = purchases.into_iter().skip(offset).take(limit).collect();

            Ok(PaginatedPurchases { items, total_count })
        }

        async fn get_running_totals(&self, booth_id: &BoothId) -> DomainResult<BoothRunningTotals> {
            let purchases = self.find_by_booth(booth_id).await?;

            let total_sales: Decimal = purchases.iter().map(|p| p.total_amount()).sum();
            let total_items: usize = purchases.iter().map(|p| p.items.len()).sum();
            let total_checkouts = purchases.len();

            Ok(BoothRunningTotals {
                total_sales,
                total_items,
                total_checkouts,
            })
        }

        async fn find_by_vendor(
            &self,
            booth_id: &BoothId,
            vendor_id: &VendorId,
        ) -> DomainResult<Vec<Purchase>> {
            let purchases = self.purchases.lock().unwrap();
            Ok(purchases
                .get(booth_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|p| p.items.iter().any(|item| &item.vendor_id == vendor_id))
                .collect())
        }

        async fn find_all(&self) -> DomainResult<Vec<Purchase>> {
            let purchases = self.purchases.lock().unwrap();
            Ok(purchases.values().flat_map(|v| v.clone()).collect())
        }

        async fn delete(&self, _id: &crate::models::PurchaseId) -> DomainResult<()> {
            Ok(())
        }

        async fn delete_from_booth(
            &self,
            _booth_id: &BoothId,
            _id: &crate::models::PurchaseId,
        ) -> DomainResult<()> {
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
            let mut booths = self.booths.lock().unwrap();
            booths.insert(booth.id.clone(), booth);
        }
    }

    #[async_trait(?Send)]
    impl BoothRepository for MockBoothRepository {
        async fn save(&self, _booth: &Booth) -> DomainResult<()> {
            Ok(())
        }

        async fn find_by_id(&self, booth_id: &BoothId) -> DomainResult<Option<Booth>> {
            let booths = self.booths.lock().unwrap();
            Ok(booths.get(booth_id).cloned())
        }

        async fn find_all(&self) -> DomainResult<Vec<Booth>> {
            let booths = self.booths.lock().unwrap();
            Ok(booths.values().cloned().collect())
        }

        async fn find_by_description_and_date(
            &self,
            description: &str,
            date: &NaiveDate,
        ) -> DomainResult<Option<Booth>> {
            let booths = self.booths.lock().unwrap();
            Ok(booths
                .values()
                .find(|booth| booth.date == *date && booth.description.trim() == description.trim())
                .cloned())
        }

        async fn delete(&self, _id: &BoothId) -> DomainResult<()> {
            Ok(())
        }
    }

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

        fn add_vendor(&self, booth_id: BoothId, vendor: Vendor) {
            let mut vendors = self.vendors.lock().unwrap();
            vendors.insert((booth_id, vendor.vendor_id.clone()), vendor);
        }
    }

    #[async_trait(?Send)]
    impl VendorRepository for MockVendorRepository {
        async fn save(&self, _vendor: &Vendor) -> DomainResult<()> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            booth_id: &BoothId,
            vendor_id: &VendorId,
        ) -> DomainResult<Option<Vendor>> {
            let vendors = self.vendors.lock().unwrap();
            Ok(vendors.get(&(booth_id.clone(), vendor_id.clone())).cloned())
        }

        async fn find_by_booth(&self, booth_id: &BoothId) -> DomainResult<Vec<Vendor>> {
            let vendors = self.vendors.lock().unwrap();
            Ok(vendors
                .iter()
                .filter(|((bid, _), _)| bid == booth_id)
                .map(|(_, v)| v.clone())
                .collect())
        }

        async fn find_all(&self) -> DomainResult<Vec<Vendor>> {
            let vendors = self.vendors.lock().unwrap();
            Ok(vendors.values().cloned().collect())
        }

        async fn delete(&self, _booth_id: &BoothId, _vendor_id: &VendorId) -> DomainResult<()> {
            Ok(())
        }

        async fn delete_from_booth(
            &self,
            _booth_id: &BoothId,
            _vendor_id: &VendorId,
        ) -> DomainResult<()> {
            Ok(())
        }
    }

    fn create_test_booth() -> Booth {
        Booth {
            id: BoothId::new(),
            description: "Test Booth".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            fees: FeeConfig {
                participation_fee: dec!(5.00),
                sales_fee_percent: dec!(10.0),
                rounding_step: dec!(0.50),
            },
            status: BoothStatus::Open,
            vendor_id_validation: crate::models::VendorIdValidation::default(),
            vendor_id_omission_rules: crate::models::VendorIdOmissionRules::empty(),
            keyboard_config: crate::models::CheckoutKeyboardConfig::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_vendor(booth_id: &BoothId, vendor_id: &str) -> Vendor {
        Vendor {
            vendor_id: VendorId::new(vendor_id.to_string()),
            booth_id: booth_id.clone(),
            name: Some(format!("Vendor {}", vendor_id)),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_generate_booth_summary() {
        let booth = create_test_booth();
        let vendor1 = create_test_vendor(&booth.id, "1");
        let vendor2 = create_test_vendor(&booth.id, "2");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor1.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor2.clone());

        // Add some purchases
        let purchase1 = Purchase::new(
            booth.id.clone(),
            vec![
                PurchaseItem::new(dec!(10.00), vendor1.vendor_id.clone()).unwrap(),
                PurchaseItem::new(dec!(5.00), vendor1.vendor_id.clone()).unwrap(),
            ],
        )
        .unwrap();
        let purchase2 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(20.00), vendor2.vendor_id.clone()).unwrap()],
        )
        .unwrap();

        purchase_repo.add_purchase(purchase1);
        purchase_repo.add_purchase(purchase2);

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);
        let summary = service
            .generate_booth_summary(&booth.id, None)
            .await
            .unwrap();

        assert_eq!(summary.total_revenue, dec!(35.00)); // 15.00 + 20.00
        assert_eq!(summary.total_purchases, 2);
        assert_eq!(summary.unique_vendors, 2);
        assert_eq!(summary.vendor_summaries.len(), 2);
        assert_eq!(summary.participation_fee, dec!(5.00));
        assert_eq!(summary.sales_fee_percent, dec!(10.0));
        assert_eq!(summary.total_participation_fees, dec!(10.00));
        assert_eq!(summary.total_sales_fees, dec!(3.50));
        assert_eq!(summary.total_booth_revenue, dec!(13.50));

        // Check vendor1 summary
        let v1_summary = summary
            .vendor_summaries
            .iter()
            .find(|v| v.vendor_id == vendor1.vendor_id)
            .unwrap();
        assert_eq!(v1_summary.gross_sales, dec!(15.00));
        assert_eq!(v1_summary.item_count, 2); // 2 items in the purchase
                                              // Fees: 5.00 participation + 1.50 sales (10% of 15.00) = 6.50
        assert_eq!(v1_summary.fees_due, dec!(6.50));
        assert_eq!(v1_summary.net_payout, dec!(8.50)); // 15.00 - 6.50
    }

    #[tokio::test]
    async fn test_generate_vendor_report() {
        let booth = create_test_booth();
        let vendor = create_test_vendor(&booth.id, "1");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor.clone());

        // Add purchases for vendor
        let purchase1 = Purchase::new(
            booth.id.clone(),
            vec![
                PurchaseItem::new(dec!(10.00), vendor.vendor_id.clone()).unwrap(),
                PurchaseItem::new(dec!(5.00), vendor.vendor_id.clone()).unwrap(),
            ],
        )
        .unwrap();
        let purchase2 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(8.00), vendor.vendor_id.clone()).unwrap()],
        )
        .unwrap();

        purchase_repo.add_purchase(purchase1);
        purchase_repo.add_purchase(purchase2);

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);
        let report = service
            .generate_vendor_report(&booth.id, &vendor.vendor_id, None)
            .await
            .unwrap();

        assert_eq!(report.sales_sum, dec!(23.00)); // 10 + 5 + 8
        assert_eq!(report.participation_fee, dec!(5.00));
        assert_eq!(report.sales_fee, dec!(2.50)); // 10% of 23.00 = 2.30, rounded to nearest 0.50 = 2.50
        assert_eq!(report.total_revenue, dec!(15.50)); // 23.00 - 5.00 - 2.50
        assert_eq!(report.items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_active_vendors() {
        let booth = create_test_booth();
        let vendor1 = create_test_vendor(&booth.id, "1");
        let _vendor2 = create_test_vendor(&booth.id, "10");
        let vendor3 = create_test_vendor(&booth.id, "2");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());

        // Add purchases (vendor2 has no purchases)
        let purchase1 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(10.00), vendor1.vendor_id.clone()).unwrap()],
        )
        .unwrap();
        let purchase2 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(20.00), vendor3.vendor_id.clone()).unwrap()],
        )
        .unwrap();

        purchase_repo.add_purchase(purchase1);
        purchase_repo.add_purchase(purchase2);

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);
        let active_vendors = service.get_active_vendors(&booth.id, None).await.unwrap();

        assert_eq!(active_vendors.len(), 2);
        // Should be sorted with smart sorting: "1", "2" (numeric sorting)
        assert_eq!(active_vendors[0].as_str(), "1");
        assert_eq!(active_vendors[1].as_str(), "2");
    }

    #[tokio::test]
    async fn test_date_range_filtering() {
        let booth = create_test_booth();
        let vendor = create_test_vendor(&booth.id, "1");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor.clone());

        // Create purchases with different timestamps
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let two_hours_ago = now - chrono::Duration::hours(2);

        let mut purchase1 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(10.00), vendor.vendor_id.clone()).unwrap()],
        )
        .unwrap();
        purchase1.timestamp = two_hours_ago;

        let mut purchase2 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(20.00), vendor.vendor_id.clone()).unwrap()],
        )
        .unwrap();
        purchase2.timestamp = one_hour_ago;

        let mut purchase3 = Purchase::new(
            booth.id.clone(),
            vec![PurchaseItem::new(dec!(30.00), vendor.vendor_id.clone()).unwrap()],
        )
        .unwrap();
        purchase3.timestamp = now;

        purchase_repo.add_purchase(purchase1);
        purchase_repo.add_purchase(purchase2);
        purchase_repo.add_purchase(purchase3);

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);

        // Test filtering to last hour (should get 2 purchases: one_hour_ago and now)
        let date_range = DateRange::new(Some(one_hour_ago - chrono::Duration::minutes(1)), None);
        let report = service
            .generate_vendor_report(&booth.id, &vendor.vendor_id, Some(date_range))
            .await
            .unwrap();

        assert_eq!(report.sales_sum, dec!(50.00)); // 20 + 30
        assert_eq!(report.items.len(), 2);
    }

    #[tokio::test]
    async fn test_mixed_vendor_purchases() {
        // Test scenario: A purchase contains items from multiple vendors
        // Each vendor's report should only include their own items
        let booth = create_test_booth();
        let vendor1 = create_test_vendor(&booth.id, "V1");
        let vendor2 = create_test_vendor(&booth.id, "V2");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor1.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor2.clone());

        // Create a purchase with items from both vendors
        let mixed_purchase = Purchase::new(
            booth.id.clone(),
            vec![
                PurchaseItem::new(dec!(10.00), vendor1.vendor_id.clone()).unwrap(),
                PurchaseItem::new(dec!(20.00), vendor2.vendor_id.clone()).unwrap(),
                PurchaseItem::new(dec!(5.00), vendor1.vendor_id.clone()).unwrap(),
            ],
        )
        .unwrap();

        purchase_repo.add_purchase(mixed_purchase);

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);

        // Test vendor1's report - should only include their items (10.00 + 5.00)
        let report1 = service
            .generate_vendor_report(&booth.id, &vendor1.vendor_id, None)
            .await
            .unwrap();

        assert_eq!(report1.sales_sum, dec!(15.00)); // 10.00 + 5.00
        assert_eq!(report1.items.len(), 2, "Vendor 1 should have 2 items");
        assert!(
            report1
                .items
                .iter()
                .all(|item| item.item.vendor_id == vendor1.vendor_id),
            "All items in vendor 1 report should belong to vendor 1"
        );

        // Test vendor2's report - should only include their item (20.00)
        let report2 = service
            .generate_vendor_report(&booth.id, &vendor2.vendor_id, None)
            .await
            .unwrap();

        assert_eq!(report2.sales_sum, dec!(20.00));
        assert_eq!(report2.items.len(), 1, "Vendor 2 should have 1 item");
        assert_eq!(
            report2.items[0].item.vendor_id, vendor2.vendor_id,
            "The item should belong to vendor 2"
        );

        // Test booth summary - should correctly aggregate per vendor
        let booth_summary = service
            .generate_booth_summary(&booth.id, None)
            .await
            .unwrap();

        assert_eq!(booth_summary.unique_vendors, 2);
        assert_eq!(booth_summary.total_revenue, dec!(35.00)); // 10 + 20 + 5

        // Find vendor summaries
        let v1_summary = booth_summary
            .vendor_summaries
            .iter()
            .find(|v| v.vendor_id == vendor1.vendor_id)
            .unwrap();
        let v2_summary = booth_summary
            .vendor_summaries
            .iter()
            .find(|v| v.vendor_id == vendor2.vendor_id)
            .unwrap();

        assert_eq!(v1_summary.gross_sales, dec!(15.00), "Vendor 1 gross sales");
        assert_eq!(v1_summary.item_count, 2, "Vendor 1 item count");

        assert_eq!(v2_summary.gross_sales, dec!(20.00), "Vendor 2 gross sales");
        assert_eq!(v2_summary.item_count, 1, "Vendor 2 item count");
    }

    #[tokio::test]
    async fn test_booth_summary_fees_match_vendor_sum() {
        let booth = create_test_booth();
        let vendor1 = create_test_vendor(&booth.id, "1");
        let vendor2 = create_test_vendor(&booth.id, "2");
        let vendor3 = create_test_vendor(&booth.id, "3");

        let purchase_repo = MockPurchaseRepository::new();
        let booth_repo = MockBoothRepository::new();
        let vendor_repo = MockVendorRepository::new();

        booth_repo.add_booth(booth.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor1.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor2.clone());
        vendor_repo.add_vendor(booth.id.clone(), vendor3.clone());

        purchase_repo.add_purchase(
            Purchase::new(
                booth.id.clone(),
                vec![PurchaseItem::new(dec!(100.00), vendor1.vendor_id.clone()).unwrap()],
            )
            .unwrap(),
        );
        purchase_repo.add_purchase(
            Purchase::new(
                booth.id.clone(),
                vec![PurchaseItem::new(dec!(518.11), vendor2.vendor_id.clone()).unwrap()],
            )
            .unwrap(),
        );
        purchase_repo.add_purchase(
            Purchase::new(
                booth.id.clone(),
                vec![PurchaseItem::new(dec!(75.25), vendor3.vendor_id.clone()).unwrap()],
            )
            .unwrap(),
        );

        let service = ReportService::new(purchase_repo, booth_repo, vendor_repo);
        let summary = service
            .generate_booth_summary(&booth.id, None)
            .await
            .unwrap();

        let sum_vendor_fees: Decimal = summary.vendor_summaries.iter().map(|v| v.fees_due).sum();
        assert_eq!(sum_vendor_fees, summary.total_booth_revenue);
        assert_eq!(
            summary.total_booth_revenue,
            summary.total_participation_fees + summary.total_sales_fees
        );
    }
}
