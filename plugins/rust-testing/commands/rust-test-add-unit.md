---
description: Add comprehensive unit tests for a function or module
---

You are helping add unit tests to Rust code following best practices.

## Your Task

Generate comprehensive unit tests for the specified function or module, covering success cases, error cases, and edge cases.

## Steps

1. **Identify Target**

   Ask the user (if not specified):
   - What function/struct/module to test?
   - Where is it located?

   Or scan the current file for testable functions.

2. **Analyze Function**

   Read the function to understand:
   - What it does
   - What inputs it takes
   - What it returns (including error types)
   - What edge cases exist

3. **Create Test Module**

   Add or update the `#[cfg(test)]` module at the bottom of the file:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       // Tests go here
   }
   ```

4. **Generate Test Cases**

   For each function, create tests for:

   **Success Cases**:
   ```rust
   #[test]
   fn test_[function]_with_valid_input() {
       // Arrange
       let input = create_valid_input();

       // Act
       let result = function(input);

       // Assert
       assert!(result.is_ok());
       assert_eq!(result.unwrap(), expected_value);
   }
   ```

   **Error Cases**:
   ```rust
   #[test]
   fn test_[function]_returns_error_on_invalid_input() {
       let result = function(invalid_input);

       assert!(result.is_err());
       assert!(matches!(result.unwrap_err(), ErrorType::Specific));
   }
   ```

   **Edge Cases**:
   ```rust
   #[test]
   fn test_[function]_with_empty_input() {
       let result = function("");
       assert!(result.is_err());
   }

   #[test]
   fn test_[function]_with_max_length_input() {
       let long_input = "x".repeat(MAX_LENGTH);
       let result = function(&long_input);
       assert!(result.is_ok());
   }
   ```

5. **Add Test Fixtures**

   Create helper functions for common test data:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       fn create_test_user() -> User {
           User {
               id: "test-id".to_string(),
               email: "test@example.com".to_string(),
           }
       }

       fn create_test_user_with_id(id: &str) -> User {
           User {
               id: id.to_string(),
               email: "test@example.com".to_string(),
           }
       }

       #[test]
       fn test_with_fixture() {
           let user = create_test_user();
           let result = validate_user(&user);
           assert!(result.is_ok());
       }
   }
   ```

6. **Add Async Tests if Needed**

   For async functions:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_async_function() {
           let result = async_function().await;
           assert!(result.is_ok());
       }

       #[tokio::test]
       async fn test_async_error_case() {
           let result = async_function_with_error().await;
           assert!(result.is_err());
       }
   }
   ```

7. **Add Mock Implementations**

   For functions using traits:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use std::collections::HashMap;

       struct MockRepository {
           data: HashMap<String, User>,
       }

       impl MockRepository {
           fn new() -> Self {
               Self { data: HashMap::new() }
           }

           fn with_user(mut self, user: User) -> Self {
               self.data.insert(user.id.clone(), user);
               self
           }
       }

       #[async_trait]
       impl UserRepository for MockRepository {
           async fn find_by_id(&self, id: &str) -> Result<User, Error> {
               self.data.get(id)
                   .cloned()
                   .ok_or(Error::NotFound(id.to_string()))
           }
       }

       #[tokio::test]
       async fn test_service_with_mock() {
           let mock = MockRepository::new()
               .with_user(create_test_user());

           let service = UserService::new(mock);
           let user = service.get_user("test-id").await.unwrap();

           assert_eq!(user.email, "test@example.com");
       }
   }
   ```

8. **Add Table-Driven Tests**

   For multiple similar test cases:

   ```rust
   #[test]
   fn test_validation_cases() {
       let test_cases = vec![
           ("", false, "Empty input"),
           ("a", true, "Single char"),
           ("abc", true, "Valid input"),
           ("x".repeat(1000).as_str(), false, "Too long"),
       ];

       for (input, should_pass, description) in test_cases {
           let result = validate(input);
           assert_eq!(
               result.is_ok(),
               should_pass,
               "Failed for case: {}",
               description
           );
       }
   }
   ```

9. **Provide Test Summary**

   After generating tests:

   ```
   ✅ Unit tests added successfully!

   ## Tests Created:

   ### Success Cases (3):
   - test_create_user_with_valid_email
   - test_update_user_success
   - test_delete_user_success

   ### Error Cases (4):
   - test_create_user_with_empty_email
   - test_create_user_with_invalid_email
   - test_update_nonexistent_user
   - test_delete_nonexistent_user

   ### Edge Cases (2):
   - test_email_max_length
   - test_special_characters_in_email

   ## Test Fixtures:
   - create_test_user()
   - create_test_user_with_email(email)

   ## Mock Implementations:
   - MockUserRepository

   ## Run Tests:
   ```bash
   cargo test
   cargo test --package [package_name]
   cargo test test_create_user -- --nocapture
   ```

   ## Next Steps:
   1. Review generated tests
   2. Add more edge cases if needed
   3. Run tests and verify they pass
   4. Check coverage with cargo tarpaulin
   ```

## Test Naming Conventions

Use descriptive names following this pattern:
- `test_[function]_[scenario]_[expected_result]`

Examples:
- `test_create_user_with_valid_email_succeeds`
- `test_create_user_with_empty_email_returns_validation_error`
- `test_parse_config_with_invalid_json_returns_parse_error`

## Important Patterns

### Testing Result Types
```rust
#[test]
fn test_returns_ok() {
    let result = function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_returns_specific_error() {
    let result = function();
    assert!(result.is_err());

    match result {
        Err(MyError::Specific) => (),  // Expected
        _ => panic!("Wrong error type"),
    }
}
```

### Testing Option Types
```rust
#[test]
fn test_returns_some() {
    let result = function();
    assert!(result.is_some());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_returns_none() {
    let result = function();
    assert!(result.is_none());
}
```

### Testing Panics
```rust
#[test]
#[should_panic(expected = "expected panic message")]
fn test_panics_on_invalid_input() {
    function_that_panics();
}
```

### Testing with assert_matches
```rust
use assert_matches::assert_matches;

#[test]
fn test_error_variant() {
    let result = function();
    assert_matches!(result, Err(Error::Specific { .. }));
}
```

## After Completion

Ask the user:
1. Do tests pass? (run `cargo test`)
2. Are there more scenarios to test?
3. Should we add integration tests?
4. Do you want to check test coverage?
