# Rust Testing Plugin

A comprehensive plugin for implementing testing best practices in Rust applications, covering unit tests, integration tests, and property-based testing.

## Overview

This plugin helps you write comprehensive tests for Rust applications, following best practices for unit testing, integration testing, test organization, and test-driven development.

## Features

### 🧪 Testing Commands

#### `/rust-test-add-unit`
Add unit tests for a specific function or module.

**Features**:
- Generate test module with #[cfg(test)]
- Create tests for success and error cases
- Add test fixtures and helpers
- Include mock implementations
- Generate property-based tests

#### `/rust-test-add-integration`
Create integration tests in the tests/ directory.

**Features**:
- Set up integration test file structure
- Create test fixtures and setup/teardown
- Add database test containers
- Generate API test clients
- Include common test utilities

#### `/rust-test-analyze`
Analyze test coverage and suggest improvements.

**Features**:
- Identify untested functions
- Find missing error case tests
- Suggest edge cases to test
- Check test organization
- Recommend testing strategies

### 🤖 Testing Expert Agent

A specialized agent (`rust-test-expert`) for comprehensive testing guidance.

**Capabilities**:
- Design test strategies for complex code
- Generate comprehensive test suites
- Create mock implementations
- Set up integration test infrastructure
- Review test quality and coverage
- Suggest property-based testing approaches

## Testing Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_case() {
        let result = my_function("valid input");
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_error_case() {
        let result = my_function("invalid");
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "panic message")]
    fn test_panic_case() {
        my_function_that_panics();
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use my_crate::*;

#[test]
fn test_full_workflow() {
    let app = setup_test_app();

    let result = app.process("input");

    assert_eq!(result, expected);
}

fn setup_test_app() -> App {
    // Setup test instance
}
```

### Async Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_function() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

### Mock Implementations

```rust
#[cfg(test)]
mod tests {
    struct MockRepository {
        data: HashMap<String, User>,
    }

    impl UserRepository for MockRepository {
        async fn find(&self, id: &str) -> Result<User, Error> {
            self.data.get(id)
                .cloned()
                .ok_or(Error::NotFound)
        }
    }

    #[tokio::test]
    async fn test_with_mock() {
        let mut mock = MockRepository::new();
        mock.data.insert("1".to_string(), test_user());

        let service = UserService::new(mock);
        let user = service.get_user("1").await.unwrap();

        assert_eq!(user.id, "1");
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip(value in any::<u32>()) {
        let serialized = serialize(value);
        let deserialized = deserialize(&serialized).unwrap();
        prop_assert_eq!(value, deserialized);
    }

    #[test]
    fn test_property(input in "[a-z]{1,10}") {
        let result = process(&input);
        prop_assert!(result.len() > 0);
    }
}
```

## Best Practices

1. **Test Organization**: Use #[cfg(test)] modules in source files for unit tests
2. **Integration Tests**: Put integration tests in tests/ directory
3. **Test Naming**: Use descriptive names (test_function_scenario_expected)
4. **Setup/Teardown**: Use helper functions for test setup
5. **Assertions**: Use specific assertions (assert_eq!, assert_matches!)
6. **Error Testing**: Test both success and error paths
7. **Edge Cases**: Test boundary conditions and edge cases
8. **Async Testing**: Use #[tokio::test] for async tests
9. **Test Fixtures**: Create reusable test data
10. **Mocking**: Use trait objects or dedicated mocking libraries

## Installation

```bash
cp -r plugins/rust-testing /path/to/your/project/plugins/
```

Register in marketplace.json:
```json
{
  "plugins": [{
    "name": "rust-testing",
    "source": "./plugins/rust-testing",
    "description": "Testing best practices for Rust",
    "version": "1.0.0"
  }]
}
```

## Usage

### Quick Start

1. **Add testing dependencies**:
   ```bash
   cargo add --dev tokio-test
   cargo add --dev proptest
   cargo add --dev wiremock
   ```

2. **Add unit tests**:
   ```
   /rust-test-add-unit
   ```

3. **Create integration tests**:
   ```
   /rust-test-add-integration
   ```

4. **Analyze coverage**:
   ```
   /rust-test-analyze
   ```

5. **Get testing help**:
   ```
   Ask rust-test-expert to help design tests for my service
   ```

## Common Testing Dependencies

```toml
[dev-dependencies]
# Async testing
tokio-test = "0.4"

# Property-based testing
proptest = "1.0"
quickcheck = "1.0"

# Mocking
mockall = "0.12"
wiremock = "0.6"  # For HTTP mocking

# Test fixtures
rstest = "0.18"

# Test containers
testcontainers = "0.15"

# Assertions
assert_matches = "1.5"
pretty_assertions = "1.4"
```

## Test Organization

```
my-project/
├── src/
│   ├── lib.rs
│   ├── module.rs        # Unit tests in #[cfg(test)] module
│   └── another.rs
├── tests/
│   ├── integration_test.rs
│   ├── api_tests.rs
│   └── common/
│       └── mod.rs       # Shared test utilities
└── benches/
    └── benchmarks.rs    # Performance benchmarks
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test

# Run with coverage
cargo tarpaulin --out Html
```

## Testing Strategies

### Test-Driven Development (TDD)
1. Write failing test
2. Implement minimum code to pass
3. Refactor
4. Repeat

### Table-Driven Tests
```rust
#[test]
fn test_multiple_cases() {
    let test_cases = vec![
        ("input1", "expected1"),
        ("input2", "expected2"),
        ("input3", "expected3"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(process(input), expected);
    }
}
```

### Snapshot Testing
```rust
#[test]
fn test_output_snapshot() {
    let output = generate_complex_output();
    insta::assert_snapshot!(output);
}
```

## Troubleshooting

### Tests not running
- Check #[cfg(test)] module
- Verify test function has #[test] attribute
- Ensure cargo test is used, not cargo run

### Async tests failing
- Use #[tokio::test] instead of #[test]
- Add tokio to dev-dependencies
- Check async runtime configuration

### Integration tests not found
- Put tests in tests/ directory at project root
- Each file in tests/ is a separate crate
- Use common/ directory for shared code

## Resources

- [Rust Book: Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example: Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [proptest documentation](https://docs.rs/proptest/)
- [mockall documentation](https://docs.rs/mockall/)

---

**Test thoroughly, ship confidently** 🦀✅
