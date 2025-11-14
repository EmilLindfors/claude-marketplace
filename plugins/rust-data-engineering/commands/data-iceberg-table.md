---
description: Create and manage Apache Iceberg tables with ACID transactions and schema evolution
---

# Apache Iceberg Tables

Help the user work with Apache Iceberg tables for data lakes with ACID transactions, time travel, and schema evolution capabilities.

## Steps

1. **Add required dependencies**:
   ```toml
   [dependencies]
   iceberg = "0.3"
   iceberg-catalog-rest = "0.3"
   arrow = "52"
   parquet = "52"
   object_store = "0.9"
   tokio = { version = "1", features = ["full"] }
   ```

2. **Set up Iceberg catalog**:
   ```rust
   use iceberg::{Catalog, TableIdent};
   use iceberg_catalog_rest::RestCatalog;

   async fn create_catalog() -> Result<RestCatalog> {
       // REST catalog (works with services like Polaris, Nessie, etc.)
       let catalog = RestCatalog::new(
           "http://localhost:8181",  // Catalog endpoint
           "warehouse",               // Warehouse location
       ).await?;

       Ok(catalog)
   }

   // For AWS Glue catalog
   // use iceberg_catalog_glue::GlueCatalog;

   // For file-based catalog (development)
   // use iceberg::catalog::FileCatalog;
   ```

3. **Create an Iceberg table**:
   ```rust
   use iceberg::{
       spec::{Schema, NestedField, PrimitiveType, Type},
       NamespaceIdent, TableCreation,
   };

   async fn create_table(catalog: &impl Catalog) -> Result<()> {
       // Define schema
       let schema = Schema::builder()
           .with_fields(vec![
               NestedField::required(1, "id", Type::Primitive(PrimitiveType::Long)),
               NestedField::required(2, "timestamp", Type::Primitive(PrimitiveType::Timestamp)),
               NestedField::required(3, "user_id", Type::Primitive(PrimitiveType::String)),
               NestedField::optional(4, "event_type", Type::Primitive(PrimitiveType::String)),
               NestedField::optional(5, "properties", Type::Primitive(PrimitiveType::String)),
           ])
           .build()?;

       // Define partitioning
       let partition_spec = iceberg::spec::PartitionSpec::builder()
           .with_spec_id(0)
           .add_partition_field(2, "year", iceberg::spec::Transform::Year)? // Partition by year
           .add_partition_field(2, "month", iceberg::spec::Transform::Month)? // Partition by month
           .build()?;

       // Define sort order (for data clustering)
       let sort_order = iceberg::spec::SortOrder::builder()
           .with_order_id(0)
           .add_sort_field(
               iceberg::spec::SortField::builder()
                   .source_id(2) // timestamp field
                   .direction(iceberg::spec::SortDirection::Ascending)
                   .null_order(iceberg::spec::NullOrder::First)
                   .build(),
           )
           .build()?;

       // Create table
       let table_creation = TableCreation::builder()
           .name("events".to_string())
           .schema(schema)
           .partition_spec(partition_spec)
           .sort_order(sort_order)
           .build();

       let namespace = NamespaceIdent::new("db".to_string());
       let table_ident = TableIdent::new(namespace, "events".to_string());

       catalog.create_table(&table_ident, table_creation).await?;

       println!("Table created: db.events");
       Ok(())
   }
   ```

4. **Load an existing table**:
   ```rust
   async fn load_table(catalog: &impl Catalog) -> Result<iceberg::Table> {
       let namespace = NamespaceIdent::new("db".to_string());
       let table_ident = TableIdent::new(namespace, "events".to_string());

       let table = catalog.load_table(&table_ident).await?;

       // Inspect table metadata
       println!("Schema: {:?}", table.metadata().current_schema());
       println!("Location: {}", table.metadata().location());
       println!("Snapshots: {}", table.metadata().snapshots().len());

       Ok(table)
   }
   ```

5. **Write data to Iceberg table**:
   ```rust
   use iceberg::writer::{IcebergWriter, RecordBatchWriter};
   use arrow::record_batch::RecordBatch;

   async fn write_data(
       table: &iceberg::Table,
       batches: Vec<RecordBatch>,
   ) -> Result<()> {
       // Create writer
       let mut writer = table
           .writer()
           .partition_by(table.metadata().default_partition_spec()?)
           .build()
           .await?;

       // Write batches
       for batch in batches {
           writer.write(&batch).await?;
       }

       // Commit (ACID transaction)
       let data_files = writer.close().await?;

       // Create snapshot
       let mut append = table.new_append();
       for file in data_files {
           append.add_data_file(file)?;
       }
       append.commit().await?;

       println!("Data written and committed");
       Ok(())
   }
   ```

6. **Read data with time travel**:
   ```rust
   use iceberg::scan::{TableScan, TableScanBuilder};

   async fn read_latest(table: &iceberg::Table) -> Result<Vec<RecordBatch>> {
       // Read latest snapshot
       let scan = table.scan().build().await?;

       let batches = scan.to_arrow().await?;

       Ok(batches)
   }

   async fn read_snapshot(
       table: &iceberg::Table,
       snapshot_id: i64,
   ) -> Result<Vec<RecordBatch>> {
       // Time travel to specific snapshot
       let scan = table
           .scan()
           .snapshot_id(snapshot_id)
           .build()
           .await?;

       let batches = scan.to_arrow().await?;

       Ok(batches)
   }

   async fn read_as_of_timestamp(
       table: &iceberg::Table,
       timestamp_ms: i64,
   ) -> Result<Vec<RecordBatch>> {
       // Time travel to specific timestamp
       let scan = table
           .scan()
           .as_of_timestamp(timestamp_ms)
           .build()
           .await?;

       let batches = scan.to_arrow().await?;

       Ok(batches)
   }
   ```

7. **Perform schema evolution**:
   ```rust
   async fn evolve_schema(table: &mut iceberg::Table) -> Result<()> {
       // Add new column
       let mut update = table.update_schema();
       update
           .add_column("new_field", Type::Primitive(PrimitiveType::String), true)?
           .commit()
           .await?;

       println!("Added column: new_field");

       // Rename column
       let mut update = table.update_schema();
       update
           .rename_column("old_name", "new_name")?
           .commit()
           .await?;

       println!("Renamed column: old_name -> new_name");

       // Delete column (metadata only)
       let mut update = table.update_schema();
       update
           .delete_column("unused_field")?
           .commit()
           .await?;

       println!("Deleted column: unused_field");

       // Update column type (limited support)
       let mut update = table.update_schema();
       update
           .update_column("numeric_field", Type::Primitive(PrimitiveType::Double))?
           .commit()
           .await?;

       // Reorder columns
       let mut update = table.update_schema();
       update
           .move_first("important_field")?
           .move_after("field_a", "field_b")?
           .commit()
           .await?;

       Ok(())
   }
   ```

8. **Query history and snapshots**:
   ```rust
   async fn inspect_history(table: &iceberg::Table) -> Result<()> {
       let metadata = table.metadata();

       // List all snapshots
       println!("Snapshots:");
       for snapshot in metadata.snapshots() {
           println!(
               "  ID: {}, Timestamp: {}, Summary: {:?}",
               snapshot.snapshot_id(),
               snapshot.timestamp_ms(),
               snapshot.summary()
           );
       }

       // Get current snapshot
       if let Some(current) = metadata.current_snapshot() {
           println!("Current snapshot: {}", current.snapshot_id());
           println!("Manifest list: {}", current.manifest_list());
       }

       // Get schema history
       println!("\nSchema versions:");
       for schema in metadata.schemas() {
           println!("  Schema ID {}: {} fields", schema.schema_id(), schema.fields().len());
       }

       Ok(())
   }
   ```

## Advanced Features

**Partition evolution**:
```rust
async fn evolve_partitioning(table: &mut iceberg::Table) -> Result<()> {
    // Change partition strategy without rewriting data
    let mut update = table.update_partition_spec();

    // Add day partitioning
    update.add_field(
        "timestamp",
        "day",
        iceberg::spec::Transform::Day,
    )?;

    // Remove old month partitioning
    update.remove_field("month")?;

    update.commit().await?;

    println!("Partition spec evolved");
    Ok(())
}
```

**Hidden partitioning**:
```rust
// Iceberg supports hidden partitioning - partition on derived values
// Users don't need to specify partition columns in queries

async fn create_table_with_hidden_partitioning(catalog: &impl Catalog) -> Result<()> {
    let schema = Schema::builder()
        .with_fields(vec![
            NestedField::required(1, "timestamp", Type::Primitive(PrimitiveType::Timestamp)),
            NestedField::required(2, "data", Type::Primitive(PrimitiveType::String)),
        ])
        .build()?;

    // Partition by year(timestamp) and month(timestamp)
    // But timestamp is a regular column, not a partition column
    let partition_spec = iceberg::spec::PartitionSpec::builder()
        .add_partition_field(1, "year", iceberg::spec::Transform::Year)?
        .add_partition_field(1, "month", iceberg::spec::Transform::Month)?
        .build()?;

    // Now queries like:
    // SELECT * FROM table WHERE timestamp >= '2024-01-01'
    // Will automatically use partition pruning

    Ok(())
}
```

**Incremental reads**:
```rust
async fn incremental_read(
    table: &iceberg::Table,
    from_snapshot_id: i64,
    to_snapshot_id: Option<i64>,
) -> Result<Vec<RecordBatch>> {
    // Read only data added between snapshots
    let scan = table
        .scan()
        .from_snapshot_id(from_snapshot_id)
        .snapshot_id(to_snapshot_id.unwrap_or_else(|| {
            table.metadata().current_snapshot().unwrap().snapshot_id()
        }))
        .build()
        .await?;

    let batches = scan.to_arrow().await?;

    Ok(batches)
}
```

**Filtering and projection**:
```rust
use iceberg::expr::{Predicate, Reference};

async fn filtered_scan(table: &iceberg::Table) -> Result<Vec<RecordBatch>> {
    // Build predicate
    let predicate = Predicate::and(
        Predicate::greater_than("timestamp", 1704067200000i64), // > 2024-01-01
        Predicate::equal("event_type", "click"),
    );

    // Scan with predicate pushdown
    let scan = table
        .scan()
        .with_filter(predicate)
        .select(&["user_id", "timestamp", "event_type"]) // Column projection
        .build()
        .await?;

    let batches = scan.to_arrow().await?;

    Ok(batches)
}
```

**Compaction (optimize files)**:
```rust
async fn compact_table(table: &iceberg::Table) -> Result<()> {
    // Read small files
    let scan = table.scan().build().await?;
    let batches = scan.to_arrow().await?;

    // Rewrite as larger, optimized files
    let mut writer = table
        .writer()
        .partition_by(table.metadata().default_partition_spec()?)
        .build()
        .await?;

    for batch in batches {
        writer.write(&batch).await?;
    }

    let new_files = writer.close().await?;

    // Atomic replace
    let mut rewrite = table.new_rewrite();
    rewrite
        .delete_files(/* old files */)
        .add_files(new_files)
        .commit()
        .await?;

    Ok(())
}
```

## Integration with DataFusion

```rust
use datafusion::prelude::*;
use iceberg::datafusion::IcebergTableProvider;

async fn query_with_datafusion(table: iceberg::Table) -> Result<()> {
    // Create DataFusion context
    let ctx = SessionContext::new();

    // Register Iceberg table
    let provider = IcebergTableProvider::try_new(table).await?;
    ctx.register_table("events", Arc::new(provider))?;

    // Query with SQL
    let df = ctx.sql("
        SELECT
            event_type,
            COUNT(*) as count
        FROM events
        WHERE timestamp >= '2024-01-01'
        GROUP BY event_type
    ").await?;

    df.show().await?;

    Ok(())
}
```

## Common Patterns

**Creating a data pipeline**:
```rust
async fn data_pipeline(
    source_store: Arc<dyn ObjectStore>,
    table: &iceberg::Table,
) -> Result<()> {
    // 1. Read from source (e.g., Parquet)
    let batches = read_parquet_files(source_store).await?;

    // 2. Transform data
    let transformed = transform_batches(batches)?;

    // 3. Write to Iceberg table
    write_data(table, transformed).await?;

    println!("Pipeline complete");
    Ok(())
}
```

**Implementing time-based retention**:
```rust
async fn expire_old_snapshots(table: &mut iceberg::Table, days: i64) -> Result<()> {
    let cutoff_ms = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);

    let mut expire = table.expire_snapshots();
    expire
        .expire_older_than(cutoff_ms)
        .retain_last(10) // Keep at least 10 snapshots
        .commit()
        .await?;

    println!("Expired snapshots older than {} days", days);
    Ok(())
}
```

**Atomic updates**:
```rust
async fn atomic_update(table: &iceberg::Table) -> Result<()> {
    // All or nothing - either entire commit succeeds or fails
    let mut transaction = table.new_transaction();

    // Multiple operations in one transaction
    transaction.append(/* new data */);
    transaction.update_schema(/* schema change */);
    transaction.update_properties(/* property change */);

    // Atomic commit
    transaction.commit().await?;

    Ok(())
}
```

## Best Practices

- **Use hidden partitioning** for cleaner queries and easier partition evolution
- **Define sort order** to cluster related data together
- **Expire old snapshots** regularly to avoid metadata bloat
- **Use schema evolution** instead of creating new tables
- **Leverage time travel** for debugging and auditing
- **Compact small files** periodically for better read performance
- **Use partition evolution** to adapt to changing data patterns
- **Enable statistics** for query optimization

## Benefits Over Raw Parquet

1. **ACID Transactions**: Atomic commits prevent partial updates
2. **Time Travel**: Query historical table states
3. **Schema Evolution**: Add/rename/reorder columns safely
4. **Partition Evolution**: Change partitioning without rewriting
5. **Hidden Partitioning**: Cleaner queries, automatic partition pruning
6. **Concurrency**: Multiple writers with optimistic concurrency
7. **Metadata Management**: Efficient metadata operations
8. **Data Lineage**: Track changes over time

## Troubleshooting

**Metadata file not found**:
- Verify catalog configuration
- Check object store permissions
- Ensure table was created successfully

**Schema mismatch on write**:
- Verify writer schema matches table schema
- Use schema evolution to add new fields
- Check for required vs. optional fields

**Slow queries**:
- Use predicate pushdown with filters
- Enable column projection
- Compact small files
- Verify partition pruning is working

**Snapshot expiration issues**:
- Ensure retain_last is set appropriately
- Don't expire too aggressively if time travel is needed
- Clean up orphaned files separately

## Resources

- [Apache Iceberg Specification](https://iceberg.apache.org/spec/)
- [iceberg-rust Documentation](https://docs.rs/iceberg/)
- [Iceberg Table Format](https://iceberg.apache.org/docs/latest/)
