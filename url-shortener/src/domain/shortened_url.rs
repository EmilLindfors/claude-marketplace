//! Domain aggregate representing a shortened URL
//!
//! This is the main aggregate root in our domain model.

use super::{OriginalUrl, ShortCode, UrlId};
use std::time::SystemTime;

/// A shortened URL aggregate
///
/// Combines all the information about a shortened URL into a single domain entity.
/// This is an aggregate root in DDD terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortenedUrl {
    id: UrlId,
    short_code: ShortCode,
    original_url: OriginalUrl,
    created_at: SystemTime,
    access_count: u64,
}

impl ShortenedUrl {
    /// Create a new ShortenedUrl
    ///
    /// # Examples
    ///
    /// ```
    /// use url_shortener::domain::{ShortenedUrl, UrlId, ShortCode, OriginalUrl};
    ///
    /// let id = UrlId::new("123".to_string());
    /// let code = ShortCode::new("abc123".to_string()).unwrap();
    /// let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
    ///
    /// let shortened = ShortenedUrl::new(id, code, url);
    /// assert_eq!(shortened.access_count(), 0);
    /// ```
    pub fn new(id: UrlId, short_code: ShortCode, original_url: OriginalUrl) -> Self {
        Self {
            id,
            short_code,
            original_url,
            created_at: SystemTime::now(),
            access_count: 0,
        }
    }

    /// Create a ShortenedUrl with a specific creation time (for testing)
    pub fn with_created_at(
        id: UrlId,
        short_code: ShortCode,
        original_url: OriginalUrl,
        created_at: SystemTime,
    ) -> Self {
        Self {
            id,
            short_code,
            original_url,
            created_at,
            access_count: 0,
        }
    }

    /// Get the unique identifier
    pub fn id(&self) -> &UrlId {
        &self.id
    }

    /// Get the short code
    pub fn short_code(&self) -> &ShortCode {
        &self.short_code
    }

    /// Get the original URL
    pub fn original_url(&self) -> &OriginalUrl {
        &self.original_url
    }

    /// Get the creation timestamp
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Get the access count
    pub fn access_count(&self) -> u64 {
        self.access_count
    }

    /// Record an access to this shortened URL
    ///
    /// This increments the access counter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use url_shortener::domain::{ShortenedUrl, UrlId, ShortCode, OriginalUrl};
    /// # let id = UrlId::new("123".to_string());
    /// # let code = ShortCode::new("abc123".to_string()).unwrap();
    /// # let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
    /// let mut shortened = ShortenedUrl::new(id, code, url);
    ///
    /// assert_eq!(shortened.access_count(), 0);
    /// shortened.record_access();
    /// assert_eq!(shortened.access_count(), 1);
    /// shortened.record_access();
    /// assert_eq!(shortened.access_count(), 2);
    /// ```
    pub fn record_access(&mut self) {
        self.access_count = self.access_count.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_url() -> ShortenedUrl {
        let id = UrlId::new("test-id".to_string());
        let code = ShortCode::new("testcode".to_string()).unwrap();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
        ShortenedUrl::new(id, code, url)
    }

    #[test]
    fn test_shortened_url_creation() {
        let url = create_test_url();
        assert_eq!(url.id().as_str(), "test-id");
        assert_eq!(url.short_code().as_str(), "testcode");
        assert_eq!(url.access_count(), 0);
    }

    #[test]
    fn test_record_access() {
        let mut url = create_test_url();

        assert_eq!(url.access_count(), 0);

        url.record_access();
        assert_eq!(url.access_count(), 1);

        url.record_access();
        assert_eq!(url.access_count(), 2);
    }

    #[test]
    fn test_access_count_saturation() {
        let id = UrlId::new("test".to_string());
        let code = ShortCode::new("test1234".to_string()).unwrap();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();

        let mut shortened = ShortenedUrl {
            id,
            short_code: code,
            original_url: url,
            created_at: SystemTime::now(),
            access_count: u64::MAX - 1,
        };

        shortened.record_access();
        assert_eq!(shortened.access_count(), u64::MAX);

        // Should saturate, not overflow
        shortened.record_access();
        assert_eq!(shortened.access_count(), u64::MAX);
    }
}
