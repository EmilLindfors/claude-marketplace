---
description: Set up Lambda Function URLs and response streaming for Rust Lambda functions
---

You are helping the user implement Lambda Function URLs and streaming responses for their Rust Lambda functions.

## Your Task

Guide the user through setting up direct HTTPS endpoints using Lambda Function URLs and implementing streaming responses for large payloads.

## Lambda Function URLs

Lambda Function URLs provide dedicated HTTP(S) endpoints for your Lambda function without API Gateway.

**Best for**:
- Simple HTTP endpoints
- Webhooks
- Direct function invocation
- Cost-sensitive applications
- No need for API Gateway features (rate limiting, API keys, etc.)

### Setup with lambda_http

Add to `Cargo.toml`:
```toml
[dependencies]
lambda_http = { version = "0.13", features = ["apigw_http"] }
lambda_runtime = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**IMPORTANT**: The `apigw_http` feature is required for Function URLs.

### Basic HTTP Handler

```rust
use lambda_http::{run, service_fn, Body, Error, Request, Response};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract path and method
    let path = event.uri().path();
    let method = event.method();

    // Extract query parameters
    let params = event.query_string_parameters();
    let name = params.first("name").unwrap_or("World");

    // Extract headers
    let user_agent = event
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Build response
    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"message": "Hello, {}!", "path": "{}", "method": "{}"}}"#,
            name, path, method
        )))?;

    Ok(response)
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

### JSON Request/Response

```rust
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct CreateUserResponse {
    id: String,
    name: String,
    email: String,
    created_at: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Parse JSON body
    let body = event.body();
    let request: CreateUserRequest = serde_json::from_slice(body)?;

    // Validate
    if request.email.is_empty() {
        return Ok(Response::builder()
            .status(400)
            .body(Body::from(r#"{"error": "Email is required"}"#))?);
    }

    // Create user
    let user = CreateUserResponse {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        email: request.email,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Return JSON response
    let response = Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&user)?))?;

    Ok(response)
}
```

### REST API Pattern

```rust
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let method = event.method();
    let path = event.uri().path();

    match (method.as_str(), path) {
        ("GET", "/users") => list_users().await,
        ("GET", path) if path.starts_with("/users/") => {
            let id = path.strip_prefix("/users/").unwrap();
            get_user(id).await
        }
        ("POST", "/users") => {
            let body = event.body();
            let request: CreateUserRequest = serde_json::from_slice(body)?;
            create_user(request).await
        }
        ("PUT", path) if path.starts_with("/users/") => {
            let id = path.strip_prefix("/users/").unwrap();
            let body = event.body();
            let request: UpdateUserRequest = serde_json::from_slice(body)?;
            update_user(id, request).await
        }
        ("DELETE", path) if path.starts_with("/users/") => {
            let id = path.strip_prefix("/users/").unwrap();
            delete_user(id).await
        }
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from(r#"{"error": "Not found"}"#))?),
    }
}

async fn list_users() -> Result<Response<Body>, Error> {
    // Implementation
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"users": []}"#))?)
}
```

### Enable Function URL

```bash
# Deploy function
cargo lambda build --release --arm64
cargo lambda deploy my-function

# Create Function URL
aws lambda create-function-url-config \
  --function-name my-function \
  --auth-type NONE \
  --cors '{
    "AllowOrigins": ["*"],
    "AllowMethods": ["GET", "POST", "PUT", "DELETE"],
    "AllowHeaders": ["content-type"],
    "MaxAge": 300
  }'

# Add permission for public access
aws lambda add-permission \
  --function-name my-function \
  --action lambda:InvokeFunctionUrl \
  --principal "*" \
  --function-url-auth-type NONE \
  --statement-id FunctionURLAllowPublicAccess
```

## Response Streaming

For large responses (up to 20MB), use streaming to send data incrementally.

**Best for**:
- Large file downloads
- Real-time data feeds
- Server-sent events (SSE)
- Reducing time to first byte

### Setup Streaming

Add to `Cargo.toml`:
```toml
[dependencies]
lambda_runtime = { version = "0.13", features = ["streaming"] }
tokio = { version = "1", features = ["macros", "io-util"] }
tokio-stream = "0.1"
```

### Basic Streaming Example

```rust
use lambda_runtime::{run, streaming, Error, LambdaEvent};
use tokio::io::AsyncWriteExt;

async fn function_handler(
    event: LambdaEvent<Request>,
    response_stream: streaming::Response,
) -> Result<(), Error> {
    let mut writer = response_stream.into_writer();

    // Stream data incrementally
    for i in 0..100 {
        let data = format!("Chunk {}\n", i);
        writer.write_all(data.as_bytes()).await?;

        // Optional: Flush to send immediately
        writer.flush().await?;

        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(streaming::service_fn(function_handler)).await
}
```

### Stream Large File from S3

```rust
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{streaming, Error, LambdaEvent};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

async fn function_handler(
    event: LambdaEvent<Request>,
    response_stream: streaming::Response,
) -> Result<(), Error> {
    let s3 = S3Client::new(&aws_config::load_from_env().await);

    // Get S3 object as stream
    let mut object = s3
        .get_object()
        .bucket("my-bucket")
        .key(&event.payload.file_key)
        .send()
        .await?;

    let mut writer = response_stream.into_writer();
    let mut body = object.body;

    // Stream S3 data directly to response
    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
    }

    Ok(())
}
```

### Server-Sent Events (SSE)

```rust
use lambda_runtime::{streaming, Error, LambdaEvent};
use tokio::io::AsyncWriteExt;
use tokio::time::{interval, Duration};

async fn function_handler(
    event: LambdaEvent<Request>,
    response_stream: streaming::Response,
) -> Result<(), Error> {
    let mut writer = response_stream.into_writer();

    // SSE headers
    let headers = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/event-stream\r\n\
                   Cache-Control: no-cache\r\n\
                   Connection: keep-alive\r\n\r\n";

    writer.write_all(headers.as_bytes()).await?;

    let mut ticker = interval(Duration::from_secs(1));

    for i in 0..30 {
        ticker.tick().await;

        // Send SSE event
        let event = format!("data: {{\"count\": {}, \"timestamp\": {}}}\n\n",
                          i,
                          chrono::Utc::now().timestamp());

        writer.write_all(event.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
```

### Configure Streaming in AWS

```bash
# Update function to use streaming
aws lambda update-function-configuration \
  --function-name my-function \
  --invoke-mode RESPONSE_STREAM

# Create streaming Function URL
aws lambda create-function-url-config \
  --function-name my-function \
  --auth-type NONE \
  --invoke-mode RESPONSE_STREAM
```

## CORS Configuration

```rust
use lambda_http::{Response, Body};

fn add_cors_headers(response: Response<Body>) -> Response<Body> {
    let (mut parts, body) = response.into_parts();

    parts.headers.insert(
        "access-control-allow-origin",
        "*".parse().unwrap(),
    );
    parts.headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    parts.headers.insert(
        "access-control-allow-headers",
        "content-type, authorization".parse().unwrap(),
    );

    Response::from_parts(parts, body)
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Handle OPTIONS preflight
    if event.method() == "OPTIONS" {
        return Ok(add_cors_headers(
            Response::builder()
                .status(200)
                .body(Body::Empty)?
        ));
    }

    let response = handle_request(event).await?;
    Ok(add_cors_headers(response))
}
```

## Authentication

### IAM Authentication

```bash
aws lambda create-function-url-config \
  --function-name my-function \
  --auth-type AWS_IAM  # Requires AWS Signature V4
```

### Custom Authentication

```rust
use lambda_http::{Request, Response, Body, Error};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Verify bearer token
    let auth_header = event
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            &header[7..]
        }
        _ => {
            return Ok(Response::builder()
                .status(401)
                .body(Body::from(r#"{"error": "Unauthorized"}"#))?);
        }
    };

    if !verify_token(token).await? {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from(r#"{"error": "Invalid token"}"#))?);
    }

    // Process authenticated request
    handle_authenticated_request(event).await
}
```

## Complete Example: REST API with Streaming

```rust
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_runtime::streaming;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ExportRequest {
    format: String,
    filters: Vec<String>,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    match (event.method().as_str(), event.uri().path()) {
        ("GET", "/health") => health_check(),
        ("POST", "/export") => {
            // For large exports, use streaming
            let request: ExportRequest = serde_json::from_slice(event.body())?;
            export_data_streaming(request).await
        }
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from(r#"{"error": "Not found"}"#))?),
    }
}

fn health_check() -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .body(Body::from(r#"{"status": "healthy"}"#))?)
}

async fn export_data_streaming(request: ExportRequest) -> Result<Response<Body>, Error> {
    // Return streaming response for large data
    // Note: This is simplified - actual streaming setup varies
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/csv")
        .header("content-disposition", "attachment; filename=export.csv")
        .body(Body::from("Streaming not available in non-streaming handler"))?)
}
```

## Comparison: Function URLs vs API Gateway

| Feature | Function URLs | API Gateway |
|---------|---------------|-------------|
| Cost | Free | $3.50/million requests |
| Setup | Simple | Complex |
| Rate Limiting | No | Yes |
| API Keys | No | Yes |
| Custom Domains | No (use CloudFront) | Yes |
| Request Validation | Manual | Built-in |
| WebSocket | No | Yes |
| Max Timeout | 15 min | 29 sec (HTTP), 15 min (REST) |
| Streaming | Yes (20MB) | Limited |

## Best Practices

- [ ] Use Function URLs for simple endpoints
- [ ] Use API Gateway for complex APIs
- [ ] Implement authentication for public endpoints
- [ ] Add CORS headers for web clients
- [ ] Use streaming for large responses (>1MB)
- [ ] Implement proper error handling
- [ ] Add request validation
- [ ] Monitor with CloudWatch Logs
- [ ] Set appropriate timeout and memory
- [ ] Use compression for large responses
- [ ] Cache responses when possible
- [ ] Document your API endpoints

Guide the user through implementing Function URLs or streaming based on their needs.
