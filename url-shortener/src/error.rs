//! Error types for the URL shortener service
//!
//! Uses thiserror for ergonomic error handling with proper error chaining.

use thiserror::Error;

/// Domain errors that can occur in the URL shortener service
#[derive(Error, Debug, PartialEq)]
pub enum UrlShortenerError {
    /// The provided URL is invalid or malformed
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    /// The short code is invalid (wrong format, length, or characters)
    #[error("Invalid short code: {0}")]
    InvalidShortCode(String),

    /// The short code is already in use
    #[error("Short code '{0}' is already in use")]
    ShortCodeAlreadyExists(String),

    /// The requested short code was not found
    #[error("Short code '{0}' not found")]
    ShortCodeNotFound(String),

    /// Repository operation failed
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// ID generation failed
    #[error("Failed to generate unique ID after {0} attempts")]
    IdGenerationFailed(usize),
}

/// Result type alias for URL shortener operations
pub type Result<T> = std::result::Result<T, UrlShortenerError>;
