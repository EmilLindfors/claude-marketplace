---
description: Add a new resource to your MCP server for providing data context
---

You are adding a new resource to an existing MCP server project.

## Your Task

Guide the user through creating a well-designed resource that provides data to AI assistants.

## Steps

### 1. Gather Resource Information

Ask the user:
```
I'll help you add a new resource to your MCP server. Please provide:

1. Resource name: (e.g., files, database_records, api_docs)
2. URI pattern: How should resources be addressed?
   Examples:
   - file:///{path}
   - db://{table}/{id}
   - api://{endpoint}
3. Content type: What kind of data?
   - Text (markdown, code, json)
   - Binary (images, files)
   - Structured (database records)
4. Source: Where does data come from?
   - File system
   - Database
   - External API
   - Generated dynamically
5. Caching: Should results be cached?
```

### 2. Create Resource Module

Generate `src/resources/{resource_name}.rs`:

```rust
use rmcp::prelude::*;
use serde::{{Deserialize, Serialize}};
use schemars::JsonSchema;
use crate::error::{{Error, Result}};
use std::path::{{Path, PathBuf}};

/// Resource information for {resource_name}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct {ResourceName}Info {{
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: String,
    pub size: Option<u64>,
}}

/// Resource content for {resource_name}
#[derive(Debug, Serialize, Deserialize)]
pub struct {ResourceName}Content {{
    pub uri: String,
    pub mime_type: String,
    pub text: Option<String>,
    pub blob: Option<Vec<u8>>,
}}

pub struct {ResourceName}Resource {{
    // Configuration
    {config_field}: {ConfigType},
}}

impl {ResourceName}Resource {{
    pub fn new({config_field}: {ConfigType}) -> Self {{
        Self {{ {config_field} }}
    }}

    /// List all available resources
    pub async fn list_resources(&self) -> Result<Vec<{ResourceName}Info>> {{
        let mut resources = Vec::new();

        // TODO: Implement resource listing logic
        // Example: Read directory, query database, fetch from API, etc.

        Ok(resources)
    }}

    /// Fetch resource content by URI
    pub async fn fetch_resource(&self, uri: &str) -> Result<{ResourceName}Content> {{
        // Parse URI
        let path = self.parse_uri(uri)?;

        // Validate access
        self.validate_access(&path)?;

        // Fetch content
        let content = self.fetch_content(&path).await?;

        Ok({ResourceName}Content {{
            uri: uri.to_string(),
            mime_type: self.detect_mime_type(&path),
            text: Some(content),
            blob: None,
        }})
    }}

    fn parse_uri(&self, uri: &str) -> Result<String> {{
        // Remove URI scheme prefix
        uri.strip_prefix("{uri_scheme}://")
            .ok_or_else(|| Error::InvalidInput {{
                field: "uri".to_string(),
                message: format!("Invalid URI scheme, expected {uri_scheme}://", uri),
            }})
            .map(|s| s.to_string())
    }}

    fn validate_access(&self, path: &str) -> Result<()> {{
        // Implement access control
        // Check permissions, validate paths, etc.
        Ok(())
    }}

    async fn fetch_content(&self, path: &str) -> Result<String> {{
        // TODO: Implement content fetching
        // Example: Read file, query database, call API
        Ok(String::new())
    }}

    fn detect_mime_type(&self, path: &str) -> String {{
        // Detect MIME type based on file extension or content
        "text/plain".to_string()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_list_resources() {{
        let resource = {ResourceName}Resource::new(/* config */);
        let list = resource.list_resources().await;
        assert!(list.is_ok());
    }}

    #[tokio::test]
    async fn test_fetch_resource() {{
        let resource = {ResourceName}Resource::new(/* config */);
        let content = resource.fetch_resource("{example_uri}").await;
        assert!(content.is_ok());
    }}

    #[tokio::test]
    async fn test_invalid_uri() {{
        let resource = {ResourceName}Resource::new(/* config */);
        let result = resource.fetch_resource("invalid://uri").await;
        assert!(result.is_err());
    }}
}}
```

### 3. Update resources/mod.rs

```rust
pub mod {resource_name};

pub use {resource_name}::{{ResourceName}Resource;
```

### 4. Register Resource in Service

Update `src/service.rs`:

```rust
use crate::resources::{resource_name}::{{ResourceName}Resource;

pub struct McpService {{
    {resource_name}_resource: {ResourceName}Resource,
    // ... other resources
}}

impl McpService {{
    pub async fn new(config: AppConfig) -> Result<Self> {{
        Ok(Self {{
            {resource_name}_resource: {ResourceName}Resource::new(/* config */),
            // ... other resources
        }})
    }}

    /// List all available resources
    pub async fn list_resources(&self) -> Result<Vec<ResourceInfo>> {{
        let mut all_resources = Vec::new();

        // Add resources from this provider
        let resources = self.{resource_name}_resource.list_resources().await?;
        all_resources.extend(resources.into_iter().map(|r| ResourceInfo {{
            uri: r.uri,
            name: r.name,
            description: r.description,
            mime_type: Some(r.mime_type),
        }}));

        Ok(all_resources)
    }}

    /// Fetch resource by URI
    pub async fn fetch_resource(&self, uri: &str) -> Result<ResourceContent> {{
        // Route to appropriate resource provider based on URI scheme
        if uri.starts_with("{uri_scheme}://") {{
            let content = self.{resource_name}_resource.fetch_resource(uri).await?;
            return Ok(ResourceContent {{
                uri: content.uri,
                mime_type: content.mime_type,
                text: content.text,
                blob: content.blob,
            }});
        }}

        Err(Error::NotFound(format!("Resource not found: {{}}", uri)))
    }}
}}
```

## Resource Implementation Examples

### File System Resource

```rust
use tokio::fs;
use walkdir::WalkDir;

pub struct FileSystemResource {{
    root_path: PathBuf,
}}

impl FileSystemResource {{
    pub async fn list_resources(&self) -> Result<Vec<FileInfo>> {{
        let mut resources = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {{
            let path = entry.path();
            let relative = path.strip_prefix(&self.root_path)?;
            let uri = format!("file:///{{}}", relative.display());

            let metadata = fs::metadata(path).await?;

            resources.push(FileInfo {{
                uri,
                name: path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                description: None,
                mime_type: detect_mime_type(path),
                size: Some(metadata.len()),
            }});
        }}

        Ok(resources)
    }}

    pub async fn fetch_resource(&self, uri: &str) -> Result<FileContent> {{
        let path = self.parse_uri(uri)?;
        let full_path = self.root_path.join(&path);

        // Security: Prevent path traversal
        if !full_path.starts_with(&self.root_path) {{
            return Err(Error::PermissionDenied(
                "Access denied".to_string()
            ));
        }}

        let content = fs::read_to_string(&full_path).await?;
        let mime_type = detect_mime_type(&full_path);

        Ok(FileContent {{
            uri: uri.to_string(),
            mime_type,
            text: Some(content),
            blob: None,
        }})
    }}
}}
```

### Database Resource

```rust
use sqlx::{{PgPool, Row}};

pub struct DatabaseResource {{
    pool: PgPool,
}}

impl DatabaseResource {{
    pub async fn list_resources(&self) -> Result<Vec<DbResourceInfo>> {{
        let tables = sqlx::query("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
            .fetch_all(&self.pool)
            .await?;

        let mut resources = Vec::new();
        for row in tables {{
            let table_name: String = row.get("tablename");

            // Get row count
            let count_query = format!("SELECT COUNT(*) FROM {{}}", table_name);
            let count: i64 = sqlx::query_scalar(&count_query)
                .fetch_one(&self.pool)
                .await?;

            resources.push(DbResourceInfo {{
                uri: format!("db://{{}}", table_name),
                name: table_name.clone(),
                description: Some(format!("Database table with {{}} records", count)),
                mime_type: "application/json".to_string(),
                size: Some(count as u64),
            }});
        }}

        Ok(resources)
    }}

    pub async fn fetch_resource(&self, uri: &str) -> Result<DbContent> {{
        let parts: Vec<&str> = uri.strip_prefix("db://")
            .ok_or_else(|| Error::InvalidInput {{
                field: "uri".to_string(),
                message: "Invalid URI".to_string(),
            }})?
            .split('/')
            .collect();

        let table = parts[0];
        let id = parts.get(1);

        let query = if let Some(id) = id {{
            format!("SELECT * FROM {{}} WHERE id = $1", table)
        }} else {{
            format!("SELECT * FROM {{}} LIMIT 100", table)
        }};

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        let json = serde_json::to_string_pretty(&rows)?;

        Ok(DbContent {{
            uri: uri.to_string(),
            mime_type: "application/json".to_string(),
            text: Some(json),
            blob: None,
        }})
    }}
}}
```

### API Resource

```rust
use reqwest::Client;

pub struct ApiResource {{
    client: Client,
    base_url: String,
    api_key: String,
}}

impl ApiResource {{
    pub async fn list_resources(&self) -> Result<Vec<ApiResourceInfo>> {{
        // Fetch available endpoints from API
        let response = self.client
            .get(&format!("{{}}/endpoints", self.base_url))
            .header("Authorization", format!("Bearer {{}}", self.api_key))
            .send()
            .await?;

        let endpoints: Vec<String> = response.json().await?;

        let resources = endpoints.into_iter().map(|endpoint| ApiResourceInfo {{
            uri: format!("api://{{}}", endpoint),
            name: endpoint.clone(),
            description: Some(format!("API endpoint: {{}}", endpoint)),
            mime_type: "application/json".to_string(),
            size: None,
        }}).collect();

        Ok(resources)
    }}

    pub async fn fetch_resource(&self, uri: &str) -> Result<ApiContent> {{
        let endpoint = uri.strip_prefix("api://")
            .ok_or_else(|| Error::InvalidInput {{
                field: "uri".to_string(),
                message: "Invalid URI".to_string(),
            }})?;

        let url = format!("{{}}/{{}}", self.base_url, endpoint);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {{}}", self.api_key))
            .send()
            .await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json")
            .to_string();

        let text = response.text().await?;

        Ok(ApiContent {{
            uri: uri.to_string(),
            mime_type: content_type,
            text: Some(text),
            blob: None,
        }})
    }}
}}
```

## Caching Pattern

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct CachedResource {{
    inner: Arc<InnerResource>,
    cache: Cache<String, ResourceContent>,
}}

impl CachedResource {{
    pub fn new(inner: Arc<InnerResource>) -> Self {{
        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self {{ inner, cache }}
    }}

    pub async fn fetch_resource(&self, uri: &str) -> Result<ResourceContent> {{
        // Check cache
        if let Some(content) = self.cache.get(uri).await {{
            return Ok(content);
        }}

        // Fetch from source
        let content = self.inner.fetch_resource(uri).await?;

        // Cache result
        self.cache.insert(uri.to_string(), content.clone()).await;

        Ok(content)
    }}
}}
```

## After Creation

```
✅ Resource '{resource_name}' added successfully!

## Files Created/Modified:
- src/resources/{resource_name}.rs - Resource implementation
- src/resources/mod.rs - Module export
- src/service.rs - Resource registration

## Next Steps:

1. **Implement resource logic:**
   Edit src/resources/{resource_name}.rs

2. **Test resource:**
   ```bash
   cargo test {resource_name}
   ```

3. **Test listing:**
   Send list_resources request via MCP client

4. **Test fetching:**
   Send fetch_resource request with a URI

## Example Usage:

List resources:
\```json
{{"jsonrpc":"2.0","method":"resources/list","params":{{}},"id":1}}
\```

Fetch resource:
\```json
{{"jsonrpc":"2.0","method":"resources/read","params":{{"uri":"{example_uri}"}},"id":2}}
\```
```
