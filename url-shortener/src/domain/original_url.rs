//! Validated original URL type

use crate::error::{Result, UrlShortenerError};
use std::fmt;
use url::Url;

/// A validated original URL
///
/// OriginalUrl ensures that the URL is valid and well-formed.
/// Once created, it's guaranteed to be a valid URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalUrl(Url);

impl OriginalUrl {
    /// Create a new validated OriginalUrl
    ///
    /// # Errors
    ///
    /// Returns `UrlShortenerError::InvalidUrl` if the URL is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// use url_shortener::domain::OriginalUrl;
    ///
    /// let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
    /// assert_eq!(url.as_str(), "https://example.com/");
    ///
    /// // Invalid URL
    /// assert!(OriginalUrl::new("not a url".to_string()).is_err());
    /// ```
    pub fn new(url: String) -> Result<Self> {
        let parsed = Url::parse(&url)
            .map_err(|e| UrlShortenerError::InvalidUrl(e.to_string()))?;

        // Ensure we have a valid scheme (http or https)
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(UrlShortenerError::InvalidUrl(
                format!("Unsupported scheme: {}. Only http and https are allowed", parsed.scheme())
            ));
        }

        Ok(Self(parsed))
    }

    /// Get the URL as a string slice
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the domain of the URL
    pub fn domain(&self) -> Option<&str> {
        self.0.domain()
    }

    /// Get the scheme (http or https)
    pub fn scheme(&self) -> &str {
        self.0.scheme()
    }
}

impl fmt::Display for OriginalUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_http_url() {
        let url = OriginalUrl::new("http://example.com".to_string()).unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.domain(), Some("example.com"));
    }

    #[test]
    fn test_valid_https_url() {
        let url = OriginalUrl::new("https://example.com/path".to_string()).unwrap();
        assert_eq!(url.scheme(), "https");
        assert!(url.as_str().contains("/path"));
    }

    #[test]
    fn test_invalid_url_format() {
        let result = OriginalUrl::new("not a url".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_scheme() {
        let result = OriginalUrl::new("ftp://example.com".to_string());
        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::InvalidUrl(_))));
    }

    #[test]
    fn test_url_with_query_params() {
        let url = OriginalUrl::new("https://example.com/search?q=test".to_string()).unwrap();
        assert!(url.as_str().contains("q=test"));
    }

    #[test]
    fn test_url_with_fragment() {
        let url = OriginalUrl::new("https://example.com/page#section".to_string()).unwrap();
        assert!(url.as_str().contains("#section"));
    }
}
