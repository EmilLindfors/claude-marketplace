//! Repository port for URL persistence
//!
//! This trait defines the interface for storing and retrieving shortened URLs.
//! Different implementations can provide different storage backends (in-memory, database, etc.)

use crate::domain::{ShortCode, ShortenedUrl};
use crate::error::Result;

/// Port for URL persistence
///
/// This trait abstracts away the storage mechanism. Implementations can use
/// any storage backend (in-memory, SQL database, NoSQL, etc.)
///
/// # Hexagonal Architecture
///
/// This is a "port" in hexagonal architecture - an interface that defines
/// what the application needs from the outside world.
pub trait UrlRepository: Send + Sync {
    /// Save a shortened URL
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The short code already exists
    /// - The storage operation fails
    fn save(&self, url: ShortenedUrl) -> Result<()>;

    /// Find a shortened URL by its short code
    ///
    /// # Errors
    ///
    /// Returns `UrlShortenerError::ShortCodeNotFound` if the code doesn't exist
    fn find_by_short_code(&self, code: &ShortCode) -> Result<ShortenedUrl>;

    /// Update an existing shortened URL
    ///
    /// Typically used to update access counts
    ///
    /// # Errors
    ///
    /// Returns an error if the URL doesn't exist or the update fails
    fn update(&self, url: ShortenedUrl) -> Result<()>;

    /// Check if a short code exists
    fn exists(&self, code: &ShortCode) -> Result<bool>;

    /// Delete a shortened URL by its short code
    ///
    /// # Errors
    ///
    /// Returns an error if the code doesn't exist or the delete fails
    fn delete(&self, code: &ShortCode) -> Result<()>;

    /// Get all shortened URLs (useful for admin/testing)
    fn list_all(&self) -> Result<Vec<ShortenedUrl>>;
}
