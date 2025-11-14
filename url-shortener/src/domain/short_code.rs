//! Validated short code type
//!
//! ShortCode is a validated type that guarantees the short code meets
//! the required format constraints.

use crate::error::{Result, UrlShortenerError};
use std::fmt;

/// A validated short code for URLs
///
/// Short codes must be:
/// - Between 4 and 12 characters long
/// - Contain only alphanumeric characters (a-z, A-Z, 0-9)
///
/// Once created, a ShortCode is guaranteed to be valid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortCode(String);

impl ShortCode {
    /// Minimum length for a short code
    pub const MIN_LENGTH: usize = 4;

    /// Maximum length for a short code
    pub const MAX_LENGTH: usize = 12;

    /// Create a new validated ShortCode
    ///
    /// # Errors
    ///
    /// Returns `UrlShortenerError::InvalidShortCode` if:
    /// - Length is outside the valid range
    /// - Contains non-alphanumeric characters
    ///
    /// # Examples
    ///
    /// ```
    /// use url_shortener::domain::ShortCode;
    ///
    /// let code = ShortCode::new("abc123".to_string()).unwrap();
    /// assert_eq!(code.as_str(), "abc123");
    ///
    /// // Too short
    /// assert!(ShortCode::new("abc".to_string()).is_err());
    ///
    /// // Invalid characters
    /// assert!(ShortCode::new("abc-123".to_string()).is_err());
    /// ```
    pub fn new(code: String) -> Result<Self> {
        Self::validate(&code)?;
        Ok(Self(code))
    }

    /// Validate a short code string
    fn validate(code: &str) -> Result<()> {
        if code.len() < Self::MIN_LENGTH {
            return Err(UrlShortenerError::InvalidShortCode(
                format!("Too short: must be at least {} characters", Self::MIN_LENGTH)
            ));
        }

        if code.len() > Self::MAX_LENGTH {
            return Err(UrlShortenerError::InvalidShortCode(
                format!("Too long: must be at most {} characters", Self::MAX_LENGTH)
            ));
        }

        if !code.chars().all(|c| c.is_alphanumeric()) {
            return Err(UrlShortenerError::InvalidShortCode(
                "Must contain only alphanumeric characters".to_string()
            ));
        }

        Ok(())
    }

    /// Get the short code as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for ShortCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_short_code() {
        let code = ShortCode::new("abc123".to_string()).unwrap();
        assert_eq!(code.as_str(), "abc123");
    }

    #[test]
    fn test_short_code_too_short() {
        let result = ShortCode::new("abc".to_string());
        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::InvalidShortCode(_))));
    }

    #[test]
    fn test_short_code_too_long() {
        let result = ShortCode::new("a".repeat(13));
        assert!(result.is_err());
    }

    #[test]
    fn test_short_code_invalid_characters() {
        let invalid_codes = vec![
            "abc-123",
            "abc_123",
            "abc 123",
            "abc@123",
            "abc#123",
        ];

        for code in invalid_codes {
            let result = ShortCode::new(code.to_string());
            assert!(result.is_err(), "Should reject: {}", code);
        }
    }

    #[test]
    fn test_short_code_valid_edge_cases() {
        // Min length
        assert!(ShortCode::new("abcd".to_string()).is_ok());

        // Max length
        assert!(ShortCode::new("a".repeat(12)).is_ok());

        // Mixed case
        assert!(ShortCode::new("AbC123".to_string()).is_ok());

        // All numbers
        assert!(ShortCode::new("123456".to_string()).is_ok());

        // All letters
        assert!(ShortCode::new("abcdef".to_string()).is_ok());
    }
}
