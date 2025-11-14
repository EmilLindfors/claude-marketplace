//! Adapters (implementations of ports)
//!
//! These are concrete implementations of the port traits.
//! Following hexagonal architecture, these are our adapters.

mod in_memory_repository;
mod random_id_generator;

pub use in_memory_repository::InMemoryUrlRepository;
pub use random_id_generator::RandomIdGenerator;
