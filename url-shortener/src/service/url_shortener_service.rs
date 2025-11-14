//! URL shortener application service
//!
//! This service orchestrates the URL shortening logic using the domain model
//! and ports for external dependencies.

use crate::domain::{OriginalUrl, ShortCode, ShortenedUrl};
use crate::error::{Result, UrlShortenerError};
use crate::ports::{IdGenerator, UrlRepository};
use std::sync::Arc;

/// Application service for URL shortening operations
///
/// This service provides the main API for the URL shortener application.
/// It depends on ports (traits) rather than concrete implementations,
/// following the dependency inversion principle.
///
/// # Examples
///
/// ```
/// use url_shortener::service::UrlShortenerService;
/// use url_shortener::adapters::{InMemoryUrlRepository, RandomIdGenerator};
/// use url_shortener::domain::OriginalUrl;
/// use std::sync::Arc;
///
/// let repository = Arc::new(InMemoryUrlRepository::new());
/// let id_generator = Arc::new(RandomIdGenerator::new());
/// let service = UrlShortenerService::new(repository, id_generator);
///
/// let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
/// let shortened = service.shorten_url(url).unwrap();
/// ```
pub struct UrlShortenerService<R, G>
where
    R: UrlRepository,
    G: IdGenerator,
{
    repository: Arc<R>,
    id_generator: Arc<G>,
}

impl<R, G> UrlShortenerService<R, G>
where
    R: UrlRepository,
    G: IdGenerator,
{
    /// Maximum attempts to generate a unique short code
    const MAX_GENERATION_ATTEMPTS: usize = 10;

    /// Create a new URL shortener service
    ///
    /// # Arguments
    ///
    /// * `repository` - Implementation of the UrlRepository port
    /// * `id_generator` - Implementation of the IdGenerator port
    pub fn new(repository: Arc<R>, id_generator: Arc<G>) -> Self {
        Self {
            repository,
            id_generator,
        }
    }

    /// Shorten a URL with an auto-generated short code
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - A unique short code cannot be generated
    /// - The repository operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use url_shortener::service::UrlShortenerService;
    /// # use url_shortener::adapters::{InMemoryUrlRepository, RandomIdGenerator};
    /// # use url_shortener::domain::OriginalUrl;
    /// # use std::sync::Arc;
    /// # let repository = Arc::new(InMemoryUrlRepository::new());
    /// # let id_generator = Arc::new(RandomIdGenerator::new());
    /// # let service = UrlShortenerService::new(repository, id_generator);
    /// let url = OriginalUrl::new("https://example.com/long/path".to_string()).unwrap();
    /// let shortened = service.shorten_url(url).unwrap();
    /// println!("Short code: {}", shortened.short_code());
    /// ```
    pub fn shorten_url(&self, original_url: OriginalUrl) -> Result<ShortenedUrl> {
        // Try to generate a unique short code
        let short_code = self.generate_unique_short_code()?;

        // Create the domain entity
        let id = self.id_generator.generate_id();
        let shortened_url = ShortenedUrl::new(id, short_code, original_url);

        // Persist it
        self.repository.save(shortened_url.clone())?;

        Ok(shortened_url)
    }

    /// Shorten a URL with a custom short code
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The short code is invalid
    /// - The short code is already in use
    /// - The repository operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use url_shortener::service::UrlShortenerService;
    /// # use url_shortener::adapters::{InMemoryUrlRepository, RandomIdGenerator};
    /// # use url_shortener::domain::{OriginalUrl, ShortCode};
    /// # use std::sync::Arc;
    /// # let repository = Arc::new(InMemoryUrlRepository::new());
    /// # let id_generator = Arc::new(RandomIdGenerator::new());
    /// # let service = UrlShortenerService::new(repository, id_generator);
    /// let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
    /// let code = ShortCode::new("custom".to_string()).unwrap();
    /// let shortened = service.shorten_url_with_code(url, code).unwrap();
    /// assert_eq!(shortened.short_code().as_str(), "custom");
    /// ```
    pub fn shorten_url_with_code(
        &self,
        original_url: OriginalUrl,
        short_code: ShortCode,
    ) -> Result<ShortenedUrl> {
        // Check if code already exists
        if self.repository.exists(&short_code)? {
            return Err(UrlShortenerError::ShortCodeAlreadyExists(
                short_code.as_str().to_string()
            ));
        }

        // Create the domain entity
        let id = self.id_generator.generate_id();
        let shortened_url = ShortenedUrl::new(id, short_code, original_url);

        // Persist it
        self.repository.save(shortened_url.clone())?;

        Ok(shortened_url)
    }

    /// Resolve a short code to its original URL
    ///
    /// This operation also records the access in the access counter.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The short code doesn't exist
    /// - The repository operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use url_shortener::service::UrlShortenerService;
    /// # use url_shortener::adapters::{InMemoryUrlRepository, RandomIdGenerator};
    /// # use url_shortener::domain::OriginalUrl;
    /// # use std::sync::Arc;
    /// # let repository = Arc::new(InMemoryUrlRepository::new());
    /// # let id_generator = Arc::new(RandomIdGenerator::new());
    /// # let service = UrlShortenerService::new(repository, id_generator);
    /// # let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
    /// # let shortened = service.shorten_url(url).unwrap();
    /// let original = service.resolve_short_code(shortened.short_code()).unwrap();
    /// assert_eq!(original.as_str(), "https://example.com/");
    /// ```
    pub fn resolve_short_code(&self, short_code: &ShortCode) -> Result<OriginalUrl> {
        // Find the shortened URL
        let mut shortened_url = self.repository.find_by_short_code(short_code)?;

        // Record the access
        shortened_url.record_access();

        // Update in repository
        self.repository.update(shortened_url.clone())?;

        Ok(shortened_url.original_url().clone())
    }

    /// Get statistics for a short code
    ///
    /// Returns the ShortenedUrl entity which includes access count and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the short code doesn't exist
    pub fn get_statistics(&self, short_code: &ShortCode) -> Result<ShortenedUrl> {
        self.repository.find_by_short_code(short_code)
    }

    /// Delete a shortened URL
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The short code doesn't exist
    /// - The repository operation fails
    pub fn delete_short_code(&self, short_code: &ShortCode) -> Result<()> {
        self.repository.delete(short_code)
    }

    /// List all shortened URLs
    ///
    /// Useful for admin interfaces or testing
    pub fn list_all(&self) -> Result<Vec<ShortenedUrl>> {
        self.repository.list_all()
    }

    /// Generate a unique short code
    ///
    /// Attempts multiple times to avoid collisions
    fn generate_unique_short_code(&self) -> Result<ShortCode> {
        for attempt in 0..Self::MAX_GENERATION_ATTEMPTS {
            let code = self.id_generator.generate_short_code()?;

            if !self.repository.exists(&code)? {
                return Ok(code);
            }

            if attempt == Self::MAX_GENERATION_ATTEMPTS - 1 {
                return Err(UrlShortenerError::IdGenerationFailed(
                    Self::MAX_GENERATION_ATTEMPTS
                ));
            }
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{InMemoryUrlRepository, RandomIdGenerator};

    fn create_service() -> UrlShortenerService<InMemoryUrlRepository, RandomIdGenerator> {
        let repository = Arc::new(InMemoryUrlRepository::new());
        let id_generator = Arc::new(RandomIdGenerator::new());
        UrlShortenerService::new(repository, id_generator)
    }

    #[test]
    fn test_shorten_url() {
        let service = create_service();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();

        let result = service.shorten_url(url);
        assert!(result.is_ok());

        let shortened = result.unwrap();
        assert_eq!(shortened.original_url().as_str(), "https://example.com/");
    }

    #[test]
    fn test_shorten_url_with_custom_code() {
        let service = create_service();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
        let code = ShortCode::new("custom".to_string()).unwrap();

        let result = service.shorten_url_with_code(url, code);
        assert!(result.is_ok());

        let shortened = result.unwrap();
        assert_eq!(shortened.short_code().as_str(), "custom");
    }

    #[test]
    fn test_custom_code_already_exists() {
        let service = create_service();
        let url1 = OriginalUrl::new("https://example1.com".to_string()).unwrap();
        let url2 = OriginalUrl::new("https://example2.com".to_string()).unwrap();
        let code = ShortCode::new("dupe12".to_string()).unwrap();

        service.shorten_url_with_code(url1, code.clone()).unwrap();

        let result = service.shorten_url_with_code(url2, code);
        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::ShortCodeAlreadyExists(_))));
    }

    #[test]
    fn test_resolve_short_code() {
        let service = create_service();
        let url = OriginalUrl::new("https://example.com/test".to_string()).unwrap();
        let shortened = service.shorten_url(url).unwrap();

        let resolved = service.resolve_short_code(shortened.short_code()).unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/test");
    }

    #[test]
    fn test_resolve_increments_access_count() {
        let service = create_service();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
        let shortened = service.shorten_url(url).unwrap();
        let code = shortened.short_code().clone();

        // Initial access count should be 0
        let stats = service.get_statistics(&code).unwrap();
        assert_eq!(stats.access_count(), 0);

        // Resolve once
        service.resolve_short_code(&code).unwrap();

        // Access count should be 1
        let stats = service.get_statistics(&code).unwrap();
        assert_eq!(stats.access_count(), 1);

        // Resolve again
        service.resolve_short_code(&code).unwrap();

        // Access count should be 2
        let stats = service.get_statistics(&code).unwrap();
        assert_eq!(stats.access_count(), 2);
    }

    #[test]
    fn test_resolve_nonexistent_code() {
        let service = create_service();
        let code = ShortCode::new("notfound".to_string()).unwrap();

        let result = service.resolve_short_code(&code);
        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::ShortCodeNotFound(_))));
    }

    #[test]
    fn test_delete_short_code() {
        let service = create_service();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
        let shortened = service.shorten_url(url).unwrap();
        let code = shortened.short_code().clone();

        // Should exist
        assert!(service.get_statistics(&code).is_ok());

        // Delete it
        service.delete_short_code(&code).unwrap();

        // Should no longer exist
        assert!(service.get_statistics(&code).is_err());
    }

    #[test]
    fn test_list_all() {
        let service = create_service();

        let urls = service.list_all().unwrap();
        assert_eq!(urls.len(), 0);

        let url1 = OriginalUrl::new("https://example1.com".to_string()).unwrap();
        let url2 = OriginalUrl::new("https://example2.com".to_string()).unwrap();

        service.shorten_url(url1).unwrap();
        service.shorten_url(url2).unwrap();

        let urls = service.list_all().unwrap();
        assert_eq!(urls.len(), 2);
    }
}
