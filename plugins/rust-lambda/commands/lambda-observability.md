---
description: Set up advanced observability for Rust Lambda with OpenTelemetry, X-Ray, and structured logging
---

You are helping the user implement comprehensive observability for their Rust Lambda functions.

## Your Task

Guide the user through setting up production-grade observability including distributed tracing, metrics, and structured logging.

## Observability Stack Options

### Option 1: AWS X-Ray (Native AWS Solution)

**Best for**:
- AWS-native monitoring
- Quick setup
- CloudWatch integration
- Basic distributed tracing needs

#### Enable X-Ray in Lambda

**Via cargo-lambda:**
```bash
cargo lambda deploy --enable-tracing
```

**Via SAM template:**
```yaml
Resources:
  MyFunction:
    Type: AWS::Serverless::Function
    Properties:
      Tracing: Active  # Enable X-Ray
```

**Via Terraform:**
```hcl
resource "aws_lambda_function" "function" {
  # ... other config ...

  tracing_config {
    mode = "Active"
  }
}
```

#### X-Ray with xray-lite

Add to `Cargo.toml`:
```toml
[dependencies]
xray-lite = "0.1"
aws-config = "1"
aws-sdk-dynamodb = "1"  # or other AWS services
```

Basic usage:
```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use xray_lite::SubsegmentContext;
use xray_lite_aws_sdk::XRayAwsSdkExtension;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // X-Ray automatically creates parent segment for Lambda

    // Create subsegment for custom operation
    let subsegment = SubsegmentContext::from_lambda_ctx(&event.context);

    // Trace AWS SDK calls
    let config = aws_config::load_from_env().await
        .xray_extension(subsegment.clone());

    let dynamodb = aws_sdk_dynamodb::Client::new(&config);

    // This DynamoDB call will be traced automatically
    let result = dynamodb
        .get_item()
        .table_name("MyTable")
        .key("id", AttributeValue::S("123".to_string()))
        .send()
        .await?;

    Ok(Response { data: result })
}
```

### Option 2: OpenTelemetry (Vendor-Neutral)

**Best for**:
- Multi-vendor monitoring
- Portability across platforms
- Advanced telemetry needs
- Custom metrics and traces

#### Setup OpenTelemetry

Add to `Cargo.toml`:
```toml
[dependencies]
lambda_runtime = "0.13"
lambda-otel-lite = "0.1"  # Lightweight OpenTelemetry for Lambda
opentelemetry = "0.22"
opentelemetry-otlp = "0.15"
opentelemetry_sdk = "0.22"
tracing = "0.1"
tracing-opentelemetry = "0.23"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

#### Basic OpenTelemetry Setup

```rust
use lambda_otel_lite::{init_telemetry, HttpTracerProviderBuilder};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use opentelemetry::trace::TracerProvider;
use tracing::{info, instrument};
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize OpenTelemetry
    let tracer_provider = HttpTracerProviderBuilder::default()
        .with_default_text_map_propagator()
        .with_stdout_client()  // For testing, use OTLP for production
        .build()?;

    let tracer = tracer_provider.tracer("my-rust-lambda");

    // Setup tracing subscriber
    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer);

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    run(service_fn(function_handler)).await
}

#[instrument(skip(event))]
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!(request_id = %event.context.request_id, "Processing request");

    let result = process_data(&event.payload).await?;

    Ok(Response { result })
}

#[instrument]
async fn process_data(request: &Request) -> Result<Data, Error> {
    info!("Processing data");

    // Your processing logic
    // All operations within this function will be traced

    Ok(Data::new())
}
```

#### OpenTelemetry with OTLP Exporter

For production, export to observability backend:

```rust
use lambda_otel_lite::HttpTracerProviderBuilder;
use opentelemetry_otlp::WithExportConfig;

let tracer_provider = HttpTracerProviderBuilder::default()
    .with_stdout_client()
    .enable_otlp(
        opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint("https://your-collector:4318")
            .with_headers([("api-key", "your-key")])
    )?
    .build()?;
```

### Option 3: Datadog Integration

**Best for**:
- Datadog users
- Comprehensive APM
- Log aggregation
- Custom metrics

Add Datadog Lambda Extension layer and configure:

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{info, instrument};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // JSON format for Datadog log parsing
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_current_span(false)
        .init();

    run(service_fn(function_handler)).await
}

#[instrument(
    skip(event),
    fields(
        request_id = %event.context.request_id,
        user_id = %event.payload.user_id,
    )
)]
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Processing user request");

    // Datadog automatically traces this
    let result = fetch_user_data(&event.payload.user_id).await?;

    Ok(Response { result })
}
```

Deploy with Datadog extension layer:
```bash
cargo lambda deploy \
  --layers arn:aws:lambda:us-east-1:464622532012:layer:Datadog-Extension-ARM:latest \
  --env-var DD_API_KEY=your-api-key \
  --env-var DD_SITE=datadoghq.com \
  --env-var DD_SERVICE=my-rust-service \
  --env-var DD_ENV=production
```

## Structured Logging Best Practices

### Using tracing with Spans

```rust
use tracing::{info, warn, error, debug, span, Level};

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let span = span!(
        Level::INFO,
        "process_request",
        request_id = %event.context.request_id,
        user_id = %event.payload.user_id,
    );

    let _enter = span.enter();

    info!("Starting request processing");

    match process_user(&event.payload.user_id).await {
        Ok(user) => {
            info!(user_name = %user.name, "User processed successfully");
            Ok(Response { user })
        }
        Err(e) => {
            error!(error = %e, "Failed to process user");
            Err(e)
        }
    }
}

#[instrument(skip(db), fields(user_id = %user_id))]
async fn process_user(user_id: &str) -> Result<User, Error> {
    debug!("Fetching user from database");

    let user = fetch_from_db(user_id).await?;

    info!(email = %user.email, "User fetched");

    Ok(user)
}
```

### JSON Structured Logging

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // JSON output for CloudWatch Insights
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_current_span(true)
        .with_span_list(true)
        .with_target(false)
        .without_time()  // CloudWatch adds timestamp
        .init();

    run(service_fn(function_handler)).await
}

// Logs will be structured JSON:
// {"level":"info","message":"Processing request","request_id":"abc123","user_id":"user456"}
```

### Custom Metrics with OpenTelemetry

```rust
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::KeyValue;

struct Metrics {
    request_counter: Counter<u64>,
    duration_histogram: Histogram<f64>,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let start = std::time::Instant::now();

    // Increment counter
    metrics.request_counter.add(
        1,
        &[
            KeyValue::new("function", "my-lambda"),
            KeyValue::new("region", "us-east-1"),
        ],
    );

    let result = process_request(&event.payload).await?;

    // Record duration
    let duration = start.elapsed().as_secs_f64();
    metrics.duration_histogram.record(
        duration,
        &[KeyValue::new("status", "success")],
    );

    Ok(result)
}
```

## CloudWatch Logs Insights Queries

With structured logging, you can query efficiently:

```
# Find errors for specific user
fields @timestamp, message, error
| filter user_id = "user456"
| filter level = "error"
| sort @timestamp desc

# Calculate p95 latency
fields duration_ms
| stats percentile(duration_ms, 95) as p95_latency by bin(5m)

# Count requests by status
fields @timestamp
| filter message = "Request completed"
| stats count() by status
```

## Distributed Tracing Pattern

For microservices calling each other:

```rust
use opentelemetry::global;
use opentelemetry::trace::{Tracer, TracerProvider, SpanKind};
use opentelemetry_http::HeaderExtractor;

async fn function_handler(event: LambdaEvent<ApiGatewayRequest>) -> Result<Response, Error> {
    let tracer = global::tracer("my-service");

    // Extract trace context from incoming request
    let parent_cx = global::get_text_map_propagator(|propagator| {
        let headers = HeaderExtractor::new(&event.payload.headers);
        propagator.extract(&headers)
    });

    // Create span with parent context
    let span = tracer
        .span_builder("handle_request")
        .with_kind(SpanKind::Server)
        .start_with_context(&tracer, &parent_cx);

    let cx = opentelemetry::Context::current_with_span(span);

    // Call downstream service with trace context
    let client = reqwest::Client::new();
    let response = client
        .get("https://downstream-service.com/api")
        .header("traceparent", extract_traceparent(&cx))
        .send()
        .await?;

    Ok(Response { data: response.text().await? })
}
```

## AWS ADOT Lambda Layer

For automatic instrumentation (limited Rust support):

```bash
# Add ADOT layer (note: Rust needs manual instrumentation)
cargo lambda deploy \
  --layers arn:aws:lambda:us-east-1:901920570463:layer:aws-otel-collector-arm64-ver-0-90-1:1 \
  --env-var AWS_LAMBDA_EXEC_WRAPPER=/opt/otel-instrument \
  --env-var OPENTELEMETRY_COLLECTOR_CONFIG_FILE=/var/task/collector.yaml
```

## Cold Start Monitoring

Track cold start vs warm start:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static COLD_START: AtomicBool = AtomicBool::new(true);

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let is_cold_start = COLD_START.swap(false, Ordering::Relaxed);

    info!(
        cold_start = is_cold_start,
        "Lambda invocation"
    );

    // Process request...

    Ok(Response {})
}
```

## Error Tracking

### Capturing Error Context

```rust
use tracing::error;
use thiserror::Error;

#[derive(Error, Debug)]
enum LambdaError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("External API error: {status}, {message}")]
    ExternalApi { status: u16, message: String },
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    match process_request(&event.payload).await {
        Ok(result) => {
            info!("Request processed successfully");
            Ok(Response { result })
        }
        Err(e) => {
            error!(
                error = %e,
                error_type = std::any::type_name_of_val(&e),
                request_id = %event.context.request_id,
                "Request failed"
            );

            // Optionally send to error tracking service
            send_to_sentry(&e, &event.context).await;

            Err(e.into())
        }
    }
}
```

## Performance Monitoring

### Measure Operation Duration

```rust
use std::time::Instant;
use tracing::info;

#[instrument]
async fn expensive_operation() -> Result<Data, Error> {
    let start = Instant::now();

    let result = do_work().await?;

    let duration = start.elapsed();
    info!(duration_ms = duration.as_millis(), "Operation completed");

    Ok(result)
}
```

### Automatic Instrumentation

```rust
use tracing::instrument;

// Automatically creates span and logs entry/exit
#[instrument(
    skip(db),  // Don't log entire db object
    fields(
        user_id = %user_id,
        operation = "fetch_user"
    ),
    err  // Log errors automatically
)]
async fn fetch_user(db: &Database, user_id: &str) -> Result<User, Error> {
    db.get_user(user_id).await
}
```

## Observability Checklist

- [ ] Enable X-Ray or OpenTelemetry tracing
- [ ] Use structured logging (JSON format)
- [ ] Add span instrumentation to key functions
- [ ] Track cold vs warm starts
- [ ] Monitor error rates and types
- [ ] Measure operation durations
- [ ] Set up CloudWatch Logs Insights queries
- [ ] Configure alerts for errors and latency
- [ ] Track custom business metrics
- [ ] Propagate trace context across services
- [ ] Set appropriate log retention
- [ ] Use log levels correctly (debug, info, warn, error)

## Recommended Stack

**For AWS-only**:
- X-Ray for tracing
- CloudWatch Logs with structured JSON
- CloudWatch Insights for queries
- xray-lite for Rust integration

**For multi-cloud/vendor-neutral**:
- OpenTelemetry for tracing
- OTLP exporter to your backend
- lambda-otel-lite for Lambda optimization
- tracing crate for structured logging

**For Datadog users**:
- Datadog Lambda Extension
- DD_TRACE_ENABLED for automatic tracing
- JSON structured logging
- Custom metrics via DogStatsD

## Dependencies

```toml
[dependencies]
# Basic tracing
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# X-Ray
xray-lite = "0.1"
xray-lite-aws-sdk = "0.1"

# OpenTelemetry
lambda-otel-lite = "0.1"
opentelemetry = "0.22"
opentelemetry-otlp = "0.15"
opentelemetry_sdk = "0.22"
tracing-opentelemetry = "0.23"

# AWS SDK (for tracing AWS calls)
aws-config = "1"
aws-sdk-dynamodb = "1"  # or other services
```

Guide the user through setting up observability appropriate for their needs and monitoring backend.
