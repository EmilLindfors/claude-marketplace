---
description: Execute SQL queries with DataFusion against Parquet, CSV, and in-memory data
---

# DataFusion Query Execution

Help the user set up DataFusion and execute SQL queries against data stored in object storage (Parquet, CSV) or in-memory.

## Steps

1. **Add required dependencies**:
   ```toml
   [dependencies]
   datafusion = "39"
   arrow = "52"
   object_store = "0.9"
   tokio = { version = "1", features = ["full"] }
   ```

2. **Create a DataFusion session context**:
   ```rust
   use datafusion::prelude::*;
   use datafusion::execution::context::{SessionContext, SessionConfig};
   use datafusion::execution::runtime_env::{RuntimeEnv, RuntimeConfig};
   use std::sync::Arc;

   async fn create_context() -> Result<SessionContext> {
       // Configure session
       let config = SessionConfig::new()
           .with_target_partitions(num_cpus::get()) // Match CPU count
           .with_batch_size(8192); // Rows per batch

       // Configure runtime
       let runtime_config = RuntimeConfig::new()
           .with_memory_limit(4 * 1024 * 1024 * 1024) // 4GB memory limit
           .with_temp_file_path("/tmp/datafusion");

       let runtime = Arc::new(RuntimeEnv::new(runtime_config)?);

       Ok(SessionContext::new_with_config_rt(config, runtime))
   }
   ```

3. **Register object store** for S3/Azure/GCS:
   ```rust
   use object_store::aws::AmazonS3Builder;

   async fn register_object_store(ctx: &SessionContext) -> Result<()> {
       // Create S3 store
       let s3 = AmazonS3Builder::from_env()
           .with_bucket_name("my-data-lake")
           .build()?;

       // Register with DataFusion
       let url = "s3://my-data-lake/";
       ctx.runtime_env().register_object_store(
           &url::Url::parse(url)?,
           Arc::new(s3),
       );

       Ok(())
   }
   ```

4. **Register Parquet tables**:
   ```rust
   use datafusion::datasource::listing::{
       ListingOptions,
       ListingTable,
       ListingTableConfig,
       ListingTableUrl,
   };
   use datafusion::datasource::file_format::parquet::ParquetFormat;

   async fn register_parquet_table(
       ctx: &SessionContext,
       table_name: &str,
       path: &str,
   ) -> Result<()> {
       // Simple registration
       ctx.register_parquet(
           table_name,
           path,
           ParquetReadOptions::default(),
       ).await?;

       Ok(())
   }

   // Advanced registration with partitioning
   async fn register_partitioned_table(
       ctx: &SessionContext,
       table_name: &str,
       path: &str,
   ) -> Result<()> {
       let table_path = ListingTableUrl::parse(path)?;

       let file_format = ParquetFormat::default();

       let listing_options = ListingOptions::new(Arc::new(file_format))
           .with_file_extension(".parquet")
           .with_target_partitions(ctx.state().config().target_partitions())
           .with_collect_stat(true); // Collect file statistics

       let config = ListingTableConfig::new(table_path)
           .with_listing_options(listing_options);

       let table = ListingTable::try_new(config)?;

       ctx.register_table(table_name, Arc::new(table))?;

       Ok(())
   }
   ```

5. **Execute SQL queries**:
   ```rust
   async fn execute_sql(ctx: &SessionContext, query: &str) -> Result<Vec<RecordBatch>> {
       // Create DataFrame from SQL
       let df = ctx.sql(query).await?;

       // Collect all results
       let batches = df.collect().await?;

       Ok(batches)
   }

   // Example queries
   async fn example_queries(ctx: &SessionContext) -> Result<()> {
       // Simple select
       let df = ctx.sql("
           SELECT user_id, event_type, COUNT(*) as count
           FROM events
           WHERE date >= '2024-01-01'
           GROUP BY user_id, event_type
           ORDER BY count DESC
           LIMIT 100
       ").await?;

       df.show().await?;

       // Window functions
       let df = ctx.sql("
           SELECT
               user_id,
               timestamp,
               amount,
               SUM(amount) OVER (
                   PARTITION BY user_id
                   ORDER BY timestamp
                   ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
               ) as running_total
           FROM transactions
       ").await?;

       df.show().await?;

       // Joins
       let df = ctx.sql("
           SELECT
               e.user_id,
               u.name,
               COUNT(*) as event_count
           FROM events e
           JOIN users u ON e.user_id = u.id
           GROUP BY e.user_id, u.name
       ").await?;

       df.show().await?;

       Ok(())
   }
   ```

6. **Use DataFrame API** as an alternative to SQL:
   ```rust
   use datafusion::prelude::*;

   async fn dataframe_api_examples(ctx: &SessionContext) -> Result<()> {
       // Get table
       let df = ctx.table("events").await?;

       // Filter
       let df = df.filter(col("timestamp").gt(lit("2024-01-01")))?;

       // Select columns
       let df = df.select(vec![
           col("user_id"),
           col("event_type"),
           col("timestamp"),
       ])?;

       // Aggregate
       let df = df.aggregate(
           vec![col("user_id"), col("event_type")],
           vec![
               count(col("*")).alias("count"),
               avg(col("duration")).alias("avg_duration"),
               max(col("timestamp")).alias("max_time"),
           ],
       )?;

       // Sort
       let df = df.sort(vec![
           col("count").sort(false, true), // DESC NULLS LAST
       ])?;

       // Limit
       let df = df.limit(0, Some(100))?;

       // Execute
       let batches = df.collect().await?;

       Ok(())
   }
   ```

7. **Stream results** for large queries:
   ```rust
   use futures::stream::StreamExt;

   async fn stream_query_results(
       ctx: &SessionContext,
       query: &str,
   ) -> Result<()> {
       let df = ctx.sql(query).await?;

       // Get streaming results
       let mut stream = df.execute_stream().await?;

       // Process batches incrementally
       let mut total_rows = 0;
       while let Some(batch) = stream.next().await {
           let batch = batch?;
           total_rows += batch.num_rows();

           // Process this batch
           process_batch(&batch)?;

           println!("Processed {} rows so far...", total_rows);
       }

       println!("Total rows: {}", total_rows);
       Ok(())
   }

   fn process_batch(batch: &RecordBatch) -> Result<()> {
       // Your processing logic
       Ok(())
   }
   ```

8. **Inspect query plans** for optimization:
   ```rust
   async fn explain_query(ctx: &SessionContext, query: &str) -> Result<()> {
       // Logical plan
       let logical_plan = ctx.sql(query).await?.into_optimized_plan()?;
       println!("Logical Plan:\n{}", logical_plan.display_indent());

       // Physical plan
       let df = ctx.sql(query).await?;
       let physical_plan = df.create_physical_plan().await?;
       println!("Physical Plan:\n{}", physical_plan.display_indent());

       // Or use EXPLAIN in SQL
       let df = ctx.sql(&format!("EXPLAIN {}", query)).await?;
       df.show().await?;

       Ok(())
   }
   ```

## Advanced Features

**Register CSV tables**:
```rust
use datafusion::datasource::file_format::csv::CsvFormat;

async fn register_csv(ctx: &SessionContext) -> Result<()> {
    ctx.register_csv(
        "users",
        "s3://my-bucket/users.csv",
        CsvReadOptions::new()
            .has_header(true)
            .delimiter(b',')
            .schema_infer_max_records(1000),
    ).await?;

    Ok(())
}
```

**Register in-memory tables**:
```rust
use datafusion::datasource::MemTable;

async fn register_memory_table(
    ctx: &SessionContext,
    name: &str,
    batches: Vec<RecordBatch>,
    schema: SchemaRef,
) -> Result<()> {
    let mem_table = MemTable::try_new(schema, vec![batches])?;
    ctx.register_table(name, Arc::new(mem_table))?;
    Ok(())
}
```

**Create temporary views**:
```rust
async fn create_view(ctx: &SessionContext) -> Result<()> {
    // Create view from query
    let df = ctx.sql("
        SELECT user_id, COUNT(*) as count
        FROM events
        GROUP BY user_id
    ").await?;

    ctx.register_table("user_counts", df.into_view())?;

    // Now query the view
    let results = ctx.sql("SELECT * FROM user_counts WHERE count > 100").await?;
    results.show().await?;

    Ok(())
}
```

**User-Defined Functions (UDFs)**:
```rust
use datafusion::logical_expr::{create_udf, Volatility, ColumnarValue};
use arrow::array::StringArray;

async fn register_udfs(ctx: &SessionContext) -> Result<()> {
    // Create scalar UDF
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

    // Use in query
    let df = ctx.sql("
        SELECT
            extract_domain(url) as domain,
            COUNT(*) as count
        FROM events
        GROUP BY domain
    ").await?;

    df.show().await?;

    Ok(())
}
```

**Write query results to Parquet**:
```rust
async fn write_query_results(
    ctx: &SessionContext,
    query: &str,
    output_path: &str,
) -> Result<()> {
    let df = ctx.sql(query).await?;

    // Write to Parquet
    df.write_parquet(
        output_path,
        DataFrameWriteOptions::new(),
        Some(WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
            .build()),
    ).await?;

    Ok(())
}
```

## Performance Optimization

**Partition pruning**:
```rust
// DataFusion automatically prunes partitions based on WHERE clauses
async fn partition_pruning_example(ctx: &SessionContext) -> Result<()> {
    // Assuming Hive-style partitioning: year=2024/month=01/...

    // This query only scans year=2024/month=01 partitions
    let df = ctx.sql("
        SELECT * FROM events
        WHERE year = 2024 AND month = 1
    ").await?;

    // Use EXPLAIN to verify partition pruning
    let explain = ctx.sql("EXPLAIN SELECT * FROM events WHERE year = 2024 AND month = 1").await?;
    explain.show().await?;

    Ok(())
}
```

**Predicate pushdown**:
```rust
// DataFusion pushes predicates to Parquet readers automatically
// This reads only relevant row groups based on statistics

let df = ctx.sql("
    SELECT * FROM events
    WHERE user_id = 'user123'
      AND timestamp >= '2024-01-01'
").await?;
```

**Projection pushdown**:
```rust
// Only requested columns are read from Parquet
let df = ctx.sql("
    SELECT user_id, timestamp
    FROM events
").await?; // Only reads user_id and timestamp columns
```

**Parallelism tuning**:
```rust
let config = SessionConfig::new()
    .with_target_partitions(16); // Increase for better parallelism

let ctx = SessionContext::new_with_config(config);
```

## Common Patterns

**Aggregating across partitions**:
```rust
async fn aggregate_partitions(ctx: &SessionContext) -> Result<()> {
    let df = ctx.sql("
        SELECT
            year,
            month,
            COUNT(*) as total_events,
            COUNT(DISTINCT user_id) as unique_users,
            AVG(duration) as avg_duration
        FROM events
        WHERE year = 2024
        GROUP BY year, month
        ORDER BY month
    ").await?;

    df.show().await?;
    Ok(())
}
```

**Time-series analysis**:
```rust
async fn time_series_analysis(ctx: &SessionContext) -> Result<()> {
    let df = ctx.sql("
        SELECT
            DATE_TRUNC('hour', timestamp) as hour,
            COUNT(*) as events_per_hour,
            AVG(value) as avg_value,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY value) as p95_value
        FROM metrics
        WHERE timestamp >= NOW() - INTERVAL '7 days'
        GROUP BY 1
        ORDER BY 1
    ").await?;

    df.show().await?;
    Ok(())
}
```

**Complex joins**:
```rust
async fn complex_join(ctx: &SessionContext) -> Result<()> {
    let df = ctx.sql("
        SELECT
            e.event_type,
            u.country,
            COUNT(*) as count,
            AVG(e.duration) as avg_duration
        FROM events e
        JOIN users u ON e.user_id = u.id
        LEFT JOIN subscriptions s ON u.id = s.user_id
        WHERE e.timestamp >= '2024-01-01'
          AND u.active = true
        GROUP BY e.event_type, u.country
        HAVING count > 100
        ORDER BY count DESC
    ").await?;

    df.show().await?;
    Ok(())
}
```

## Best Practices

- **Use partition pruning** by filtering on partition columns (year, month, day)
- **Select only needed columns** to leverage projection pushdown
- **Configure appropriate parallelism** based on CPU cores and data size
- **Use EXPLAIN** to verify query optimization
- **Stream large results** instead of collecting all at once
- **Register statistics** when creating tables for better query planning
- **Create views** for commonly used queries
- **Use UDFs** for custom business logic

## Troubleshooting

**Out of memory**:
- Reduce batch size: `.with_batch_size(4096)`
- Set memory limit: `.with_memory_limit()`
- Stream results instead of collecting
- Enable spilling to disk with temp_file_path

**Slow queries**:
- Use EXPLAIN to inspect query plan
- Verify partition pruning is working
- Check if predicates can be pushed down
- Increase parallelism: `.with_target_partitions()`
- Ensure object store is registered correctly

**Schema errors**:
- Verify table registration: `ctx.table("name").await?.schema()`
- Check for schema evolution in Parquet files
- Use explicit schema for CSV files
- Handle NULL values appropriately

**Partition not found**:
- Verify path format matches Hive partitioning
- Check object store URL registration
- List files to debug: `store.list(prefix).await`
