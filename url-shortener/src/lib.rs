//! # URL Shortener Library
//!
//! A modern Rust URL shortener implementation demonstrating:
//! - **Hexagonal Architecture** (Ports and Adapters pattern)
//! - **Type-Driven Design** (Newtype pattern, validated types)
//! - **Error Handling** with `thiserror`
//! - **Comprehensive Testing**
//!
//! ## Architecture
//!
//! This library follows hexagonal architecture principles:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Application Core              │
//! │  ┌─────────────────────────────────┐   │
//! │  │       Domain Layer              │   │
//! │  │  - UrlId, ShortCode             │   │
//! │  │  - OriginalUrl, ShortenedUrl    │   │
//! │  └─────────────────────────────────┘   │
//! │  ┌─────────────────────────────────┐   │
//! │  │       Service Layer             │   │
//! │  │  - UrlShortenerService          │   │
//! │  └─────────────────────────────────┘   │
//! │  ┌─────────────────────────────────┐   │
//! │  │       Ports (Traits)            │   │
//! │  │  - UrlRepository                │   │
//! │  │  - IdGenerator                  │   │
//! │  └─────────────────────────────────┘   │
//! └─────────────────────────────────────────┘
//!         ↑                    ↑
//!         │                    │
//! ┌───────────────┐    ┌───────────────┐
//! │   Adapters    │    │   Adapters    │
//! │ InMemoryRepo  │    │ RandomIdGen   │
//! └───────────────┘    └───────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use url_shortener::{
//!     service::UrlShortenerService,
//!     adapters::{InMemoryUrlRepository, RandomIdGenerator},
//!     domain::OriginalUrl,
//! };
//! use std::sync::Arc;
//!
//! // Set up dependencies
//! let repository = Arc::new(InMemoryUrlRepository::new());
//! let id_generator = Arc::new(RandomIdGenerator::new());
//! let service = UrlShortenerService::new(repository, id_generator);
//!
//! // Shorten a URL
//! let url = OriginalUrl::new("https://example.com/very/long/path".to_string())?;
//! let shortened = service.shorten_url(url)?;
//!
//! println!("Short code: {}", shortened.short_code());
//! println!("Original URL: {}", shortened.original_url());
//!
//! // Resolve the short code
//! let original = service.resolve_short_code(shortened.short_code())?;
//! println!("Resolved to: {}", original);
//!
//! # Ok::<(), url_shortener::error::UrlShortenerError>(())
//! ```
//!
//! ## Type Safety with Newtype Pattern
//!
//! All domain types use the newtype pattern to prevent mixing up values:
//!
//! ```
//! use url_shortener::domain::{UrlId, ShortCode, OriginalUrl};
//!
//! let id = UrlId::new("abc123".to_string());
//! let code = ShortCode::new("mycode".to_string())?;
//! let url = OriginalUrl::new("https://example.com".to_string())?;
//!
//! // These are all different types - can't be mixed up!
//! // let x: UrlId = code; // Compile error!
//! # Ok::<(), url_shortener::error::UrlShortenerError>(())
//! ```
//!
//! ## Custom Short Codes
//!
//! ```
//! # use url_shortener::{
//! #     service::UrlShortenerService,
//! #     adapters::{InMemoryUrlRepository, RandomIdGenerator},
//! #     domain::{OriginalUrl, ShortCode},
//! # };
//! # use std::sync::Arc;
//! # let repository = Arc::new(InMemoryUrlRepository::new());
//! # let id_generator = Arc::new(RandomIdGenerator::new());
//! # let service = UrlShortenerService::new(repository, id_generator);
//! let url = OriginalUrl::new("https://example.com".to_string())?;
//! let code = ShortCode::new("custom".to_string())?;
//!
//! let shortened = service.shorten_url_with_code(url, code)?;
//! assert_eq!(shortened.short_code().as_str(), "custom");
//! # Ok::<(), url_shortener::error::UrlShortenerError>(())
//! ```
//!
//! ## Features
//!
//! - ✅ **Type-safe domain model** with validated types
//! - ✅ **Hexagonal architecture** for testability and flexibility
//! - ✅ **Thread-safe** with Arc and RwLock
//! - ✅ **Comprehensive error handling** with thiserror
//! - ✅ **Access counting** for analytics
//! - ✅ **Custom short codes** support
//! - ✅ **Well-tested** with unit and integration tests

pub mod adapters;
pub mod domain;
pub mod error;
pub mod ports;
pub mod service;

// Re-export commonly used types
pub use error::{Result, UrlShortenerError};
