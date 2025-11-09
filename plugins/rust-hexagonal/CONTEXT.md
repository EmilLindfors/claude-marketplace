# Rust Hexagonal Architecture Plugin - Context

## Purpose

This plugin helps developers implement hexagonal architecture (ports and adapters pattern) in Rust applications. It provides commands, agents, and best practices for creating maintainable, testable, and flexible Rust codebases.

## Key Concepts

### Hexagonal Architecture in Rust

Hexagonal architecture separates the application into three main areas:

1. **Domain Layer** (The Hexagon)
   - Contains business logic and domain models
   - No dependencies on external frameworks or libraries
   - Pure Rust code that's easy to test
   - Example: User validation, business rules, calculations

2. **Ports Layer** (Interfaces)
   - Defined using Rust traits
   - Two types:
     - **Driving Ports** (Primary): Interfaces the domain exposes (e.g., `UserService`)
     - **Driven Ports** (Secondary): Interfaces the domain requires (e.g., `UserRepository`)
   - Enable dependency inversion

3. **Adapters Layer** (Implementations)
   - Concrete implementations of ports
   - Two types:
     - **Driving Adapters**: REST APIs, CLI, gRPC servers
     - **Driven Adapters**: Database repos, HTTP clients, file systems
   - Can be swapped without affecting domain

### Why Rust is Perfect for Hexagonal Architecture

1. **Traits**: Natural fit for defining ports
2. **Zero-cost abstractions**: No runtime overhead from the pattern
3. **Type safety**: Compiler ensures correct implementations
4. **Compile-time polymorphism**: Static dispatch by default
5. **Ownership**: Clear boundaries prevent coupling

## Implementation Patterns

### Pattern 1: Trait-based Ports

```rust
// Driven port (what domain needs)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: UserId) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
}

// Domain service uses the port
pub struct UserService<R: UserRepository> {
    repo: R,
}
```

### Pattern 2: Generic Domain Services

```rust
// Domain service is generic over its dependencies
pub struct OrderService<R, P>
where
    R: OrderRepository,
    P: PaymentGateway,
{
    order_repo: R,
    payment: P,
}

impl<R, P> OrderService<R, P>
where
    R: OrderRepository,
    P: PaymentGateway,
{
    pub fn new(order_repo: R, payment: P) -> Self {
        Self { order_repo, payment }
    }

    pub async fn place_order(&self, order: Order) -> Result<OrderId, DomainError> {
        // Business logic here
        self.payment.charge(&order.amount).await?;
        self.order_repo.save(&order).await
    }
}
```

### Pattern 3: Error Handling

```rust
// Domain error type
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Repository error")]
    RepositoryError(#[from] RepositoryError),
}

// Adapter error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Connection error: {0}")]
    Connection(String),
}
```

### Pattern 4: Dependency Injection

```rust
// Application composition root
pub struct Application {
    user_service: UserService<PostgresUserRepository>,
}

impl Application {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .connect(database_url)
            .await?;

        let user_repo = PostgresUserRepository::new(pool);
        let user_service = UserService::new(user_repo);

        Ok(Self { user_service })
    }
}
```

## Command Usage Patterns

### Initializing a New Project

When starting a new Rust project with hexagonal architecture:

1. Use `/rust-hex-init` to create the structure
2. Define your domain models first
3. Identify what external dependencies you need (ports)
4. Create driven ports as traits
5. Implement adapters for each port
6. Wire everything together in main.rs

### Adding New Features

When adding a new feature:

1. Start with domain logic - what does the business need?
2. Identify if you need new ports (external dependencies)
3. Use `/rust-hex-add-port` for new interfaces
4. Use `/rust-hex-add-adapter` for implementations
5. Update domain services to use the new ports

### Refactoring Existing Code

When refactoring to hexagonal architecture:

1. Identify domain logic scattered in the codebase
2. Extract business rules into domain services
3. Identify external dependencies (DB, HTTP, files)
4. Create port traits for these dependencies
5. Create adapters implementing the ports
6. Use the `rust-hex-architect` agent for guidance

## Testing Strategies

### Unit Testing Domain

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserRepository {
        users: HashMap<UserId, User>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find(&self, id: UserId) -> Result<User, DomainError> {
            self.users.get(&id)
                .cloned()
                .ok_or(DomainError::UserNotFound(id.to_string()))
        }
    }

    #[tokio::test]
    async fn test_user_service() {
        let mock_repo = MockUserRepository::new();
        let service = UserService::new(mock_repo);

        // Test business logic without real database
        let result = service.get_user(UserId::new()).await;
        assert!(result.is_err());
    }
}
```

### Integration Testing Adapters

```rust
#[cfg(test)]
mod integration_tests {
    use testcontainers::*;

    #[tokio::test]
    async fn test_postgres_adapter() {
        // Spin up real database for integration tests
        let container = postgres::Postgres::default();
        let pool = create_pool(&container).await;

        let repo = PostgresUserRepository::new(pool);

        // Test actual database operations
        let user = User::new("test");
        repo.save(&user).await.unwrap();

        let found = repo.find(user.id()).await.unwrap();
        assert_eq!(found.name(), "test");
    }
}
```

## Common Pitfalls

### ❌ Leaking Adapter Types into Domain

```rust
// BAD: Domain depends on adapter type
pub struct UserService {
    repo: PostgresUserRepository, // ❌ Coupled to Postgres
}
```

```rust
// GOOD: Domain depends on port trait
pub struct UserService<R: UserRepository> {
    repo: R, // ✅ Generic over any implementation
}
```

### ❌ Too Many Small Ports

```rust
// BAD: Over-engineered with too many tiny ports
pub trait FindUser { ... }
pub trait SaveUser { ... }
pub trait DeleteUser { ... }
```

```rust
// GOOD: Cohesive port with related operations
pub trait UserRepository {
    async fn find(&self, id: UserId) -> Result<User, Error>;
    async fn save(&self, user: &User) -> Result<(), Error>;
    async fn delete(&self, id: UserId) -> Result<(), Error>;
}
```

### ❌ Anemic Domain Model

```rust
// BAD: Domain has no behavior, just data
pub struct User {
    pub id: String,
    pub email: String,
}

// Business logic in service instead of domain
impl UserService {
    pub fn validate_email(&self, email: &str) -> bool { ... }
}
```

```rust
// GOOD: Domain has behavior
pub struct User {
    id: UserId,
    email: Email, // Value object with validation
}

impl User {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        let email = Email::try_from(email)?; // Validation in domain
        Ok(Self { id: UserId::generate(), email })
    }
}
```

## Dependencies

Common dependencies for hexagonal architecture in Rust:

```toml
[dependencies]
# Async support
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Error handling
thiserror = "1.0"
anyhow = "1.0" # For application errors

# Serialization (for adapters)
serde = { version = "1.0", features = ["derive"] }

# Example adapters
# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# HTTP server
axum = "0.7"
```

## Agent Capabilities

The `rust-hex-architect` agent can:

1. **Analyze** existing codebases for architecture issues
2. **Design** new port interfaces based on requirements
3. **Generate** complete adapter implementations
4. **Refactor** legacy code to hexagonal architecture
5. **Review** PRs for architectural consistency
6. **Suggest** improvements for testability

## Best Practices Summary

1. **Domain First**: Always start with domain models and business logic
2. **Trait Ports**: Use traits to define clear contracts
3. **Dependency Direction**: Dependencies point inward (adapters → ports → domain)
4. **Pure Domain**: Keep domain free of framework dependencies
5. **Error Handling**: Use domain-specific error types
6. **Testing**: Mock ports for unit tests, real adapters for integration tests
7. **Composition Root**: Wire dependencies in main.rs or application struct
8. **Documentation**: Document port contracts clearly

## Resources

- [Hexagonal Architecture (Alistair Cockburn)](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust)
- [Rust Async Trait](https://docs.rs/async-trait/)
- [Domain-Driven Design in Rust](https://github.com/rust-ddd)

---

This context guide helps Claude Code understand how to effectively use and teach hexagonal architecture patterns in Rust.
