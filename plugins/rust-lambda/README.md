# Rust Lambda Plugin

A comprehensive plugin for building, deploying, and optimizing AWS Lambda functions using Rust with cargo-lambda and the lambda-runtime library.

## Overview

This plugin provides guidance and best practices for developing AWS Lambda functions in Rust, covering everything from initial setup to advanced optimization strategies. Learn how to build both IO-intensive async lambdas and compute-intensive sync lambdas.

## Features

### 🚀 Development Commands

#### `/lambda-new`
Create a new Rust Lambda function project.

**Features**:
- Initialize project with cargo-lambda
- Set up proper project structure
- Configure runtime and dependencies
- Generate basic handler templates

#### `/lambda-build`
Build Lambda function for deployment.

**Features**:
- Cross-compile for AWS Lambda runtime
- Optimize binary size
- Support for ARM64 and x86_64
- Release builds with proper flags

#### `/lambda-deploy`
Deploy Lambda function to AWS.

**Features**:
- Deploy with cargo-lambda
- Update existing functions
- Configure environment variables
- Set memory and timeout

#### `/lambda-github-actions`
Set up GitHub Actions CI/CD for Lambda deployment.

**Features**:
- Complete workflow templates
- AWS credentials configuration
- Build and deploy automation
- Multi-architecture support

#### `/lambda-optimize-io`
Optimize Lambda for IO-intensive workloads.

**Features**:
- Async/await best practices
- Concurrent request handling
- Connection pooling strategies
- Efficient async runtime usage

#### `/lambda-optimize-compute`
Optimize Lambda for compute-intensive workloads.

**Features**:
- Rayon parallel processing
- spawn_blocking patterns
- Mixed async/sync strategies
- Thread pool configuration

#### `/lambda-iac`
Set up Infrastructure as Code with SAM, Terraform, or CDK.

**Features**:
- SAM templates for serverless deployments
- Terraform modules for Lambda infrastructure
- CDK examples in TypeScript/Python
- LocalStack integration for local testing

#### `/lambda-observability`
Implement advanced observability with OpenTelemetry and X-Ray.

**Features**:
- AWS X-Ray integration with xray-lite
- OpenTelemetry setup with lambda-otel-lite
- Structured logging with tracing
- Distributed tracing patterns
- CloudWatch Logs Insights queries

#### `/lambda-secrets`
Manage secrets and configuration securely.

**Features**:
- AWS Secrets Manager integration
- Parameter Store usage patterns
- Parameters and Secrets Lambda Extension
- Caching strategies
- Environment-specific configuration

#### `/lambda-function-urls`
Set up Lambda Function URLs and streaming responses.

**Features**:
- Direct HTTPS endpoints without API Gateway
- REST API patterns with lambda_http
- Response streaming for large payloads (up to 20MB)
- CORS configuration
- Authentication patterns

#### `/lambda-cost`
Deep dive into cost optimization strategies.

**Features**:
- AWS Lambda Power Tuning integration
- Memory vs CPU allocation strategies
- Rust-specific optimizations
- ARM64 savings (20% cheaper)
- Cost monitoring and alerts

#### `/lambda-advanced`
Advanced topics: extensions, containers, and more.

**Features**:
- Custom Lambda Extensions in Rust
- Container image deployments
- VPC configuration
- Reserved concurrency
- Blue/green deployments
- Multi-region strategies

### 🤖 Lambda Expert Agent

A specialized agent (`rust-lambda-expert`) for Lambda development.

**Capabilities**:
- Design Lambda architectures
- Optimize for cold start performance
- Review Lambda code for best practices
- Debug Lambda-specific issues
- Guide on async vs sync patterns

## Getting Started

### Prerequisites

1. **Install Rust** (1.75.0 or later recommended):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install cargo-lambda**:
   ```bash
   # macOS/Linux
   brew tap cargo-lambda/cargo-lambda
   brew install cargo-lambda

   # Or using pip
   pip install cargo-lambda

   # Or from source
   cargo install cargo-lambda
   ```

3. **Install Zig** (for cross-compilation):
   ```bash
   # macOS
   brew install zig

   # Linux
   # Download from https://ziglang.org/download/
   ```

### Quick Start

1. **Create a new Lambda project**:
   ```bash
   cargo lambda new my-lambda-function
   cd my-lambda-function
   ```

2. **Test locally**:
   ```bash
   cargo lambda watch

   # In another terminal
   cargo lambda invoke --data-ascii '{"key": "value"}'
   ```

3. **Build for Lambda**:
   ```bash
   cargo lambda build --release
   ```

4. **Deploy to AWS**:
   ```bash
   cargo lambda deploy
   ```

## Core Dependencies

Add these to your `Cargo.toml`:

```toml
[package]
name = "my-lambda"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core Lambda runtime
lambda_runtime = "0.13"
tokio = { version = "1", features = ["macros"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1"
thiserror = "1"

# Tracing/Logging
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# For compute-intensive tasks
rayon = "1.10"

# For async HTTP requests (IO-intensive)
reqwest = { version = "0.12", features = ["json"] }

[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Reduce number of codegen units
strip = true        # Strip symbols from binary
```

## Lambda Patterns

### Pattern 1: IO-Intensive Lambda (Async)

For lambdas that make many API calls, database queries, or external requests:

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
struct Request {
    user_ids: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    users: Vec<User>,
}

#[derive(Serialize)]
struct User {
    id: String,
    name: String,
}

/// IO-intensive lambda: maximize async concurrency
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Processing {} user IDs", event.payload.user_ids.len());

    // Process requests concurrently using async
    let user_futures = event.payload.user_ids
        .into_iter()
        .map(|user_id| async move {
            // Each request is async and non-blocking
            fetch_user_from_api(&user_id).await
        });

    // Wait for all requests to complete concurrently
    let users = futures::future::try_join_all(user_futures).await?;

    Ok(Response { users })
}

async fn fetch_user_from_api(user_id: &str) -> Result<User, Error> {
    // Async HTTP request - doesn't block the runtime
    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://api.example.com/users/{}", user_id))
        .send()
        .await?
        .json::<User>()
        .await?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
```

**Key Points**:
- Use async throughout for IO operations
- Leverage `futures::future::try_join_all` for concurrency
- Don't block the async runtime
- Efficient for high-latency, low-CPU operations

### Pattern 2: Compute-Intensive Lambda (Sync with Rayon)

For lambdas that do CPU-heavy processing (image processing, data transformation, etc.):

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::info;

#[derive(Deserialize)]
struct Request {
    numbers: Vec<i64>,
}

#[derive(Serialize)]
struct Response {
    results: Vec<i64>,
}

/// Compute-intensive lambda: use sync processing with Rayon
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Processing {} numbers", event.payload.numbers.len());

    let numbers = event.payload.numbers;

    // Async at the boundaries: spawn_blocking for CPU work
    let results = task::spawn_blocking(move || {
        // Use Rayon for parallel CPU-intensive work
        numbers
            .par_iter()
            .map(|&num| expensive_computation(num))
            .collect::<Vec<_>>()
    })
    .await?;

    Ok(Response { results })
}

/// CPU-intensive synchronous function
fn expensive_computation(n: i64) -> i64 {
    // Simulate expensive computation
    // (e.g., cryptographic hashing, image processing, etc.)
    (0..1000).fold(n, |acc, _| {
        acc.wrapping_mul(31).wrapping_add(17)
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
```

**Key Points**:
- Use `tokio::task::spawn_blocking` to run sync code
- Use Rayon for parallel CPU work within the blocking task
- Keep async only at input/output boundaries
- Efficient for CPU-bound operations

### Pattern 3: Mixed Workload Lambda

For lambdas that combine IO and compute:

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::info;

#[derive(Deserialize)]
struct Request {
    image_urls: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    processed_count: usize,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Processing {} images", event.payload.image_urls.len());

    // Step 1: Async IO - Download all images concurrently
    let download_futures = event.payload.image_urls
        .into_iter()
        .map(|url| async move {
            let response = reqwest::get(&url).await?;
            let bytes = response.bytes().await?;
            Ok::<_, Error>(bytes.to_vec())
        });

    let images = futures::future::try_join_all(download_futures).await?;

    // Step 2: Sync compute - Process images in parallel with Rayon
    let processed = task::spawn_blocking(move || {
        images
            .par_iter()
            .map(|image_data| {
                // CPU-intensive image processing
                process_image(image_data)
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .await??;

    // Step 3: Async IO - Upload results concurrently
    let upload_futures = processed
        .into_iter()
        .enumerate()
        .map(|(i, data)| async move {
            upload_to_s3(&format!("processed-{}.jpg", i), &data).await
        });

    futures::future::try_join_all(upload_futures).await?;

    Ok(Response {
        processed_count: event.payload.image_urls.len(),
    })
}

fn process_image(data: &[u8]) -> Result<Vec<u8>, Error> {
    // CPU-intensive synchronous processing
    // (e.g., resize, filter, compress)
    Ok(data.to_vec()) // Placeholder
}

async fn upload_to_s3(key: &str, data: &[u8]) -> Result<(), Error> {
    // Async S3 upload
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
```

**Key Points**:
- Async for all IO (downloads, uploads, API calls)
- Sync with Rayon for CPU-intensive work
- Use `spawn_blocking` to transition between async and sync
- Maximizes both IO concurrency and CPU utilization

## GitHub Actions CI/CD

### Complete Workflow Example

Create `.github/workflows/deploy-lambda.yml`:

```yaml
name: Deploy Lambda

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    permissions:
      id-token: write   # Required for AWS OIDC
      contents: read

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Zig
        uses: goto-bus-stop/setup-zig@v2
        with:
          version: 0.11.0

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Run tests
        run: cargo test --verbose

      - name: Build Lambda (x86_64)
        run: cargo lambda build --release --output-format zip

      - name: Build Lambda (ARM64)
        run: cargo lambda build --release --arm64 --output-format zip

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: us-east-1

      - name: Deploy to AWS Lambda
        if: github.ref == 'refs/heads/main'
        run: |
          cargo lambda deploy \
            --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE }} \
            --region us-east-1 \
            my-lambda-function
```

### Using AWS Credentials (Alternative)

If not using OIDC, you can use AWS credentials directly:

```yaml
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1
```

### Multi-Function Deployment

For deploying multiple Lambda functions:

```yaml
      - name: Deploy all Lambda functions
        if: github.ref == 'refs/heads/main'
        run: |
          for func in $(cargo lambda list); do
            echo "Deploying $func..."
            cargo lambda deploy --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE }} $func
          done
```

## Performance Optimization

### Cold Start Optimization

1. **Minimize binary size**:
   ```toml
   [profile.release]
   opt-level = 'z'       # Optimize for size
   lto = true            # Link Time Optimization
   codegen-units = 1     # Better optimization
   strip = true          # Remove debug symbols
   panic = 'abort'       # Smaller binary
   ```

2. **Use function-level initialization**:
   ```rust
   use std::sync::OnceLock;

   static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

   async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
       // Initialize once, reuse across invocations
       let client = HTTP_CLIENT.get_or_init(|| {
           reqwest::Client::builder()
               .timeout(Duration::from_secs(10))
               .build()
               .unwrap()
       });

       // Use client...
   }
   ```

3. **ARM64 for better performance**:
   ```bash
   cargo lambda build --release --arm64
   ```

### Memory and Timeout Configuration

```bash
# Set memory (more memory = more CPU)
cargo lambda deploy --memory 512

# Set timeout
cargo lambda deploy --timeout 30

# Set environment variables
cargo lambda deploy --env-var KEY1=value1 --env-var KEY2=value2
```

## Tracing and Logging

### Setup Tracing

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing for CloudWatch
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!(request_id = %event.context.request_id, "Processing request");

    // Your logic here

    Ok(response)
}
```

### Environment Variables for Log Level

Set `RUST_LOG` environment variable when deploying:

```bash
cargo lambda deploy --env-var RUST_LOG=debug
```

## Error Handling Best Practices

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum LambdaError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Lambda runtime requires errors to implement std::error::Error
impl From<LambdaError> for lambda_runtime::Error {
    fn from(err: LambdaError) -> Self {
        Box::new(err)
    }
}
```

## Common Event Types

### API Gateway Event

```rust
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::StatusCode;

async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let body = event.payload.body.unwrap_or_default();

    Ok(ApiGatewayProxyResponse {
        status_code: StatusCode::OK.as_u16() as i64,
        body: Some(format!("Received: {}", body)),
        ..Default::default()
    })
}
```

### S3 Event

```rust
use aws_lambda_events::event::s3::S3Event;

async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    for record in event.payload.records {
        let bucket = record.s3.bucket.name.unwrap();
        let key = record.s3.object.key.unwrap();

        info!("Processing s3://{}/{}", bucket, key);
    }

    Ok(())
}
```

### SQS Event

```rust
use aws_lambda_events::event::sqs::SqsEvent;

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    for record in event.payload.records {
        let body = record.body.unwrap_or_default();
        info!("Processing message: {}", body);

        // Process message...
    }

    Ok(())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler() {
        let event = LambdaEvent::new(
            Request { user_ids: vec!["123".to_string()] },
            lambda_runtime::Context::default(),
        );

        let result = function_handler(event).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```bash
# Start local Lambda emulator
cargo lambda watch

# Invoke with test data
cargo lambda invoke --data-file tests/fixtures/event.json
```

## Best Practices

1. **Use Async for IO**: Network calls, database queries, file operations
2. **Use Sync for Compute**: CPU-intensive calculations, data processing
3. **Reuse Connections**: Initialize clients once using `OnceLock` or `lazy_static`
4. **Optimize Binary Size**: Use release profile optimizations
5. **Structure Errors**: Use `thiserror` for clear error types
6. **Add Tracing**: Use structured logging for observability
7. **Test Locally**: Use `cargo lambda watch` for fast iteration
8. **Monitor Cold Starts**: Use ARM64 and size optimizations
9. **Set Appropriate Memory**: More memory = more CPU (and cost)
10. **Use spawn_blocking**: For CPU work in async context

## Troubleshooting

### Binary too large
- Enable `strip = true` in release profile
- Use `opt-level = 'z'` for size optimization
- Check dependencies with `cargo tree`

### Cold starts too slow
- Build for ARM64 architecture
- Reduce binary size
- Move initialization outside handler
- Consider Lambda SnapStart (Java) or provisioned concurrency

### Out of memory errors
- Increase Lambda memory allocation
- Use streaming for large files
- Profile memory usage locally

### Timeout errors
- Increase timeout setting
- Optimize async concurrency
- Use spawn_blocking for CPU work
- Profile with `cargo flamegraph`

## Resources

- [cargo-lambda Documentation](https://www.cargo-lambda.info/)
- [lambda_runtime Crate](https://docs.rs/lambda_runtime)
- [AWS Lambda Rust Guide](https://docs.aws.amazon.com/lambda/latest/dg/lambda-rust.html)
- [AWS Lambda Events](https://docs.rs/aws_lambda_events)
- [Tokio Documentation](https://tokio.rs/)
- [Rayon Documentation](https://docs.rs/rayon)

## Installation

```bash
cp -r plugins/rust-lambda /path/to/your/project/plugins/
```

Register in marketplace.json:
```json
{
  "plugins": [{
    "name": "rust-lambda",
    "source": "./plugins/rust-lambda",
    "description": "Build and deploy AWS Lambda functions with Rust",
    "version": "1.0.0"
  }]
}
```

## Usage

### Quick Commands

```bash
# Create new Lambda
/lambda-new

# Build for deployment
/lambda-build

# Deploy to AWS
/lambda-deploy

# Setup CI/CD
/lambda-github-actions

# Optimize for IO workloads
/lambda-optimize-io

# Optimize for compute workloads
/lambda-optimize-compute
```

### Get Expert Help

```
Ask rust-lambda-expert to help optimize my Lambda for cold starts
Ask rust-lambda-expert to review my Lambda architecture
```

---

**Build fast, efficient Lambda functions with Rust** 🦀⚡
