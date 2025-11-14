# Rust Lambda Plugin - Context

## Purpose

This plugin helps developers build, deploy, and optimize AWS Lambda functions using Rust. It provides comprehensive guidance on using cargo-lambda, the lambda_runtime crate, and best practices for both IO-intensive and compute-intensive workloads.

## Core Technologies

### cargo-lambda

**cargo-lambda** is a Cargo subcommand to help you work with AWS Lambda. It provides several commands to streamline Lambda development:

- `cargo lambda new` - Create new Lambda projects with templates
- `cargo lambda build` - Build Lambda functions for deployment (cross-compilation)
- `cargo lambda watch` - Run Lambda locally with hot-reload
- `cargo lambda invoke` - Test Lambda locally with events
- `cargo lambda deploy` - Deploy to AWS Lambda

**Key Features**:
- Cross-compilation from macOS, Windows, Linux to Linux (Lambda runtime)
- Powered by Zig for cross-compilation
- Support for both x86_64 and ARM64 (Graviton2)
- Automatic binary optimization
- Local testing and development

**Installation**:
```bash
# Via Homebrew (macOS/Linux)
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda

# Via pip
pip install cargo-lambda

# From source
cargo install cargo-lambda
```

**GitHub Repository**: https://github.com/cargo-lambda/cargo-lambda

### lambda_runtime

The **lambda_runtime** crate is the official Rust runtime for AWS Lambda, maintained by AWS Labs.

**Crate**: `lambda_runtime = "0.13"`
**Docs**: https://docs.rs/lambda_runtime
**GitHub**: https://github.com/awslabs/aws-lambda-rust-runtime

**Key Components**:

1. **`LambdaEvent<T>`**: Wrapper around the event payload with context
   ```rust
   pub struct LambdaEvent<T> {
       pub payload: T,      // The actual event data
       pub context: Context, // Lambda execution context
   }
   ```

2. **`Context`**: Lambda execution metadata
   ```rust
   pub struct Context {
       pub request_id: String,
       pub deadline: u64,
       pub invoked_function_arn: String,
       pub env_config: Config,
       // ... more fields
   }
   ```

3. **`run()` function**: Starts the Lambda runtime
   ```rust
   use lambda_runtime::{run, service_fn, Error};

   #[tokio::main]
   async fn main() -> Result<(), Error> {
       run(service_fn(my_handler)).await
   }
   ```

4. **`service_fn()`**: Converts async function to Lambda service
   ```rust
   async fn my_handler(event: LambdaEvent<MyEvent>) -> Result<Response, Error> {
       // Handler logic
   }
   ```

### Tokio Runtime

Lambda functions use Tokio as the async runtime. The `#[tokio::main]` macro sets up the runtime:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}
```

**Important**: Lambda's execution model works well with async because:
- Lambdas handle one event at a time per container
- Async allows concurrent operations within a single event
- Non-blocking IO reduces execution time and cost

## Lambda Execution Model

### How Lambda Works

1. **Cold Start**: Lambda creates new container, loads runtime, initializes code
2. **Warm Start**: Reuses existing container for subsequent invocations
3. **Single Event**: Each container processes one event at a time
4. **Concurrency**: Multiple containers handle concurrent requests

### Implications for Rust

- **Initialization**: Code outside handler runs once per container (cold start)
- **Handler**: Runs for each invocation
- **State**: Can maintain state between invocations in same container
- **Concurrency**: Within single event, use async for concurrent operations

```rust
use std::sync::OnceLock;

// Initialized once per container (cold start)
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Reused across invocations in same container
    let client = HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
    });

    // Per-invocation logic
    let data = client.get("https://api.example.com/data").await?;
    Ok(Response { data })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Runs once per container
    tracing_subscriber::fmt().init();

    run(service_fn(function_handler)).await
}
```

## Async vs Sync: When to Use Each

### Async (IO-Intensive Workloads)

**Use when**:
- Making HTTP/API calls
- Database queries
- File IO (especially S3, DynamoDB)
- Message queue operations
- Multiple independent IO operations

**Why async?**:
- Non-blocking: While waiting for IO, other work can proceed
- Concurrent: Handle multiple IO operations simultaneously
- Efficient: No thread per operation overhead
- Cost-effective: Reduce Lambda execution time

**Pattern**:
```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // All IO operations are async

    // Concurrent API calls
    let (user, posts, comments) = tokio::try_join!(
        fetch_user(user_id),
        fetch_posts(user_id),
        fetch_comments(user_id),
    )?;

    // Concurrent database queries
    let futures = ids.iter().map(|id| query_database(id));
    let results = futures::future::try_join_all(futures).await?;

    Ok(Response { user, posts, comments, results })
}
```

**Key Libraries**:
- `tokio` - Async runtime
- `reqwest` - Async HTTP client
- `aws-sdk-*` - AWS SDKs (all async)
- `sqlx` - Async SQL
- `futures` - Async utilities

### Sync (Compute-Intensive Workloads)

**Use when**:
- CPU-heavy calculations
- Data processing (parsing, transformation)
- Image/video processing
- Encryption/hashing
- Parallel data processing

**Why sync?**:
- No async overhead for CPU work
- Rayon provides efficient parallelism
- Better CPU utilization for compute tasks
- Simpler mental model for CPU-bound code

**Pattern**:
```rust
use tokio::task;
use rayon::prelude::*;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let data = event.payload.data;

    // Move CPU work to blocking thread pool
    let results = task::spawn_blocking(move || {
        // Use Rayon for parallel computation
        data.par_iter()
            .map(|item| expensive_computation(item))
            .collect::<Vec<_>>()
    })
    .await?;

    Ok(Response { results })
}

// Pure CPU work - synchronous
fn expensive_computation(data: &Data) -> Result {
    // CPU-intensive work here
}
```

**Key Libraries**:
- `rayon` - Data parallelism
- `tokio::task::spawn_blocking` - Run sync code in async context

### spawn_blocking vs Rayon

#### tokio::task::spawn_blocking

**Purpose**: Run blocking/sync code without blocking async runtime

**Thread Pool**:
- Dedicated blocking thread pool (separate from async runtime)
- Default: 512 threads max
- Good for: Blocking IO, legacy sync APIs

**Usage**:
```rust
let result = task::spawn_blocking(|| {
    // Blocking operation
    std::fs::read_to_string("file.txt")
})
.await?;
```

**Characteristics**:
- Each task gets one thread
- Good for blocking IO
- NOT optimized for CPU-bound work
- Can spawn many tasks (up to 512)

#### Rayon

**Purpose**: Efficient data parallelism for CPU-bound work

**Thread Pool**:
- Fixed size based on CPU cores
- Default: Number of logical CPUs
- Good for: CPU-intensive parallel processing

**Usage**:
```rust
use rayon::prelude::*;

let results: Vec<_> = data
    .par_iter()
    .map(|item| cpu_intensive_work(item))
    .collect();
```

**Characteristics**:
- Work-stealing scheduler
- Optimized for CPU parallelism
- Limited threads (matches CPU cores)
- Best for compute tasks

#### When to Use Which

**Use spawn_blocking when**:
- Calling blocking IO (file system, blocking database drivers)
- Wrapping legacy sync APIs
- Single blocking operation
- Unknown/variable blocking duration

**Use Rayon when**:
- CPU-intensive parallel work
- Processing large collections
- Data parallelism
- Predictable compute work

**Combine both**:
```rust
// Wrap Rayon work in spawn_blocking
task::spawn_blocking(move || {
    data.par_iter()
        .map(|item| cpu_work(item))
        .collect::<Vec<_>>()
})
.await?
```

## Mixed Workload Pattern

For Lambdas that do both IO and compute:

```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Phase 1: Async IO - Download data
    let download_futures = event.payload.urls
        .into_iter()
        .map(|url| async move {
            reqwest::get(&url).await?.bytes().await
        });

    let raw_data = futures::future::try_join_all(download_futures).await?;

    // Phase 2: Sync compute - Process data with Rayon
    let processed_data = task::spawn_blocking(move || {
        raw_data
            .par_iter()
            .map(|bytes| {
                // CPU-intensive processing
                process_data(bytes)
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .await??;

    // Phase 3: Async IO - Upload results
    let upload_futures = processed_data
        .into_iter()
        .enumerate()
        .map(|(i, data)| async move {
            upload_to_s3(&format!("result-{}.dat", i), &data).await
        });

    futures::future::try_join_all(upload_futures).await?;

    Ok(Response { success: true })
}
```

**Pattern**:
1. **Async at boundaries**: Input/output operations
2. **Sync in middle**: CPU-intensive processing
3. **Use spawn_blocking**: To bridge async and sync
4. **Use Rayon**: For parallel CPU work

## Performance Considerations

### Cold Start Optimization

**Problem**: First invocation (cold start) is slower due to:
- Container initialization
- Runtime loading
- Code initialization

**Solutions**:

1. **Minimize binary size**:
   ```toml
   [profile.release]
   opt-level = 'z'     # Optimize for size
   lto = true          # Link-time optimization
   codegen-units = 1   # Single codegen unit
   strip = true        # Strip symbols
   panic = 'abort'     # Smaller panic handler
   ```

2. **Use ARM64 (Graviton2)**:
   ```bash
   cargo lambda build --release --arm64
   ```
   - 20% better price/performance
   - Often faster cold starts

3. **Lazy initialization**:
   ```rust
   use std::sync::OnceLock;

   static CLIENT: OnceLock<Client> = OnceLock::new();

   fn get_client() -> &'static Client {
       CLIENT.get_or_init(|| Client::new())
   }
   ```

4. **Reduce dependencies**:
   - Audit with `cargo tree`
   - Use feature flags to disable unused features
   - Consider lighter alternatives

### Memory Configuration

**Lambda memory affects**:
- Available RAM
- CPU allocation (proportional to memory)
- Cost

**Guidelines**:
- **128-512 MB**: Simple, fast functions
- **512-1024 MB**: Standard workloads
- **1024-3008 MB**: CPU-intensive, more memory needed
- **3008+ MB**: Heavy compute, large datasets

**Testing**:
```bash
# Deploy with different memory settings
cargo lambda deploy --memory 512
cargo lambda deploy --memory 1024

# Monitor execution time and cost
```

### Concurrency Patterns

**Within single event** (one Lambda invocation):

```rust
// Sequential (slow)
let user = fetch_user().await?;
let posts = fetch_posts().await?;
let comments = fetch_comments().await?;

// Concurrent (fast)
let (user, posts, comments) = tokio::try_join!(
    fetch_user(),
    fetch_posts(),
    fetch_comments(),
)?;

// Dynamic concurrency
let futures: Vec<_> = ids
    .iter()
    .map(|id| fetch_data(id))
    .collect();

let results = futures::future::try_join_all(futures).await?;
```

## Event Types and Patterns

### API Gateway

```rust
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::StatusCode;
use serde_json::json;

async fn handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let path = event.payload.path.unwrap_or_default();
    let method = event.payload.http_method;

    let body = match (method.as_str(), path.as_str()) {
        ("GET", "/users") => {
            let users = fetch_users().await?;
            serde_json::to_string(&users)?
        }
        ("POST", "/users") => {
            let input: CreateUser = serde_json::from_str(
                &event.payload.body.unwrap_or_default()
            )?;
            let user = create_user(input).await?;
            serde_json::to_string(&user)?
        }
        _ => json!({"error": "Not found"}).to_string(),
    };

    Ok(ApiGatewayProxyResponse {
        status_code: StatusCode::OK.as_u16() as i64,
        body: Some(body),
        headers: Default::default(),
        ..Default::default()
    })
}
```

### S3 Events

```rust
use aws_lambda_events::event::s3::S3Event;

async fn handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    for record in event.payload.records {
        let bucket = record.s3.bucket.name.unwrap();
        let key = record.s3.object.key.unwrap();
        let size = record.s3.object.size.unwrap();

        info!("Processing s3://{}/{} ({} bytes)", bucket, key, size);

        // Download from S3
        let data = download_from_s3(&bucket, &key).await?;

        // Process (sync if CPU-intensive)
        let processed = task::spawn_blocking(move || {
            process_file(&data)
        })
        .await??;

        // Upload result
        upload_to_s3(&bucket, &format!("processed/{}", key), &processed).await?;
    }

    Ok(())
}
```

### SQS Events

```rust
use aws_lambda_events::event::sqs::SqsEvent;

async fn handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Process messages concurrently
    let futures = event.payload.records
        .into_iter()
        .map(|record| async move {
            let body = record.body.unwrap_or_default();
            let message: Message = serde_json::from_str(&body)?;

            process_message(message).await
        });

    futures::future::try_join_all(futures).await?;

    Ok(())
}
```

### EventBridge (CloudWatch Events)

```rust
use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use serde::Deserialize;

#[derive(Deserialize)]
struct ScheduledEventDetail {
    action: String,
}

async fn handler(
    event: LambdaEvent<EventBridgeEvent<ScheduledEventDetail>>,
) -> Result<(), Error> {
    info!("Scheduled event: {}", event.payload.detail.action);

    match event.payload.detail.action.as_str() {
        "cleanup" => cleanup_old_data().await?,
        "report" => generate_report().await?,
        _ => warn!("Unknown action"),
    }

    Ok(())
}
```

## Error Handling

### Using thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum LambdaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

// Lambda runtime requires Box<dyn std::error::Error>
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response> {
    let user = fetch_user(&event.payload.user_id)
        .await?
        .ok_or_else(|| LambdaError::NotFound("User not found".to_string()))?;

    Ok(Response { user })
}
```

## Tracing and Logging

### Setup

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // CloudWatch-compatible logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .without_time())  // CloudWatch adds timestamp
        .init();

    run(service_fn(function_handler)).await
}
```

### Usage in Handler

```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let request_id = &event.context.request_id;

    info!(request_id, "Processing request");

    debug!(request_id, user_id = %event.payload.user_id, "Fetching user");

    match fetch_user(&event.payload.user_id).await {
        Ok(user) => {
            info!(request_id, user_id = %user.id, "User found");
            Ok(Response { user })
        }
        Err(e) => {
            error!(request_id, error = %e, "Failed to fetch user");
            Err(e.into())
        }
    }
}
```

### Environment Variable

Set in deployment:
```bash
cargo lambda deploy --env-var RUST_LOG=debug
```

Or in CloudFormation/Terraform:
```yaml
Environment:
  Variables:
    RUST_LOG: info
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::Context;

    #[tokio::test]
    async fn test_handler() {
        let event = LambdaEvent {
            payload: Request {
                user_id: "123".to_string(),
            },
            context: Context::default(),
        };

        let response = function_handler(event).await;
        assert!(response.is_ok());
    }

    #[test]
    fn test_sync_computation() {
        let result = expensive_computation(&input);
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use lambda_runtime::{LambdaEvent, Context};

#[tokio::test]
async fn test_full_workflow() {
    // Setup
    let event = LambdaEvent {
        payload: create_test_event(),
        context: Context::default(),
    };

    // Execute
    let response = my_lambda::function_handler(event).await.unwrap();

    // Verify
    assert_eq!(response.status, "success");
}
```

### Local Testing

```bash
# Start local Lambda
cargo lambda watch

# Invoke with test event
cargo lambda invoke --data-ascii '{"user_id": "123"}'

# Invoke with file
cargo lambda invoke --data-file tests/events/user-event.json

# With environment variables
RUST_LOG=debug cargo lambda watch
```

## GitHub Actions CI/CD

### Key Steps

1. **Install Rust toolchain**
2. **Cache dependencies**
3. **Install Zig** (for cross-compilation)
4. **Install cargo-lambda**
5. **Run tests**
6. **Build for Lambda**
7. **Configure AWS credentials**
8. **Deploy**

### AWS Authentication

**Option 1: OIDC (Recommended)**:
- No long-lived credentials
- More secure
- Requires AWS IAM OIDC provider setup

**Option 2: Access Keys**:
- Simple setup
- Use secrets for credentials
- Rotate regularly

### Multi-Architecture Builds

```yaml
- name: Build for x86_64
  run: cargo lambda build --release --output-format zip

- name: Build for ARM64
  run: cargo lambda build --release --arm64 --output-format zip
```

## Common Patterns Summary

| Workload Type | Approach | Key Tools |
|---------------|----------|-----------|
| IO-only | Full async | tokio, reqwest, aws-sdk |
| Compute-only | spawn_blocking + rayon | rayon, tokio::task |
| Mixed | Async boundaries, sync middle | Both |
| API Gateway | Async with structured responses | aws_lambda_events, http |
| S3 Processing | Async download/upload, sync processing | aws-sdk-s3, rayon |
| Scheduled Tasks | Async for orchestration | EventBridge events |

## Best Practices Checklist

- [ ] Use async for all IO operations
- [ ] Use spawn_blocking + rayon for CPU work
- [ ] Initialize expensive resources once (OnceLock)
- [ ] Optimize binary size in release profile
- [ ] Structure errors with thiserror
- [ ] Add comprehensive tracing
- [ ] Test locally with cargo lambda watch
- [ ] Use appropriate memory allocation
- [ ] Consider ARM64 for better performance
- [ ] Implement proper error handling
- [ ] Set timeouts appropriately
- [ ] Use environment variables for configuration
- [ ] Monitor CloudWatch logs and metrics
- [ ] Profile and optimize hot paths
- [ ] Document expected event formats

## Resources

- **cargo-lambda**: https://github.com/cargo-lambda/cargo-lambda
- **lambda_runtime**: https://github.com/awslabs/aws-lambda-rust-runtime
- **AWS Lambda Rust**: https://docs.aws.amazon.com/lambda/latest/dg/lambda-rust.html
- **Tokio**: https://tokio.rs/
- **Rayon**: https://github.com/rayon-rs/rayon
- **aws_lambda_events**: https://docs.rs/aws_lambda_events

---

This context helps Claude Code understand Rust Lambda development patterns and guide users effectively.
