use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use domain::{
    Booth, BoothId, BoothRepository, CheckoutKeyboardConfig, FeeConfig, ItemId, Purchase,
    PurchaseId, PurchaseItem, PurchaseRepository, Vendor, VendorId, VendorIdOmissionRules,
    VendorIdValidation, VendorRepository,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::backup_format::BoothBackupData;
use super::error::ExportError;
use super::export_service::ExportService;
use super::qr_import::QrChunk;

pub const QR_CHUNK_SIZE: usize = 1_800;
pub const QR_WARNING_THRESHOLD: usize = 5;
pub const MAX_QR_CODES: usize = 10;
pub const QR_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrBoothBackupData {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub app_version: String,
    pub booth: QrBooth,
    pub vendors: Vec<QrVendor>,
    pub purchases: Vec<QrPurchase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrBooth {
    pub id: BoothId,
    pub description: String,
    pub date: NaiveDate,
    pub fees: QrFeeConfig,
    pub vendor_id_validation: VendorIdValidation,
    pub vendor_id_omission_rules: VendorIdOmissionRules,
    pub keyboard_config: QrCheckoutKeyboardConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrFeeConfig {
    pub participation_fee: String,
    pub sales_fee_percent: String,
    pub rounding_step: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrCheckoutKeyboardConfig {
    pub quick_amounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrVendor {
    pub vendor_id: VendorId,
    pub booth_id: BoothId,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrPurchase {
    pub id: PurchaseId,
    pub booth_id: BoothId,
    pub items: Vec<QrPurchaseItem>,
    pub timestamp: DateTime<Utc>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QrPurchaseItem {
    pub id: ItemId,
    pub amount: String,
    pub vendor_id: VendorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportScope {
    Today,
    Week,
    Month,
    Full,
}

impl ExportScope {
    pub fn days(self) -> Option<i64> {
        match self {
            Self::Today => Some(1),
            Self::Week => Some(7),
            Self::Month => Some(30),
            Self::Full => None,
        }
    }

    pub fn filter_purchases(self, purchases: &[Purchase]) -> Vec<Purchase> {
        let Some(days) = self.days() else {
            return purchases.to_vec();
        };

        let cutoff = Utc::now() - Duration::days(days);
        purchases
            .iter()
            .filter(|purchase| purchase.timestamp >= cutoff)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QrExport {
    pub backup: BoothBackupData,
    pub compressed_bytes: Vec<u8>,
    pub chunks: Vec<QrChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedQrChunk {
    pub chunk: QrChunk,
    pub svg: String,
}

#[derive(Clone)]
pub struct QrExportService {
    export_service: ExportService,
}

impl QrExportService {
    pub fn new(
        booth_repository: Arc<dyn BoothRepository>,
        vendor_repository: Arc<dyn VendorRepository>,
        purchase_repository: Arc<dyn PurchaseRepository>,
    ) -> Self {
        Self {
            export_service: ExportService::new(
                booth_repository,
                vendor_repository,
                purchase_repository,
            ),
        }
    }

    pub fn with_export_service(export_service: ExportService) -> Self {
        Self { export_service }
    }

    pub async fn export_booth_as_qr(
        &self,
        booth_id: &BoothId,
        scope: ExportScope,
    ) -> Result<QrExport, ExportError> {
        let mut backup = self.export_service.export_booth(booth_id).await?;
        backup.purchases = scope.filter_purchases(&backup.purchases);

        let compressed_bytes = serialize_and_compress_backup(&backup)?;
        let chunks = create_chunks(&compressed_bytes)?;

        Ok(QrExport {
            backup,
            compressed_bytes,
            chunks,
        })
    }

    pub fn render_svg_chunks(
        &self,
        chunks: &[QrChunk],
    ) -> Result<Vec<RenderedQrChunk>, ExportError> {
        chunks
            .iter()
            .cloned()
            .map(|chunk| {
                let payload = serialize_chunk_payload(&chunk)?;
                let svg = render_qr_svg(&payload)?;
                Ok(RenderedQrChunk { chunk, svg })
            })
            .collect()
    }
}

pub fn serialize_and_compress_backup(backup: &BoothBackupData) -> Result<Vec<u8>, ExportError> {
    let qr_backup = QrBoothBackupData::from_backup(backup);
    let bytes = rmp_serde::to_vec_named(&qr_backup)
        .map_err(|err| ExportError::Serialization(err.to_string()))?;
    compress_data(&bytes)
}

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, ExportError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|err| ExportError::Compression(err.to_string()))?;
    encoder
        .finish()
        .map_err(|err| ExportError::Compression(err.to_string()))
}

pub fn create_chunks(data: &[u8]) -> Result<Vec<QrChunk>, ExportError> {
    let total_chunks = data.len().max(1).div_ceil(QR_CHUNK_SIZE);
    if total_chunks > MAX_QR_CODES {
        return Err(ExportError::TooManyQrCodes {
            required: total_chunks,
            maximum: MAX_QR_CODES,
        });
    }

    let hash = hash_bytes(data);
    let mut chunks = Vec::with_capacity(total_chunks);

    for index in 0..total_chunks {
        let start = index * QR_CHUNK_SIZE;
        let end = (start + QR_CHUNK_SIZE).min(data.len());
        let chunk_data = &data[start..end];
        chunks.push(QrChunk {
            v: QR_FORMAT_VERSION,
            i: index,
            t: total_chunks,
            h: hash.clone(),
            d: BASE64_STANDARD.encode(chunk_data),
        });
    }

    Ok(chunks)
}

pub fn serialize_chunk_payload(chunk: &QrChunk) -> Result<String, ExportError> {
    serde_json::to_string(chunk).map_err(|err| ExportError::Serialization(err.to_string()))
}

pub fn render_qr_svg(payload: &str) -> Result<String, ExportError> {
    let code = QrCode::with_error_correction_level(payload.as_bytes(), EcLevel::M)
        .map_err(|err| ExportError::QrGeneration(err.to_string()))?;

    Ok(code
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#111111"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

pub fn estimate_qr_count(vendor_count: usize, purchase_count: usize, scope: ExportScope) -> usize {
    let filtered_purchase_count = match scope {
        ExportScope::Today => purchase_count.saturating_mul(3).div_ceil(100),
        ExportScope::Week => purchase_count.saturating_mul(21).div_ceil(100),
        ExportScope::Month => purchase_count.saturating_mul(9).div_ceil(10),
        ExportScope::Full => purchase_count,
    };

    let total_binary_size = 400 + (vendor_count * 80) + (filtered_purchase_count * 200);
    let compressed_estimate = total_binary_size.saturating_mul(30).div_ceil(100);
    compressed_estimate.max(1).div_ceil(QR_CHUNK_SIZE).max(1)
}

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn parse_decimal(value: &str, field: &str) -> Result<Decimal, String> {
    Decimal::from_str(value).map_err(|err| format!("invalid decimal for {field}: {err}"))
}

fn decimal_to_string(decimal: Decimal) -> String {
    decimal.normalize().to_string()
}

impl QrBoothBackupData {
    pub(crate) fn from_backup(backup: &BoothBackupData) -> Self {
        Self {
            version: backup.version,
            created_at: backup.created_at,
            app_version: backup.app_version.clone(),
            booth: QrBooth::from_booth(&backup.booth),
            vendors: backup.vendors.iter().map(QrVendor::from_vendor).collect(),
            purchases: backup
                .purchases
                .iter()
                .map(QrPurchase::from_purchase)
                .collect(),
        }
    }

    pub(crate) fn into_backup(self) -> Result<BoothBackupData, String> {
        Ok(BoothBackupData {
            version: self.version,
            created_at: self.created_at,
            app_version: self.app_version,
            booth: self.booth.into_booth()?,
            vendors: self
                .vendors
                .into_iter()
                .map(QrVendor::into_vendor)
                .collect::<Result<Vec<_>, _>>()?,
            purchases: self
                .purchases
                .into_iter()
                .map(QrPurchase::into_purchase)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl QrBooth {
    fn from_booth(booth: &Booth) -> Self {
        Self {
            id: booth.id,
            description: booth.description.clone(),
            date: booth.date,
            fees: QrFeeConfig::from_fee_config(&booth.fees),
            vendor_id_validation: booth.vendor_id_validation.clone(),
            vendor_id_omission_rules: booth.vendor_id_omission_rules.clone(),
            keyboard_config: QrCheckoutKeyboardConfig::from_keyboard_config(&booth.keyboard_config),
            created_at: booth.created_at,
            updated_at: booth.updated_at,
        }
    }

    fn into_booth(self) -> Result<Booth, String> {
        Ok(Booth {
            id: self.id,
            description: self.description,
            date: self.date,
            fees: self.fees.into_fee_config()?,
            vendor_id_validation: self.vendor_id_validation,
            vendor_id_omission_rules: self.vendor_id_omission_rules,
            keyboard_config: self.keyboard_config.into_keyboard_config()?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl QrFeeConfig {
    fn from_fee_config(fees: &FeeConfig) -> Self {
        Self {
            participation_fee: decimal_to_string(fees.participation_fee),
            sales_fee_percent: decimal_to_string(fees.sales_fee_percent),
            rounding_step: decimal_to_string(fees.rounding_step),
        }
    }

    fn into_fee_config(self) -> Result<FeeConfig, String> {
        Ok(FeeConfig {
            participation_fee: parse_decimal(&self.participation_fee, "participation_fee")?,
            sales_fee_percent: parse_decimal(&self.sales_fee_percent, "sales_fee_percent")?,
            rounding_step: parse_decimal(&self.rounding_step, "rounding_step")?,
        })
    }
}

impl QrCheckoutKeyboardConfig {
    fn from_keyboard_config(config: &CheckoutKeyboardConfig) -> Self {
        Self {
            quick_amounts: config
                .quick_amounts
                .iter()
                .copied()
                .map(decimal_to_string)
                .collect(),
        }
    }

    fn into_keyboard_config(self) -> Result<CheckoutKeyboardConfig, String> {
        Ok(CheckoutKeyboardConfig {
            quick_amounts: self
                .quick_amounts
                .into_iter()
                .map(|value| parse_decimal(&value, "keyboard_config.quick_amounts"))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl QrVendor {
    fn from_vendor(vendor: &Vendor) -> Self {
        Self {
            vendor_id: vendor.vendor_id.clone(),
            booth_id: vendor.booth_id,
            name: vendor.name.clone(),
            created_at: vendor.created_at,
        }
    }

    fn into_vendor(self) -> Result<Vendor, String> {
        Ok(Vendor {
            vendor_id: self.vendor_id,
            booth_id: self.booth_id,
            name: self.name,
            created_at: self.created_at,
        })
    }
}

impl QrPurchase {
    fn from_purchase(purchase: &Purchase) -> Self {
        Self {
            id: purchase.id,
            booth_id: purchase.booth_id,
            items: purchase
                .items
                .iter()
                .map(QrPurchaseItem::from_purchase_item)
                .collect(),
            timestamp: purchase.timestamp,
            note: purchase.note.clone(),
        }
    }

    fn into_purchase(self) -> Result<Purchase, String> {
        Ok(Purchase {
            id: self.id,
            booth_id: self.booth_id,
            items: self
                .items
                .into_iter()
                .map(QrPurchaseItem::into_purchase_item)
                .collect::<Result<Vec<_>, _>>()?,
            timestamp: self.timestamp,
            note: self.note,
        })
    }
}

impl QrPurchaseItem {
    fn from_purchase_item(item: &PurchaseItem) -> Self {
        Self {
            id: item.id,
            amount: decimal_to_string(item.amount),
            vendor_id: item.vendor_id.clone(),
        }
    }

    fn into_purchase_item(self) -> Result<PurchaseItem, String> {
        Ok(PurchaseItem {
            id: self.id,
            amount: parse_decimal(&self.amount, "purchase_item.amount")?,
            vendor_id: self.vendor_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use super::*;

    fn sample_backup() -> BoothBackupData {
        let booth = Booth::new(
            "Spring Market 2026".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
            FeeConfig {
                participation_fee: dec!(10.00),
                sales_fee_percent: dec!(15.00),
                rounding_step: dec!(0.50),
            },
        )
        .unwrap();
        let vendor = Vendor::new(VendorId::new("12".to_string()), booth.id)
            .with_name("Ada Vendor".to_string());
        let purchase = Purchase::new(
            booth.id,
            vec![PurchaseItem::new(dec!(42.00), vendor.vendor_id.clone()).unwrap()],
        )
        .unwrap();

        let mut backup = BoothBackupData::new(booth, "test-version");
        backup.vendors = vec![vendor];
        backup.purchases = vec![purchase];
        backup
    }

    #[test]
    fn compresses_backup_data() {
        let backup = sample_backup();
        let bytes = serialize_and_compress_backup(&backup).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn creates_chunk_metadata() {
        let data = vec![7_u8; QR_CHUNK_SIZE * 2 + 7];
        let chunks = create_chunks(&data).unwrap();

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].i, 0);
        assert_eq!(chunks[1].i, 1);
        assert_eq!(chunks[2].i, 2);
        assert!(chunks.iter().all(|chunk| chunk.t == 3));
        assert!(chunks.iter().all(|chunk| chunk.v == QR_FORMAT_VERSION));
        assert!(chunks.iter().all(|chunk| chunk.h == chunks[0].h));
    }

    #[test]
    fn rejects_exports_exceeding_chunk_limit() {
        let data = vec![0_u8; QR_CHUNK_SIZE * (MAX_QR_CODES + 1)];
        let error = create_chunks(&data).unwrap_err();

        match error {
            ExportError::TooManyQrCodes { required, maximum } => {
                assert_eq!(required, MAX_QR_CODES + 1);
                assert_eq!(maximum, MAX_QR_CODES);
            }
            other => panic!("Expected TooManyQrCodes, got {other:?}"),
        }
    }

    #[test]
    fn renders_svg_qr_code() {
        let chunk = create_chunks(b"hello qr").unwrap().remove(0);
        let payload = serialize_chunk_payload(&chunk).unwrap();
        let svg = render_qr_svg(&payload).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("path"));
    }

    #[test]
    fn estimates_qr_count_with_scope() {
        assert_eq!(estimate_qr_count(20, 100, ExportScope::Today), 1);
        assert!(estimate_qr_count(100, 2_000, ExportScope::Full) > MAX_QR_CODES);
    }

    #[test]
    fn qr_transfer_format_roundtrips_backup_data() {
        let backup = sample_backup();
        let transfer = QrBoothBackupData::from_backup(&backup);
        let restored = transfer.into_backup().unwrap();
        assert_eq!(restored, backup);
    }
}
