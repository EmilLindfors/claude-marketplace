//! Type-safe URL identifier using the newtype pattern
//!
//! UrlId wraps a String to prevent mixing up identifiers with other string values.

use std::fmt;

/// Unique identifier for a URL
///
/// Uses the newtype pattern to ensure type safety - you cannot accidentally
/// pass a regular String where a UrlId is expected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UrlId(String);

impl UrlId {
    /// Create a new UrlId from a String
    ///
    /// # Examples
    ///
    /// ```
    /// use url_shortener::domain::UrlId;
    ///
    /// let id = UrlId::new("abc123".to_string());
    /// ```
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the inner String value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String value
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for UrlId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UrlId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_id_creation() {
        let id = UrlId::new("test123".to_string());
        assert_eq!(id.as_str(), "test123");
    }

    #[test]
    fn test_url_id_equality() {
        let id1 = UrlId::new("abc".to_string());
        let id2 = UrlId::new("abc".to_string());
        let id3 = UrlId::new("xyz".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_url_id_display() {
        let id = UrlId::new("display_test".to_string());
        assert_eq!(format!("{}", id), "display_test");
    }
}
