# Rust Testing Plugin - Context

## Purpose

This plugin helps developers write comprehensive, maintainable tests for Rust applications using unit tests, integration tests, property-based testing, and mocking strategies.

## Key Testing Principles

### 1. Unit Tests in Source Files

Unit tests live in the same file as the code they test:

```rust
// src/user.rs
pub struct User {
    pub id: String,
    pub email: String,
}

impl User {
    pub fn new(id: String, email: String) -> Result<Self, ValidationError> {
        if email.is_empty() {
            return Err(ValidationError::EmptyEmail);
        }
        Ok(Self { id, email })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_success() {
        let user = User::new("1".to_string(), "test@example.com".to_string()).unwrap();
        assert_eq!(user.id, "1");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_new_user_empty_email() {
        let result = User::new("1".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(result, Err(ValidationError::EmptyEmail)));
    }
}
```

### 2. Integration Tests in tests/ Directory

Integration tests are separate crates that test your public API:

```rust
// tests/user_service_integration.rs
use my_crate::*;

#[tokio::test]
async fn test_create_and_retrieve_user() {
    let service = setup_test_service().await;

    let user = service.create_user("test@example.com").await.unwrap();
    let retrieved = service.get_user(&user.id).await.unwrap();

    assert_eq!(user.id, retrieved.id);
}

async fn setup_test_service() -> UserService {
    let pool = create_test_db_pool().await;
    UserService::new(pool)
}
```

### 3. Async Testing

Use `#[tokio::test]` for async tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (result1, result2) = tokio::join!(
            async_op1(),
            async_op2()
        );

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
```

## Testing Patterns

### Pattern 1: Arrange-Act-Assert (AAA)

```rust
#[test]
fn test_user_validation() {
    // Arrange
    let email = "invalid-email";
    let user = User::new("1".to_string(), email.to_string());

    // Act
    let result = validate_user(&user);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ValidationError::InvalidEmail);
}
```

### Pattern 2: Table-Driven Tests

```rust
#[test]
fn test_multiple_validation_cases() {
    let test_cases = vec![
        ("", false, "Empty email"),
        ("invalid", false, "No @ sign"),
        ("test@example.com", true, "Valid email"),
        ("test@", false, "No domain"),
    ];

    for (email, should_pass, description) in test_cases {
        let result = validate_email(email);
        assert_eq!(result.is_ok(), should_pass, "Failed for: {}", description);
    }
}
```

### Pattern 3: Mock Implementations

For hexagonal architecture, create test doubles:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockUserRepository {
        users: HashMap<String, User>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self { users: HashMap::new() }
        }

        fn with_user(mut self, user: User) -> Self {
            self.users.insert(user.id.clone(), user);
            self
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: &str) -> Result<User, Error> {
            self.users.get(id)
                .cloned()
                .ok_or(Error::NotFound(id.to_string()))
        }

        async fn save(&self, user: &User) -> Result<(), Error> {
            Ok(())  // Mock implementation
        }
    }

    #[tokio::test]
    async fn test_service_with_mock() {
        let mock_repo = MockUserRepository::new()
            .with_user(User { id: "1".to_string(), email: "test@example.com".to_string() });

        let service = UserService::new(mock_repo);
        let user = service.get_user("1").await.unwrap();

        assert_eq!(user.email, "test@example.com");
    }
}
```

### Pattern 4: Test Fixtures

Create reusable test data:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_user() -> User {
        User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    fn test_user_with_email(email: &str) -> User {
        User {
            id: "test-id".to_string(),
            email: email.to_string(),
        }
    }

    #[test]
    fn test_with_fixture() {
        let user = test_user();
        let result = process_user(&user);
        assert!(result.is_ok());
    }
}
```

### Pattern 5: Property-Based Testing

Test properties that should always hold:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_serialization_roundtrip(user_id in "[a-z0-9]{1,10}", email in "[a-z]+@[a-z]+\\.com") {
        let user = User {
            id: user_id.clone(),
            email: email.clone(),
        };

        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&serialized).unwrap();

        prop_assert_eq!(user.id, deserialized.id);
        prop_assert_eq!(user.email, deserialized.email);
    }

    #[test]
    fn test_validation_properties(s in ".*") {
        // Email validation should never panic
        let _ = validate_email(&s);
    }
}
```

### Pattern 6: Integration Test Setup

```rust
// tests/common/mod.rs
use sqlx::PgPool;
use testcontainers::*;

pub async fn setup_test_db() -> PgPool {
    let container = postgres::Postgres::default();
    let connection_string = format!(
        "postgres://postgres@localhost:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    let pool = PgPool::connect(&connection_string).await.unwrap();

    // Run migrations
    sqlx::migrate!().run(&pool).await.unwrap();

    pool
}

// tests/integration_test.rs
mod common;

#[tokio::test]
async fn test_with_real_database() {
    let pool = common::setup_test_db().await;
    let repo = PostgresUserRepository::new(pool);

    let user = User::new("1".to_string(), "test@example.com".to_string()).unwrap();
    repo.save(&user).await.unwrap();

    let retrieved = repo.find_by_id("1").await.unwrap();
    assert_eq!(retrieved.email, "test@example.com");
}
```

## Test Organization

### Unit Tests
- In same file as code
- Use `#[cfg(test)]` module
- Test private functions
- Focus on single units

### Integration Tests
- In `tests/` directory
- Each file is a separate crate
- Test public API only
- Use real or test implementations

### Benchmark Tests
- In `benches/` directory
- Use criterion.rs
- Measure performance

## Testing Best Practices

### ✅ DO

1. **Test both success and error paths**
```rust
#[test]
fn test_success() { /* ... */ }

#[test]
fn test_error_not_found() { /* ... */ }

#[test]
fn test_error_invalid_input() { /* ... */ }
```

2. **Use descriptive test names**
```rust
#[test]
fn test_user_creation_with_valid_email_succeeds() { /* ... */ }

#[test]
fn test_user_creation_with_empty_email_returns_validation_error() { /* ... */ }
```

3. **Test edge cases**
```rust
#[test]
fn test_empty_input() { /* ... */ }

#[test]
fn test_max_length_input() { /* ... */ }

#[test]
fn test_special_characters() { /* ... */ }
```

4. **Use specific assertions**
```rust
assert_eq!(result, expected);  // Not just assert!(result == expected)
assert!(matches!(error, ErrorType::Specific));
```

### ❌ DON'T

1. **Don't test implementation details**
```rust
// BAD: Testing internal state
#[test]
fn test_internal_counter() {
    let obj = MyStruct::new();
    assert_eq!(obj.internal_counter, 0);  // Testing private implementation
}

// GOOD: Testing behavior
#[test]
fn test_operation_count() {
    let obj = MyStruct::new();
    assert_eq!(obj.operation_count(), 0);  // Testing public behavior
}
```

2. **Don't have assertions in setup**
```rust
// BAD
fn setup() -> User {
    let user = create_user();
    assert!(user.is_valid());  // Don't assert in setup
    user
}

// GOOD
fn setup() -> User {
    create_user()
}

#[test]
fn test() {
    let user = setup();
    assert!(user.is_valid());  // Assert in test
}
```

3. **Don't ignore test failures**
```rust
// BAD
#[test]
#[ignore]  // Don't just ignore failing tests
fn test_something() { /* ... */ }
```

## Common Test Utilities

### Custom Assertions
```rust
#[cfg(test)]
mod test_utils {
    pub fn assert_error<T, E: std::fmt::Debug>(
        result: Result<T, E>,
        expected_error: E
    ) where E: PartialEq {
        match result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => assert_eq!(e, expected_error),
        }
    }
}
```

### Test Helpers
```rust
#[cfg(test)]
mod helpers {
    pub fn create_test_config() -> Config {
        Config {
            database_url: "postgres://localhost/test".to_string(),
            port: 8080,
        }
    }
}
```

## Coverage and Quality

### Measuring Coverage
```bash
# Using tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Using llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Running Tests
```bash
# All tests
cargo test

# Specific test
cargo test test_user_creation

# Show output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Integration tests only
cargo test --test integration_test
```

## Resources

- [Rust Book: Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest](https://docs.rs/proptest/)
- [mockall](https://docs.rs/mockall/)
- [rstest](https://docs.rs/rstest/)
- [testcontainers](https://docs.rs/testcontainers/)

---

This context helps Claude Code write comprehensive, maintainable tests for Rust applications.
