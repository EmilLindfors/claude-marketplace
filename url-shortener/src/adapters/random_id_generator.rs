//! Random ID generator implementation

use crate::domain::{ShortCode, UrlId};
use crate::error::{Result, UrlShortenerError};
use crate::ports::IdGenerator;
use rand::Rng;

/// Random ID generator using alphanumeric characters
///
/// Generates random IDs and short codes using secure random number generation.
pub struct RandomIdGenerator {
    short_code_length: usize,
}

impl RandomIdGenerator {
    /// Default length for generated short codes
    pub const DEFAULT_SHORT_CODE_LENGTH: usize = 6;

    /// Maximum attempts to generate a unique code
    const MAX_ATTEMPTS: usize = 100;

    /// Create a new random ID generator with default settings
    pub fn new() -> Self {
        Self {
            short_code_length: Self::DEFAULT_SHORT_CODE_LENGTH,
        }
    }

    /// Create a random ID generator with a specific short code length
    ///
    /// # Panics
    ///
    /// Panics if the length is outside the valid range for ShortCode
    pub fn with_length(length: usize) -> Self {
        assert!(
            length >= ShortCode::MIN_LENGTH && length <= ShortCode::MAX_LENGTH,
            "Short code length must be between {} and {}",
            ShortCode::MIN_LENGTH,
            ShortCode::MAX_LENGTH
        );

        Self {
            short_code_length: length,
        }
    }

    /// Generate a random alphanumeric string
    fn generate_alphanumeric(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl Default for RandomIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for RandomIdGenerator {
    fn generate_id(&self) -> UrlId {
        // Generate a longer ID for uniqueness
        let id = self.generate_alphanumeric(16);
        UrlId::new(id)
    }

    fn generate_short_code(&self) -> Result<ShortCode> {
        // Try multiple times to generate a valid code
        for attempt in 0..Self::MAX_ATTEMPTS {
            let code = self.generate_alphanumeric(self.short_code_length);

            match ShortCode::new(code) {
                Ok(short_code) => return Ok(short_code),
                Err(_) if attempt < Self::MAX_ATTEMPTS - 1 => continue,
                Err(e) => return Err(e),
            }
        }

        Err(UrlShortenerError::IdGenerationFailed(Self::MAX_ATTEMPTS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let generator = RandomIdGenerator::new();
        let id1 = generator.generate_id();
        let id2 = generator.generate_id();

        // IDs should be different
        assert_ne!(id1, id2);

        // IDs should be the expected length
        assert_eq!(id1.as_str().len(), 16);
        assert_eq!(id2.as_str().len(), 16);

        // IDs should be alphanumeric
        assert!(id1.as_str().chars().all(|c| c.is_alphanumeric()));
        assert!(id2.as_str().chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_short_code() {
        let generator = RandomIdGenerator::new();
        let code1 = generator.generate_short_code().unwrap();
        let code2 = generator.generate_short_code().unwrap();

        // Codes should likely be different (could collide but very unlikely)
        assert_ne!(code1, code2);

        // Codes should be the expected length
        assert_eq!(code1.as_str().len(), RandomIdGenerator::DEFAULT_SHORT_CODE_LENGTH);
        assert_eq!(code2.as_str().len(), RandomIdGenerator::DEFAULT_SHORT_CODE_LENGTH);

        // Codes should be alphanumeric
        assert!(code1.as_str().chars().all(|c| c.is_alphanumeric()));
        assert!(code2.as_str().chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_custom_length() {
        let generator = RandomIdGenerator::with_length(8);
        let code = generator.generate_short_code().unwrap();

        assert_eq!(code.as_str().len(), 8);
    }

    #[test]
    #[should_panic]
    fn test_invalid_length_too_short() {
        RandomIdGenerator::with_length(ShortCode::MIN_LENGTH - 1);
    }

    #[test]
    #[should_panic]
    fn test_invalid_length_too_long() {
        RandomIdGenerator::with_length(ShortCode::MAX_LENGTH + 1);
    }

    #[test]
    fn test_generate_multiple_unique_codes() {
        let generator = RandomIdGenerator::new();
        let mut codes = std::collections::HashSet::new();

        // Generate many codes and ensure they're mostly unique
        for _ in 0..100 {
            let code = generator.generate_short_code().unwrap();
            codes.insert(code.as_str().to_string());
        }

        // With 100 6-character alphanumeric codes, we should have very high uniqueness
        // (62^6 = ~56 billion possible combinations)
        assert!(codes.len() >= 95, "Expected at least 95 unique codes, got {}", codes.len());
    }
}
