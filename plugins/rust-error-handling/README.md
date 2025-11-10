# Rust Error Handling Plugin

A comprehensive plugin for implementing robust error handling patterns in Rust applications.

## Overview

This plugin helps you implement best practices for error handling in Rust using the Result type, custom error types, and proper error propagation patterns.

## Features

### 📝 Error Handling Commands

#### `/rust-error-add-type`
Create a new custom error type with proper error variants.

**Features**:
- Interactive prompts for error name and variants
- Automatic `thiserror` or manual implementation
- From trait implementations for error conversion
- Display and Debug implementations
- Documentation templates

#### `/rust-error-refactor`
Refactor code from panic-based error handling to Result-based.

**Features**:
- Scan code for unwrap() and expect() calls
- Suggest Result-based alternatives
- Convert panic-prone code to proper error handling
- Add error propagation with ? operator
- Update function signatures to return Result

### 🤖 Error Handling Agent

A specialized agent (`rust-error-expert`) for complex error handling tasks.

**Capabilities**:
- Analyze error handling patterns in codebase
- Design error type hierarchies
- Refactor from panic to Result
- Review error handling for best practices
- Generate error types with proper conversions
- Suggest error recovery strategies

## Error Handling Patterns

### Custom Error Types with thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

### Error Propagation with ?

```rust
fn process_data() -> Result<Data, MyError> {
    let content = read_file()?; // Auto-converts using From trait
    let parsed = parse_content(&content)?;
    Ok(parsed)
}
```

### Error Context with anyhow

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;

    toml::from_str(&content)
        .context("Failed to parse config")
}
```

### Result Type Aliases

```rust
// Define a result type alias for your library
pub type Result<T> = std::result::Result<T, MyError>;

pub fn do_work() -> Result<Output> {
    // Now you can use Result without the error type
    Ok(output)
}
```

## Best Practices

1. **Use Result for recoverable errors**: Don't use panic! for expected errors
2. **Use custom error types**: Define domain-specific errors with thiserror
3. **Implement From traits**: Enable automatic error conversion with ?
4. **Add context to errors**: Use anyhow or custom context methods
5. **Don't overuse unwrap/expect**: Only use in tests or when panic is appropriate
6. **Layer error types**: Different error types for different layers (domain, infra)
7. **Document error conditions**: Explain when functions return which errors
8. **Test error cases**: Write tests for error paths, not just happy paths

## Installation

```bash
cp -r plugins/rust-error-handling /path/to/your/project/plugins/
```

Register in marketplace.json:
```json
{
  "plugins": [{
    "name": "rust-error-handling",
    "source": "./plugins/rust-error-handling",
    "description": "Error handling best practices for Rust",
    "version": "1.0.0"
  }]
}
```

## Usage

### Quick Start

1. **Add error handling dependencies**:
   ```bash
   cargo add thiserror anyhow
   ```

2. **Create a custom error type**:
   ```
   /rust-error-add-type
   ```

3. **Refactor existing code**:
   ```
   /rust-error-refactor
   ```

4. **Get expert help**:
   ```
   Ask rust-error-expert to review my error handling
   ```

## Common Dependencies

```toml
[dependencies]
# For library error types
thiserror = "1.0"

# For application error handling
anyhow = "1.0"

# For error reporting
color-eyre = "0.6"  # Alternative to anyhow
```

## Error Type Patterns

### Simple Enum Errors
```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
}
```

### Layered Errors
```rust
// Domain errors
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Business rule violated: {0}")]
    BusinessRule(String),
}

// Infrastructure errors
#[derive(Error, Debug)]
pub enum InfraError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Network error")]
    Network(#[from] reqwest::Error),
}

// Application errors (combines both)
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infra(#[from] InfraError),
}
```

## Testing Error Handling

```rust
#[test]
fn test_error_handling() {
    let result = parse_config("invalid");

    assert!(result.is_err());

    match result {
        Err(ConfigError::InvalidValue { field, .. }) => {
            assert_eq!(field, "port");
        }
        _ => panic!("Expected InvalidValue error"),
    }
}
```

## Troubleshooting

### thiserror not working
- Ensure `thiserror = "1.0"` is in Cargo.toml
- Check that derive macro is enabled

### Error conversion not working
- Verify #[from] attribute is used
- Check that From trait is in scope
- Ensure error types are compatible

## Resources

- [Rust Error Handling Best Practices](https://rust10x.com/best-practices/error-handling)
- [thiserror documentation](https://docs.rs/thiserror/)
- [anyhow documentation](https://docs.rs/anyhow/)
- [Error Handling in Rust (Rust Book)](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

---

**Handle errors gracefully, write robust Rust** 🦀✨
