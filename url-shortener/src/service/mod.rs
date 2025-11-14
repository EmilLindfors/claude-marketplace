//! Application service layer
//!
//! The service layer orchestrates domain logic and uses ports to interact
//! with external dependencies.

mod url_shortener_service;

pub use url_shortener_service::UrlShortenerService;
