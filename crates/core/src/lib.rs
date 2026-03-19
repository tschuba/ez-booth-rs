// Core domain library for ez-booth-rs
// Pure business logic with no external dependencies (except core utilities)

pub mod entities;
pub mod error;
pub mod services;
pub mod validation;

// Re-export commonly used types
pub use entities::{Booth, Purchase, PurchaseItem, Vendor};
pub use error::{CoreError, Result};
