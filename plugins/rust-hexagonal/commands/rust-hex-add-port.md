---
description: Add a new port (interface) to your hexagonal architecture
---

You are helping add a new port (interface/trait) to a Rust hexagonal architecture project.

## Your Task

Guide the user through creating a new port trait and provide the implementation scaffold.

## Steps

1. **Ask User for Port Details**

   Ask the following questions (if not already provided):
   - Port name (e.g., "UserRepository", "PaymentGateway", "EmailService")
   - Port type: "driving" (primary - what domain offers) or "driven" (secondary - what domain needs)
   - Brief description of what this port does
   - Required methods (you can suggest common ones based on the name)

2. **Determine Port Type**

   **Driving Ports** (Primary):
   - Use cases, application services
   - What the application offers to external actors
   - Usually have an "execute" or similar method
   - Example: `CreateUserUseCase`, `GetOrderDetails`

   **Driven Ports** (Secondary):
   - Repositories, gateways, external services
   - What the domain needs from infrastructure
   - Usually CRUD or external API operations
   - Example: `UserRepository`, `EmailGateway`, `PaymentService`

3. **Create the Port Trait**

   Based on the port type, create the trait in the appropriate file.

   **For Driven Ports** (in `src/ports/driven.rs`):
   ```rust
   /// [Description of what this port does]
   ///
   /// This port is implemented by adapters that provide [functionality].
   #[async_trait]
   pub trait [PortName]: Send + Sync {
       /// [Method description]
       async fn [method_name](&self, [params]) -> Result<[ReturnType], [ErrorType]>;

       // Add more methods as needed
   }

   /// Error type for [PortName]
   #[derive(Debug, thiserror::Error)]
   pub enum [PortName]Error {
       #[error("Not found: {0}")]
       NotFound(String),

       #[error("Operation failed: {0}")]
       OperationFailed(String),

       #[error("Unknown error: {0}")]
       Unknown(String),
   }
   ```

   **For Driving Ports** (in `src/ports/driving.rs`):
   ```rust
   /// [Description of use case]
   ///
   /// This use case [what it does for the user].
   #[async_trait]
   pub trait [UseCaseName]: Send + Sync {
       async fn execute(&self, input: [UseCaseName]Input) -> Result<[UseCaseName]Output, [UseCaseName]Error>;
   }

   /// Input for [UseCaseName]
   #[derive(Debug, Clone)]
   pub struct [UseCaseName]Input {
       // Input fields
   }

   /// Output for [UseCaseName]
   #[derive(Debug, Clone)]
   pub struct [UseCaseName]Output {
       // Output fields
   }

   /// Error type for [UseCaseName]
   #[derive(Debug, thiserror::Error)]
   pub enum [UseCaseName]Error {
       #[error("Invalid input: {0}")]
       InvalidInput(String),

       #[error("Use case failed: {0}")]
       Failed(String),
   }
   ```

4. **Common Port Patterns**

   Suggest appropriate methods based on the port name:

   **Repository Patterns**:
   ```rust
   async fn find_by_id(&self, id: &str) -> Result<Entity, Error>;
   async fn find_all(&self) -> Result<Vec<Entity>, Error>;
   async fn save(&self, entity: &Entity) -> Result<(), Error>;
   async fn update(&self, entity: &Entity) -> Result<(), Error>;
   async fn delete(&self, id: &str) -> Result<(), Error>;
   ```

   **Service/Gateway Patterns**:
   ```rust
   async fn send(&self, data: &Data) -> Result<Response, Error>;
   async fn query(&self, params: QueryParams) -> Result<QueryResult, Error>;
   ```

   **Cache Patterns**:
   ```rust
   async fn get(&self, key: &str) -> Result<Option<Value>, Error>;
   async fn set(&self, key: &str, value: Value) -> Result<(), Error>;
   async fn delete(&self, key: &str) -> Result<(), Error>;
   ```

5. **Create Adapter Scaffold**

   After creating the port, automatically create a scaffold for an adapter implementation:

   **For Driven Ports** (create in `src/adapters/driven/`):
   ```rust
   //! [PortName] adapter implementation
   //!
   //! This adapter implements the [PortName] port using [technology].

   use crate::ports::driven::[PortName];
   use async_trait::async_trait;

   /// [Technology] implementation of [PortName]
   pub struct [TechnologyName][PortName] {
       // Configuration fields
       // e.g., connection pool, client, config
   }

   impl [TechnologyName][PortName] {
       pub fn new(/* config params */) -> Self {
           Self {
               // Initialize fields
           }
       }
   }

   #[async_trait]
   impl [PortName] for [TechnologyName][PortName] {
       async fn [method_name](&self, [params]) -> Result<[ReturnType], [ErrorType]> {
           // TODO: Implement using [technology]
           todo!("Implement [method_name]")
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_[method_name]() {
           // TODO: Add tests
       }
   }
   ```

6. **Update Module Exports**

   Add the new port to the appropriate module file:

   For driven ports, add to `src/ports/driven.rs`:
   ```rust
   pub use self::[port_name]::*;
   mod [port_name];
   ```

   Or add to the existing file if it's small.

   For adapters, add to `src/adapters/driven/mod.rs`:
   ```rust
   pub mod [adapter_name];
   ```

7. **Provide Usage Example**

   Show the user how to use the new port:

   ```rust
   // In a domain service
   use crate::ports::driven::[PortName];

   pub struct MyService<R>
   where
       R: [PortName],
   {
       [port_field]: R,
   }

   impl<R> MyService<R>
   where
       R: [PortName],
   {
       pub fn new([port_field]: R) -> Self {
           Self { [port_field] }
       }

       pub async fn do_something(&self) -> Result<(), Error> {
           self.[port_field].[method]().await?;
           Ok(())
       }
   }
   ```

8. **Suggest Next Steps**

   Tell the user:
   ```
   ✅ Port '[PortName]' created successfully!

   ## Files Created/Modified:
   - `src/ports/[driving|driven].rs` - Port trait definition
   - `src/adapters/[driving|driven]/[adapter_name].rs` - Adapter scaffold

   ## Next Steps:

   1. Review the port trait and adjust methods as needed
   2. Implement the adapter for your specific technology
   3. Add the adapter to your application's dependency injection
   4. Write tests for the adapter

   ## Example Usage:
   [Show usage example from step 7]

   ## Implement Adapter:
   To implement the adapter for a specific technology (e.g., PostgreSQL, HTTP):
   - Use `/rust-hex-add-adapter` to create additional implementations
   - Or manually edit `src/adapters/[driving|driven]/[adapter_name].rs`
   ```

## Port Naming Conventions

- **Repositories**: `[Entity]Repository` (e.g., `UserRepository`, `OrderRepository`)
- **Gateways**: `[Service]Gateway` (e.g., `PaymentGateway`, `EmailGateway`)
- **Services**: `[Domain]Service` (e.g., `AuthenticationService`, `NotificationService`)
- **Use Cases**: `[Action][Entity]` (e.g., `CreateUser`, `GetOrderDetails`)

## Important Notes

- Always use `#[async_trait]` for async trait methods
- Include `Send + Sync` bounds for thread safety
- Define custom error types using `thiserror`
- Add documentation comments explaining the port's purpose
- Follow Rust naming conventions (PascalCase for traits, snake_case for methods)

## After Completion

Confirm with the user that the port was created successfully and ask if they want to:
1. Add more methods to the port
2. Create additional adapters for this port
3. Create another port
