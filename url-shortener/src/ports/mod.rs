//! Ports (interfaces) for external dependencies
//!
//! This module defines the traits that adapters must implement.
//! Following hexagonal architecture, these are our ports.

mod repository;
mod id_generator;

pub use repository::UrlRepository;
pub use id_generator::IdGenerator;
