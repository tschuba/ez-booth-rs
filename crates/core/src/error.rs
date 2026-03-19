use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CoreError {
    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid booth name: {0}")]
    InvalidBoothName(String),

    #[error("Invalid vendor ID: {0}")]
    InvalidVendorId(String),

    #[error("Invalid price: {0}")]
    InvalidPrice(String),

    #[error("Invalid commission rate: must be between 0 and 100, got {0}")]
    InvalidCommissionRate(f64),

    // Business logic errors
    #[error("Booth not found: {0}")]
    BoothNotFound(String),

    #[error("Vendor not found: {0}")]
    VendorNotFound(String),

    #[error("Purchase not found: {0}")]
    PurchaseNotFound(String),

    #[error("Booth already exists: {0}")]
    BoothAlreadyExists(String),

    #[error("Cannot delete booth with active purchases")]
    BoothHasPurchases,

    #[error("Cannot modify completed purchase")]
    PurchaseCompleted,

    #[error("Empty purchase: at least one item required")]
    EmptyPurchase,

    // Calculation errors
    #[error("Calculation error: {0}")]
    Calculation(String),

    // Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
}
