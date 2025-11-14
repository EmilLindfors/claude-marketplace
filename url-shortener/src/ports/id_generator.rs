//! ID generator port
//!
//! Abstracts the generation of unique identifiers and short codes

use crate::domain::{ShortCode, UrlId};
use crate::error::Result;

/// Port for generating unique identifiers
///
/// This trait abstracts ID generation. Implementations can use:
/// - Random generation
/// - UUIDs
/// - Sequential IDs
/// - Hash-based generation
pub trait IdGenerator: Send + Sync {
    /// Generate a unique URL identifier
    fn generate_id(&self) -> UrlId;

    /// Generate a unique short code
    ///
    /// # Errors
    ///
    /// Returns an error if a unique code cannot be generated
    fn generate_short_code(&self) -> Result<ShortCode>;
}
