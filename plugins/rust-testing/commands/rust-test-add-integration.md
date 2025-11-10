---
description: Create integration tests in the tests/ directory
---

You are helping create integration tests for a Rust project in the tests/ directory.

## Your Task

Set up integration test infrastructure and create comprehensive integration tests that test the public API.

## Steps

1. **Ask for Test Details**

   Ask the user:
   - What feature/module to integration test?
   - Do you need database/HTTP mocking?
   - Should we use testcontainers for real databases?

2. **Create Integration Test File**

   Create a new file in `tests/` directory:

   ```
   tests/
   ├── [feature]_integration.rs
   └── common/
       └── mod.rs  # Shared test utilities
   ```

3. **Set Up Common Utilities**

   Create `tests/common/mod.rs`:

   ```rust
   // tests/common/mod.rs
   #![allow(dead_code)]

   use my_crate::*;

   pub fn setup() {
       // Common setup logic
   }

   pub fn teardown() {
       // Common cleanup logic
   }
   ```

4. **Generate Integration Test**

   Create the integration test file:

   ```rust
   // tests/[feature]_integration.rs
   use my_crate::*;

   mod common;

   #[test]
   fn test_[feature]_complete_workflow() {
       common::setup();

       // Arrange
       let app = create_test_application();

       // Act
       let result = app.execute_workflow();

       // Assert
       assert!(result.is_ok());

       common::teardown();
   }

   fn create_test_application() -> Application {
       // Setup test instance with test configuration
       Application::new(test_config())
   }

   fn test_config() -> Config {
       Config {
           database_url: "postgres://localhost/test_db".to_string(),
           // ... other test config
       }
   }
   ```

5. **Add Async Integration Tests**

   For async applications:

   ```rust
   #[tokio::test]
   async fn test_async_integration() {
       let app = setup_test_app().await;

       let result = app.process_request(test_request()).await;

       assert!(result.is_ok());
   }

   async fn setup_test_app() -> Application {
       Application::new(test_config()).await.unwrap()
   }
   ```

6. **Add Database Integration Tests**

   Using testcontainers:

   ```rust
   use testcontainers::{clients, images::postgres::Postgres};
   use sqlx::PgPool;

   #[tokio::test]
   async fn test_with_real_database() {
       let docker = clients::Cli::default();
       let postgres = docker.run(Postgres::default());

       let connection_string = format!(
           "postgres://postgres@localhost:{}/postgres",
           postgres.get_host_port_ipv4(5432)
       );

       let pool = PgPool::connect(&connection_string).await.unwrap();

       // Run migrations
       sqlx::migrate!("./migrations")
           .run(&pool)
           .await
           .unwrap();

       // Now test with real database
       let repo = PostgresRepository::new(pool);
       let service = MyService::new(repo);

       let result = service.create_item("test").await;

       assert!(result.is_ok());
   }
   ```

   Or create a helper in common:

   ```rust
   // tests/common/mod.rs
   use sqlx::PgPool;
   use testcontainers::*;

   pub async fn setup_test_database() -> PgPool {
       let docker = clients::Cli::default();
       let postgres = docker.run(images::postgres::Postgres::default());

       let url = format!(
           "postgres://postgres@localhost:{}/postgres",
           postgres.get_host_port_ipv4(5432)
       );

       let pool = PgPool::connect(&url).await.unwrap();
       sqlx::migrate!().run(&pool).await.unwrap();

       pool
   }

   // tests/database_integration.rs
   mod common;

   #[tokio::test]
   async fn test_repository() {
       let pool = common::setup_test_database().await;
       let repo = MyRepository::new(pool);

       // Test repository operations
   }
   ```

7. **Add HTTP API Integration Tests**

   For testing HTTP APIs:

   ```rust
   use axum::body::Body;
   use axum::http::{Request, StatusCode};
   use tower::ServiceExt;

   #[tokio::test]
   async fn test_api_endpoint() {
       let app = create_test_app();

       let response = app
           .oneshot(
               Request::builder()
                   .uri("/api/users")
                   .method("GET")
                   .body(Body::empty())
                   .unwrap(),
           )
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::OK);

       let body = hyper::body::to_bytes(response.into_body())
           .await
           .unwrap();

       let users: Vec<User> = serde_json::from_slice(&body).unwrap();
       assert!(!users.is_empty());
   }
   ```

   Or using reqwest for full HTTP testing:

   ```rust
   #[tokio::test]
   async fn test_api_full_request() {
       let server = spawn_test_server().await;

       let client = reqwest::Client::new();
       let response = client
           .get(format!("{}/api/users", server.url()))
           .send()
           .await
           .unwrap();

       assert_eq!(response.status(), 200);

       let users: Vec<User> = response.json().await.unwrap();
       assert!(!users.is_empty());
   }

   async fn spawn_test_server() -> TestServer {
       // Start server on random port for testing
       TestServer::spawn().await
   }
   ```

8. **Add HTTP Mocking with wiremock**

   For testing external API calls:

   ```rust
   use wiremock::{MockServer, Mock, ResponseTemplate};
   use wiremock::matchers::{method, path};

   #[tokio::test]
   async fn test_external_api_integration() {
       let mock_server = MockServer::start().await;

       Mock::given(method("GET"))
           .and(path("/external/api"))
           .respond_with(ResponseTemplate::new(200).set_body_json(
               serde_json::json!({
                   "status": "ok",
                   "data": "test"
               })
           ))
           .mount(&mock_server)
           .await;

       let client = ExternalApiClient::new(&mock_server.uri());
       let result = client.fetch_data().await.unwrap();

       assert_eq!(result.status, "ok");
   }
   ```

9. **Add Multi-Step Workflow Tests**

   Test complete user workflows:

   ```rust
   #[tokio::test]
   async fn test_complete_user_workflow() {
       let app = setup_test_app().await;

       // Step 1: Create user
       let user_id = app
           .create_user("test@example.com")
           .await
           .unwrap();

       // Step 2: Retrieve user
       let user = app
           .get_user(&user_id)
           .await
           .unwrap();

       assert_eq!(user.email, "test@example.com");

       // Step 3: Update user
       app.update_user(&user_id, "new@example.com")
           .await
           .unwrap();

       // Step 4: Verify update
       let updated = app
           .get_user(&user_id)
           .await
           .unwrap();

       assert_eq!(updated.email, "new@example.com");

       // Step 5: Delete user
       app.delete_user(&user_id)
           .await
           .unwrap();

       // Step 6: Verify deletion
       let result = app.get_user(&user_id).await;
       assert!(result.is_err());
   }
   ```

10. **Update Dev Dependencies**

    Ensure required dependencies are in Cargo.toml:

    ```toml
    [dev-dependencies]
    tokio = { version = "1", features = ["full", "test-util"] }
    testcontainers = "0.15"
    wiremock = "0.6"
    reqwest = { version = "0.11", features = ["json"] }
    ```

11. **Provide Summary**

    ```
    ✅ Integration tests created successfully!

    ## Files Created:
    - `tests/[feature]_integration.rs` - Main integration test file
    - `tests/common/mod.rs` - Shared test utilities

    ## Tests Added:
    - test_[feature]_complete_workflow
    - test_database_operations
    - test_api_endpoints
    - test_external_api_integration

    ## Infrastructure:
    - Database test setup with testcontainers
    - HTTP mocking with wiremock
    - Test configuration helpers

    ## Dependencies Added:
    [List of dev dependencies]

    ## Running Integration Tests:

    ```bash
    # Run all integration tests
    cargo test --test [feature]_integration

    # Run specific test
    cargo test test_complete_workflow

    # Run with output
    cargo test --test [feature]_integration -- --nocapture

    # Run all integration tests
    cargo test --tests
    ```

    ## Next Steps:
    1. Review and customize test cases
    2. Add more workflow scenarios
    3. Run tests: `cargo test --tests`
    4. Check if you need more test infrastructure
    ```

## Integration Test Patterns

### Setup/Teardown Pattern
```rust
struct TestContext {
    pool: PgPool,
    // Other resources
}

impl TestContext {
    async fn new() -> Self {
        // Setup
        Self {
            pool: setup_database().await,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup
    }
}

#[tokio::test]
async fn test_with_context() {
    let ctx = TestContext::new().await;
    // Use ctx.pool
}
```

### Parallel Test Isolation
```rust
#[tokio::test]
async fn test_isolated_1() {
    let db = create_unique_test_db("test1").await;
    // Each test gets its own database
}

#[tokio::test]
async fn test_isolated_2() {
    let db = create_unique_test_db("test2").await;
    // Runs in parallel without interference
}
```

## Important Notes

- Integration tests are in separate crates from src/
- Each file in tests/ is a separate binary
- Common code goes in tests/common/ (not tests/common.rs)
- Use real implementations when possible
- Mock external services
- Clean up resources after tests
- Tests should be independent and runnable in any order

## After Completion

Ask the user:
1. Did the integration tests pass?
2. Do you need more test scenarios?
3. Should we add performance/load tests?
4. Do you want to set up CI/CD for these tests?
