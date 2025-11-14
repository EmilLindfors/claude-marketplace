---
description: Initialize a new Rust MCP server project with best practices
---

You are initializing a new Rust MCP server project using the rmcp crate.

## Your Task

Create a complete, production-ready MCP server project structure with all necessary files, dependencies, and best practices configured.

## Steps

### 1. Gather Requirements

Ask the user:
- **Project name**: What should the project be called?
- **Transport type**: stdio (local), SSE (cloud), or HTTP (web service)?
- **Features needed**: Tools, resources, prompts, or all three?
- **Additional features**: Database, caching, authentication?

Example interaction:
```
I'll help you initialize a new MCP server project. Let me gather some information:

1. What would you like to name your project? (e.g., my-mcp-server)
2. Which transport will you use?
   - stdio (for local/subprocess use)
   - sse (for cloud deployments)
   - http (for web services)
3. What capabilities do you need?
   - Tools (functions AI can invoke)
   - Resources (data sources for context)
   - Prompts (conversation templates)
   - All of the above
4. Do you need any additional features?
   - Database integration (PostgreSQL, SQLite)
   - Caching (Redis, in-memory)
   - Authentication/authorization
   - None
```

### 2. Create Project Structure

Create the following directory structure:

```
{project_name}/
├── Cargo.toml
├── .gitignore
├── README.md
├── LICENSE
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   └── example.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   └── example.rs
│   └── prompts/
│       ├── mod.rs
│       └── example.rs
├── tests/
│   ├── integration_test.rs
│   └── tool_tests.rs
├── config/
│   ├── default.toml
│   └── local.example.toml
├── .github/
│   └── workflows/
│       └── ci.yml
├── clippy.toml
├── rustfmt.toml
└── deny.toml
```

### 3. Generate Cargo.toml

Create `Cargo.toml` based on requirements:

```toml
[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.75"
authors = ["{author}"]
description = "MCP server built with rmcp"
license = "MIT"

[dependencies]
# Core MCP
rmcp = { version = "0.8", features = ["server"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Configuration
config = "0.14"

# Async utilities
futures-util = "0.3"

# Optional: Database
{database_deps}

# Optional: Caching
{cache_deps}

# Optional: Authentication
{auth_deps}

# Optional: Transport-specific
{transport_deps}

[dev-dependencies]
mockall = "0.13"
proptest = "1.5"
testcontainers = "0.20"

[profile.release]
opt-level = 'z'      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
debug = true         # Keep debug info for profiling

[profile.dev.package."*"]
opt-level = 2        # Optimize dependencies in dev mode
```

### 4. Create Main Entry Point

Generate `src/main.rs` based on transport type:

**For stdio:**
```rust
use {project_name}::{{config::AppConfig, service::McpService}};
use rmcp::prelude::*;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (stderr only for stdio)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting MCP server");

    // Load configuration
    let config = AppConfig::load()?;

    // Create service
    let service = McpService::new(config).await?;

    // Create stdio transport
    let transport = rmcp::transport::stdio::stdio_transport();

    info!("Serving via stdio");

    // Serve
    match service.serve(transport).await {
        Ok(_) => info!("Server terminated normally"),
        Err(e) => error!("Server error: {{}}", e),
    }

    Ok(())
}
```

**For HTTP/SSE:**
```rust
use {project_name}::{{config::AppConfig, service::McpService}};
use axum::{{routing::post, Router}};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("Starting MCP server");

    // Load configuration
    let config = AppConfig::load()?;
    let addr = format!("{{}}:{{}}", config.server.host, config.server.port);

    // Create service
    let service = Arc::new(McpService::new(config).await?);

    // Create router
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(service);

    info!("Listening on {{}}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_mcp_request(/* handler implementation */) {{
    // MCP request handling
}}
```

### 5. Create Library Structure

Generate `src/lib.rs`:

```rust
pub mod config;
pub mod error;
pub mod service;

// Feature modules
#[cfg(feature = "tools")]
pub mod tools;

#[cfg(feature = "resources")]
pub mod resources;

#[cfg(feature = "prompts")]
pub mod prompts;

// Re-exports
pub use error::{{Error, Result}};
pub use service::McpService;
```

### 6. Create Example Tool

Generate `src/tools/example.rs`:

```rust
use rmcp::prelude::*;
use serde::{{Deserialize, Serialize}};
use schemars::JsonSchema;
use crate::error::{{Error, Result}};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CalculateParams {
    pub a: f64,
    pub b: f64,
    pub operation: Operation,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[tool(tool_box)]
pub struct ExampleTools;

#[tool(tool_box)]
impl ExampleTools {
    #[tool(description = "Perform a mathematical calculation")]
    pub async fn calculate(&self, #[tool(aggr)] params: CalculateParams) -> Result<f64> {
        let result = match params.operation {
            Operation::Add => params.a + params.b,
            Operation::Subtract => params.a - params.b,
            Operation::Multiply => params.a * params.b,
            Operation::Divide => {
                if params.b == 0.0 {
                    return Err(Error::InvalidInput {
                        field: "b".to_string(),
                        message: "Cannot divide by zero".to_string(),
                    });
                }
                params.a / params.b
            }
        };

        Ok(result)
    }

    #[tool(description = "Echo back a message")]
    pub async fn echo(&self, message: String) -> String {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_add() {
        let tools = ExampleTools;
        let params = CalculateParams {
            a: 5.0,
            b: 3.0,
            operation: Operation::Add,
        };

        let result = tools.calculate(params).await.unwrap();
        assert_eq!(result, 8.0);
    }

    #[tokio::test]
    async fn test_calculate_divide_by_zero() {
        let tools = ExampleTools;
        let params = CalculateParams {
            a: 5.0,
            b: 0.0,
            operation: Operation::Divide,
        };

        let result = tools.calculate(params).await;
        assert!(result.is_err());
    }
}
```

### 7. Create Error Types

Generate `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Resource not found: {{0}}")]
    NotFound(String),

    #[error("Invalid input: {{field}} - {{message}}")]
    InvalidInput { field: String, message: String },

    #[error("Configuration error: {{0}}")]
    ConfigError(#[from] config::ConfigError),

    #[error("IO error: {{0}}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {{0}}")]
    JsonError(#[from] serde_json::Error),

    #[error("Internal error: {{0}}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::InvalidInput {{ .. }} => "INVALID_INPUT",
            Self::ConfigError(_) => "CONFIG_ERROR",
            Self::IoError(_) => "IO_ERROR",
            Self::JsonError(_) => "JSON_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
```

### 8. Create Configuration

Generate `src/config.rs` and `config/default.toml`:

**src/config.rs:**
```rust
use config::{{Config, ConfigError, Environment, File}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP"))
            .build()?
            .try_deserialize()
    }
}
```

**config/default.toml:**
```toml
[server]
host = "127.0.0.1"
port = 3000
timeout_seconds = 30

[logging]
level = "info"
format = "json"
```

### 9. Create CI/CD Pipeline

Generate `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Check formatting
      run: cargo fmt -- --check

    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Run tests
      run: cargo test --all-features

    - name: Build
      run: cargo build --release

    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit
```

### 10. Create Tool Configuration Files

Generate `clippy.toml`:
```toml
cognitive-complexity-threshold = 30
single-char-binding-names-threshold = 5
too-many-arguments-threshold = 7
```

Generate `rustfmt.toml`:
```toml
edition = "2024"
max_width = 100
hard_tabs = false
tab_spaces = 4
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
```

Generate `deny.toml`:
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]

[bans]
multiple-versions = "warn"
```

### 11. Create Documentation

Generate comprehensive `README.md` with:
- Project description
- Features
- Installation instructions
- Configuration guide
- Usage examples
- Development setup
- Testing instructions
- Deployment guide

### 12. Create .gitignore

```
/target
Cargo.lock
*.swp
*.swo
*~
.DS_Store
config/local.toml
.env
```

### 13. Initialize Git

```bash
cd {project_name}
git init
git add .
git commit -m "Initial commit: MCP server with rmcp"
```

## After Creation

Inform the user:

```
✅ MCP server project '{project_name}' created successfully!

## Project Structure
- src/main.rs           - Server entry point
- src/lib.rs           - Library root
- src/tools/           - Tool implementations
- src/resources/       - Resource implementations
- src/prompts/         - Prompt definitions
- config/              - Configuration files
- tests/               - Integration tests

## Next Steps

1. **Test the setup:**
   ```bash
   cd {project_name}
   cargo test
   cargo run
   ```

2. **Add your first tool:**
   ```
   /mcp-add-tool
   ```

3. **Configure for your needs:**
   - Edit config/default.toml
   - Update dependencies in Cargo.toml
   - Customize tools/resources/prompts

4. **Set up development environment:**
   ```bash
   # Install development tools
   cargo install bacon cargo-nextest cargo-audit

   # Start bacon for continuous feedback
   bacon clippy
   ```

## Resources
- Project README: {project_name}/README.md
- MCP Documentation: https://modelcontextprotocol.io
- rmcp Crate: https://docs.rs/rmcp

Happy coding! 🦀
```

## Important Notes

- Use the user's provided name and email for author fields
- Adjust dependencies based on selected features
- Include only requested capabilities (tools/resources/prompts)
- Add database/cache dependencies only if requested
- Generate working example code that compiles
- Include comprehensive tests
- Set up CI/CD from the start
- Configure modern Rust tooling (clippy, rustfmt, deny)
