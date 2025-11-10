---
description: Add a new adapter implementation for an existing port
---

You are helping create a new adapter implementation for an existing port in a Rust hexagonal architecture project.

## Your Task

Create a concrete adapter implementation for a port trait, following best practices for the specific technology being used.

## Steps

1. **List Available Ports**

   First, scan the project to find existing ports:
   - Check `src/ports/driven.rs` for driven ports
   - Check `src/ports/driving.rs` for driving ports

   Display them to the user:
   ```
   Available Ports:

   Driven Ports (Secondary):
   - UserRepository
   - PaymentGateway
   - EmailService

   Driving Ports (Primary):
   - CreateUserUseCase
   - ProcessOrderUseCase
   ```

2. **Ask User for Details**

   Ask (if not already provided):
   - Which port to implement?
   - What technology/adapter type? (e.g., PostgreSQL, MongoDB, HTTP, InMemory, Mock)
   - Any configuration needed? (connection strings, API keys, etc.)

3. **Create Adapter Implementation**

   Based on the technology, create the appropriate adapter.

   **Database Adapters (PostgreSQL example)**:
   ```rust
   //! PostgreSQL implementation of [PortName]
   //!
   //! This adapter implements [PortName] using PostgreSQL via SQLx.

   use crate::domain::models::[Entity];
   use crate::ports::driven::{[PortName], [PortName]Error};
   use async_trait::async_trait;
   use sqlx::PgPool;

   /// PostgreSQL implementation of [PortName]
   pub struct Postgres[PortName] {
       pool: PgPool,
   }

   impl Postgres[PortName] {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
   }

   #[async_trait]
   impl [PortName] for Postgres[PortName] {
       async fn find_by_id(&self, id: &str) -> Result<[Entity], [PortName]Error> {
           sqlx::query_as!(
               [Entity],
               "SELECT * FROM [table_name] WHERE id = $1",
               id
           )
           .fetch_one(&self.pool)
           .await
           .map_err(|e| match e {
               sqlx::Error::RowNotFound => [PortName]Error::NotFound(id.to_string()),
               _ => [PortName]Error::Database(e.to_string()),
           })
       }

       async fn save(&self, entity: &[Entity]) -> Result<(), [PortName]Error> {
           sqlx::query!(
               "INSERT INTO [table_name] (id, [fields]) VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET [fields] = $2",
               entity.id(),
               // other fields
           )
           .execute(&self.pool)
           .await
           .map_err(|e| [PortName]Error::Database(e.to_string()))?;

           Ok(())
       }

       async fn delete(&self, id: &str) -> Result<(), [PortName]Error> {
           sqlx::query!("DELETE FROM [table_name] WHERE id = $1", id)
               .execute(&self.pool)
               .await
               .map_err(|e| [PortName]Error::Database(e.to_string()))?;

           Ok(())
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;
       // Recommend using testcontainers for integration tests

       #[sqlx::test]
       async fn test_find_by_id(pool: PgPool) {
           let repo = Postgres[PortName]::new(pool);
           // Test implementation
       }
   }
   ```

   **HTTP Client Adapters**:
   ```rust
   //! HTTP implementation of [PortName]
   //!
   //! This adapter implements [PortName] using reqwest HTTP client.

   use crate::ports::driven::{[PortName], [PortName]Error};
   use async_trait::async_trait;
   use reqwest::Client;
   use serde::{Deserialize, Serialize};

   /// HTTP client implementation of [PortName]
   pub struct Http[PortName] {
       client: Client,
       base_url: String,
       api_key: Option<String>,
   }

   impl Http[PortName] {
       pub fn new(base_url: String, api_key: Option<String>) -> Self {
           Self {
               client: Client::new(),
               base_url,
               api_key,
           }
       }
   }

   #[async_trait]
   impl [PortName] for Http[PortName] {
       async fn [method](&self, [params]) -> Result<[ReturnType], [PortName]Error> {
           let url = format!("{}/[endpoint]", self.base_url);

           let mut request = self.client.get(&url);

           if let Some(key) = &self.api_key {
               request = request.header("Authorization", format!("Bearer {}", key));
           }

           let response = request
               .send()
               .await
               .map_err(|e| [PortName]Error::Network(e.to_string()))?;

           if !response.status().is_success() {
               return Err([PortName]Error::HttpError(response.status().as_u16()));
           }

           response
               .json::<[ReturnType]>()
               .await
               .map_err(|e| [PortName]Error::Deserialization(e.to_string()))
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;
       use wiremock::{MockServer, Mock, ResponseTemplate};
       use wiremock::matchers::{method, path};

       #[tokio::test]
       async fn test_[method]() {
           let mock_server = MockServer::start().await;

           Mock::given(method("GET"))
               .and(path("/[endpoint]"))
               .respond_with(ResponseTemplate::new(200).set_body_json(/* mock response */))
               .mount(&mock_server)
               .await;

           let adapter = Http[PortName]::new(mock_server.uri(), None);
           // Test implementation
       }
   }
   ```

   **In-Memory Adapters (for testing)**:
   ```rust
   //! In-memory implementation of [PortName]
   //!
   //! This adapter provides an in-memory implementation useful for testing.

   use crate::domain::models::[Entity];
   use crate::ports::driven::{[PortName], [PortName]Error};
   use async_trait::async_trait;
   use std::collections::HashMap;
   use std::sync::Arc;
   use tokio::sync::RwLock;

   /// In-memory implementation of [PortName]
   #[derive(Clone)]
   pub struct InMemory[PortName] {
       storage: Arc<RwLock<HashMap<String, [Entity]>>>,
   }

   impl InMemory[PortName] {
       pub fn new() -> Self {
           Self {
               storage: Arc::new(RwLock::new(HashMap::new())),
           }
       }
   }

   impl Default for InMemory[PortName] {
       fn default() -> Self {
           Self::new()
       }
   }

   #[async_trait]
   impl [PortName] for InMemory[PortName] {
       async fn find_by_id(&self, id: &str) -> Result<[Entity], [PortName]Error> {
           let storage = self.storage.read().await;
           storage
               .get(id)
               .cloned()
               .ok_or_else(|| [PortName]Error::NotFound(id.to_string()))
       }

       async fn save(&self, entity: &[Entity]) -> Result<(), [PortName]Error> {
           let mut storage = self.storage.write().await;
           storage.insert(entity.id().to_string(), entity.clone());
           Ok(())
       }

       async fn delete(&self, id: &str) -> Result<(), [PortName]Error> {
           let mut storage = self.storage.write().await;
           storage
               .remove(id)
               .ok_or_else(|| [PortName]Error::NotFound(id.to_string()))?;
           Ok(())
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_save_and_find() {
           let repo = InMemory[PortName]::new();
           // Test implementation
       }
   }
   ```

   **Redis Cache Adapter**:
   ```rust
   //! Redis implementation of [CacheName]
   //!
   //! This adapter implements caching using Redis.

   use crate::ports::driven::{[CacheName], [CacheName]Error};
   use async_trait::async_trait;
   use redis::{AsyncCommands, Client};
   use serde::{de::DeserializeOwned, Serialize};

   /// Redis implementation of [CacheName]
   pub struct Redis[CacheName] {
       client: Client,
   }

   impl Redis[CacheName] {
       pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
           let client = Client::open(redis_url)?;
           Ok(Self { client })
       }
   }

   #[async_trait]
   impl [CacheName] for Redis[CacheName] {
       async fn get<T>(&self, key: &str) -> Result<Option<T>, [CacheName]Error>
       where
           T: DeserializeOwned,
       {
           let mut conn = self.client.get_async_connection()
               .await
               .map_err(|e| [CacheName]Error::Connection(e.to_string()))?;

           let value: Option<String> = conn.get(key)
               .await
               .map_err(|e| [CacheName]Error::Operation(e.to_string()))?;

           match value {
               Some(v) => {
                   let parsed = serde_json::from_str(&v)
                       .map_err(|e| [CacheName]Error::Serialization(e.to_string()))?;
                   Ok(Some(parsed))
               }
               None => Ok(None),
           }
       }

       async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), [CacheName]Error>
       where
           T: Serialize,
       {
           let mut conn = self.client.get_async_connection()
               .await
               .map_err(|e| [CacheName]Error::Connection(e.to_string()))?;

           let serialized = serde_json::to_string(value)
               .map_err(|e| [CacheName]Error::Serialization(e.to_string()))?;

           if let Some(ttl) = ttl_seconds {
               conn.set_ex(key, serialized, ttl)
                   .await
                   .map_err(|e| [CacheName]Error::Operation(e.to_string()))?;
           } else {
               conn.set(key, serialized)
                   .await
                   .map_err(|e| [CacheName]Error::Operation(e.to_string()))?;
           }

           Ok(())
       }
   }
   ```

4. **Update Module Exports**

   Add to `src/adapters/driven/mod.rs` (or `driving/mod.rs`):
   ```rust
   pub mod [adapter_name];
   ```

5. **Update Dependencies**

   Check if required dependencies are in Cargo.toml and suggest additions:

   For PostgreSQL:
   ```toml
   sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
   ```

   For HTTP:
   ```toml
   reqwest = { version = "0.11", features = ["json"] }
   ```

   For Redis:
   ```toml
   redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
   ```

6. **Provide Integration Example**

   Show how to wire up the adapter in the application:

   ```rust
   // In main.rs or application setup
   use crate::adapters::driven::[Adapter];
   use crate::domain::services::[Service];

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Setup adapter
       let pool = PgPoolOptions::new()
           .connect("postgresql://localhost/mydb")
           .await?;

       let adapter = [Adapter]::new(pool);

       // Create domain service with adapter
       let service = [Service]::new(adapter);

       // Use the service
       service.do_work().await?;

       Ok(())
   }
   ```

7. **Suggest Testing Approach**

   Based on adapter type:

   **For Database Adapters**:
   ```
   Testing recommendations:
   1. Use `sqlx::test` macro for integration tests
   2. Consider testcontainers for isolated test databases
   3. Create test fixtures for common scenarios
   4. Test error cases (not found, connection errors)
   ```

   **For HTTP Adapters**:
   ```
   Testing recommendations:
   1. Use wiremock or mockito for HTTP mocking
   2. Test success and error responses
   3. Test authentication/authorization
   4. Test timeout and retry logic
   ```

   **For In-Memory Adapters**:
   ```
   Testing recommendations:
   1. Test concurrent access with multiple threads
   2. Verify data consistency
   3. Test all CRUD operations
   ```

8. **Provide Summary**

   Tell the user:
   ```
   ✅ Adapter '[AdapterName]' created successfully!

   ## Files Created/Modified:
   - `src/adapters/[driving|driven]/[adapter_name].rs` - Adapter implementation
   - `src/adapters/[driving|driven]/mod.rs` - Module export

   ## Dependencies to Add:
   [List required Cargo.toml dependencies]

   ## Next Steps:

   1. Add dependencies to Cargo.toml
   2. Implement the TODO items in the adapter
   3. Write tests for the adapter
   4. Integrate adapter in your application setup

   ## Testing:
   ```bash
   cargo test
   ```

   ## Example Integration:
   [Show integration example from step 6]
   ```

## Technology Templates

Maintain templates for common technologies:
- PostgreSQL, MySQL, SQLite (via sqlx)
- MongoDB (via mongodb crate)
- Redis (via redis crate)
- HTTP clients (via reqwest)
- gRPC (via tonic)
- InMemory (HashMap/RwLock)
- Mock (for testing)

## Important Notes

- Always implement `Send + Sync` for thread safety
- Use appropriate error mapping from library errors to port errors
- Include comprehensive tests
- Add documentation comments
- Follow Rust async best practices
- Consider connection pooling for database adapters

## After Completion

Ask the user if they want to:
1. Create another adapter for the same port (e.g., a test double)
2. Add more methods to the adapter
3. Create integration tests
