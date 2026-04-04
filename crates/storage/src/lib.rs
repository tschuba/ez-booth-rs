pub mod diagnostics;
pub mod error;
pub mod error_log;
pub mod export;
pub mod indexeddb;
pub mod repositories;

pub use diagnostics::{
    create_session_id, load_storage_diagnostics, record_backup_completed, run_integrity_check,
    IntegrityStatus, StorageDiagnostics,
};
pub use error::StorageError;
pub use error_log::{
    retention_cutoff, ErrorLogContext, ErrorLogDeviceInfo, ErrorLogEntry, ERROR_LOG_RETENTION_DAYS,
    ERROR_LOG_RETENTION_LIMIT,
};
pub use indexeddb::Database;
pub use repositories::{ErrorLogRepository, IndexedDbErrorLogRepository};
