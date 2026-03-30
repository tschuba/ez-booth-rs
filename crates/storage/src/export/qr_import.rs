use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use domain::{BoothRepository, PurchaseRepository, VendorRepository};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use super::backup_format::BoothBackupData;
use super::error::ImportError;
use super::import_service::{ConflictStrategy, ImportService, ImportSummary};
use super::import_validator::ImportValidator;
use super::qr_binary_format::{
    detect_payload_format, latin1_string_to_bytes, BinaryQrChunk, QrPayloadFormat,
};
use super::qr_export::{hash_bytes, QrBoothBackupData, QR_FORMAT_VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrChunk {
    pub v: u32,
    pub i: usize,
    pub t: usize,
    pub h: [u8; 32],
    pub d: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LegacyQrChunk {
    v: u32,
    i: usize,
    t: usize,
    h: String,
    d: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectorStatus {
    ChunkAdded,
    Duplicate,
    Complete,
}

#[derive(Debug, Clone, Default)]
pub struct QrChunkCollector {
    expected_total: Option<usize>,
    expected_hash: Option<[u8; 32]>,
    expected_version: Option<u32>,
    received_chunks: HashMap<usize, QrChunk>,
}

impl QrChunkCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_chunk(&mut self, chunk: QrChunk) -> Result<CollectorStatus, ImportError> {
        validate_chunk_metadata(&chunk)?;

        if let Some(expected_total) = self.expected_total {
            if chunk.t != expected_total {
                return Err(ImportError::InconsistentChunks(format!(
                    "expected {expected_total} total chunks, received {}",
                    chunk.t
                )));
            }
        } else {
            self.expected_total = Some(chunk.t);
        }

        if let Some(expected_version) = self.expected_version {
            if chunk.v != expected_version {
                return Err(ImportError::InconsistentChunks(format!(
                    "expected QR version {expected_version}, received {}",
                    chunk.v
                )));
            }
        } else {
            self.expected_version = Some(chunk.v);
        }

        if let Some(expected_hash) = &self.expected_hash {
            if chunk.h != *expected_hash {
                return Err(ImportError::InconsistentChunks(
                    "chunk hash does not match the current collection".to_string(),
                ));
            }
        } else {
            self.expected_hash = Some(chunk.h);
        }

        if self.received_chunks.contains_key(&chunk.i) {
            return Ok(CollectorStatus::Duplicate);
        }

        let index = chunk.i;
        self.received_chunks.insert(index, chunk);

        if self.is_complete() {
            Ok(CollectorStatus::Complete)
        } else {
            Ok(CollectorStatus::ChunkAdded)
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.expected_total, Some(total) if total > 0 && self.received_chunks.len() == total)
    }

    pub fn progress(&self) -> (usize, Option<usize>) {
        (self.received_chunks.len(), self.expected_total)
    }

    pub fn reassemble_bytes(&self) -> Result<Vec<u8>, ImportError> {
        let expected_total = self
            .expected_total
            .ok_or_else(|| ImportError::IncompleteChunks {
                received: self.received_chunks.len(),
                expected: 0,
            })?;

        if self.received_chunks.len() != expected_total {
            return Err(ImportError::IncompleteChunks {
                received: self.received_chunks.len(),
                expected: expected_total,
            });
        }

        let mut bytes = Vec::new();
        for index in 0..expected_total {
            let chunk =
                self.received_chunks
                    .get(&index)
                    .ok_or_else(|| ImportError::IncompleteChunks {
                        received: self.received_chunks.len(),
                        expected: expected_total,
                    })?;

            bytes.extend_from_slice(&chunk.d);
        }

        let expected_hash = self.expected_hash.unwrap_or_default();
        if hash_bytes(&bytes) != expected_hash {
            return Err(ImportError::HashMismatch);
        }

        Ok(bytes)
    }

    pub fn reassemble_backup(&self) -> Result<BoothBackupData, ImportError> {
        let compressed = self.reassemble_bytes()?;
        let decompressed = decompress_data(&compressed)?;
        deserialize_backup(&decompressed)
    }
}

#[derive(Clone)]
pub struct QrImportService {
    import_service: ImportService,
    validator: ImportValidator,
}

impl QrImportService {
    pub fn new(
        booth_repository: Arc<dyn BoothRepository>,
        vendor_repository: Arc<dyn VendorRepository>,
        purchase_repository: Arc<dyn PurchaseRepository>,
    ) -> Self {
        Self {
            import_service: ImportService::new(
                booth_repository,
                vendor_repository,
                purchase_repository,
            ),
            validator: ImportValidator::new(),
        }
    }

    pub fn with_dependencies(import_service: ImportService, validator: ImportValidator) -> Self {
        Self {
            import_service,
            validator,
        }
    }

    pub fn parse_chunk_payload(&self, raw: &str) -> Result<QrChunk, ImportError> {
        parse_chunk_payload(raw)
    }

    pub fn collect_backup(&self, chunks: Vec<QrChunk>) -> Result<BoothBackupData, ImportError> {
        let mut collector = QrChunkCollector::new();
        for chunk in chunks {
            collector.add_chunk(chunk)?;
        }

        let backup = collector.reassemble_backup()?;
        self.validator.validate_booth_backup_data(backup)
    }

    pub async fn import_chunks(
        &self,
        chunks: Vec<QrChunk>,
        strategy: ConflictStrategy,
    ) -> Result<ImportSummary, ImportError> {
        let backup = self.collect_backup(chunks)?;
        self.import_service
            .import_booth_backup(backup, strategy)
            .await
    }
}

pub fn parse_chunk_payload(raw: &str) -> Result<QrChunk, ImportError> {
    match detect_payload_format(raw)? {
        QrPayloadFormat::JsonV1 => parse_legacy_chunk_payload(raw),
        QrPayloadFormat::BinaryV2 => parse_binary_chunk_payload(raw),
    }
}

pub fn validate_chunk_metadata(chunk: &QrChunk) -> Result<(), ImportError> {
    if chunk.v != 1 && chunk.v != QR_FORMAT_VERSION {
        return Err(ImportError::InvalidQrPayload(format!(
            "unsupported QR format version {}",
            chunk.v
        )));
    }

    if chunk.t == 0 {
        return Err(ImportError::InvalidQrPayload(
            "chunk count must be greater than zero".to_string(),
        ));
    }

    if chunk.i >= chunk.t {
        return Err(ImportError::InvalidQrPayload(format!(
            "chunk index {} is out of bounds for total {}",
            chunk.i, chunk.t
        )));
    }

    if chunk.h == [0_u8; 32] {
        return Err(ImportError::InvalidQrPayload(
            "chunk hash must not be empty".to_string(),
        ));
    }

    if chunk.d.is_empty() {
        return Err(ImportError::InvalidQrPayload(
            "chunk data must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn parse_legacy_chunk_payload(raw: &str) -> Result<QrChunk, ImportError> {
    let chunk: LegacyQrChunk =
        serde_json::from_str(raw).map_err(|err| ImportError::InvalidQrPayload(err.to_string()))?;

    let hash_vec = decode_hex_hash(&chunk.h)?;
    let data = BASE64_STANDARD
        .decode(&chunk.d)
        .map_err(|err| ImportError::InvalidQrPayload(err.to_string()))?;

    let normalized = QrChunk {
        v: chunk.v,
        i: chunk.i,
        t: chunk.t,
        h: hash_vec,
        d: data,
    };
    validate_chunk_metadata(&normalized)?;
    Ok(normalized)
}

fn parse_binary_chunk_payload(raw: &str) -> Result<QrChunk, ImportError> {
    let bytes = latin1_string_to_bytes(raw)?;
    let chunk = BinaryQrChunk::decode_from_bytes(&bytes)?;
    let normalized = QrChunk {
        v: u32::from(chunk.version),
        i: usize::from(chunk.index),
        t: usize::from(chunk.total),
        h: chunk.hash,
        d: chunk.data,
    };
    validate_chunk_metadata(&normalized)?;
    Ok(normalized)
}

fn decode_hex_hash(value: &str) -> Result<[u8; 32], ImportError> {
    if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ImportError::InvalidQrPayload(
            "chunk hash must be a 64-character hex string".to_string(),
        ));
    }

    let mut hash = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = (chunk[0] as char)
            .to_digit(16)
            .ok_or_else(|| ImportError::InvalidQrPayload("invalid hash digit".to_string()))?;
        let low = (chunk[1] as char)
            .to_digit(16)
            .ok_or_else(|| ImportError::InvalidQrPayload("invalid hash digit".to_string()))?;
        hash[index] = ((high << 4) | low) as u8;
    }
    Ok(hash)
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, ImportError> {
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|err| ImportError::Decompression(err.to_string()))?;
    Ok(output)
}

pub fn deserialize_backup(data: &[u8]) -> Result<BoothBackupData, ImportError> {
    let qr_backup: QrBoothBackupData =
        rmp_serde::from_slice(data).map_err(|err| ImportError::Deserialization(err.to_string()))?;
    qr_backup
        .into_backup()
        .map_err(ImportError::Deserialization)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use domain::{Booth, FeeConfig, Purchase, PurchaseItem, Vendor, VendorId};
    use rust_decimal_macros::dec;

    use super::*;
    use crate::export::qr_export::{compress_data, create_chunks};

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
    fn collector_detects_duplicate_chunks() {
        let mut collector = QrChunkCollector::new();
        let chunk = create_chunks(b"hello world").unwrap().remove(0);

        assert_eq!(
            collector.add_chunk(chunk.clone()).unwrap(),
            CollectorStatus::Complete
        );
        assert_eq!(
            collector.add_chunk(chunk).unwrap(),
            CollectorStatus::Duplicate
        );
    }

    #[test]
    fn collector_reassembles_and_verifies_hash() {
        let backup = sample_backup();
        let encoded = rmp_serde::to_vec_named(&QrBoothBackupData::from_backup(&backup)).unwrap();
        let compressed = compress_data(&encoded).unwrap();
        let chunks = create_chunks(&compressed).unwrap();
        let mut collector = QrChunkCollector::new();

        for chunk in chunks {
            collector.add_chunk(chunk).unwrap();
        }

        let decoded = collector.reassemble_backup().unwrap();
        assert_eq!(decoded, backup);
    }

    #[test]
    fn collector_accepts_out_of_order_chunks() {
        let backup = sample_backup();
        let encoded = rmp_serde::to_vec_named(&QrBoothBackupData::from_backup(&backup)).unwrap();
        let compressed = compress_data(&encoded).unwrap();
        let mut chunks = create_chunks(&compressed).unwrap();
        chunks.reverse();
        let mut collector = QrChunkCollector::new();

        for chunk in chunks {
            collector.add_chunk(chunk).unwrap();
        }

        let decoded = collector.reassemble_backup().unwrap();
        assert_eq!(decoded, backup);
    }

    #[test]
    fn collector_rejects_hash_mismatch() {
        let mut collector = QrChunkCollector::new();
        let mut chunks = create_chunks(b"hello world").unwrap();
        chunks[0].h = [0_u8; 32];

        let error = collector.add_chunk(chunks.remove(0)).unwrap_err();
        assert!(matches!(error, ImportError::InvalidQrPayload(_)));
    }

    #[test]
    fn collector_rejects_reassembled_hash_mismatch() {
        let mut collector = QrChunkCollector::new();
        let mut chunks = create_chunks(b"hello world").unwrap();
        chunks[0].h = [9_u8; 32];

        collector.add_chunk(chunks.remove(0)).unwrap();
        let error = collector.reassemble_bytes().unwrap_err();
        assert!(matches!(error, ImportError::HashMismatch));
    }

    #[test]
    fn parse_chunk_rejects_invalid_metadata() {
        let error =
            parse_chunk_payload(r#"{"v":99,"i":0,"t":1,"h":"abc","d":"abcd"}"#).unwrap_err();
        assert!(matches!(error, ImportError::InvalidQrPayload(_)));
    }

    #[test]
    fn collector_rejects_inconsistent_total_chunks() {
        let mut collector = QrChunkCollector::new();
        let first = QrChunk {
            v: QR_FORMAT_VERSION,
            i: 0,
            t: 2,
            h: [10_u8; 32],
            d: b"hello".to_vec(),
        };
        let second = QrChunk {
            t: 3,
            i: 1,
            ..first.clone()
        };

        assert_eq!(collector.add_chunk(first).unwrap(), CollectorStatus::ChunkAdded);
        let error = collector.add_chunk(second).unwrap_err();
        assert!(matches!(error, ImportError::InconsistentChunks(_)));
    }

    #[test]
    fn collector_rejects_invalid_base64_data() {
        let error = parse_chunk_payload(r#"{"v":1,"i":0,"t":1,"h":"001122","d":"%%%not-base64%%%"}"#)
            .unwrap_err();
        assert!(matches!(error, ImportError::InvalidQrPayload(_)));
    }

    #[test]
    fn collector_reports_decompression_error_for_non_gzip_payload() {
        let raw_bytes = b"not a gzip stream";
        let mut collector = QrChunkCollector::new();
        collector
            .add_chunk(QrChunk {
                v: QR_FORMAT_VERSION,
                i: 0,
                t: 1,
                h: hash_bytes(raw_bytes),
                d: raw_bytes.to_vec(),
            })
            .unwrap();

        let error = collector.reassemble_backup().unwrap_err();
        assert!(matches!(error, ImportError::Decompression(_)));
    }

    #[test]
    fn collector_reports_deserialization_error_for_invalid_messagepack() {
        let invalid_messagepack = compress_data(b"not messagepack").unwrap();
        let mut collector = QrChunkCollector::new();
        collector
            .add_chunk(QrChunk {
                v: QR_FORMAT_VERSION,
                i: 0,
                t: 1,
                h: hash_bytes(&invalid_messagepack),
                d: invalid_messagepack,
            })
            .unwrap();

        let error = collector.reassemble_backup().unwrap_err();
        assert!(matches!(error, ImportError::Deserialization(_)));
    }

    #[test]
    fn decompress_roundtrip() {
        let data = vec![42_u8; 4096];
        let compressed = compress_data(&data).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn parses_binary_chunk_payload() {
        let chunk = create_chunks(b"hello binary").unwrap().remove(0);
        let payload = crate::export::qr_export::serialize_chunk_payload(&chunk).unwrap();
        let parsed = parse_chunk_payload(&payload).unwrap();
        assert_eq!(parsed, chunk);
        assert_eq!(parsed.v, QR_FORMAT_VERSION);
    }

    #[test]
    fn parses_legacy_json_chunk_payload() {
        let payload = r#"{"v":1,"i":0,"t":1,"h":"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824","d":"aGVsbG8="}"#;
        let parsed = parse_chunk_payload(payload).unwrap();
        assert_eq!(parsed.v, 1);
        assert_eq!(parsed.i, 0);
        assert_eq!(parsed.t, 1);
        assert_eq!(parsed.d, b"hello");
    }

    #[test]
    fn collector_rejects_mixed_qr_versions() {
        let mut collector = QrChunkCollector::new();
        let binary = create_chunks(b"hello").unwrap().remove(0);
        collector.add_chunk(binary).unwrap();

        let legacy = parse_chunk_payload(
            r#"{"v":1,"i":0,"t":1,"h":"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824","d":"aGVsbG8="}"#,
        )
        .unwrap();

        let error = collector.add_chunk(legacy).unwrap_err();
        assert!(matches!(error, ImportError::InconsistentChunks(_)));
    }
}
