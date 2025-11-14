//! Domain models using type-driven design patterns
//!
//! This module demonstrates:
//! - Newtype pattern for type safety
//! - Validated types that guarantee invariants
//! - Rich domain models with behavior

mod short_code;
mod url_id;
mod original_url;
mod shortened_url;

pub use short_code::ShortCode;
pub use url_id::UrlId;
pub use original_url::OriginalUrl;
pub use shortened_url::ShortenedUrl;
