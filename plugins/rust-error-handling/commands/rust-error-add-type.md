---
description: Create a new custom error type with thiserror
---

You are helping create a custom error type in Rust using the `thiserror` crate for robust error handling.

## Your Task

Generate a well-structured custom error type with appropriate variants and conversions.

## Steps

1. **Ask for Error Type Details**

   Ask the user (if not provided):
   - Error type name (e.g., "ConfigError", "DatabaseError", "ValidationError")
   - What domain/module is this for?
   - What error variants are needed? (suggest common ones based on context)
   - Should it wrap any external error types? (std::io::Error, sqlx::Error, etc.)

2. **Determine Common Patterns**

   Based on the error name, suggest appropriate variants:

   **For *ValidationError**:
   - Required(String) - missing required field
   - Invalid { field, value } - invalid field value
   - OutOfRange { min, max } - value out of range

   **For *DatabaseError / *RepositoryError**:
   - NotFound(String)
   - Connection (#[from] lib::Error)
   - Query(String)
   - Transaction

   **For *NetworkError / *ApiError**:
   - Connection (#[from] reqwest::Error)
   - Timeout
   - InvalidResponse(String)
   - Unauthorized

   **For *ConfigError**:
   - MissingField(String)
   - InvalidValue { field, value }
   - ParseError (#[from] toml::Error or serde_json::Error)

3. **Create Error Type File**

   Generate the error type with thiserror:

   ```rust
   use thiserror::Error;

   /// Error type for [domain/module]
   ///
   /// This error type represents all possible errors that can occur
   /// when [description of what can go wrong].
   #[derive(Error, Debug)]
   pub enum [ErrorName] {
       /// [Description of when this error occurs]
       #[error("[User-friendly error message]")]
       [Variant1](String),

       /// [Description]
       #[error("[Message with field]: {field}")]
       [Variant2] {
           field: String,
       },

       /// Wraps [external error type]
       #[error("[Context message]")]
       [Variant3](#[from] [ExternalError]),

       /// [Description]
       #[error("[Message with source error]")]
       [Variant4](#[source] [ExternalError]),
   }
   ```

4. **Add Result Type Alias**

   Create a Result alias for convenience:

   ```rust
   /// Result type alias for [module] operations
   pub type Result<T> = std::result::Result<T, [ErrorName]>;
   ```

5. **Create Comprehensive Example**

   Provide a complete, real-world example:

   ```rust
   use thiserror::Error;

   /// Errors that can occur during user operations
   #[derive(Error, Debug)]
   pub enum UserError {
       /// User not found with the given identifier
       #[error("User not found: {0}")]
       NotFound(String),

       /// Invalid email format
       #[error("Invalid email: {0}")]
       InvalidEmail(String),

       /// Database operation failed
       #[error("Database error")]
       Database(#[from] sqlx::Error),

       /// User already exists
       #[error("User already exists: {email}")]
       AlreadyExists { email: String },

       /// Validation failed
       #[error("Validation failed: {0}")]
       Validation(String),
   }

   /// Result type for user operations
   pub type Result<T> = std::result::Result<T, UserError>;

   // Example usage
   pub async fn find_user(id: &str) -> Result<User> {
       let user = query_user(id).await?; // Auto-converts sqlx::Error
       validate_user(&user)?;
       Ok(user)
   }

   fn validate_user(user: &User) -> Result<()> {
       if user.email.is_empty() {
           return Err(UserError::InvalidEmail("Email cannot be empty".to_string()));
       }
       Ok(())
   }
   ```

6. **Add to Appropriate Module**

   Determine where to place the error type:
   - If it's a domain error: `src/domain/errors.rs` or `src/domain/[module]/error.rs`
   - If it's an infrastructure error: `src/infrastructure/errors.rs`
   - If it's a service error: `src/services/[service]/error.rs`

   Update the module's `mod.rs`:
   ```rust
   mod error;
   pub use error::{[ErrorName], Result};
   ```

7. **Update Cargo.toml**

   Ensure thiserror is added:
   ```toml
   [dependencies]
   thiserror = "1.0"
   ```

8. **Add Tests**

   Create tests for error handling:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_error_display() {
           let error = UserError::NotFound("user123".to_string());
           assert_eq!(error.to_string(), "User not found: user123");
       }

       #[test]
       fn test_error_conversion() {
           fn returns_io_error() -> std::io::Result<()> {
               Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
           }

           fn wraps_error() -> Result<()> {
               returns_io_error().map_err(|e| UserError::Validation(e.to_string()))?;
               Ok(())
           }

           assert!(wraps_error().is_err());
       }

       #[test]
       fn test_error_matching() {
           let error = UserError::InvalidEmail("test".to_string());

           match error {
               UserError::InvalidEmail(email) => assert_eq!(email, "test"),
               _ => panic!("Wrong error variant"),
           }
       }
   }
   ```

9. **Provide Usage Guidance**

   Show how to use the error type:

   ```
   ✅ Error type '[ErrorName]' created successfully!

   ## Usage Examples:

   ### Returning Errors
   ```rust
   fn do_work() -> Result<Output> {
       if condition {
           return Err([ErrorName]::SomeVariant("details".to_string()));
       }
       Ok(output)
   }
   ```

   ### Error Propagation
   ```rust
   fn process() -> Result<Data> {
       let result = risky_operation()?; // Auto-converts with #[from]
       Ok(result)
   }
   ```

   ### Error Matching
   ```rust
   match operation() {
       Ok(value) => println!("Success: {:?}", value),
       Err([ErrorName]::NotFound(id)) => eprintln!("Not found: {}", id),
       Err([ErrorName]::Validation(msg)) => eprintln!("Validation: {}", msg),
       Err(e) => eprintln!("Other error: {}", e),
   }
   ```

   ## Next Steps:

   1. Review and adjust error variants as needed
   2. Use this error type in your functions
   3. Add more variants as you discover new error cases
   4. Consider creating error conversion helpers if needed

   ## Testing:
   ```bash
   cargo test
   ```
   ```

## Error Message Guidelines

When creating error messages:

1. **Be Specific**: Include relevant context
   ```rust
   #[error("Failed to load config from {path}")]
   ConfigLoad { path: String }
   ```

2. **Be User-Friendly**: Write for humans
   ```rust
   #[error("The email address '{0}' is not valid")]
   InvalidEmail(String)
   ```

3. **Include Details**: Help with debugging
   ```rust
   #[error("Database query failed: {query}")]
   QueryFailed { query: String }
   ```

4. **Use Action Words**: Describe what went wrong
   ```rust
   #[error("Failed to connect to database at {url}")]
   ConnectionFailed { url: String }
   ```

## Common Patterns

### Simple Error
```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("Something went wrong: {0}")]
    Generic(String),
}
```

### Error with Fields
```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid value for {field}: expected {expected}, got {actual}")]
    InvalidValue {
        field: String,
        expected: String,
        actual: String,
    },
}
```

### Wrapped External Error
```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO operation failed")]
    Io(#[from] std::io::Error),

    #[error("Serialization failed")]
    Serde(#[from] serde_json::Error),
}
```

### Error with Source
```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("Operation failed")]
    Failed(#[source] Box<dyn std::error::Error>),
}
```

## Important Notes

- Always derive `Error` and `Debug`
- Use `#[from]` for automatic From impl
- Use `#[source]` to preserve error chain
- Keep error messages concise but informative
- Consider adding Result type alias
- Add documentation comments
- Include tests for error cases

## After Completion

Ask the user:
1. Do you want to add more error variants?
2. Should we create conversion helpers?
3. Do you want to integrate this with existing error types?
