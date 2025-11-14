---
description: Optimize Rust Lambda function for IO-intensive workloads with async patterns
---

You are helping the user optimize their Lambda function for IO-intensive workloads.

## Your Task

Guide the user to optimize their Lambda for maximum IO performance using async/await patterns.

## IO-Intensive Characteristics

Functions that:
- Make multiple HTTP/API requests
- Query databases
- Read/write from S3, DynamoDB
- Call external services
- Process message queues
- Send notifications

**Goal**: Maximize concurrency to reduce wall-clock time and cost

## Key Optimization Strategies

### 1. Concurrent Operations

Replace sequential operations with concurrent ones:

**❌ Sequential (Slow)**:
```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Each operation waits for previous one - slow!
    let user = fetch_user().await?;
    let posts = fetch_posts().await?;
    let comments = fetch_comments().await?;

    Ok(Response { user, posts, comments })
}
```

**✅ Concurrent (Fast)**:
```rust
use futures::future::try_join_all;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // All operations run concurrently - fast!
    let (user, posts, comments) = tokio::try_join!(
        fetch_user(),
        fetch_posts(),
        fetch_comments(),
    )?;

    Ok(Response { user, posts, comments })
}
```

**Performance impact**: 3 sequential 100ms calls = 300ms. Concurrent = ~100ms.

### 2. Parallel Collection Processing

**❌ Sequential iteration**:
```rust
let mut results = Vec::new();
for id in user_ids {
    let data = fetch_data(&id).await?;
    results.push(data);
}
```

**✅ Concurrent iteration**:
```rust
use futures::future::try_join_all;

let futures = user_ids
    .iter()
    .map(|id| fetch_data(id));

let results = try_join_all(futures).await?;
```

**Alternative with buffer (limits concurrency)**:
```rust
use futures::stream::{self, StreamExt};

let results = stream::iter(user_ids)
    .map(|id| fetch_data(&id))
    .buffer_unordered(10)  // Max 10 concurrent requests
    .collect::<Vec<_>>()
    .await;
```

### 3. Reuse Connections

**❌ Creating new client each time**:
```rust
async fn fetch_data(url: &str) -> Result<Data, Error> {
    let client = reqwest::Client::new();  // New connection every call!
    client.get(url).send().await?.json().await
}
```

**✅ Shared client with connection pooling**:
```rust
use std::sync::OnceLock;
use reqwest::Client;

// Initialized once per container
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap()
    })
}

async fn fetch_data(url: &str) -> Result<Data, Error> {
    let client = get_client();  // Reuses connections!
    client.get(url).send().await?.json().await
}
```

### 4. Database Connection Pooling

**For PostgreSQL with sqlx**:
```rust
use std::sync::OnceLock;
use sqlx::{PgPool, postgres::PgPoolOptions};

static DB_POOL: OnceLock<PgPool> = OnceLock::new();

async fn get_pool() -> &'static PgPool {
    DB_POOL.get_or_init(|| async {
        PgPoolOptions::new()
            .max_connections(5)  // Limit connections
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap()
    }).await
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let pool = get_pool().await;

    // Use connection pool for queries
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;

    Ok(Response { user })
}
```

**For DynamoDB**:
```rust
use std::sync::OnceLock;
use aws_sdk_dynamodb::Client;

static DYNAMODB_CLIENT: OnceLock<Client> = OnceLock::new();

async fn get_dynamodb() -> &'static Client {
    DYNAMODB_CLIENT.get_or_init(|| async {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    }).await
}
```

### 5. Batch Operations

When possible, use batch APIs:

**❌ Individual requests**:
```rust
for key in keys {
    let item = dynamodb.get_item()
        .table_name("MyTable")
        .key("id", AttributeValue::S(key))
        .send()
        .await?;
}
```

**✅ Batch request**:
```rust
let batch_keys = keys
    .iter()
    .map(|key| {
        [(
            "id".to_string(),
            AttributeValue::S(key.clone())
        )].into()
    })
    .collect();

let response = dynamodb.batch_get_item()
    .request_items("MyTable", KeysAndAttributes::builder()
        .set_keys(Some(batch_keys))
        .build()?)
    .send()
    .await?;
```

## Complete IO-Optimized Example

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use reqwest::Client;
use futures::future::try_join_all;
use tracing::info;

#[derive(Deserialize)]
struct Request {
    user_ids: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    users: Vec<UserData>,
}

#[derive(Serialize)]
struct UserData {
    user: User,
    posts: Vec<Post>,
    followers: usize,
}

// Shared HTTP client with connection pooling
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .unwrap()
    })
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    info!("Processing {} users", event.payload.user_ids.len());

    // Process all users concurrently
    let user_futures = event.payload.user_ids
        .into_iter()
        .map(|user_id| fetch_user_data(user_id));

    let users = try_join_all(user_futures).await?;

    Ok(Response { users })
}

async fn fetch_user_data(user_id: String) -> Result<UserData, Error> {
    let client = get_client();

    // Fetch all user data concurrently
    let (user, posts, followers) = tokio::try_join!(
        fetch_user(client, &user_id),
        fetch_posts(client, &user_id),
        fetch_follower_count(client, &user_id),
    )?;

    Ok(UserData { user, posts, followers })
}

async fn fetch_user(client: &Client, user_id: &str) -> Result<User, Error> {
    let response = client
        .get(format!("https://api.example.com/users/{}", user_id))
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

async fn fetch_posts(client: &Client, user_id: &str) -> Result<Vec<Post>, Error> {
    let response = client
        .get(format!("https://api.example.com/users/{}/posts", user_id))
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

async fn fetch_follower_count(client: &Client, user_id: &str) -> Result<usize, Error> {
    let response: FollowerResponse = client
        .get(format!("https://api.example.com/users/{}/followers", user_id))
        .send()
        .await?
        .json()
        .await?;
    Ok(response.count)
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

## AWS SDK Optimization

### S3 Concurrent Operations

```rust
use aws_sdk_s3::Client;
use futures::future::try_join_all;

async fn download_multiple_files(
    s3: &Client,
    bucket: &str,
    keys: Vec<String>,
) -> Result<Vec<Bytes>, Error> {
    let futures = keys.iter().map(|key| async move {
        s3.get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?
            .body
            .collect()
            .await
            .map(|data| data.into_bytes())
    });

    try_join_all(futures).await
}
```

### DynamoDB Parallel Queries

```rust
async fn query_multiple_partitions(
    dynamodb: &Client,
    partition_keys: Vec<String>,
) -> Result<Vec<Item>, Error> {
    let futures = partition_keys.iter().map(|pk| {
        dynamodb
            .query()
            .table_name("MyTable")
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", AttributeValue::S(pk.clone()))
            .send()
    });

    let results = try_join_all(futures).await?;

    let items = results
        .into_iter()
        .flat_map(|r| r.items.unwrap_or_default())
        .collect();

    Ok(items)
}
```

## Timeout and Retry Configuration

```rust
use reqwest::Client;
use std::time::Duration;

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))      // Total request timeout
            .connect_timeout(Duration::from_secs(10)) // Connection timeout
            .pool_max_idle_per_host(10)            // Connection pool size
            .tcp_keepalive(Duration::from_secs(60)) // Keep connections alive
            .build()
            .unwrap()
    })
}
```

With retries:
```rust
use tower::{ServiceBuilder, ServiceExt};
use tower::retry::RetryLayer;

// Add to dependencies: tower = { version = "0.4", features = ["retry"] }

async fn fetch_with_retry(url: &str) -> Result<Response, Error> {
    let client = get_client();

    for attempt in 1..=3 {
        match client.get(url).send().await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < 3 => {
                tokio::time::sleep(Duration::from_millis(100 * attempt)).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    unreachable!()
}
```

## Streaming Large Responses

For large files or responses:

```rust
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

async fn download_large_file(s3: &Client, bucket: &str, key: &str) -> Result<(), Error> {
    let mut object = s3
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    // Stream to avoid loading entire file in memory
    let mut body = object.body;

    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        // Process chunk
        process_chunk(&chunk).await?;
    }

    Ok(())
}
```

## Concurrency Limits

Control concurrency to avoid overwhelming external services:

```rust
use futures::stream::{self, StreamExt};

async fn process_with_limit(
    items: Vec<Item>,
    max_concurrent: usize,
) -> Result<Vec<Output>, Error> {
    let results = stream::iter(items)
        .map(|item| async move {
            process_item(item).await
        })
        .buffer_unordered(max_concurrent)  // Limit concurrent operations
        .collect::<Vec<_>>()
        .await;

    results.into_iter().collect()
}
```

## Error Handling for Async Operations

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum LambdaError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Timeout: operation took too long")]
    Timeout,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Use timeout for async operations
    let result = tokio::time::timeout(
        Duration::from_secs(25),  // Lambda timeout - buffer
        process_request(event.payload)
    )
    .await
    .map_err(|_| LambdaError::Timeout)??;

    Ok(result)
}
```

## Performance Checklist

Apply these optimizations:

- [ ] Use `tokio::try_join!` for fixed concurrent operations
- [ ] Use `futures::future::try_join_all` for dynamic collections
- [ ] Initialize clients/pools once with `OnceLock`
- [ ] Configure connection pooling for HTTP clients
- [ ] Use batch APIs when available
- [ ] Set appropriate timeouts
- [ ] Add retries for transient failures
- [ ] Stream large responses
- [ ] Limit concurrency to avoid overwhelming services
- [ ] Use `buffer_unordered` for controlled parallelism
- [ ] Avoid blocking operations in async context
- [ ] Monitor cold start times
- [ ] Test with realistic event sizes

## Dependencies for IO Optimization

Add to `Cargo.toml`:

```toml
[dependencies]
lambda_runtime = "0.13"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
futures = "0.3"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# AWS SDKs
aws-config = "1"
aws-sdk-s3 = "1"
aws-sdk-dynamodb = "1"

# Database (if needed)
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Optional: retries
tower = { version = "0.4", features = ["retry", "timeout"] }
```

## Testing IO Performance

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_performance() {
        let start = std::time::Instant::now();

        let results = fetch_multiple_users(vec!["1", "2", "3"]).await.unwrap();

        let duration = start.elapsed();

        // Should be ~100ms (concurrent), not ~300ms (sequential)
        assert!(duration.as_millis() < 150);
        assert_eq!(results.len(), 3);
    }
}
```

## Monitoring

Add instrumentation to track IO performance:

```rust
use tracing::{info, instrument};

#[instrument(skip(client))]
async fn fetch_user(client: &Client, user_id: &str) -> Result<User, Error> {
    let start = std::time::Instant::now();

    let result = client
        .get(format!("https://api.example.com/users/{}", user_id))
        .send()
        .await?
        .json()
        .await?;

    info!(user_id, duration_ms = start.elapsed().as_millis(), "User fetched");

    Ok(result)
}
```

After optimization, verify:
- Cold start time (should be minimal)
- Warm execution time (should be low due to concurrency)
- Memory usage (should be moderate)
- Error rates (should be low with retries)
- CloudWatch metrics show improved performance
