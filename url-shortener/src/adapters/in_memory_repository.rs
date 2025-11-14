//! In-memory implementation of UrlRepository
//!
//! Uses a HashMap for storage with interior mutability pattern

use crate::domain::{ShortCode, ShortenedUrl};
use crate::error::{Result, UrlShortenerError};
use crate::ports::UrlRepository;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory URL repository using a HashMap
///
/// This adapter implements the UrlRepository port using an in-memory HashMap.
/// Uses RwLock for thread-safe concurrent access.
///
/// # Examples
///
/// ```
/// use url_shortener::adapters::InMemoryUrlRepository;
/// use url_shortener::ports::UrlRepository;
/// use url_shortener::domain::{UrlId, ShortCode, OriginalUrl, ShortenedUrl};
///
/// let repo = InMemoryUrlRepository::new();
///
/// let id = UrlId::new("123".to_string());
/// let code = ShortCode::new("abc123".to_string()).unwrap();
/// let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
/// let shortened = ShortenedUrl::new(id, code.clone(), url);
///
/// repo.save(shortened).unwrap();
/// let found = repo.find_by_short_code(&code).unwrap();
/// ```
#[derive(Clone)]
pub struct InMemoryUrlRepository {
    storage: Arc<RwLock<HashMap<String, ShortenedUrl>>>,
}

impl InMemoryUrlRepository {
    /// Create a new empty in-memory repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of stored URLs (useful for testing)
    pub fn len(&self) -> usize {
        self.storage.read().unwrap().len()
    }

    /// Check if the repository is empty
    pub fn is_empty(&self) -> bool {
        self.storage.read().unwrap().is_empty()
    }
}

impl Default for InMemoryUrlRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlRepository for InMemoryUrlRepository {
    fn save(&self, url: ShortenedUrl) -> Result<()> {
        let mut storage = self.storage.write()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire write lock: {}", e)
            ))?;

        let key = url.short_code().as_str().to_string();

        if storage.contains_key(&key) {
            return Err(UrlShortenerError::ShortCodeAlreadyExists(key));
        }

        storage.insert(key, url);
        Ok(())
    }

    fn find_by_short_code(&self, code: &ShortCode) -> Result<ShortenedUrl> {
        let storage = self.storage.read()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire read lock: {}", e)
            ))?;

        storage.get(code.as_str())
            .cloned()
            .ok_or_else(|| UrlShortenerError::ShortCodeNotFound(code.as_str().to_string()))
    }

    fn update(&self, url: ShortenedUrl) -> Result<()> {
        let mut storage = self.storage.write()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire write lock: {}", e)
            ))?;

        let key = url.short_code().as_str().to_string();

        if !storage.contains_key(&key) {
            return Err(UrlShortenerError::ShortCodeNotFound(key));
        }

        storage.insert(key, url);
        Ok(())
    }

    fn exists(&self, code: &ShortCode) -> Result<bool> {
        let storage = self.storage.read()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire read lock: {}", e)
            ))?;

        Ok(storage.contains_key(code.as_str()))
    }

    fn delete(&self, code: &ShortCode) -> Result<()> {
        let mut storage = self.storage.write()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire write lock: {}", e)
            ))?;

        let key = code.as_str();

        if !storage.contains_key(key) {
            return Err(UrlShortenerError::ShortCodeNotFound(key.to_string()));
        }

        storage.remove(key);
        Ok(())
    }

    fn list_all(&self) -> Result<Vec<ShortenedUrl>> {
        let storage = self.storage.read()
            .map_err(|e| UrlShortenerError::RepositoryError(
                format!("Failed to acquire read lock: {}", e)
            ))?;

        Ok(storage.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OriginalUrl, UrlId};

    fn create_test_url(code: &str) -> ShortenedUrl {
        let id = UrlId::new(format!("id-{}", code));
        let short_code = ShortCode::new(code.to_string()).unwrap();
        let url = OriginalUrl::new("https://example.com".to_string()).unwrap();
        ShortenedUrl::new(id, short_code, url)
    }

    #[test]
    fn test_save_and_find() {
        let repo = InMemoryUrlRepository::new();
        let url = create_test_url("test1234");
        let code = ShortCode::new("test1234".to_string()).unwrap();

        repo.save(url.clone()).unwrap();

        let found = repo.find_by_short_code(&code).unwrap();
        assert_eq!(found.short_code(), url.short_code());
    }

    #[test]
    fn test_duplicate_short_code() {
        let repo = InMemoryUrlRepository::new();
        let url1 = create_test_url("dupe1234");
        let url2 = create_test_url("dupe1234");

        repo.save(url1).unwrap();
        let result = repo.save(url2);

        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::ShortCodeAlreadyExists(_))));
    }

    #[test]
    fn test_find_not_found() {
        let repo = InMemoryUrlRepository::new();
        let code = ShortCode::new("notfound".to_string()).unwrap();

        let result = repo.find_by_short_code(&code);
        assert!(result.is_err());
        assert!(matches!(result, Err(UrlShortenerError::ShortCodeNotFound(_))));
    }

    #[test]
    fn test_update() {
        let repo = InMemoryUrlRepository::new();
        let mut url = create_test_url("updt1234");
        let code = ShortCode::new("updt1234".to_string()).unwrap();

        repo.save(url.clone()).unwrap();

        // Update access count
        url.record_access();
        repo.update(url.clone()).unwrap();

        let found = repo.find_by_short_code(&code).unwrap();
        assert_eq!(found.access_count(), 1);
    }

    #[test]
    fn test_exists() {
        let repo = InMemoryUrlRepository::new();
        let url = create_test_url("exst1234");
        let code = ShortCode::new("exst1234".to_string()).unwrap();

        assert!(!repo.exists(&code).unwrap());

        repo.save(url).unwrap();

        assert!(repo.exists(&code).unwrap());
    }

    #[test]
    fn test_delete() {
        let repo = InMemoryUrlRepository::new();
        let url = create_test_url("dele1234");
        let code = ShortCode::new("dele1234".to_string()).unwrap();

        repo.save(url).unwrap();
        assert!(repo.exists(&code).unwrap());

        repo.delete(&code).unwrap();
        assert!(!repo.exists(&code).unwrap());
    }

    #[test]
    fn test_list_all() {
        let repo = InMemoryUrlRepository::new();

        let urls = repo.list_all().unwrap();
        assert_eq!(urls.len(), 0);

        repo.save(create_test_url("list1234")).unwrap();
        repo.save(create_test_url("list5678")).unwrap();

        let urls = repo.list_all().unwrap();
        assert_eq!(urls.len(), 2);
    }
}
