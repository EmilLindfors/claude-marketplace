---
description: Initialize a hexagonal architecture project structure for Rust
---

You are helping initialize a Rust project with hexagonal architecture (ports and adapters pattern).

## Your Task

Create a complete hexagonal architecture directory structure with example files to help the user get started.

## Steps

1. **Verify or Create Base Directory Structure**

   Create the following structure:
   ```
   src/
   ├── domain/
   │   ├── mod.rs
   │   ├── models.rs
   │   └── services.rs
   ├── ports/
   │   ├── mod.rs
   │   ├── driving.rs
   │   └── driven.rs
   ├── adapters/
   │   ├── mod.rs
   │   ├── driving/
   │   │   └── mod.rs
   │   └── driven/
   │       └── mod.rs
   └── lib.rs (or main.rs if it exists)
   ```

2. **Create Domain Layer Files**

   **src/domain/mod.rs**:
   ```rust
   //! Domain layer - Core business logic
   //!
   //! This layer contains:
   //! - Domain models (entities, value objects)
   //! - Business rules and validations
   //! - Domain services
   //!
   //! The domain layer has NO dependencies on ports or adapters.

   pub mod models;
   pub mod services;
   ```

   **src/domain/models.rs**:
   ```rust
   //! Domain models and entities
   //!
   //! Define your business entities here with their behaviors.

   use serde::{Deserialize, Serialize};

   /// Example domain entity
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ExampleEntity {
       id: String,
       name: String,
   }

   impl ExampleEntity {
       pub fn new(id: String, name: String) -> Result<Self, ValidationError> {
           if name.is_empty() {
               return Err(ValidationError::EmptyName);
           }
           Ok(Self { id, name })
       }

       pub fn id(&self) -> &str {
           &self.id
       }

       pub fn name(&self) -> &str {
           &self.name
       }
   }

   #[derive(Debug, thiserror::Error)]
   pub enum ValidationError {
       #[error("Name cannot be empty")]
       EmptyName,
   }
   ```

   **src/domain/services.rs**:
   ```rust
   //! Domain services - Business logic orchestration
   //!
   //! Domain services coordinate between entities and use ports
   //! for external dependencies.

   use super::models::{ExampleEntity, ValidationError};
   use crate::ports::driven::ExampleRepository;

   /// Example domain service
   pub struct ExampleService<R>
   where
       R: ExampleRepository,
   {
       repository: R,
   }

   impl<R> ExampleService<R>
   where
       R: ExampleRepository,
   {
       pub fn new(repository: R) -> Self {
           Self { repository }
       }

       pub async fn get_entity(&self, id: &str) -> Result<ExampleEntity, ServiceError> {
           self.repository
               .find_by_id(id)
               .await
               .map_err(ServiceError::Repository)
       }

       pub async fn create_entity(&self, name: String) -> Result<ExampleEntity, ServiceError> {
           let entity = ExampleEntity::new(uuid::Uuid::new_v4().to_string(), name)
               .map_err(ServiceError::Validation)?;

           self.repository
               .save(&entity)
               .await
               .map_err(ServiceError::Repository)?;

           Ok(entity)
       }
   }

   #[derive(Debug, thiserror::Error)]
   pub enum ServiceError {
       #[error("Validation error: {0}")]
       Validation(#[from] ValidationError),

       #[error("Repository error: {0}")]
       Repository(#[from] crate::ports::driven::RepositoryError),
   }
   ```

3. **Create Ports Layer Files**

   **src/ports/mod.rs**:
   ```rust
   //! Ports layer - Interfaces for adapters
   //!
   //! Ports define the contracts between the domain and the outside world:
   //! - Driving ports: What the domain offers to the outside
   //! - Driven ports: What the domain needs from the outside

   pub mod driving;
   pub mod driven;
   ```

   **src/ports/driving.rs**:
   ```rust
   //! Driving ports (Primary ports)
   //!
   //! These are the interfaces that the domain exposes to the outside world.
   //! Driving adapters (like REST APIs, CLI) will use these interfaces.

   use crate::domain::models::ExampleEntity;
   use async_trait::async_trait;

   /// Example driving port - what the application offers
   #[async_trait]
   pub trait ExampleUseCase: Send + Sync {
       async fn execute(&self, input: UseCaseInput) -> Result<ExampleEntity, UseCaseError>;
   }

   pub struct UseCaseInput {
       pub name: String,
   }

   #[derive(Debug, thiserror::Error)]
   pub enum UseCaseError {
       #[error("Invalid input: {0}")]
       InvalidInput(String),

       #[error("Service error: {0}")]
       Service(#[from] crate::domain::services::ServiceError),
   }
   ```

   **src/ports/driven.rs**:
   ```rust
   //! Driven ports (Secondary ports)
   //!
   //! These are the interfaces that the domain needs from the outside world.
   //! Driven adapters (like database repositories, HTTP clients) implement these.

   use crate::domain::models::ExampleEntity;
   use async_trait::async_trait;

   /// Example repository port - what the domain needs
   #[async_trait]
   pub trait ExampleRepository: Send + Sync {
       async fn find_by_id(&self, id: &str) -> Result<ExampleEntity, RepositoryError>;
       async fn save(&self, entity: &ExampleEntity) -> Result<(), RepositoryError>;
       async fn delete(&self, id: &str) -> Result<(), RepositoryError>;
   }

   #[derive(Debug, thiserror::Error)]
   pub enum RepositoryError {
       #[error("Not found: {0}")]
       NotFound(String),

       #[error("Database error: {0}")]
       Database(String),

       #[error("Unknown error: {0}")]
       Unknown(String),
   }
   ```

4. **Create Adapters Layer Files**

   **src/adapters/mod.rs**:
   ```rust
   //! Adapters layer - Implementations of ports
   //!
   //! Adapters connect the domain to the outside world:
   //! - Driving adapters: REST API, CLI, gRPC
   //! - Driven adapters: Database, HTTP clients, file systems

   pub mod driving;
   pub mod driven;
   ```

   **src/adapters/driving/mod.rs**:
   ```rust
   //! Driving adapters
   //!
   //! These adapters expose the application to the outside world.
   //! Examples: REST API, CLI, gRPC server, GraphQL

   // Example: REST API adapter would go here
   // pub mod rest_api;
   ```

   **src/adapters/driven/mod.rs**:
   ```rust
   //! Driven adapters
   //!
   //! These adapters implement the ports needed by the domain.
   //! Examples: PostgreSQL repository, HTTP client, Redis cache

   // Example adapter implementations would go here
   // pub mod postgres_repository;
   // pub mod http_client;
   ```

5. **Update lib.rs or main.rs**

   Add to the top of `src/lib.rs` (or `src/main.rs` if no lib.rs exists):
   ```rust
   //! Hexagonal Architecture Application
   //!
   //! This application follows the hexagonal architecture pattern:
   //! - Domain: Core business logic
   //! - Ports: Interfaces (traits)
   //! - Adapters: Implementations

   pub mod domain;
   pub mod ports;
   pub mod adapters;
   ```

6. **Update Cargo.toml**

   Ensure the following dependencies are in Cargo.toml:
   ```toml
   [dependencies]
   # Async runtime
   tokio = { version = "1", features = ["full"] }
   async-trait = "0.1"

   # Error handling
   thiserror = "1.0"
   anyhow = "1.0"

   # Serialization
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"

   # UUID generation
   uuid = { version = "1.0", features = ["v4", "serde"] }

   # Example: Database (uncomment if needed)
   # sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }

   # Example: HTTP (uncomment if needed)
   # axum = "0.7"
   # reqwest = { version = "0.11", features = ["json"] }
   ```

7. **Provide Usage Instructions**

   After creating all files, tell the user:
   ```
   ✅ Hexagonal architecture structure initialized!

   ## Next Steps:

   1. Review the generated structure in `src/`
   2. Define your domain models in `src/domain/models.rs`
   3. Implement business logic in `src/domain/services.rs`
   4. Create port traits in `src/ports/` for external dependencies
   5. Implement adapters in `src/adapters/` for each port

   ## Example Commands:
   - Add a new port: `/rust-hex-add-port`
   - Add an adapter: `/rust-hex-add-adapter`
   - Get architecture help: Ask the `rust-hex-architect` agent

   ## Running the Code:
   ```bash
   cargo build
   cargo test
   ```

   ## Architecture Layers:
   - **Domain**: Pure business logic (no external dependencies)
   - **Ports**: Trait definitions (interfaces)
   - **Adapters**: Concrete implementations

   Dependencies flow: Adapters → Ports → Domain
   ```

## Important Notes

- Create directories only if they don't exist
- Don't overwrite existing files without asking the user first
- If files already exist, ask if they want to merge or skip
- Use proper Rust formatting and conventions
- Add helpful comments explaining the architecture

## After Completion

Tell the user about the structure created and suggest next steps for their specific use case.
