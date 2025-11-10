# Rust Error Handling Plugin - Context

## Purpose

This plugin helps developers implement robust, idiomatic error handling in Rust applications using Result types, custom error enums, and proper error propagation.

## Key Principles

### 1. Use Result for Recoverable Errors

Rust's type system makes error handling explicit and safe:

```rust
// ❌ BAD: Using panic for expected errors
fn parse_port(s: &str) -> u16 {
    s.parse().unwrap() // Crashes on invalid input
}

// ✅ GOOD: Using Result
fn parse_port(s: &str) -> Result<u16, ParseError> {
    s.parse().map_err(|_| ParseError::InvalidFormat)
}
```

### 2. Custom Error Types with thiserror

Use `thiserror` for library error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

### 3. Error Propagation with ?

The `?` operator provides concise error propagation:

```rust
fn process() -> Result<Data, Error> {
    let file = File::open("data.txt")?; // Returns early on error
    let content = read_content(file)?;
    let parsed = parse(content)?;
    Ok(parsed)
}
```

### 4. Error Context with anyhow

Use `anyhow` for application-level error handling:

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let bytes = std::fs::read("config.toml")
        .context("Failed to read config.toml")?;

    toml::from_slice(&bytes)
        .context("Failed to parse TOML")
}
```

## Error Type Patterns

### Pattern 1: Simple Error Enum

For straightforward error cases:

```rust
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field '{0}' is required")]
    Required(String),

    #[error("Field '{field}' has invalid value: {value}")]
    Invalid { field: String, value: String },

    #[error("Value must be between {min} and {max}")]
    OutOfRange { min: i32, max: i32 },
}
```

### Pattern 2: Wrapped External Errors

For wrapping errors from dependencies:

```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed")]
    Connection(#[from] sqlx::Error),

    #[error("Query failed: {0}")]
    Query(String),

    #[error("Transaction failed")]
    Transaction(#[source] sqlx::Error),
}
```

### Pattern 3: Layered Errors

Different error types for different layers:

```rust
// Domain layer errors
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid user: {0}")]
    InvalidUser(String),

    #[error("Permission denied")]
    PermissionDenied,
}

// Repository layer errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

// Application layer errors (combines both)
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}
```

### Pattern 4: Result Type Alias

Create type aliases for cleaner signatures:

```rust
// In your crate root
pub type Result<T> = std::result::Result<T, Error>;

// Now you can use it without specifying error type
pub fn do_something() -> Result<Output> {
    Ok(output)
}
```

## Error Conversion

### Using #[from] for Automatic Conversion

```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error")]
    Io(#[from] std::io::Error), // Implements From<std::io::Error>

    #[error("Parse error")]
    Parse(#[from] serde_json::Error), // Implements From<serde_json::Error>
}

// Now ? operator auto-converts
fn process() -> Result<(), MyError> {
    let file = File::open("data.json")?; // io::Error -> MyError
    let data: Data = serde_json::from_reader(file)?; // serde_json::Error -> MyError
    Ok(())
}
```

### Manual Error Conversion

```rust
impl From<ParseIntError> for MyError {
    fn from(err: ParseIntError) -> Self {
        MyError::Parse(err.to_string())
    }
}
```

## Common Anti-Patterns

### ❌ Overusing unwrap/expect

```rust
// BAD: Will panic on error
let value = risky_operation().unwrap();

// GOOD: Handle the error
let value = risky_operation().map_err(|e| MyError::from(e))?;
```

### ❌ Swallowing Errors

```rust
// BAD: Ignoring errors
let _ = dangerous_operation();

// GOOD: Handle or propagate
dangerous_operation()?;
```

### ❌ String as Error Type

```rust
// BAD: String errors lose type information
fn do_work() -> Result<(), String> {
    Err("something went wrong".to_string())
}

// GOOD: Custom error type
#[derive(Error, Debug)]
pub enum WorkError {
    #[error("Operation failed: {0}")]
    Failed(String),
}

fn do_work() -> Result<(), WorkError> {
    Err(WorkError::Failed("something went wrong".to_string()))
}
```

### ❌ Too Generic Errors

```rust
// BAD: One generic error variant
#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Generic(String),
}

// GOOD: Specific error variants
#[derive(Error, Debug)]
pub enum Error {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Database connection failed")]
    DatabaseConnection,
}
```

## Testing Error Handling

### Test Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let result = validate_email("");

        assert!(result.is_err());

        match result {
            Err(ValidationError::Required(field)) => {
                assert_eq!(field, "email");
            }
            _ => panic!("Expected Required error"),
        }
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let my_error: MyError = io_error.into();

        assert!(matches!(my_error, MyError::Io(_)));
    }
}
```

## Library vs Application Errors

### Library Errors (use thiserror)

```rust
// Libraries should use custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Processing failed")]
    ProcessingFailed(#[from] std::io::Error),
}
```

### Application Errors (use anyhow)

```rust
// Applications can use anyhow for flexibility
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    start_server(config)
        .context("Failed to start server")?;

    Ok(())
}
```

## Error Reporting

### User-Friendly Error Messages

```rust
#[derive(Error, Debug)]
pub enum UserError {
    #[error("Unable to connect to database. Please check your connection settings.")]
    DatabaseConnection,

    #[error("The file '{0}' does not exist. Please check the path.")]
    FileNotFound(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
}
```

### Debug vs Display

```rust
// Display for user-friendly messages
#[error("Configuration error: {0}")]
ConfigError(String),

// Debug includes source error for logging
#[error("Database query failed")]
QueryFailed(#[source] sqlx::Error),
```

## When to Use What

| Scenario | Recommendation |
|----------|---------------|
| Library code | Use `thiserror` with custom error types |
| Application code | Use `anyhow` with context |
| Domain layer | Custom error types with `thiserror` |
| Prototyping | `anyhow` or `Box<dyn Error>` |
| Error conversion | `#[from]` attribute |
| Adding context | `anyhow::Context` trait |
| Unrecoverable errors | `panic!` or `unwrap` (rarely) |
| Test code | `unwrap`, `expect` acceptable |

## Best Practices Summary

1. ✅ Use `Result` for all recoverable errors
2. ✅ Define custom error types with `thiserror`
3. ✅ Use `#[from]` for automatic error conversion
4. ✅ Add context with `anyhow::Context` or custom methods
5. ✅ Test error cases, not just success paths
6. ✅ Document when functions return which errors
7. ✅ Use error type aliases for cleaner APIs
8. ✅ Layer errors appropriately (domain, infra, app)
9. ❌ Avoid `unwrap/expect` except in tests
10. ❌ Don't use String as error type

## Resources

- [Rust Error Handling Guide](https://rust10x.com/best-practices/error-handling)
- [thiserror crate](https://docs.rs/thiserror/)
- [anyhow crate](https://docs.rs/anyhow/)
- [Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

---

This context helps Claude Code understand and teach robust error handling patterns in Rust.
