---
description: Write Parquet files with optimal compression, encoding, and row group sizing
---

# Write Parquet Files

Help the user write Parquet files to object storage with production-quality settings for compression, encoding, row group sizing, and statistics.

## Steps

1. **Add required dependencies**:
   ```toml
   [dependencies]
   parquet = "52"
   arrow = "52"
   object_store = "0.9"
   tokio = { version = "1", features = ["full"] }
   ```

2. **Create a basic Parquet writer**:
   ```rust
   use parquet::arrow::AsyncArrowWriter;
   use parquet::basic::{Compression, ZstdLevel};
   use parquet::file::properties::WriterProperties;
   use object_store::{ObjectStore, path::Path};
   use arrow::record_batch::RecordBatch;

   async fn write_parquet(
       store: Arc<dyn ObjectStore>,
       path: &str,
       batches: Vec<RecordBatch>,
       schema: SchemaRef,
   ) -> Result<()> {
       let path = Path::from(path);

       // Create buffered writer for object store
       let object_store_writer = object_store::buffered::BufWriter::new(
           store.clone(),
           path.clone()
       );

       // Create Arrow writer
       let mut writer = AsyncArrowWriter::try_new(
           object_store_writer,
           schema,
           None, // Use default properties
       )?;

       // Write batches
       for batch in batches {
           writer.write(&batch).await?;
       }

       // Close writer (flushes and finalizes file)
       writer.close().await?;

       Ok(())
   }
   ```

3. **Configure writer properties** for production use:
   ```rust
   use parquet::file::properties::{WriterProperties, WriterVersion};
   use parquet::basic::{Compression, Encoding, ZstdLevel};

   fn create_writer_properties() -> WriterProperties {
       WriterProperties::builder()
           // Use Parquet 2.0 format
           .set_writer_version(WriterVersion::PARQUET_2_0)

           // Compression: ZSTD level 3 (balanced)
           .set_compression(Compression::ZSTD(
               ZstdLevel::try_new(3).unwrap()
           ))

           // Row group size: ~500MB uncompressed or 100M rows
           .set_max_row_group_size(100_000_000)

           // Data page size: 1MB
           .set_data_page_size_limit(1024 * 1024)

           // Enable dictionary encoding
           .set_dictionary_enabled(true)

           // Write batch size
           .set_write_batch_size(1024)

           // Enable statistics for predicate pushdown
           .set_statistics_enabled(parquet::file::properties::EnabledStatistics::Page)

           // Metadata
           .set_created_by("my-app v1.0".to_string())

           .build()
   }

   async fn write_with_properties(
       store: Arc<dyn ObjectStore>,
       path: &str,
       batches: Vec<RecordBatch>,
       schema: SchemaRef,
   ) -> Result<()> {
       let path = Path::from(path);
       let writer_obj = object_store::buffered::BufWriter::new(store, path);

       let props = create_writer_properties();

       let mut writer = AsyncArrowWriter::try_new(
           writer_obj,
           schema,
           Some(props),
       )?;

       for batch in batches {
           writer.write(&batch).await?;
       }

       writer.close().await?;
       Ok(())
   }
   ```

4. **Set column-specific properties** for optimal encoding:
   ```rust
   use parquet::schema::types::ColumnPath;

   fn create_column_specific_properties() -> WriterProperties {
       WriterProperties::builder()
           // High-entropy data: use stronger compression
           .set_column_compression(
               ColumnPath::from("raw_data"),
               Compression::ZSTD(ZstdLevel::try_new(6).unwrap()),
           )

           // Low-cardinality columns: use dictionary encoding
           .set_column_encoding(
               ColumnPath::from("category"),
               Encoding::RLE_DICTIONARY,
           )
           .set_column_compression(
               ColumnPath::from("category"),
               Compression::SNAPPY,
           )

           // Timestamp columns: use delta encoding
           .set_column_encoding(
               ColumnPath::from("timestamp"),
               Encoding::DELTA_BINARY_PACKED,
           )

           // High-frequency data: faster compression
           .set_column_compression(
               ColumnPath::from("metric"),
               Compression::SNAPPY,
           )

           .build()
   }
   ```

5. **Implement streaming writes** for large datasets:
   ```rust
   use futures::stream::StreamExt;

   async fn write_stream(
       store: Arc<dyn ObjectStore>,
       path: &str,
       mut batch_stream: impl Stream<Item = Result<RecordBatch>> + Unpin,
       schema: SchemaRef,
   ) -> Result<()> {
       let path = Path::from(path);
       let writer_obj = object_store::buffered::BufWriter::new(store, path);

       let props = create_writer_properties();
       let mut writer = AsyncArrowWriter::try_new(writer_obj, schema, Some(props))?;

       // Write batches as they arrive
       while let Some(batch) = batch_stream.next().await {
           let batch = batch?;
           writer.write(&batch).await?;
       }

       writer.close().await?;
       Ok(())
   }
   ```

6. **Implement partitioned writes**:
   ```rust
   use chrono::NaiveDate;

   async fn write_partitioned(
       store: Arc<dyn ObjectStore>,
       base_path: &str,
       date: NaiveDate,
       partition_id: usize,
       batch: RecordBatch,
       schema: SchemaRef,
   ) -> Result<()> {
       // Create partitioned path: base/year=2024/month=01/day=15/part-00000.parquet
       let path = format!(
           "{}/year={}/month={:02}/day={:02}/part-{:05}.parquet",
           base_path,
           date.year(),
           date.month(),
           date.day(),
           partition_id
       );

       write_parquet(store, &path, vec![batch], schema).await
   }

   // Write multiple partitions
   async fn write_all_partitions(
       store: Arc<dyn ObjectStore>,
       base_path: &str,
       partitioned_data: HashMap<NaiveDate, Vec<RecordBatch>>,
       schema: SchemaRef,
   ) -> Result<()> {
       for (date, batches) in partitioned_data {
           for (partition_id, batch) in batches.into_iter().enumerate() {
               write_partitioned(
                   store.clone(),
                   base_path,
                   date,
                   partition_id,
                   batch,
                   schema.clone(),
               ).await?;
           }
       }
       Ok(())
   }
   ```

7. **Add proper error handling and validation**:
   ```rust
   use thiserror::Error;

   #[derive(Error, Debug)]
   enum ParquetWriteError {
       #[error("Object store error: {0}")]
       ObjectStore(#[from] object_store::Error),

       #[error("Parquet error: {0}")]
       Parquet(#[from] parquet::errors::ParquetError),

       #[error("Arrow error: {0}")]
       Arrow(#[from] arrow::error::ArrowError),

       #[error("Empty batch: cannot write empty data")]
       EmptyBatch,

       #[error("Schema mismatch: {0}")]
       SchemaMismatch(String),
   }

   async fn write_with_validation(
       store: Arc<dyn ObjectStore>,
       path: &str,
       batches: Vec<RecordBatch>,
       schema: SchemaRef,
   ) -> Result<(), ParquetWriteError> {
       // Validate input
       if batches.is_empty() {
           return Err(ParquetWriteError::EmptyBatch);
       }

       // Verify schema consistency
       for batch in &batches {
           if batch.schema() != schema {
               return Err(ParquetWriteError::SchemaMismatch(
                   format!("Batch schema does not match expected schema")
               ));
           }
       }

       let path = Path::from(path);
       let writer_obj = object_store::buffered::BufWriter::new(store, path);
       let props = create_writer_properties();

       let mut writer = AsyncArrowWriter::try_new(writer_obj, schema, Some(props))?;

       for batch in batches {
           writer.write(&batch).await?;
       }

       writer.close().await?;
       Ok(())
   }
   ```

## Performance Tuning

**Optimal row group sizing**:
```rust
// Calculate appropriate row group size based on data
fn calculate_row_group_size(schema: &Schema, target_bytes: usize) -> usize {
    // Estimate bytes per row
    let bytes_per_row: usize = schema
        .fields()
        .iter()
        .map(|field| estimate_field_size(field.data_type()))
        .sum();

    // Target ~500MB per row group
    target_bytes / bytes_per_row.max(1)
}

fn estimate_field_size(data_type: &DataType) -> usize {
    match data_type {
        DataType::Int32 => 4,
        DataType::Int64 => 8,
        DataType::Float64 => 8,
        DataType::Utf8 => 50, // Estimate average string length
        DataType::Timestamp(_, _) => 8,
        DataType::Boolean => 1,
        _ => 100, // Conservative estimate for complex types
    }
}

let row_group_size = calculate_row_group_size(&schema, 500 * 1024 * 1024);

let props = WriterProperties::builder()
    .set_max_row_group_size(row_group_size)
    .build();
```

**Compression codec selection**:
```rust
fn choose_compression(use_case: CompressionUseCase) -> Compression {
    match use_case {
        CompressionUseCase::Balanced => Compression::ZSTD(ZstdLevel::try_new(3).unwrap()),
        CompressionUseCase::MaxCompression => Compression::ZSTD(ZstdLevel::try_new(9).unwrap()),
        CompressionUseCase::FastWrite => Compression::SNAPPY,
        CompressionUseCase::FastRead => Compression::SNAPPY,
        CompressionUseCase::Archive => Compression::ZSTD(ZstdLevel::try_new(19).unwrap()),
    }
}

enum CompressionUseCase {
    Balanced,
    MaxCompression,
    FastWrite,
    FastRead,
    Archive,
}
```

## Common Patterns

**Batching small records**:
```rust
use arrow::array::{RecordBatchOptions, ArrayRef};

async fn batch_and_write<T>(
    store: Arc<dyn ObjectStore>,
    path: &str,
    records: Vec<T>,
    schema: SchemaRef,
    batch_size: usize,
) -> Result<()>
where
    T: IntoRecordBatch,
{
    let path = Path::from(path);
    let writer_obj = object_store::buffered::BufWriter::new(store, path);
    let props = create_writer_properties();

    let mut writer = AsyncArrowWriter::try_new(writer_obj, schema.clone(), Some(props))?;

    // Process in batches
    for chunk in records.chunks(batch_size) {
        let batch = records_to_batch(chunk, schema.clone())?;
        writer.write(&batch).await?;
    }

    writer.close().await?;
    Ok(())
}
```

**Append to existing files (via temp + rename)**:
```rust
// Parquet doesn't support appending, so read + rewrite
async fn append_to_parquet(
    store: Arc<dyn ObjectStore>,
    path: &str,
    new_batches: Vec<RecordBatch>,
) -> Result<()> {
    // 1. Read existing data
    let existing_batches = read_parquet(store.clone(), path).await?;

    // 2. Combine with new data
    let mut all_batches = existing_batches;
    all_batches.extend(new_batches);

    // 3. Write to temp location
    let temp_path = format!("{}.tmp", path);
    write_parquet(
        store.clone(),
        &temp_path,
        all_batches,
        schema,
    ).await?;

    // 4. Atomic rename
    let from = Path::from(temp_path);
    let to = Path::from(path);
    store.rename(&from, &to).await?;

    Ok(())
}
```

**Writing with progress tracking**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

async fn write_with_progress(
    store: Arc<dyn ObjectStore>,
    path: &str,
    batches: Vec<RecordBatch>,
    schema: SchemaRef,
) -> Result<()> {
    let pb = ProgressBar::new(batches.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
    );

    let path = Path::from(path);
    let writer_obj = object_store::buffered::BufWriter::new(store, path);
    let props = create_writer_properties();

    let mut writer = AsyncArrowWriter::try_new(writer_obj, schema, Some(props))?;

    for (idx, batch) in batches.iter().enumerate() {
        writer.write(batch).await?;
        pb.set_position(idx as u64 + 1);
        pb.set_message(format!("{} rows written", batch.num_rows()));
    }

    writer.close().await?;
    pb.finish_with_message("Complete");

    Ok(())
}
```

## Best Practices

- **Use ZSTD(3) compression** for balanced performance (recommended for production)
- **Set row group size to 100MB-1GB** uncompressed for optimal S3 scanning
- **Enable statistics** for predicate pushdown optimization
- **Use dictionary encoding** for low-cardinality columns (categories, enums)
- **Write to temp location + rename** for atomic writes
- **Partition large datasets** by date or other logical grouping
- **Set column-specific properties** for heterogeneous data
- **Validate schema consistency** across all batches before writing

## Troubleshooting

**Slow writes**:
- Reduce compression level (use SNAPPY or ZSTD(1))
- Increase row group size to reduce overhead
- Use buffered writer (already included in examples)
- Write multiple files in parallel

**Large file sizes**:
- Increase compression level (ZSTD(6-9))
- Enable dictionary encoding for appropriate columns
- Check for redundant data that could be normalized

**Memory issues**:
- Reduce batch size
- Write smaller row groups
- Stream data instead of collecting all batches first

**Compatibility issues**:
- Use WriterVersion::PARQUET_2_0 for best compatibility
- Avoid advanced features if targeting older readers
- Test with target systems (Spark, Hive, etc.)

## Compression Comparison

| Codec | Write Speed | Read Speed | Ratio | Best For |
|-------|-------------|------------|-------|----------|
| Uncompressed | Fastest | Fastest | 1x | Development only |
| SNAPPY | Very Fast | Very Fast | 2-3x | Hot data, real-time |
| ZSTD(1) | Fast | Fast | 2.5-3x | High write throughput |
| ZSTD(3) | Fast | Fast | 3-4x | **Production default** |
| ZSTD(6) | Medium | Fast | 4-5x | Cold storage |
| ZSTD(9) | Slow | Fast | 5-6x | Archive, long-term |
