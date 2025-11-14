---
description: Configure object_store for cloud storage (S3, Azure, GCS, or local filesystem)
---

# Object Store Setup

Help the user configure the `object_store` crate for their cloud provider or local filesystem.

## Steps

1. **Identify the storage backend** by asking the user which provider they want to use:
   - Amazon S3
   - Azure Blob Storage
   - Google Cloud Storage
   - Local filesystem (for development/testing)

2. **Add the dependency** to their Cargo.toml:
   ```toml
   [dependencies]
   object_store = { version = "0.9", features = ["aws", "azure", "gcp"] }
   tokio = { version = "1", features = ["full"] }
   ```

3. **Create the appropriate builder** based on their choice:

   **For Amazon S3**:
   ```rust
   use object_store::aws::AmazonS3Builder;
   use object_store::ObjectStore;
   use std::sync::Arc;

   let s3 = AmazonS3Builder::new()
       .with_region("us-east-1")
       .with_bucket_name("my-data-lake")
       .with_access_key_id(access_key)
       .with_secret_access_key(secret_key)
       // Production settings
       .with_retry(RetryConfig {
           max_retries: 3,
           retry_timeout: Duration::from_secs(10),
           ..Default::default()
       })
       .build()?;

   let store: Arc<dyn ObjectStore> = Arc::new(s3);
   ```

   **For Azure Blob Storage**:
   ```rust
   use object_store::azure::MicrosoftAzureBuilder;

   let azure = MicrosoftAzureBuilder::new()
       .with_account("mystorageaccount")
       .with_container_name("mycontainer")
       .with_access_key(access_key)
       .build()?;

   let store: Arc<dyn ObjectStore> = Arc::new(azure);
   ```

   **For Google Cloud Storage**:
   ```rust
   use object_store::gcs::GoogleCloudStorageBuilder;

   let gcs = GoogleCloudStorageBuilder::new()
       .with_service_account_key(service_account_json)
       .with_bucket_name("my-bucket")
       .build()?;

   let store: Arc<dyn ObjectStore> = Arc::new(gcs);
   ```

   **For Local Filesystem**:
   ```rust
   use object_store::local::LocalFileSystem;

   let local = LocalFileSystem::new_with_prefix("/tmp/data-lake")?;
   let store: Arc<dyn ObjectStore> = Arc::new(local);
   ```

4. **Test the connection** by listing objects or performing a simple operation:
   ```rust
   // List objects with a prefix
   let prefix = Some(&Path::from("data/"));
   let mut list = store.list(prefix);

   while let Some(meta) = list.next().await {
       let meta = meta?;
       println!("{}: {} bytes", meta.location, meta.size);
   }
   ```

5. **Add error handling** and configuration management:
   ```rust
   use object_store::Error as ObjectStoreError;

   async fn create_store() -> Result<Arc<dyn ObjectStore>, ObjectStoreError> {
       // Get credentials from environment or config
       let region = std::env::var("AWS_REGION")
           .unwrap_or_else(|_| "us-east-1".to_string());
       let bucket = std::env::var("S3_BUCKET")?;

       let s3 = AmazonS3Builder::from_env()
           .with_region(&region)
           .with_bucket_name(&bucket)
           .build()?;

       Ok(Arc::new(s3))
   }
   ```

## Best Practices

- **Use Arc<dyn ObjectStore>** for shared ownership across threads
- **Configure retry logic** for production resilience
- **Store credentials securely** using environment variables or secret managers
- **Use LocalFileSystem** for testing to avoid cloud costs
- **Enable request timeouts** to prevent hanging operations
- **Set up connection pooling** for better performance

## Common Patterns

**Environment-based configuration**:
```rust
let s3 = AmazonS3Builder::from_env()
    .with_bucket_name(&bucket)
    .build()?;
```

**Multipart upload for large files**:
```rust
let multipart = store.put_multipart(&path).await?;
for chunk in chunks {
    multipart.put_part(chunk).await?;
}
multipart.complete().await?;
```

**Streaming downloads**:
```rust
let result = store.get(&path).await?;
let mut stream = result.into_stream();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // Process chunk
}
```
