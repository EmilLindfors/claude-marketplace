---
description: Refactor code from panic-based to Result-based error handling
---

You are helping refactor Rust code to use proper error handling with Result types instead of panic-based error handling.

## Your Task

Analyze code for panic-prone patterns and refactor to use Result-based error handling.

## Steps

1. **Scan for Panic-Prone Code**

   Search the codebase for:
   - `.unwrap()` calls
   - `.expect()` calls
   - `panic!()` macros
   - `.unwrap_or_default()` where errors should be handled
   - Indexing that could panic (e.g., `vec[0]`)

   Use grep to find these patterns:
   ```
   - unwrap()
   - expect(
   - panic!(
   ```

2. **Categorize Findings**

   Group findings by severity:
   - **Critical**: Production code with unwrap/panic
   - **Warning**: expect() with poor messages
   - **Info**: Test code (acceptable to use unwrap)

   Report to user:
   ```
   Found panic-prone patterns:

   Critical (5):
   - src/api/handler.rs:42 - .unwrap() on database query
   - src/config.rs:15 - .expect("Failed") on file read
   ...

   Info (2):
   - tests/integration.rs:10 - .unwrap() (OK in tests)
   ...
   ```

3. **Ask User for Scope**

   Ask which files or functions to refactor:
   - All critical issues?
   - Specific file or module?
   - Specific function?

4. **Refactor Each Pattern**

   For each panic-prone pattern, apply appropriate refactoring:

   **Pattern 1: unwrap() on Option**

   Before:
   ```rust
   fn get_user(id: &str) -> User {
       let user = users.get(id).unwrap();
       user
   }
   ```

   After:
   ```rust
   fn get_user(id: &str) -> Result<User, UserError> {
       users.get(id)
           .cloned()
           .ok_or_else(|| UserError::NotFound(id.to_string()))
   }
   ```

   **Pattern 2: unwrap() on Result**

   Before:
   ```rust
   fn load_config() -> Config {
       let content = std::fs::read_to_string("config.toml").unwrap();
       toml::from_str(&content).unwrap()
   }
   ```

   After:
   ```rust
   fn load_config() -> Result<Config, ConfigError> {
       let content = std::fs::read_to_string("config.toml")
           .map_err(|e| ConfigError::FileRead(e))?;

       toml::from_str(&content)
           .map_err(|e| ConfigError::Parse(e))
   }
   ```

   **Pattern 3: expect() with bad message**

   Before:
   ```rust
   let value = dangerous_op().expect("Failed");
   ```

   After:
   ```rust
   let value = dangerous_op()
       .map_err(|e| MyError::OperationFailed(format!("Dangerous operation failed: {}", e)))?;
   ```

   **Pattern 4: panic! for validation**

   Before:
   ```rust
   fn create_user(email: String) -> User {
       if email.is_empty() {
           panic!("Email cannot be empty");
       }
       User { email }
   }
   ```

   After:
   ```rust
   fn create_user(email: String) -> Result<User, ValidationError> {
       if email.is_empty() {
           return Err(ValidationError::Required("email".to_string()));
       }
       Ok(User { email })
   }
   ```

   **Pattern 5: Vec indexing**

   Before:
   ```rust
   fn get_first(items: &Vec<Item>) -> Item {
       items[0].clone()
   }
   ```

   After:
   ```rust
   fn get_first(items: &Vec<Item>) -> Result<Item, ItemError> {
       items.first()
           .cloned()
           .ok_or(ItemError::Empty)
   }
   ```

5. **Update Function Signatures**

   When refactoring a function to return Result:
   - Change return type from `T` to `Result<T, ErrorType>`
   - Add error type if it doesn't exist
   - Update all return statements

   Before:
   ```rust
   fn process_data(input: &str) -> Data {
       // ...
   }
   ```

   After:
   ```rust
   fn process_data(input: &str) -> Result<Data, ProcessError> {
       // ...
       Ok(data)
   }
   ```

6. **Update Call Sites**

   Find all places where the refactored function is called and update them:

   **If caller already returns Result**:
   ```rust
   fn caller() -> Result<Output, Error> {
       let data = process_data(input)?; // Use ? operator
       Ok(output)
   }
   ```

   **If caller doesn't handle errors yet**:
   ```rust
   // Option 1: Propagate error
   fn caller() -> Result<Output, Error> {
       let data = process_data(input)?;
       Ok(output)
   }

   // Option 2: Handle locally
   fn caller() -> Output {
       match process_data(input) {
           Ok(data) => process(data),
           Err(e) => {
               log::error!("Failed to process: {}", e);
               default_output()
           }
       }
   }
   ```

7. **Add Error Types if Needed**

   If refactoring requires new error types, create them:

   ```rust
   #[derive(thiserror::Error, Debug)]
   pub enum ProcessError {
       #[error("Invalid input: {0}")]
       InvalidInput(String),

       #[error("IO error")]
       Io(#[from] std::io::Error),

       #[error("Parse error")]
       Parse(#[from] serde_json::Error),
   }
   ```

8. **Update Tests**

   Refactor tests to handle Result types:

   Before:
   ```rust
   #[test]
   fn test_process() {
       let result = process_data("input");
       assert_eq!(result.value, 42);
   }
   ```

   After:
   ```rust
   #[test]
   fn test_process_success() {
       let result = process_data("input").unwrap(); // OK in tests
       assert_eq!(result.value, 42);
   }

   #[test]
   fn test_process_error() {
       let result = process_data("invalid");
       assert!(result.is_err());
       match result {
           Err(ProcessError::InvalidInput(_)) => (),
           _ => panic!("Expected InvalidInput error"),
       }
   }
   ```

9. **Run Tests and Fix**

   After refactoring:
   ```bash
   cargo test
   cargo clippy
   ```

   Fix any compilation errors or test failures.

10. **Provide Refactoring Summary**

    Show what was changed:
    ```
    ✅ Refactored error handling

    ## Changes Made:

    ### Files Modified:
    - src/api/handler.rs
    - src/config.rs
    - src/database/query.rs

    ### Functions Refactored:
    - `load_config` - Now returns Result<Config, ConfigError>
    - `get_user` - Now returns Result<User, UserError>
    - `execute_query` - Now returns Result<Data, DatabaseError>

    ### New Error Types Created:
    - ConfigError in src/config.rs
    - UserError in src/domain/user.rs

    ### Patterns Replaced:
    - 8 unwrap() calls → ? operator with proper error handling
    - 3 expect() calls → descriptive error variants
    - 2 panic!() calls → Result returns

    ## Next Steps:

    1. Run tests: `cargo test`
    2. Review error messages for clarity
    3. Consider adding error context where needed
    4. Update API documentation

    ## Before/After Example:

    Before:
    ```rust
    fn load_config() -> Config {
        let content = std::fs::read_to_string("config.toml").unwrap();
        toml::from_str(&content).unwrap()
    }
    ```

    After:
    ```rust
    fn load_config() -> Result<Config, ConfigError> {
        let content = std::fs::read_to_string("config.toml")?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    ```
    ```

## Refactoring Guidelines

1. **Don't refactor test code**: `unwrap()` is acceptable in tests
2. **Preserve behavior**: Make sure logic stays the same
3. **Update incrementally**: Refactor one function at a time
4. **Test after each change**: Ensure nothing breaks
5. **Add error context**: Make errors informative
6. **Consider backwards compatibility**: Use deprecation if needed

## When to Keep unwrap/expect

Keep these patterns when:
- In test code (`#[cfg(test)]`)
- After explicitly checking with `if let Some` or `is_some()`
- When panic is truly the desired behavior (e.g., invalid constants)
- In example code or documentation

## Common Refactoring Patterns

### Option::unwrap → ok_or
```rust
// Before
let value = map.get(key).unwrap();

// After
let value = map.get(key)
    .ok_or(Error::NotFound(key.to_string()))?;
```

### Result::unwrap → ?
```rust
// Before
let data = parse_data(&input).unwrap();

// After
let data = parse_data(&input)?;
```

### panic! → return Err
```rust
// Before
if !is_valid(&input) {
    panic!("Invalid input");
}

// After
if !is_valid(&input) {
    return Err(Error::InvalidInput);
}
```

## Important Notes

- Create comprehensive error types before refactoring
- Update documentation to reflect new error returns
- Consider API compatibility for public functions
- Add migration guide if it's a library
- Use `#[deprecated]` for gradual migration

## After Completion

Ask the user:
1. Did all tests pass?
2. Are there more files to refactor?
3. Should we add more error context?
4. Do you want to update error documentation?
