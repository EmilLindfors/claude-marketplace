---
description: Add a new tool to your MCP server with proper typing and tests
---

You are adding a new tool to an existing MCP server project.

## Your Task

Guide the user through creating a well-designed, type-safe tool with comprehensive tests.

## Steps

### 1. Gather Tool Information

Ask the user about the tool:
```
I'll help you add a new tool to your MCP server. Please provide:

1. Tool name: (e.g., search_files, analyze_code, fetch_weather)
2. Description: What does this tool do? (This helps AI understand when to use it)
3. Parameters: What inputs does it need?
   - Parameter name, type, required/optional
4. Return type: What does it return?
5. Does it need external services? (database, API, file system, etc.)
```

Example:
```
Tool: search_files
Description: Search for files matching a pattern in a directory
Parameters:
  - directory: String (required) - Directory to search in
  - pattern: String (required) - Glob pattern to match
  - recursive: bool (optional, default true) - Search recursively
  - limit: Option<u32> (optional) - Maximum results
Return: Vec<FileInfo>
External: File system access
```

### 2. Create Tool Module File

Generate `src/tools/{tool_name}.rs`:

```rust
use rmcp::prelude::*;
use serde::{{Deserialize, Serialize}};
use schemars::JsonSchema;
use crate::error::{{Error, Result}};

/// {Tool description}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct {ToolName}Params {
    #[serde(default)]
    pub {param1}: {Type1},
    pub {param2}: {Type2},
    #[serde(skip_serializing_if = "Option::is_none")]
    pub {param3}: Option<{Type3}>,
}

/// Result type for {tool_name}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct {ToolName}Result {
    pub {field1}: {FieldType1},
    pub {field2}: {FieldType2},
}

#[tool(tool_box)]
pub struct {ToolName}Tool {{
    // Service dependencies
    #[cfg(feature = "database")]
    pool: Arc<PgPool>,
}}

impl {ToolName}Tool {
    pub fn new(/* dependencies */) -> Self {
        Self {{
            // Initialize dependencies
        }}
    }
}

#[tool(tool_box)]
impl {ToolName}Tool {
    #[tool(description = "{Full tool description for AI}")]
    pub async fn {function_name}(
        &self,
        #[tool(aggr)] params: {ToolName}Params,
    ) -> Result<{ToolName}Result> {
        // Validate inputs
        if params.{param}.is_empty() {
            return Err(Error::InvalidInput {{
                field: "{param}".to_string(),
                message: "Cannot be empty".to_string(),
            }});
        }

        // Implementation
        todo!("Implement {tool_name} logic")

        // Return result
        Ok({ToolName}Result {{
            {field1}: value1,
            {field2}: value2,
        }})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_{function_name}_success() {
        let tool = {ToolName}Tool::new();
        let params = {ToolName}Params {{
            {param1}: test_value1,
            {param2}: test_value2,
            {param3}: None,
        }};

        let result = tool.{function_name}(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_{function_name}_validation_error() {
        let tool = {ToolName}Tool::new();
        let params = {ToolName}Params {{
            {param1}: invalid_value,
            {param2}: test_value2,
            {param3}: None,
        }};

        let result = tool.{function_name}(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_{function_name}_with_optional_params() {
        let tool = {ToolName}Tool::new();
        let params = {ToolName}Params {{
            {param1}: test_value1,
            {param2}: test_value2,
            {param3}: Some(test_value3),
        }};

        let result = tool.{function_name}(params).await;
        assert!(result.is_ok());
    }
}
```

### 3. Update tools/mod.rs

Add the new module:

```rust
pub mod {tool_name};

pub use {tool_name}::{{ToolName}Tool;
```

### 4. Register Tool in Service

Update `src/service.rs` or `src/lib.rs` to include the tool:

```rust
use crate::tools::{tool_name}::{{ToolName}Tool;

#[tool(tool_box)]
pub struct McpService {{
    {tool_name}: {ToolName}Tool,
    // ... other tools
}}

impl McpService {{
    pub async fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {{
            {tool_name}: {ToolName}Tool::new(/* deps */),
            // ... other tools
        }})
    }}
}}

// Delegate tool calls
#[tool(tool_box)]
impl McpService {{
    #[tool(description = "{description}")]
    pub async fn {function_name}(
        &self,
        #[tool(aggr)] params: {ToolName}Params,
    ) -> Result<{ToolName}Result> {
        self.{tool_name}.{function_name}(params).await
    }}
}}
```

### 5. Add Integration Test

Create `tests/{tool_name}_test.rs`:

```rust
use {crate_name}::{{
    config::AppConfig,
    service::McpService,
    tools::{tool_name}::{{ToolName}Params,
}};

#[tokio::test]
async fn test_{tool_name}_integration() {
    // Setup
    let config = AppConfig::load().unwrap();
    let service = McpService::new(config).await.unwrap();

    // Test
    let params = {ToolName}Params {{
        // test params
    }};

    let result = service.{function_name}(params).await;

    // Verify
    assert!(result.is_ok());
    let result = result.unwrap();
    // Add more assertions
}
```

### 6. Update Documentation

Add to README.md:

```markdown
### {ToolName}

**Description:** {Tool description}

**Parameters:**
- `{param1}` ({Type1}, required): {Description}
- `{param2}` ({Type2}, required): {Description}
- `{param3}` ({Type3}, optional): {Description}

**Returns:** {ReturnType}

**Example:**
\```json
{{
  "jsonrpc": "2.0",
  "method": "{function_name}",
  "params": {{
    "{param1}": "value1",
    "{param2}": "value2"
  }},
  "id": 1
}}
\```

**Response:**
\```json
{{
  "jsonrpc": "2.0",
  "result": {{
    "{field1}": "result1",
    "{field2}": "result2"
  }},
  "id": 1
}}
\```
```

## Common Tool Patterns

### File System Tool

```rust
use tokio::fs;
use glob::glob;

#[tool(tool_box)]
impl FileSystemTool {
    #[tool(description = "Search for files matching a pattern")]
    pub async fn search_files(
        &self,
        directory: String,
        pattern: String,
    ) -> Result<Vec<String>> {
        let search_pattern = format!("{}/{}}", directory, pattern);

        let mut results = Vec::new();
        for entry in glob(&search_pattern).map_err(|e| Error::Internal(e.to_string()))? {
            if let Ok(path) = entry {
                results.push(path.display().to_string());
            }
        }

        Ok(results)
    }
}
```

### API Integration Tool

```rust
use reqwest::Client;

#[tool(tool_box)]
struct ApiTool {
    client: Client,
    api_key: String,
}

#[tool(tool_box)]
impl ApiTool {
    #[tool(description = "Fetch data from external API")]
    pub async fn fetch_data(&self, endpoint: String) -> Result<String> {
        let url = format!("https://api.example.com/{}", endpoint);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| Error::ExternalError(e.to_string()))?;

        let text = response.text().await
            .map_err(|e| Error::ExternalError(e.to_string()))?;

        Ok(text)
    }
}
```

### Database Tool

```rust
use sqlx::PgPool;

#[tool(tool_box)]
struct DatabaseTool {
    pool: PgPool,
}

#[tool(tool_box)]
impl DatabaseTool {
    #[tool(description = "Query database records")]
    pub async fn query_records(&self, table: String, limit: u32) -> Result<Vec<Record>> {
        let query = format!("SELECT * FROM {} LIMIT ${}", table, limit);

        let records = sqlx::query_as::<_, Record>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(records)
    }
}
```

## After Creation

Inform the user:

```
✅ Tool '{tool_name}' added successfully!

## Files Created/Modified:
- src/tools/{tool_name}.rs - Tool implementation
- src/tools/mod.rs - Module export
- src/service.rs - Tool registration
- tests/{tool_name}_test.rs - Integration tests
- README.md - Documentation

## Next Steps:

1. **Implement the tool logic:**
   ```bash
   # Edit the TODO section in src/tools/{tool_name}.rs
   ```

2. **Run tests:**
   ```bash
   cargo test {tool_name}
   ```

3. **Test manually:**
   ```bash
   cargo run
   # Then send a JSON-RPC request
   ```

4. **Add more tools:**
   ```
   /mcp-add-tool
   ```

## Example JSON-RPC Request:

\```bash
echo '{{"jsonrpc":"2.0","method":"{function_name}","params":{{"param1":"value1"}},"id":1}}' | cargo run
\```

Happy coding! 🦀
```

## Important Notes

- Validate all inputs before processing
- Use proper error types from Error enum
- Add comprehensive tests (success, error, edge cases)
- Document parameters and return values clearly
- Use async properly (don't block)
- Consider rate limiting for external APIs
- Add logging with tracing for debugging
