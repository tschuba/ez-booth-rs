mod backup_format;
mod error;
mod export_service;
mod import_service;
mod import_validator;
mod qr_binary_format;
mod qr_export;
mod qr_import;

pub use backup_format::{
    generate_booth_backup_filename, generate_full_backup_filename, sanitize_filename_component,
    BackupData, BoothBackupData, BACKUP_FILE_EXTENSION, BACKUP_FORMAT_VERSION,
};
pub use error::ExportError;
pub use error::{ImportError, SkippedRecord, ValidationFailure};
pub use export_service::{ExportService, SerializedBackup};
pub use import_service::{ConflictStrategy, ImportService, ImportSummary};
pub use import_validator::ImportValidator;
pub use qr_binary_format::{
    bytes_to_latin1_string, detect_payload_format, latin1_string_to_bytes, BinaryQrChunk,
    QrPayloadFormat, QR_BINARY_FORMAT_VERSION, QR_BINARY_MAGIC,
};
pub use qr_export::{
    create_chunks, estimate_qr_count, hash_bytes, render_qr_svg, serialize_and_compress_backup,
    serialize_chunk_payload, ExportScope, QrExport, QrExportService, RenderedQrChunk, MAX_QR_CODES,
    QR_CHUNK_SIZE, QR_FORMAT_VERSION, QR_WARNING_THRESHOLD,
};
pub use qr_import::{
    decompress_data, deserialize_backup, parse_chunk_payload, CollectorStatus, QrChunk,
    QrChunkCollector, QrImportService,
};
