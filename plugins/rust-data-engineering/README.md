# Rust Data Engineering Plugin

Expert guidance for building data pipelines, lakehouse architectures, and analytical workloads in Rust using modern data engineering tools.

## Overview

This plugin provides comprehensive support for Rust-based data engineering workflows, focusing on cloud-native data lakes, analytical query engines, and efficient data formats. Perfect for teams building ETL pipelines, lakehouse architectures, or analytical applications.

## Key Technologies

- **object_store** - Unified cloud storage abstraction (S3, Azure Blob, GCS, local)
- **Apache Arrow** - High-performance columnar memory format
- **Apache Parquet** - Efficient columnar storage format
- **DataFusion** - Fast SQL query engine built on Arrow
- **Apache Iceberg** - Table format for large analytical datasets
- **CSV/JSON** - Structured data parsing and writing

## Features

### Commands

#### Cloud Storage & Object Store

- `/data-object-store-setup` - Configure object_store for S3, Azure, GCS, or local storage
- `/data-object-store-read` - Read objects with streaming, buffering, and retry strategies
- `/data-object-store-write` - Write objects with multipart uploads and atomic operations

#### Parquet Operations

- `/data-parquet-read` - Read Parquet files with predicate pushdown and column projection
- `/data-parquet-write` - Write Parquet files with compression, encoding, and row group tuning
- `/data-parquet-schema` - Define and evolve Parquet schemas

#### DataFusion Queries

- `/data-datafusion-setup` - Create DataFusion execution context and register data sources
- `/data-datafusion-query` - Execute SQL queries against Parquet, CSV, and in-memory data
- `/data-datafusion-custom` - Build custom table providers and UDFs

#### Iceberg Tables

- `/data-iceberg-table` - Create and manage Iceberg tables
- `/data-iceberg-query` - Query Iceberg tables with time travel
- `/data-iceberg-evolution` - Handle schema evolution and partitioning

#### Data Pipeline Patterns

- `/data-pipeline-etl` - Design ETL pipelines with streaming and batch processing
- `/data-partition-strategy` - Implement partitioning strategies (Hive, date-based, custom)
- `/data-schema-evolution` - Handle schema changes safely across versions

### Expert Agent

- `@data-engineering-expert` - Specialized agent for data engineering architecture decisions, performance optimization, and data lake best practices

## Installation

Add this plugin to your Claude Code marketplace:

```bash
# From the marketplace root
cd .claude-plugin
# Plugin will be auto-discovered from plugins/rust-data-engineering
```

The plugin includes commands and an expert agent for data engineering workflows.

## Quick Start

### 1. Set Up Object Store

```bash
/data-object-store-setup
```

Creates object_store configuration for your cloud provider:

```rust
use object_store::{aws::AmazonS3Builder, ObjectStore};
use std::sync::Arc;

let s3 = AmazonS3Builder::new()
    .with_region("us-east-1")
    .with_bucket_name("my-data-lake")
    .with_access_key_id(access_key)
    .with_secret_access_key(secret_key)
    .build()?;

let store: Arc<dyn ObjectStore> = Arc::new(s3);
```

### 2. Read Parquet Files

```bash
/data-parquet-read
```

Efficiently read Parquet data with predicate pushdown:

```rust
use parquet::arrow::async_reader::ParquetObjectReader;
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use object_store::path::Path;

let path = Path::from("data/events/year=2024/month=01/data.parquet");
let meta = store.head(&path).await?;

let reader = ParquetObjectReader::new(store.clone(), meta);
let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

// Column projection
let builder = builder.with_projection(vec![0, 2, 5]);

// Predicate pushdown
let builder = builder.with_row_filter(/* ... */);

let mut stream = builder.build()?;
while let Some(batch) = stream.next().await {
    let batch = batch?;
    // Process RecordBatch
}
```

### 3. Query with DataFusion

```bash
/data-datafusion-query
```

Run SQL queries against cloud-stored Parquet files:

```rust
use datafusion::prelude::*;
use datafusion::execution::context::SessionContext;

let ctx = SessionContext::new();

// Register object store
let url = "s3://my-data-lake/";
ctx.runtime_env()
    .register_object_store(url, store.clone());

// Register Parquet table
ctx.register_parquet(
    "events",
    "s3://my-data-lake/data/events/",
    ParquetReadOptions::default(),
).await?;

// Execute SQL
let df = ctx.sql("
    SELECT user_id, COUNT(*) as event_count
    FROM events
    WHERE date >= '2024-01-01'
    GROUP BY user_id
    ORDER BY event_count DESC
    LIMIT 100
").await?;

df.show().await?;
```

### 4. Work with Iceberg Tables

```bash
/data-iceberg-table
```

Create and manage Iceberg tables for ACID guarantees and schema evolution:

```rust
use iceberg_rust::{Table, TableIdentifier};
use iceberg_rust::catalog::Catalog;

// Open Iceberg table
let catalog = /* ... */;
let table = catalog.load_table(
    &TableIdentifier::new(&["db", "events"])
).await?;

// Time travel query
let snapshot_id = table.metadata()
    .snapshots()
    .get(5)
    .snapshot_id();

let scan = table.scan()
    .snapshot_id(snapshot_id)
    .build()?;

// Schema evolution
let mut update = table.update_schema();
update.add_column("new_field", DataType::String)?;
update.commit().await?;
```

## Common Workflows

### ETL Pipeline Architecture

Use `/data-pipeline-etl` to design production ETL pipelines:

1. **Extract** - Read from object_store with retry and backpressure
2. **Transform** - Use DataFusion or Arrow compute kernels
3. **Load** - Write Parquet with optimal row group sizes and compression

```rust
// Streaming ETL
async fn process_batch(
    store: Arc<dyn ObjectStore>,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    // Read with streaming
    let reader = ParquetObjectReader::new(/* ... */);
    let stream = ParquetRecordBatchStreamBuilder::new(reader)
        .await?
        .build()?;

    // Transform with Arrow compute
    let transformed = stream.map(|batch| {
        let batch = batch?;
        apply_transformations(batch)
    });

    // Write with optimal settings
    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::default()))
        .set_max_row_group_size(1_000_000)
        .build();

    write_parquet_stream(store, output_path, transformed, props).await?;
    Ok(())
}
```

### Partitioning Strategy

Use `/data-partition-strategy` for efficient data organization:

```
data-lake/
└── events/
    ├── year=2024/
    │   ├── month=01/
    │   │   ├── day=01/
    │   │   │   ├── part-00000.parquet
    │   │   │   └── part-00001.parquet
    │   │   └── day=02/
    │   └── month=02/
```

Enables partition pruning in queries:
```sql
SELECT * FROM events
WHERE year = 2024 AND month = 1 AND day >= 15
-- Only scans relevant partitions
```

### Schema Evolution

Use `/data-schema-evolution` to safely evolve schemas:

```rust
// Add optional fields (backward compatible)
schema_update.add_column("new_metric", DataType::Float64)?;

// Rename fields (maintain compatibility)
schema_update.rename_column("old_name", "new_name")?;

// Parquet handles missing fields gracefully
// Old files return NULL for new columns
```

## Performance Best Practices

### Object Store Optimization

- Use `with_retry()` for resilient cloud operations
- Enable `with_concurrent_request_limit()` for rate limiting
- Configure multipart uploads for large files (>100MB)

### Parquet Tuning

- **Row group size**: 100MB-1GB for optimal S3 scanning
- **Compression**: ZSTD for balanced compression/speed, Snappy for speed
- **Page size**: 1MB default works well for most cases
- **Column encoding**: Use dictionary encoding for low-cardinality columns

### DataFusion Query Optimization

- Enable predicate pushdown and column projection
- Use `EXPLAIN` to inspect query plans
- Partition data to enable partition pruning
- Create statistics for better query planning

### Iceberg Benefits

- **ACID transactions**: Atomic commits prevent partial updates
- **Time travel**: Query historical table states
- **Schema evolution**: Add/rename/reorder columns safely
- **Partition evolution**: Change partitioning without rewriting data
- **Hidden partitioning**: Partition on derived values transparently

## Integration Examples

### S3 + Parquet + DataFusion

```rust
use object_store::aws::AmazonS3Builder;
use datafusion::prelude::*;

// S3 object store
let s3 = AmazonS3Builder::from_env()
    .with_bucket_name("data-lake")
    .build()?;

// DataFusion context
let ctx = SessionContext::new();
ctx.runtime_env()
    .register_object_store("s3://data-lake/", Arc::new(s3));

// Query Parquet on S3
ctx.register_parquet("sales", "s3://data-lake/sales/", Default::default()).await?;
let df = ctx.sql("SELECT product, SUM(amount) FROM sales GROUP BY product").await?;
```

### Streaming ETL with Arrow

```rust
use arrow::compute::kernels::filter;
use arrow::compute::kernels::aggregate;

// Stream large datasets
let mut stream = parquet_stream;
while let Some(batch) = stream.next().await {
    let batch = batch?;

    // Filter rows
    let predicate = /* boolean array */;
    let filtered = filter::filter_record_batch(&batch, &predicate)?;

    // Compute aggregates
    let values = filtered.column(0);
    let sum = aggregate::sum(values)?;

    // Write results
}
```

## Troubleshooting

### Large File Memory Issues

**Problem**: OOM when reading large Parquet files

**Solution**: Use streaming readers and limit batch size:
```rust
let builder = ParquetRecordBatchStreamBuilder::new(reader)
    .await?
    .with_batch_size(8192); // Smaller batches
```

### S3 Rate Limiting

**Problem**: 503 SlowDown errors from S3

**Solution**: Configure retry and backoff:
```rust
let s3 = AmazonS3Builder::new()
    .with_retry(RetryConfig::default())
    .build()?;
```

### Schema Mismatch Errors

**Problem**: Incompatible schemas when reading multiple files

**Solution**: Use schema evolution or unified schema:
```rust
// Read with schema adaptation
let options = ParquetReadOptions {
    schema_force_view: Some(unified_schema),
    ..Default::default()
};
```

## Resources

- [object_store crate](https://docs.rs/object_store/)
- [Apache Arrow Rust](https://docs.rs/arrow/)
- [Apache Parquet Rust](https://docs.rs/parquet/)
- [DataFusion User Guide](https://arrow.apache.org/datafusion/)
- [Apache Iceberg Rust](https://github.com/apache/iceberg-rust)
- [Parquet Format Specification](https://parquet.apache.org/docs/)

## Author

Emil Lindfors

## Version

1.0.0
