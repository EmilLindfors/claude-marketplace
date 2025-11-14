---
description: Optimize Rust Lambda function for compute-intensive workloads using Rayon and spawn_blocking
---

You are helping the user optimize their Lambda function for compute-intensive workloads.

## Your Task

Guide the user to optimize their Lambda for CPU-intensive operations using synchronous parallel processing with Rayon and spawn_blocking.

## Compute-Intensive Characteristics

Functions that:
- Process large datasets
- Perform mathematical computations
- Transform/parse data
- Image/video processing
- Compression/decompression
- Encryption/hashing
- Machine learning inference

**Goal**: Maximize CPU utilization without blocking async runtime

## Key Principle: Async at Boundaries, Sync in Middle

```
Input (async) → CPU Work (sync) → Output (async)
```

```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Phase 1: Async input (if needed)
    let data = fetch_input_data().await?;

    // Phase 2: Sync compute with spawn_blocking + Rayon
    let results = tokio::task::spawn_blocking(move || {
        data.par_iter()
            .map(|item| expensive_computation(item))
            .collect::<Vec<_>>()
    })
    .await?;

    // Phase 3: Async output (if needed)
    upload_results(&results).await?;

    Ok(Response { results })
}
```

## Core Pattern: spawn_blocking + Rayon

### Why This Pattern?

1. **spawn_blocking**: Moves work off async runtime to blocking thread pool
2. **Rayon**: Efficiently parallelizes CPU work across available cores
3. **Together**: Best CPU utilization without blocking async operations

### Basic Pattern

```rust
use tokio::task;
use rayon::prelude::*;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let numbers = event.payload.numbers;

    // Move CPU work to blocking thread pool
    let results = task::spawn_blocking(move || {
        // Use Rayon for parallel computation
        numbers
            .par_iter()
            .map(|&n| cpu_intensive_work(n))
            .collect::<Vec<_>>()
    })
    .await?;

    Ok(Response { results })
}

// Pure CPU work - synchronous, no async
fn cpu_intensive_work(n: i64) -> i64 {
    // Heavy computation here
    (0..10000).fold(n, |acc, _| {
        acc.wrapping_mul(31).wrapping_add(17)
    })
}
```

## Complete Compute-Optimized Example

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::{info, instrument};

#[derive(Deserialize)]
struct Request {
    data: Vec<DataPoint>,
    algorithm: String,
}

#[derive(Serialize)]
struct Response {
    processed: Vec<ProcessedData>,
    stats: Statistics,
}

#[derive(Debug, Clone, Deserialize)]
struct DataPoint {
    values: Vec<f64>,
    metadata: String,
}

#[derive(Debug, Serialize)]
struct ProcessedData {
    result: f64,
    classification: String,
}

#[derive(Debug, Serialize)]
struct Statistics {
    count: usize,
    mean: f64,
    std_dev: f64,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let data = event.payload.data;
    let algorithm = event.payload.algorithm;

    info!("Processing {} data points with {}", data.len(), algorithm);

    // CPU-intensive work in spawn_blocking
    let results = task::spawn_blocking(move || {
        process_data_parallel(data, &algorithm)
    })
    .await??;

    Ok(results)
}

// All CPU work happens here - synchronous and parallel
fn process_data_parallel(data: Vec<DataPoint>, algorithm: &str) -> Result<Response, Error> {
    // Parallel processing with Rayon
    let processed: Vec<ProcessedData> = data
        .par_iter()
        .map(|point| {
            let result = match algorithm {
                "standard" => compute_standard(&point.values),
                "advanced" => compute_advanced(&point.values),
                _ => compute_standard(&point.values),
            };

            let classification = classify_result(result);

            ProcessedData { result, classification }
        })
        .collect();

    // Compute statistics
    let stats = compute_statistics(&processed);

    Ok(Response { processed, stats })
}

// Pure computation - no IO, no async
fn compute_standard(values: &[f64]) -> f64 {
    // CPU-intensive mathematical computation
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;

    values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn compute_advanced(values: &[f64]) -> f64 {
    // Even more intensive computation
    let mut result = 0.0;
    for &v in values {
        for i in 0..1000 {
            result += v * (i as f64).sin();
        }
    }
    result
}

fn classify_result(value: f64) -> String {
    match value {
        x if x < 0.0 => "low".to_string(),
        x if x < 10.0 => "medium".to_string(),
        _ => "high".to_string(),
    }
}

fn compute_statistics(processed: &[ProcessedData]) -> Statistics {
    let count = processed.len();
    let mean = processed.iter().map(|p| p.result).sum::<f64>() / count as f64;

    let variance = processed
        .iter()
        .map(|p| (p.result - mean).powi(2))
        .sum::<f64>() / count as f64;

    let std_dev = variance.sqrt();

    Statistics { count, mean, std_dev }
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

## Advanced Rayon Patterns

### Custom Thread Pool

```rust
use rayon::ThreadPoolBuilder;
use std::sync::OnceLock;

static THREAD_POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

fn get_thread_pool() -> &'static rayon::ThreadPool {
    THREAD_POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .unwrap()
    })
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let data = event.payload.data;

    let results = task::spawn_blocking(move || {
        let pool = get_thread_pool();

        pool.install(|| {
            data.par_iter()
                .map(|item| process(item))
                .collect::<Vec<_>>()
        })
    })
    .await?;

    Ok(Response { results })
}
```

### Parallel Fold (Reduce)

```rust
fn parallel_sum(numbers: Vec<i64>) -> i64 {
    numbers
        .par_iter()
        .fold(|| 0i64, |acc, &x| acc + expensive_transform(x))
        .reduce(|| 0, |a, b| a + b)
}

fn expensive_transform(n: i64) -> i64 {
    // CPU work
    (0..1000).fold(n, |acc, _| acc.wrapping_mul(31))
}
```

### Parallel Chunks

```rust
use rayon::prelude::*;

fn process_in_chunks(data: Vec<u8>) -> Vec<Vec<u8>> {
    data.par_chunks(1024)  // Process in 1KB chunks
        .map(|chunk| {
            // Expensive processing per chunk
            compress_chunk(chunk)
        })
        .collect()
}

fn compress_chunk(chunk: &[u8]) -> Vec<u8> {
    // CPU-intensive compression
    chunk.to_vec()  // Placeholder
}
```

### Parallel Chain

```rust
fn multi_stage_processing(data: Vec<DataPoint>) -> Vec<Output> {
    data.par_iter()
        .filter(|point| point.is_valid())
        .map(|point| normalize(point))
        .map(|normalized| transform(normalized))
        .filter(|result| result.score > 0.5)
        .collect()
}
```

## Mixed IO + Compute Pattern

For functions that do both IO and compute:

```rust
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Phase 1: Async IO - Download data
    let raw_data = download_from_s3(&event.payload.bucket, &event.payload.key).await?;

    // Phase 2: Sync compute - Process data
    let processed = task::spawn_blocking(move || {
        process_data_parallel(raw_data)
    })
    .await??;

    // Phase 3: Async IO - Upload results
    upload_to_s3(&event.payload.output_bucket, &processed).await?;

    Ok(Response { success: true })
}

fn process_data_parallel(data: Vec<u8>) -> Result<Vec<ProcessedChunk>, Error> {
    // Parse and process in parallel
    let chunks: Vec<Vec<u8>> = data
        .chunks(1024)
        .map(|c| c.to_vec())
        .collect();

    let results = chunks
        .par_iter()
        .map(|chunk| {
            // CPU-intensive per chunk
            parse_and_transform(chunk)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}
```

## Image Processing Example

```rust
async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    for record in event.payload.records {
        let bucket = record.s3.bucket.name.unwrap();
        let key = record.s3.object.key.unwrap();

        // Async: Download image
        let image_data = download_from_s3(&bucket, &key).await?;

        // Sync: Process image with Rayon
        let processed = task::spawn_blocking(move || {
            process_image_parallel(image_data)
        })
        .await??;

        // Async: Upload result
        let output_key = format!("processed/{}", key);
        upload_to_s3(&bucket, &output_key, &processed).await?;
    }

    Ok(())
}

fn process_image_parallel(image_data: Vec<u8>) -> Result<Vec<u8>, Error> {
    // Parse image
    let img = parse_image(&image_data)?;
    let (width, height) = img.dimensions();

    // Process rows in parallel
    let rows: Vec<Vec<Pixel>> = (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    let pixel = img.get_pixel(x, y);
                    apply_filter(pixel)  // CPU-intensive
                })
                .collect()
        })
        .collect();

    // Flatten and encode
    encode_image(rows)
}
```

## Data Transformation Example

```rust
#[derive(Deserialize)]
struct CsvRow {
    id: String,
    values: Vec<f64>,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Async: Download CSV from S3
    let csv_data = download_csv(&event.payload.s3_key).await?;

    // Sync: Parse and transform with Rayon
    let transformed = task::spawn_blocking(move || {
        parse_and_transform_csv(csv_data)
    })
    .await??;

    // Async: Write to database
    write_to_database(&transformed).await?;

    Ok(Response { rows_processed: transformed.len() })
}

fn parse_and_transform_csv(csv_data: String) -> Result<Vec<TransformedRow>, Error> {
    let rows: Vec<CsvRow> = csv_data
        .lines()
        .skip(1)  // Skip header
        .map(|line| parse_csv_line(line))
        .collect::<Result<Vec<_>, _>>()?;

    // Parallel transformation
    let transformed = rows
        .par_iter()
        .map(|row| {
            // CPU-intensive transformation
            TransformedRow {
                id: row.id.clone(),
                mean: calculate_mean(&row.values),
                median: calculate_median(&row.values),
                std_dev: calculate_std_dev(&row.values),
                outliers: detect_outliers(&row.values),
            }
        })
        .collect();

    Ok(transformed)
}
```

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ComputeError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Computation failed: {0}")]
    ComputationFailed(String),

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let data = event.payload.data;

    if data.is_empty() {
        return Err(ComputeError::InvalidInput("Empty data".to_string()).into());
    }

    let results = task::spawn_blocking(move || {
        process_with_validation(data)
    })
    .await??;  // Handle both JoinError and computation errors

    Ok(Response { results })
}

fn process_with_validation(data: Vec<DataPoint>) -> Result<Vec<Output>, ComputeError> {
    let results: Result<Vec<_>, _> = data
        .par_iter()
        .map(|point| {
            if !point.is_valid() {
                return Err(ComputeError::InvalidInput(
                    format!("Invalid point: {:?}", point)
                ));
            }

            process_point(point)
                .map_err(|e| ComputeError::ComputationFailed(e.to_string()))
        })
        .collect();

    results
}
```

## Memory Configuration

For compute-intensive Lambda, more memory = more CPU:

```bash
# More memory for more CPU power
cargo lambda deploy my-function --memory 3008

# Lambda vCPU allocation:
# 1769 MB = 1 full vCPU
# 3008 MB = ~1.77 vCPU
# 10240 MB = 6 vCPU
```

**Recommendation**: Test different memory settings to find optimal cost/performance

## Performance Optimization Checklist

- [ ] Use `tokio::task::spawn_blocking` for CPU work
- [ ] Use Rayon `.par_iter()` for parallel processing
- [ ] Keep async only at IO boundaries
- [ ] Avoid async/await inside CPU-intensive functions
- [ ] Use appropriate Lambda memory (more memory = more CPU)
- [ ] Minimize data copying (use references where possible)
- [ ] Profile to find hot paths
- [ ] Consider chunking for very large datasets
- [ ] Use `par_chunks` for better cache locality
- [ ] Test with realistic data sizes
- [ ] Monitor CPU utilization in CloudWatch

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
lambda_runtime = "0.13"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
rayon = "1.10"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1"
thiserror = "1"

# Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Optional: CPU count
num_cpus = "1"

# For image processing (example)
# image = "0.24"

# For CSV processing (example)
# csv = "1"
```

## Testing Performance

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_performance() {
        let data = vec![DataPoint::new(); 1000];

        let start = std::time::Instant::now();

        let results = task::spawn_blocking(move || {
            process_data_parallel(data)
        })
        .await
        .unwrap();

        let duration = start.elapsed();

        println!("Processed {} items in {:?}", results.len(), duration);

        // Verify parallelism is effective
        assert!(duration.as_millis() < 5000);
    }

    #[test]
    fn test_computation() {
        let input = DataPoint::example();
        let result = cpu_intensive_work(&input);
        assert!(result.is_valid());
    }
}
```

## Benchmarking

Use Criterion for benchmarking:

```rust
// benches/compute_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_computation(c: &mut Criterion) {
    let data = vec![1i64; 10000];

    c.bench_function("sequential", |b| {
        b.iter(|| {
            data.iter()
                .map(|&n| black_box(expensive_computation(n)))
                .collect::<Vec<_>>()
        })
    });

    c.bench_function("parallel", |b| {
        b.iter(|| {
            data.par_iter()
                .map(|&n| black_box(expensive_computation(n)))
                .collect::<Vec<_>>()
        })
    });
}

criterion_group!(benches, benchmark_computation);
criterion_main!(benches);
```

Run with:
```bash
cargo bench
```

## Monitoring

Add instrumentation:

```rust
use tracing::{info, instrument};

#[instrument(skip(data))]
fn process_data_parallel(data: Vec<DataPoint>) -> Result<Vec<Output>, Error> {
    let start = std::time::Instant::now();
    let count = data.len();

    let results = data
        .par_iter()
        .map(|point| process_point(point))
        .collect::<Result<Vec<_>, _>>()?;

    let duration = start.elapsed();

    info!(
        count,
        duration_ms = duration.as_millis(),
        throughput = count as f64 / duration.as_secs_f64(),
        "Processing complete"
    );

    Ok(results)
}
```

After optimization, verify:
- CPU utilization is high (check CloudWatch)
- Execution time scales with data size
- Memory usage is within limits
- Cost is optimized for workload
