# Rust Data Engineering - Technical Context

This document provides deep technical context for building production data engineering systems in Rust using object_store, Arrow, Parquet, DataFusion, and Iceberg.

## Core Architecture Patterns

### 1. Object Store Abstraction Layer

The `object_store` crate provides a unified interface for cloud storage:

```rust
pub trait ObjectStore: std::fmt::Display + Send + Sync + Debug + 'static {
    async fn put(&self, location: &Path, bytes: Bytes) -> Result<PutResult>;
    async fn get(&self, location: &Path) -> Result<GetResult>;
    async fn delete(&self, location: &Path) -> Result<()>;
    async fn list(&self, prefix: Option<&Path>) -> BoxStream<'_, Result<ObjectMeta>>;
    async fn head(&self, location: &Path) -> Result<ObjectMeta>;
}
```

#### Key Implementations

**Amazon S3**:
```rust
use object_store::aws::AmazonS3Builder;

let s3 = AmazonS3Builder::new()
    .with_region("us-east-1")
    .with_bucket_name("my-bucket")
    .with_access_key_id(access_key)
    .with_secret_access_key(secret_key)
    // Advanced configs
    .with_retry(RetryConfig {
        max_retries: 3,
        retry_timeout: Duration::from_secs(10),
        ..Default::default()
    })
    .with_allow_http(false) // Enforce HTTPS
    .with_virtual_hosted_style_request(true) // Use virtual-hosted style URLs
    .build()?;
```

**Azure Blob Storage**:
```rust
use object_store::azure::MicrosoftAzureBuilder;

let azure = MicrosoftAzureBuilder::new()
    .with_account("myaccount")
    .with_container_name("mycontainer")
    .with_access_key(access_key)
    .build()?;
```

**Google Cloud Storage**:
```rust
use object_store::gcs::GoogleCloudStorageBuilder;

let gcs = GoogleCloudStorageBuilder::new()
    .with_service_account_key(service_account_json)
    .with_bucket_name("my-bucket")
    .build()?;
```

**Local Filesystem** (for testing):
```rust
use object_store::local::LocalFileSystem;

let local = LocalFileSystem::new_with_prefix("/tmp/data")?;
```

#### Best Practices

1. **Use Arc<dyn ObjectStore>** for shared ownership:
```rust
use std::sync::Arc;
use object_store::ObjectStore;

let store: Arc<dyn ObjectStore> = Arc::new(s3);
// Can clone Arc and share across threads/tasks
```

2. **Configure retries** for production resilience:
```rust
let retry_config = RetryConfig {
    max_retries: 3,
    retry_timeout: Duration::from_secs(10),
    backoff: BackoffConfig::default(),
};
```

3. **Use multipart uploads** for large files:
```rust
let multipart = store.put_multipart(&path).await?;
// Write chunks
multipart.put_part(bytes1).await?;
multipart.put_part(bytes2).await?;
// Commit
multipart.complete().await?;
```

4. **Stream large objects** instead of loading into memory:
```rust
let result = store.get(&path).await?;
let stream = result.into_stream();
// Process stream chunk by chunk
```

### 2. Apache Arrow Memory Model

Arrow provides a columnar in-memory format optimized for analytics.

#### Core Types

**RecordBatch**: Columnar data structure
```rust
use arrow::record_batch::RecordBatch;
use arrow::array::{Int32Array, StringArray};
use arrow::datatypes::{Schema, Field, DataType};
use std::sync::Arc;

let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Int32, false),
    Field::new("name", DataType::Utf8, false),
]));

let batch = RecordBatch::try_new(
    schema.clone(),
    vec![
        Arc::new(Int32Array::from(vec![1, 2, 3])),
        Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
    ],
)?;
```

**Schema Definition**:
```rust
use arrow::datatypes::{Schema, Field, DataType};

let schema = Schema::new(vec![
    Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
    Field::new("user_id", DataType::Utf8, false),
    Field::new("event_type", DataType::Utf8, false),
    Field::new("properties", DataType::Struct(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, true),
    ].into()), true),
    Field::new("metrics", DataType::List(Arc::new(
        Field::new("item", DataType::Float64, true)
    )), true),
]);
```

#### Arrow Compute Kernels

Use built-in compute functions for efficient operations:

```rust
use arrow::compute::kernels::{filter, aggregate, sort, cast};

// Filter
let predicate = /* BooleanArray */;
let filtered = filter::filter_record_batch(&batch, &predicate)?;

// Aggregate
let sum = aggregate::sum(batch.column(0))?;
let min = aggregate::min(batch.column(1))?;
let max = aggregate::max(batch.column(1))?;

// Sort
let sort_options = SortOptions::default();
let indices = sort::lexsort_to_indices(&[(&batch.column(0), sort_options)])?;
let sorted = filter::take_record_batch(&batch, &indices)?;

// Cast
let casted = cast::cast(batch.column(0), &DataType::Float64)?;
```

### 3. Parquet File Format

Parquet is a columnar storage format optimized for analytics.

#### File Structure

```
Parquet File
├── Row Group 1 (100MB - 1GB recommended)
│   ├── Column Chunk: id
│   │   ├── Data Pages (compressed)
│   │   └── Dictionary Page (optional)
│   ├── Column Chunk: name
│   └── Column Chunk: timestamp
├── Row Group 2
└── Footer (schema + metadata)
```

#### Writing Parquet Files

**Async Writer with object_store**:
```rust
use parquet::arrow::AsyncArrowWriter;
use parquet::basic::{Compression, Encoding, ZstdLevel};
use parquet::file::properties::{WriterProperties, WriterVersion};

// Configure writer properties
let props = WriterProperties::builder()
    .set_writer_version(WriterVersion::PARQUET_2_0)
    .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
    .set_encoding(Encoding::PLAIN)
    .set_dictionary_enabled(true) // Enable for low-cardinality columns
    .set_max_row_group_size(100_000_000) // 100M rows or 1GB uncompressed
    .set_data_page_size_limit(1024 * 1024) // 1MB pages
    .set_write_batch_size(1024)
    .set_created_by("my-app v1.0".to_string())
    .build();

// Write to object store
let path = Path::from("data/output.parquet");
let object_store_writer = object_store::buffered::BufWriter::new(store.clone(), path.clone());

let mut writer = AsyncArrowWriter::try_new(
    object_store_writer,
    schema.clone(),
    Some(props),
)?;

// Write batches
writer.write(&batch1).await?;
writer.write(&batch2).await?;

// Close and flush
writer.close().await?;
```

**Column-specific properties**:
```rust
use parquet::schema::types::ColumnPath;

let props = WriterProperties::builder()
    // Set compression per column
    .set_column_compression(
        ColumnPath::from("high_entropy_data"),
        Compression::ZSTD(ZstdLevel::try_new(6)?),
    )
    .set_column_compression(
        ColumnPath::from("low_entropy_data"),
        Compression::SNAPPY,
    )
    // Set encoding per column
    .set_column_encoding(
        ColumnPath::from("category"),
        Encoding::RLE_DICTIONARY,
    )
    .build();
```

#### Reading Parquet Files

**Async Reader with predicate pushdown**:
```rust
use parquet::arrow::async_reader::{ParquetObjectReader, ParquetRecordBatchStreamBuilder};
use parquet::arrow::ProjectionMask;
use parquet::file::metadata::ParquetMetaData;

// Create reader
let path = Path::from("data/input.parquet");
let meta = store.head(&path).await?;
let reader = ParquetObjectReader::new(store.clone(), meta);

// Build stream with optimizations
let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

// Column projection (only read needed columns)
let schema = builder.schema();
let projection = ProjectionMask::roots(schema, vec![0, 2, 5]); // Column indices
let builder = builder.with_projection(projection);

// Row group filtering (skip entire row groups)
let metadata: &ParquetMetaData = builder.metadata();
let row_groups_to_read: Vec<usize> = metadata
    .row_groups()
    .iter()
    .enumerate()
    .filter_map(|(idx, rg)| {
        // Check statistics to see if row group might contain data
        let stats = rg.column(0).statistics()?;
        if stats.min_bytes() >= threshold {
            Some(idx)
        } else {
            None
        }
    })
    .collect();
let builder = builder.with_row_groups(row_groups_to_read);

// Build stream
let mut stream = builder.build()?;
while let Some(batch) = stream.next().await {
    let batch = batch?;
    // Process batch
}
```

**Reading metadata only**:
```rust
let reader = ParquetObjectReader::new(store.clone(), meta);
let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;
let metadata = builder.metadata();

// Inspect schema
println!("Schema: {:?}", builder.schema());

// Inspect row groups
for (idx, rg) in metadata.row_groups().iter().enumerate() {
    println!("Row Group {}: {} rows", idx, rg.num_rows());

    // Column statistics
    for (col_idx, col) in rg.columns().iter().enumerate() {
        if let Some(stats) = col.statistics() {
            println!("  Column {}: min={:?}, max={:?}, null_count={:?}",
                col_idx, stats.min_bytes(), stats.max_bytes(), stats.null_count());
        }
    }
}
```

#### Compression Comparison

| Codec | Ratio | Speed | CPU | Use Case |
|-------|-------|-------|-----|----------|
| Uncompressed | 1x | Fastest | None | Local/fast networks only |
| Snappy | 2-3x | Very Fast | Low | Real-time streaming |
| ZSTD(3) | 3-4x | Fast | Medium | Balanced (recommended) |
| ZSTD(6) | 4-5x | Medium | High | Cold storage |
| ZSTD(9) | 5-6x | Slow | Very High | Archival |

**Recommendation**: Use ZSTD(3) for production data lakes. It provides excellent compression with minimal CPU overhead.

### 4. DataFusion Query Engine

DataFusion is a query execution framework built on Arrow.

#### Session Context Setup

```rust
use datafusion::prelude::*;
use datafusion::execution::context::SessionContext;
use datafusion::execution::runtime_env::{RuntimeEnv, RuntimeConfig};
use datafusion::execution::memory_pool::GreedyMemoryPool;

// Configure runtime
let runtime_config = RuntimeConfig::new()
    .with_memory_limit(4 * 1024 * 1024 * 1024) // 4GB
    .with_temp_file_path("/tmp/datafusion");

let runtime = Arc::new(RuntimeEnv::new(runtime_config)?);

// Create session context
let session_config = SessionConfig::new()
    .with_target_partitions(8) // Parallelism
    .with_batch_size(8192); // Batch size

let ctx = SessionContext::new_with_config_rt(session_config, runtime);
```

#### Registering Data Sources

**Parquet on object store**:
```rust
use datafusion::datasource::listing::{ListingOptions, ListingTableUrl};
use datafusion::datasource::file_format::parquet::ParquetFormat;

// Register object store
let url = "s3://my-bucket/";
ctx.runtime_env().register_object_store(
    &url::Url::parse(url)?,
    store.clone(),
);

// Register Parquet table with partitions
let table_path = "s3://my-bucket/data/events/";
let options = ParquetReadOptions::default();

ctx.register_parquet("events", table_path, options).await?;

// Or with more control
let listing_url = ListingTableUrl::parse(table_path)?;
let listing_options = ListingOptions::new(Arc::new(ParquetFormat::default()))
    .with_file_extension(".parquet")
    .with_target_partitions(ctx.state().config().target_partitions());

let table = listing_options
    .infer_table(&ctx.state(), &listing_url)
    .await?;

ctx.register_table("events", table)?;
```

**CSV files**:
```rust
use datafusion::datasource::file_format::csv::CsvFormat;

ctx.register_csv(
    "users",
    "s3://my-bucket/users.csv",
    CsvReadOptions::new()
        .has_header(true)
        .delimiter(b',')
        .schema_infer_max_records(1000),
).await?;
```

**In-memory RecordBatches**:
```rust
use datafusion::datasource::MemTable;

let mem_table = MemTable::try_new(schema.clone(), vec![vec![batch]])?;
ctx.register_table("temp_data", Arc::new(mem_table))?;
```

#### Executing Queries

**SQL queries**:
```rust
let df = ctx.sql("
    SELECT
        DATE_TRUNC('day', timestamp) as day,
        event_type,
        COUNT(*) as event_count,
        COUNT(DISTINCT user_id) as unique_users
    FROM events
    WHERE timestamp >= '2024-01-01'
        AND event_type IN ('click', 'view', 'purchase')
    GROUP BY 1, 2
    ORDER BY 1 DESC, 3 DESC
").await?;

// Collect results
let batches = df.collect().await?;

// Or stream results
let mut stream = df.execute_stream().await?;
while let Some(batch) = stream.next().await {
    let batch = batch?;
    // Process incrementally
}
```

**DataFrame API**:
```rust
let df = ctx.table("events").await?
    .filter(col("timestamp").gt(lit("2024-01-01")))?
    .select(vec![
        col("user_id"),
        col("event_type"),
        col("timestamp"),
    ])?
    .aggregate(
        vec![col("user_id")],
        vec![count(col("event_type")).alias("event_count")],
    )?
    .sort(vec![col("event_count").sort(false, true)])?; // DESC NULLS LAST

let results = df.collect().await?;
```

#### Custom Table Providers

For custom data sources:

```rust
use datafusion::datasource::{TableProvider, TableType};
use datafusion::execution::context::SessionState;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::logical_expr::TableProviderFilterPushDown;

#[derive(Debug)]
struct MyTableProvider {
    schema: SchemaRef,
    // Your data source
}

#[async_trait]
impl TableProvider for MyTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        state: &SessionState,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // Return execution plan that reads your data
        // Apply projection, filters, and limit for optimization
        todo!()
    }

    fn supports_filter_pushdown(
        &self,
        filter: &Expr,
    ) -> Result<TableProviderFilterPushDown> {
        // Indicate if you can push down this filter
        Ok(TableProviderFilterPushDown::Inexact)
    }
}
```

#### User-Defined Functions (UDFs)

**Scalar UDF**:
```rust
use datafusion::logical_expr::{create_udf, Volatility};
use arrow::array::StringArray;

let extract_domain = create_udf(
    "extract_domain",
    vec![DataType::Utf8],
    Arc::new(DataType::Utf8),
    Volatility::Immutable,
    Arc::new(|args: &[ColumnarValue]| {
        let urls = args[0].clone().into_array(1)?;
        let urls = urls.as_any().downcast_ref::<StringArray>().unwrap();

        let domains: StringArray = urls
            .iter()
            .map(|url| {
                url.and_then(|u| url::Url::parse(u).ok())
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
            })
            .collect();

        Ok(ColumnarValue::Array(Arc::new(domains)))
    }),
);

ctx.register_udf(extract_domain);

// Use in SQL
ctx.sql("SELECT extract_domain(url) FROM events").await?;
```

**Aggregate UDF (UDAF)**:
```rust
use datafusion::logical_expr::{create_udaf, AggregateUDF};
use datafusion::physical_plan::Accumulator;

// Implement custom aggregation (e.g., median, percentile)
```

### 5. Apache Iceberg Table Format

Iceberg provides ACID transactions, schema evolution, and time travel for data lakes.

#### Table Structure

```
s3://bucket/warehouse/db.db/events/
├── metadata/
│   ├── v1.metadata.json        # Schema, partition spec
│   ├── v2.metadata.json
│   └── snap-123.avro          # Snapshot manifest
├── data/
│   ├── year=2024/month=01/
│   │   ├── file1.parquet
│   │   └── file2.parquet
│   └── year=2024/month=02/
└── snapshots/
```

#### Creating Tables

```rust
use iceberg_rust::{
    catalog::{Catalog, CatalogBuilder},
    spec::{Schema, PartitionSpec, SortOrder, TableMetadata},
    table::Table,
};

// Define schema
let schema = Schema::builder()
    .with_field(
        Field::required("timestamp", Type::Timestamp).into(),
    )
    .with_field(
        Field::required("user_id", Type::String).into(),
    )
    .with_field(
        Field::optional("event_data", Type::String).into(),
    )
    .build();

// Define partitioning
let partition_spec = PartitionSpec::builder()
    .with_partition_field("timestamp", "year", Transform::Year)
    .with_partition_field("timestamp", "month", Transform::Month)
    .build();

// Define sort order (for data clustering)
let sort_order = SortOrder::builder()
    .with_order_field("timestamp", SortDirection::Ascending)
    .build();

// Create table
let catalog = /* ... */;
catalog.create_table(
    &namespace,
    "events",
    schema,
    partition_spec,
    sort_order,
).await?;
```

#### Reading with Time Travel

```rust
// Load table
let table = catalog.load_table(&table_id).await?;

// Read latest snapshot
let scan = table.scan().build()?;

// Read specific snapshot
let snapshot_id = table.metadata()
    .snapshots()
    .get(5)
    .snapshot_id();
let scan = table.scan()
    .snapshot_id(snapshot_id)
    .build()?;

// Read as of timestamp
let scan = table.scan()
    .as_of_timestamp(timestamp_ms)
    .build()?;

// Execute scan with DataFusion
let batches = scan.to_arrow().await?;
```

#### Schema Evolution

```rust
// Add column
let mut update = table.update_schema();
update.add_column("new_field", Type::String, true)?;
update.commit().await?;

// Rename column
update.rename_column("old_name", "new_name")?;
update.commit().await?;

// Delete column (metadata only, data preserved)
update.delete_column("unused_field")?;
update.commit().await?;

// Reorder columns
update.move_first("important_field")?;
update.move_after("field_a", "field_b")?;
update.commit().await?;
```

#### Partition Evolution

```rust
// Change partitioning without rewriting data
let mut update = table.update_partition_spec();
update.add_field("timestamp", "day", Transform::Day)?;
update.remove_field("month")?;
update.commit().await?;
```

#### ACID Transactions

```rust
// Atomic commit of new data
let mut append = table.append();
append.add_data_file(data_file)?;
append.commit().await?;

// Concurrent writes are serialized
// Last write wins with optimistic concurrency
```

## Performance Optimization Patterns

### 1. Partitioning Strategies

**Hive-style partitioning**:
```
data/
└── events/
    └── year=2024/
        └── month=01/
            └── day=15/
                └── part-00000.parquet
```

Benefits:
- Partition pruning in queries
- Easy to manage retention (delete old partitions)
- Standard across tools (Spark, Hive, Trino)

**Implementation**:
```rust
use object_store::path::Path;

fn partition_path(date: NaiveDate, partition: usize) -> Path {
    Path::from(format!(
        "events/year={}/month={:02}/day={:02}/part-{:05}.parquet",
        date.year(),
        date.month(),
        date.day(),
        partition
    ))
}
```

**Query optimization**:
```sql
-- Efficiently prunes to single partition
SELECT * FROM events
WHERE year = 2024 AND month = 1 AND day = 15

-- Scans entire table (year filter not on partition column)
SELECT * FROM events
WHERE timestamp >= '2024-01-15'
```

### 2. Row Group Sizing

**Optimal row group size**: 100MB - 1GB uncompressed

Too small:
- More overhead in footer
- Less efficient compression
- More small S3 requests

Too large:
- Can't skip irrelevant data
- Requires more memory

**Calculate row group size**:
```rust
let bytes_per_row = 500; // Estimate from schema
let target_row_group_bytes = 500 * 1024 * 1024; // 500MB
let max_row_group_size = target_row_group_bytes / bytes_per_row;

let props = WriterProperties::builder()
    .set_max_row_group_size(max_row_group_size)
    .build();
```

### 3. Predicate Pushdown

DataFusion automatically pushes predicates down to Parquet reader:

```rust
// Query
let df = ctx.sql("SELECT * FROM events WHERE user_id = 'user123'").await?;

// DataFusion pushes filter to Parquet reader
// Only row groups with matching statistics are read
// Only matching rows are decoded
```

**Enable statistics** when writing:
```rust
let props = WriterProperties::builder()
    .set_statistics_enabled(EnabledStatistics::Page) // Or Chunk
    .build();
```

### 4. Column Projection

Only read needed columns:

```rust
// Only reads 3 columns instead of all
let df = ctx.sql("SELECT user_id, timestamp, event_type FROM events").await?;
```

**Impact**: 10x+ speedup when selecting few columns from wide tables.

### 5. Compression Strategy

**By column type**:
- **IDs, timestamps**: ZSTD (good compression)
- **Text**: ZSTD with dictionary encoding
- **High-cardinality strings**: ZSTD without dictionary
- **Metrics**: ZSTD or Snappy

```rust
let props = WriterProperties::builder()
    .set_column_compression(
        ColumnPath::from("user_id"),
        Compression::ZSTD(ZstdLevel::try_new(3)?),
    )
    .set_column_dictionary_enabled(
        ColumnPath::from("user_id"),
        true,
    )
    .build();
```

### 6. Parallelism Tuning

**DataFusion parallelism**:
```rust
let config = SessionConfig::new()
    .with_target_partitions(num_cpus::get()); // Match CPU count

let ctx = SessionContext::new_with_config(config);
```

**Object store concurrency**:
```rust
// Limit concurrent S3 requests
let s3 = AmazonS3Builder::new()
    .with_client_options(ClientOptions::new()
        .with_allow_http(false)
        .with_timeout(Duration::from_secs(30))
        .with_connect_timeout(Duration::from_secs(5))
        .with_pool_max_idle_per_host(10) // Connection pool
    )
    .build()?;
```

## Common Anti-Patterns

### 1. Reading Entire Files into Memory

**Bad**:
```rust
let result = store.get(&path).await?;
let bytes = result.bytes().await?; // Loads entire file
```

**Good**:
```rust
let result = store.get(&path).await?;
let mut stream = result.into_stream();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // Process incrementally
}
```

### 2. Small Parquet Files

**Bad**: Writing many small files (< 10MB)

**Good**: Batch writes into larger files (100MB - 1GB)

```rust
// Accumulate batches
let mut batches = vec![];
for batch in source {
    batches.push(batch);

    if estimated_size(&batches) > 500 * 1024 * 1024 {
        write_parquet_file(&batches).await?;
        batches.clear();
    }
}
```

### 3. Not Using Predicate Pushdown

**Bad**:
```rust
// Read all data then filter
let df = ctx.table("events").await?;
let batches = df.collect().await?;
let filtered: Vec<_> = batches.into_iter()
    .filter(/* ... */)
    .collect();
```

**Good**:
```rust
// Push filter to Parquet reader
let df = ctx.sql("SELECT * FROM events WHERE date = '2024-01-01'").await?;
let batches = df.collect().await?;
```

### 4. Ignoring Schema Evolution

**Bad**: Assume schema never changes

**Good**: Handle optional fields and version mismatches:
```rust
// Schema v1: {id, name}
// Schema v2: {id, name, email}

// Old files will have NULL for email
let df = ctx.sql("SELECT id, name, COALESCE(email, 'unknown') FROM events").await?;
```

### 5. No Retry Logic

**Bad**: Single request fails entire pipeline

**Good**: Implement retry with backoff:
```rust
use object_store::RetryConfig;

let s3 = AmazonS3Builder::new()
    .with_retry(RetryConfig {
        max_retries: 3,
        retry_timeout: Duration::from_secs(10),
        ..Default::default()
    })
    .build()?;
```

## Testing Strategies

### 1. Use LocalFileSystem for Tests

```rust
#[cfg(test)]
mod tests {
    use object_store::local::LocalFileSystem;

    #[tokio::test]
    async fn test_pipeline() {
        let store = LocalFileSystem::new_with_prefix(
            tempfile::tempdir()?.path()
        )?;

        // Test with local store instead of S3
        run_pipeline(Arc::new(store)).await?;
    }
}
```

### 2. Property-Based Testing with Arrow

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip(data: Vec<i32>) {
        let array = Int32Array::from(data.clone());
        let batch = RecordBatch::try_new(schema, vec![Arc::new(array)])?;

        // Write to Parquet
        write_parquet(&batch).await?;

        // Read back
        let read_batch = read_parquet().await?;

        // Verify
        assert_eq!(batch, read_batch);
    }
}
```

### 3. Integration Tests with Testcontainers

```rust
use testcontainers::{clients::Cli, images::minio::MinIO};

#[tokio::test]
async fn test_with_minio() {
    let docker = Cli::default();
    let minio = docker.run(MinIO::default());

    let endpoint = format!("http://localhost:{}", minio.get_host_port_ipv4(9000));

    let s3 = AmazonS3Builder::new()
        .with_endpoint(endpoint)
        .with_access_key_id("minioadmin")
        .with_secret_access_key("minioadmin")
        .with_bucket_name("test")
        .with_allow_http(true)
        .build()?;

    // Test with real S3-compatible store
}
```

## Migration Patterns

### From CSV to Parquet

```rust
async fn convert_csv_to_parquet(
    store: Arc<dyn ObjectStore>,
    csv_path: &Path,
    parquet_path: &Path,
) -> Result<()> {
    // Read CSV with DataFusion
    let ctx = SessionContext::new();
    ctx.runtime_env().register_object_store("file://", store.clone());

    ctx.register_csv("data", csv_path.as_ref(), CsvReadOptions::default()).await?;

    // Query to DataFrame
    let df = ctx.table("data").await?;

    // Write as Parquet
    df.write_parquet(
        parquet_path.as_ref(),
        DataFrameWriteOptions::new(),
        Some(WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
            .build()),
    ).await?;

    Ok(())
}
```

### From JSON to Parquet

```rust
use datafusion::datasource::file_format::json::JsonFormat;

ctx.register_json("data", json_path, JsonReadOptions::default()).await?;
let df = ctx.table("data").await?;
df.write_parquet(parquet_path, DataFrameWriteOptions::new(), None).await?;
```

## Monitoring and Observability

### Tracking Metrics

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref BYTES_READ: Counter = Counter::new("bytes_read", "Bytes read").unwrap();
    static ref QUERY_DURATION: Histogram = Histogram::new("query_duration_seconds", "Query duration").unwrap();
}

async fn query_with_metrics(ctx: &SessionContext) -> Result<Vec<RecordBatch>> {
    let timer = QUERY_DURATION.start_timer();

    let batches = ctx.sql("SELECT ...").await?.collect().await?;

    let bytes: usize = batches.iter()
        .map(|b| b.get_array_memory_size())
        .sum();
    BYTES_READ.inc_by(bytes as f64);

    timer.observe_duration();
    Ok(batches)
}
```

### Logging Best Practices

```rust
use tracing::{info, warn, error, debug};

#[instrument(skip(store))]
async fn read_parquet(store: Arc<dyn ObjectStore>, path: &Path) -> Result<Vec<RecordBatch>> {
    info!("Reading Parquet file: {}", path);

    let meta = store.head(path).await?;
    debug!("File size: {} bytes", meta.size);

    let reader = ParquetObjectReader::new(store, meta);
    let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

    let num_row_groups = builder.metadata().num_row_groups();
    info!("Row groups: {}", num_row_groups);

    // ...
}
```

## Decision Trees

### When to use Iceberg vs. raw Parquet?

**Use Iceberg if**:
- Need ACID transactions
- Schema evolves frequently
- Want time travel queries
- Multiple writers updating same table
- Need partition evolution

**Use raw Parquet if**:
- Write-once, read-many workload
- Schema is stable
- Simpler infrastructure requirements
- Cost optimization (fewer metadata files)

### When to use DataFusion vs. direct Parquet reading?

**Use DataFusion if**:
- Need SQL queries
- Complex joins or aggregations
- Query multiple files/sources
- Want automatic optimization

**Use direct Parquet if**:
- Simple file reading
- Minimize dependencies
- Full control over I/O
- Streaming individual files

### Compression codec selection?

**Use ZSTD(3)** for:
- General purpose storage
- Balanced compression/speed
- Cloud data lakes

**Use Snappy** for:
- Real-time streaming
- CPU-constrained environments
- Frequently accessed hot data

**Use ZSTD(6-9)** for:
- Cold/archival storage
- Minimize storage costs
- Rarely accessed data

**Use Uncompressed** for:
- Extremely fast local networks
- Already compressed data
- Development/testing only
