---
description: Read Parquet files efficiently with predicate pushdown and column projection
---

# Read Parquet Files

Help the user read Parquet files from object storage with optimal performance using predicate pushdown, column projection, and row group filtering.

## Steps

1. **Add required dependencies**:
   ```toml
   [dependencies]
   parquet = "52"
   arrow = "52"
   object_store = "0.9"
   tokio = { version = "1", features = ["full"] }
   futures = "0.3"
   ```

2. **Create a basic Parquet reader** from object_store:
   ```rust
   use parquet::arrow::async_reader::{ParquetObjectReader, ParquetRecordBatchStreamBuilder};
   use object_store::{ObjectStore, path::Path};
   use arrow::record_batch::RecordBatch;
   use futures::stream::StreamExt;

   async fn read_parquet(
       store: Arc<dyn ObjectStore>,
       path: &str,
   ) -> Result<Vec<RecordBatch>> {
       let path = Path::from(path);

       // Get file metadata
       let meta = store.head(&path).await?;

       // Create reader
       let reader = ParquetObjectReader::new(store, meta);

       // Build stream
       let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;
       let mut stream = builder.build()?;

       // Collect batches
       let mut batches = Vec::new();
       while let Some(batch) = stream.next().await {
           batches.push(batch?);
       }

       Ok(batches)
   }
   ```

3. **Add column projection** to read only needed columns:
   ```rust
   use parquet::arrow::ProjectionMask;

   let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

   // Get schema to determine column indices
   let schema = builder.schema();
   println!("Available columns: {:?}", schema.fields());

   // Project specific columns by index
   let projection = ProjectionMask::roots(schema, vec![0, 2, 5]);
   let builder = builder.with_projection(projection);

   // Or project by column name (helper function)
   fn project_columns(builder: ParquetRecordBatchStreamBuilder<ParquetObjectReader>,
                      column_names: &[&str]) -> ParquetRecordBatchStreamBuilder<ParquetObjectReader> {
       let schema = builder.schema();
       let indices: Vec<usize> = column_names
           .iter()
           .filter_map(|name| schema.column_with_name(name).map(|(idx, _)| idx))
           .collect();

       let projection = ProjectionMask::roots(schema, indices);
       builder.with_projection(projection)
   }

   let builder = project_columns(builder, &["user_id", "timestamp", "event_type"]);
   ```

4. **Add row group filtering** using statistics:
   ```rust
   use parquet::file::metadata::ParquetMetaData;

   let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;
   let metadata = builder.metadata();

   // Filter row groups based on statistics
   let row_groups_to_read: Vec<usize> = metadata
       .row_groups()
       .iter()
       .enumerate()
       .filter_map(|(idx, rg)| {
           // Example: filter by min/max values
           let col_metadata = rg.column(0); // First column
           if let Some(stats) = col_metadata.statistics() {
               // Check if row group might contain relevant data
               // This is pseudo-code; actual implementation depends on data type
               if stats_match_predicate(stats) {
                   return Some(idx);
               }
           }
           None
       })
       .collect();

   let builder = builder.with_row_groups(row_groups_to_read);
   ```

5. **Implement streaming processing** for large files:
   ```rust
   async fn process_large_parquet(
       store: Arc<dyn ObjectStore>,
       path: &str,
   ) -> Result<()> {
       let path = Path::from(path);
       let meta = store.head(&path).await?;
       let reader = ParquetObjectReader::new(store, meta);

       let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

       // Limit batch size to control memory usage
       let builder = builder.with_batch_size(8192);

       let mut stream = builder.build()?;

       // Process batches incrementally
       while let Some(batch) = stream.next().await {
           let batch = batch?;

           // Process this batch
           println!("Processing batch with {} rows", batch.num_rows());
           process_batch(&batch)?;

           // Batch is dropped here, freeing memory
       }

       Ok(())
   }

   fn process_batch(batch: &RecordBatch) -> Result<()> {
       // Your processing logic
       Ok(())
   }
   ```

6. **Add comprehensive error handling**:
   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   enum ParquetReadError {
       #[error("Object store error: {0}")]
       ObjectStore(#[from] object_store::Error),

       #[error("Parquet error: {0}")]
       Parquet(#[from] parquet::errors::ParquetError),

       #[error("Arrow error: {0}")]
       Arrow(#[from] arrow::error::ArrowError),

       #[error("File not found: {0}")]
       FileNotFound(String),
   }

   async fn read_with_error_handling(
       store: Arc<dyn ObjectStore>,
       path: &str,
   ) -> Result<Vec<RecordBatch>, ParquetReadError> {
       let path = Path::from(path);

       // Check if file exists
       if !store.head(&path).await.is_ok() {
           return Err(ParquetReadError::FileNotFound(path.to_string()));
       }

       let meta = store.head(&path).await?;
       let reader = ParquetObjectReader::new(store, meta);
       let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;
       let mut stream = builder.build()?;

       let mut batches = Vec::new();
       while let Some(batch) = stream.next().await {
           batches.push(batch?);
       }

       Ok(batches)
   }
   ```

## Performance Optimization

**Reading with all optimizations**:
```rust
async fn optimized_read(
    store: Arc<dyn ObjectStore>,
    path: &str,
    columns: &[&str],
) -> Result<Vec<RecordBatch>> {
    let path = Path::from(path);
    let meta = store.head(&path).await?;
    let reader = ParquetObjectReader::new(store, meta);

    let mut builder = ParquetRecordBatchStreamBuilder::new(reader).await?;

    // 1. Column projection
    let schema = builder.schema();
    let indices: Vec<usize> = columns
        .iter()
        .filter_map(|name| schema.column_with_name(name).map(|(idx, _)| idx))
        .collect();
    let projection = ProjectionMask::roots(schema, indices);
    builder = builder.with_projection(projection);

    // 2. Batch size tuning
    builder = builder.with_batch_size(8192);

    // 3. Row group filtering (if applicable)
    // builder = builder.with_row_groups(filtered_row_groups);

    let mut stream = builder.build()?;

    let mut batches = Vec::new();
    while let Some(batch) = stream.next().await {
        batches.push(batch?);
    }

    Ok(batches)
}
```

## Reading Metadata Only

```rust
async fn read_metadata(
    store: Arc<dyn ObjectStore>,
    path: &str,
) -> Result<()> {
    let path = Path::from(path);
    let meta = store.head(&path).await?;
    let reader = ParquetObjectReader::new(store, meta);

    let builder = ParquetRecordBatchStreamBuilder::new(reader).await?;
    let metadata = builder.metadata();

    println!("Schema: {:?}", builder.schema());
    println!("Number of row groups: {}", metadata.num_row_groups());
    println!("Total rows: {}", metadata.file_metadata().num_rows());

    for (idx, rg) in metadata.row_groups().iter().enumerate() {
        println!("Row Group {}: {} rows", idx, rg.num_rows());

        for (col_idx, col) in rg.columns().iter().enumerate() {
            if let Some(stats) = col.statistics() {
                println!("  Column {}: min={:?}, max={:?}, null_count={:?}",
                    col_idx,
                    stats.min_bytes(),
                    stats.max_bytes(),
                    stats.null_count()
                );
            }
        }
    }

    Ok(())
}
```

## Common Patterns

**Reading multiple files in parallel**:
```rust
use futures::stream::{self, StreamExt};

async fn read_multiple_files(
    store: Arc<dyn ObjectStore>,
    paths: Vec<String>,
) -> Result<Vec<RecordBatch>> {
    let results = stream::iter(paths)
        .map(|path| {
            let store = store.clone();
            async move {
                read_parquet(store, &path).await
            }
        })
        .buffer_unordered(10) // Process 10 files concurrently
        .collect::<Vec<_>>()
        .await;

    // Flatten results
    let mut all_batches = Vec::new();
    for result in results {
        all_batches.extend(result?);
    }

    Ok(all_batches)
}
```

**Reading partitioned data**:
```rust
async fn read_partition(
    store: Arc<dyn ObjectStore>,
    base_path: &str,
    year: i32,
    month: u32,
) -> Result<Vec<RecordBatch>> {
    let partition_path = format!("{}/year={}/month={:02}/", base_path, year, month);

    // List all files in partition
    let prefix = Some(&Path::from(partition_path));
    let files: Vec<_> = store.list(prefix)
        .filter_map(|meta| async move {
            meta.ok().and_then(|m| {
                if m.location.as_ref().ends_with(".parquet") {
                    Some(m.location.to_string())
                } else {
                    None
                }
            })
        })
        .collect()
        .await;

    // Read all files
    read_multiple_files(store, files).await
}
```

## Best Practices

- **Use column projection** to read only needed columns (10x+ speedup for wide tables)
- **Stream large files** instead of collecting all batches into memory
- **Check metadata first** to understand file structure before reading
- **Use batch_size** to control memory usage (8192-65536 rows per batch)
- **Filter row groups** using statistics when possible
- **Read multiple files in parallel** for partitioned datasets
- **Handle schema evolution** by checking schema before processing

## Troubleshooting

**Out of memory errors**:
- Reduce batch size: `.with_batch_size(4096)`
- Stream instead of collecting: process batches one at a time
- Use column projection to read fewer columns

**Slow reads**:
- Enable column projection if reading wide tables
- Check if row group filtering is possible
- Increase parallelism when reading multiple files
- Verify network connectivity to object store

**Schema mismatch**:
- Read metadata first to inspect actual schema
- Handle optional columns that may not exist in older files
- Use schema evolution strategies from DataFusion
