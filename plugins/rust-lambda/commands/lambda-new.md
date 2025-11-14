---
description: Create a new Rust Lambda function project with cargo-lambda
---

You are helping the user create a new Rust Lambda function project using cargo-lambda.

## Your Task

Guide the user through creating a new Lambda function project with the following steps:

1. **Check if cargo-lambda is installed**:
   - Run `cargo lambda --version` to verify installation
   - If not installed, provide installation instructions:
     ```bash
     # Via Homebrew (macOS/Linux)
     brew tap cargo-lambda/cargo-lambda
     brew install cargo-lambda

     # Via pip
     pip install cargo-lambda

     # From source
     cargo install cargo-lambda
     ```

2. **Ask for project details** (if not provided):
   - Function name
   - Event type (API Gateway, S3, SQS, EventBridge, custom, or basic)
   - Workload type (IO-intensive, compute-intensive, or mixed)

3. **Create the project**:
   ```bash
   cargo lambda new <function-name>
   ```
   Or with event type:
   ```bash
   cargo lambda new <function-name> --event-type <type>
   ```

4. **Set up the basic structure** based on workload type:

   **For IO-intensive** (default):
   - Use full async/await
   - Add dependencies: tokio, reqwest, aws-sdk crates as needed
   - Example handler with concurrent operations

   **For compute-intensive**:
   - Add rayon to Cargo.toml
   - Example using spawn_blocking + rayon
   - Async only at boundaries

   **For mixed**:
   - Both patterns combined
   - Async for IO, sync for compute

5. **Add essential dependencies** to Cargo.toml:
   ```toml
   [dependencies]
   lambda_runtime = "0.13"
   tokio = { version = "1", features = ["macros"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   anyhow = "1"
   thiserror = "1"
   tracing = { version = "0.1", features = ["log"] }
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }

   # Add based on workload:
   # For compute: rayon = "1.10"
   # For HTTP: reqwest = { version = "0.12", features = ["json"] }
   # For AWS services: aws-sdk-* crates
   ```

6. **Configure release profile** for optimization:
   ```toml
   [profile.release]
   opt-level = 'z'     # Optimize for size
   lto = true          # Link-time optimization
   codegen-units = 1   # Better optimization
   strip = true        # Remove debug symbols
   panic = 'abort'     # Smaller panic handler
   ```

7. **Create example handler** matching the selected pattern

8. **Test locally**:
   ```bash
   cd <function-name>
   cargo lambda watch

   # In another terminal:
   cargo lambda invoke --data-ascii '{"key": "value"}'
   ```

## Event Type Templates

Provide appropriate code based on event type:

- **basic**: Simple JSON request/response
- **apigw**: API Gateway proxy request/response
- **s3**: S3 event processing
- **sqs**: SQS message processing
- **eventbridge**: EventBridge/CloudWatch Events

## Example Handlers

Show the user a complete working example for their chosen pattern:

### IO-Intensive Example
```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    user_ids: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    count: usize,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Concurrent async operations
    let futures = event.payload.user_ids
        .into_iter()
        .map(|id| fetch_user_data(&id));

    let results = futures::future::try_join_all(futures).await?;

    Ok(Response { count: results.len() })
}

async fn fetch_user_data(id: &str) -> Result<(), Error> {
    // Async IO operation
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
```

### Compute-Intensive Example
```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Deserialize)]
struct Request {
    numbers: Vec<i64>,
}

#[derive(Serialize)]
struct Response {
    results: Vec<i64>,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let numbers = event.payload.numbers;

    // CPU work in spawn_blocking with Rayon
    let results = task::spawn_blocking(move || {
        numbers
            .par_iter()
            .map(|&n| expensive_computation(n))
            .collect::<Vec<_>>()
    })
    .await?;

    Ok(Response { results })
}

fn expensive_computation(n: i64) -> i64 {
    // CPU-intensive work
    (0..1000).fold(n, |acc, _| acc.wrapping_mul(31))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
```

## Next Steps

After creating the project, suggest:
1. Review and customize the handler
2. Add tests
3. Test locally with `cargo lambda watch`
4. Build with `/lambda-build`
5. Deploy with `/lambda-deploy`

Be helpful and guide the user through any questions or issues they encounter.
