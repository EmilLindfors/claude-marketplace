---
description: Set up comprehensive testing for your MCP server
---

You are setting up testing infrastructure for an MCP server project.

## Your Task

Create comprehensive tests including unit tests, integration tests, and protocol validation.

## Steps

### 1. Run Initial Checks

```bash
# Check if project compiles
cargo check

# Run existing tests
cargo test

# Check for common issues
cargo clippy
```

### 2. Set Up Test Infrastructure

Install testing dependencies in `Cargo.toml`:

```toml
[dev-dependencies]
mockall = "0.13"          # Mocking
proptest = "1.5"          # Property-based testing
testcontainers = "0.20"   # Integration testing with containers
wiremock = "0.6"          # HTTP mocking
tokio-test = "0.4"        # Async test utilities
rstest = "0.22"           # Fixture-based testing
```

### 3. Create Unit Tests for Tools

Generate `tests/unit/tool_tests.rs`:

```rust
use {crate}::tools::*;
use mockall::predicate::*;

#[tokio::test]
async fn test_tool_success_case() {
    // Arrange
    let tool = MyTool::new(/* mocked deps */);
    let params = MyToolParams {
        field: "test".to_string(),
    };

    // Act
    let result = tool.execute(params).await;

    // Assert
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.field, "expected");
}

#[tokio::test]
async fn test_tool_validation_error() {
    let tool = MyTool::new();
    let params = MyToolParams {
        field: "".to_string(), // Invalid
    };

    let result = tool.execute(params).await;
    assert!(matches!(result, Err(Error::InvalidInput { .. })));
}

#[tokio::test]
async fn test_tool_not_found_error() {
    let tool = MyTool::new();
    let params = MyToolParams {
        field: "nonexistent".to_string(),
    };

    let result = tool.execute(params).await;
    assert!(matches!(result, Err(Error::NotFound(_))));
}
```

### 4. Create Property-Based Tests

Generate `tests/property/prop_tests.rs`:

```rust
use proptest::prelude::*;
use {crate}::tools::*;

proptest! {
    #[test]
    fn test_tool_handles_any_string(input in ".*") {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tool = MyTool::new();

        runtime.block_on(async {
            let params = MyToolParams { field: input.clone() };
            let result = tool.execute(params).await;

            // Should not panic, either Ok or Err
            assert!(result.is_ok() || result.is_err());
        });
    }

    #[test]
    fn test_calculation_commutative(
        a in -1000i32..1000,
        b in -1000i32..1000
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tool = CalculatorTool::new();

        runtime.block_on(async {
            let result1 = tool.add(a, b).await.unwrap();
            let result2 = tool.add(b, a).await.unwrap();
            assert_eq!(result1, result2);
        });
    }
}
```

### 5. Create Integration Tests

Generate `tests/integration/server_test.rs`:

```rust
use {crate}::{{config::AppConfig, service::McpService}};
use rmcp::prelude::*;

#[tokio::test]
async fn test_server_lifecycle() {
    // Create test config
    let config = AppConfig {{
        // test configuration
    }};

    // Create service
    let service = McpService::new(config).await.unwrap();

    // Test tool invocation
    let result = service.my_tool(MyToolParams {{
        field: "test".to_string(),
    }}).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_resources() {
    let config = AppConfig::default();
    let service = McpService::new(config).await.unwrap();

    let resources = service.list_resources().await.unwrap();
    assert!(!resources.is_empty());
}

#[tokio::test]
async fn test_fetch_resource() {
    let config = AppConfig::default();
    let service = McpService::new(config).await.unwrap();

    let content = service.fetch_resource("test://resource").await.unwrap();
    assert!(!content.text.is_none());
}

#[tokio::test]
async fn test_list_prompts() {
    let config = AppConfig::default();
    let service = McpService::new(config).await.unwrap();

    let prompts = service.list_prompts();
    assert!(!prompts.is_empty());
}
```

### 6. Create Protocol Tests

Generate `tests/protocol/jsonrpc_test.rs`:

```rust
use serde_json::json;

#[tokio::test]
async fn test_jsonrpc_tool_call() {
    let service = create_test_service().await;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "my_tool",
            "arguments": {
                "field": "test"
            }
        },
        "id": 1
    });

    let response = service.handle_request(request).await.unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn test_jsonrpc_error_handling() {
    let service = create_test_service().await;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "nonexistent_method",
        "params": {},
        "id": 1
    });

    let response = service.handle_request(request).await.unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
}
```

### 7. Create Database Tests (if applicable)

Generate `tests/integration/database_test.rs`:

```rust
use testcontainers::{{clients, images::postgres::Postgres}};
use sqlx::PgPool;

#[tokio::test]
async fn test_database_operations() {
    // Start test database
    let docker = clients::Cli::default();
    let postgres = docker.run(Postgres::default());

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );

    let pool = PgPool::connect(&connection_string).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    // Test service with real database
    let service = create_service_with_db(pool).await;

    // Test operations
    let result = service.create_record(/* params */).await;
    assert!(result.is_ok());

    let record = service.get_record("id").await;
    assert!(record.is_ok());
}
```

### 8. Create API Tests (if applicable)

Generate `tests/integration/api_test.rs`:

```rust
use wiremock::{{matchers::{{method, path}}, Mock, MockServer, ResponseTemplate}};

#[tokio::test]
async fn test_external_api_call() {
    // Start mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "value"
        })))
        .mount(&mock_server)
        .await;

    // Create service with mock URL
    let service = create_service_with_api_url(mock_server.uri()).await;

    // Test API call
    let result = service.fetch_data().await;
    assert!(result.is_ok());
}
```

### 9. Create Performance Tests

Generate `tests/performance/bench_test.rs`:

```rust
use criterion::{{black_box, criterion_group, criterion_main, Criterion}};

fn benchmark_tool(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let tool = MyTool::new();

    c.bench_function("tool_execution", |b| {
        b.iter(|| {
            runtime.block_on(async {
                tool.execute(black_box(MyToolParams {
                    field: "test".to_string(),
                }))
                .await
                .unwrap()
            })
        })
    });
}

criterion_group!(benches, benchmark_tool);
criterion_main!(benches);
```

### 10. Create Test Utilities

Generate `tests/common/mod.rs`:

```rust
use {crate}::{{config::AppConfig, service::McpService}};

pub async fn create_test_service() -> McpService {
    let config = AppConfig {
        // test configuration
    };

    McpService::new(config).await.unwrap()
}

pub fn create_test_config() -> AppConfig {
    AppConfig {
        // test values
    }
}

pub async fn setup_test_database() -> PgPool {
    // Database setup logic
}
```

### 11. Run All Tests

Create test script `scripts/test.sh`:

```bash
#!/bin/bash
set -e

echo "Running all tests..."

# Unit tests
echo "Running unit tests..."
cargo test --lib

# Integration tests
echo "Running integration tests..."
cargo test --test '*'

# Doc tests
echo "Running doc tests..."
cargo test --doc

# With all features
echo "Running tests with all features..."
cargo test --all-features

# Property-based tests (longer)
echo "Running property tests..."
PROPTEST_CASES=1000 cargo test --test prop_tests

echo "All tests passed!"
```

### 12. Set Up CI Testing

Update `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Run tests
      run: cargo test --all-features --verbose

    - name: Run integration tests
      run: cargo test --test '*' --verbose

    - name: Generate coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml

    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

### 13. Create Test Documentation

Generate `tests/README.md`:

```markdown
# Testing Guide

## Running Tests

\```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
\```

## Test Organization

- `tests/unit/` - Unit tests
- `tests/integration/` - Integration tests
- `tests/protocol/` - Protocol compliance tests
- `tests/performance/` - Performance benchmarks
- `tests/common/` - Shared test utilities

## Writing Tests

See examples in each directory.
```

## After Setup

```
✅ Test infrastructure set up successfully!

## Test Suite Overview:
- Unit tests for tools/resources/prompts
- Integration tests for full server
- Protocol compliance tests
- Property-based tests
- Performance benchmarks

## Run Tests:

\```bash
# All tests
cargo test

# With coverage
cargo tarpaulin

# Watch mode
cargo watch -x test

# Benchmarks
cargo bench
\```

## Next Steps:

1. **Run tests:** `cargo test`
2. **Check coverage:** `cargo tarpaulin --out Html`
3. **Add more tests:** As you add features
4. **Set up CI:** Tests run on every push

## Tips:

- Write tests before implementing features (TDD)
- Aim for >80% code coverage
- Test error cases, not just happy path
- Use property-based tests for algorithms
- Mock external dependencies

Happy testing! 🧪
```
