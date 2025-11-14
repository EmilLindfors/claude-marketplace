# Rust MCP Server Plugin

A comprehensive plugin for creating Model Context Protocol (MCP) servers in Rust using the rmcp crate.

## Overview

This plugin provides everything you need to build production-ready MCP servers in Rust, leveraging the official rmcp SDK. MCP enables AI assistants to securely access external tools, data sources, and capabilities through a standardized protocol.

## What is MCP?

Model Context Protocol (MCP) is an open protocol that standardizes how applications provide context to Large Language Models (LLMs). With MCP servers, you can:

- Expose custom tools that AI assistants can invoke
- Provide resources (files, data, APIs) for AI context
- Define prompts that guide AI interactions
- Build secure, composable AI integrations

## Features

### Commands

#### `/mcp-init`
Initialize a new Rust MCP server project with best practices and modern patterns.

**Features:**
- Generate complete project structure
- Configure Cargo.toml with rmcp dependencies
- Set up development tooling (clippy, rustfmt, deny)
- Create example tool, resource, and prompt
- Include comprehensive tests
- Add CI/CD pipeline

#### `/mcp-add-tool`
Add a new tool to your MCP server.

**Features:**
- Interactive tool definition
- Type-safe parameter handling
- Automatic schema generation
- Error handling patterns
- Unit test scaffolding
- Documentation templates

#### `/mcp-add-resource`
Add a new resource to your MCP server.

**Features:**
- Resource URI schema design
- Async data fetching patterns
- Caching strategies
- Error handling
- Resource listing support
- MIME type handling

#### `/mcp-add-prompt`
Add a new prompt to your MCP server.

**Features:**
- Prompt template design
- Dynamic argument handling
- Context injection patterns
- Multi-turn conversation support
- Testing strategies

#### `/mcp-test`
Test your MCP server locally with comprehensive validation.

**Features:**
- Unit tests for all tools/resources/prompts
- Integration tests with mock transports
- JSON-RPC protocol validation
- Performance benchmarking
- Security testing

#### `/mcp-deploy`
Deploy your MCP server to production.

**Features:**
- Build optimized release binaries
- Container/Docker support
- Environment configuration
- Logging and monitoring setup
- Health check endpoints
- Deployment documentation

### Expert Agent

#### `mcp-architect`
Specialized agent for designing and implementing MCP servers.

**Capabilities:**
- Architecture design and review
- Tool/resource/prompt design patterns
- Performance optimization
- Security best practices
- Testing strategies
- Integration guidance

### Skills

#### RMCP Quickstart
Get started with rmcp quickly - installation, basic concepts, and your first MCP server.

**Topics:**
- rmcp crate overview and features
- Transport types (stdio, SSE, HTTP)
- Basic server structure
- Handler implementation
- Tool macro usage
- Testing your first server

#### MCP Tools Guide
Master creating MCP tools with type-safe parameters and automatic schema generation.

**Topics:**
- `#[tool]` macro usage
- Parameter types and validation
- Result types and error handling
- Async tool implementation
- Schema generation
- Testing tools

#### MCP Resources Guide
Implement resources that provide data and files to AI assistants.

**Topics:**
- Resource URI patterns
- Resource listing
- Content fetching
- Caching strategies
- Streaming large resources
- Resource updates/subscriptions

#### MCP Prompts Guide
Create powerful prompts that guide AI interactions.

**Topics:**
- Prompt templates
- Dynamic arguments
- Context injection
- Multi-turn conversations
- Prompt chaining
- Testing prompts

#### MCP Transport Guide
Understand different transport mechanisms for MCP servers.

**Topics:**
- stdio transport (subprocess)
- SSE transport (Server-Sent Events)
- HTTP streamable transport
- Custom transports
- Security considerations
- Performance tuning

#### MCP Best Practices
Production-ready patterns and architecture for MCP servers.

**Topics:**
- Error handling patterns
- Logging and observability
- Security (authentication, authorization)
- Performance optimization
- Testing strategies
- Deployment patterns
- Monitoring and debugging

## rmcp Crate Overview

**Version:** 0.8.3 (as of November 2025)
**Repository:** https://github.com/modelcontextprotocol/rust-sdk
**Alternative:** https://github.com/4t145/rmcp (BEST Rust SDK for MCP)

### Key Features

- **Clean API:** Minimal boilerplate with powerful macros
- **Async-first:** Built on tokio for high performance
- **Type-safe:** Leverages Rust's type system for correctness
- **Multiple transports:** stdio, SSE, streamable HTTP
- **Production-ready:** Used by major projects

### Quick Example

```rust
use rmcp::prelude::*;

#[tool(tool_box)]
struct Calculator;

#[tool(tool_box)]
impl Calculator {
    #[tool(description = "Add two numbers")]
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[tool(description = "Multiply two numbers")]
    async fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = Calculator;
    let transport = stdio_transport();
    service.serve(transport).await?;
    Ok(())
}
```

## Installation

Add to your project's `.claudecode/plugins/` or install from marketplace:

```bash
# Local installation
cp -r plugins/rust-mcp-server /path/to/your/project/.claudecode/plugins/

# Or add to marketplace.json
```

Register in marketplace.json:
```json
{
  "plugins": [{
    "name": "rust-mcp-server",
    "source": "./plugins/rust-mcp-server",
    "description": "Create Rust MCP servers with rmcp crate",
    "version": "1.0.0",
    "keywords": ["rust", "mcp", "rmcp", "model-context-protocol"]
  }]
}
```

## Prerequisites

- Rust 1.75+ (recommend 1.85+ for Rust 2024 Edition)
- cargo installed
- Basic understanding of async Rust (tokio)

## Quick Start

1. **Initialize a new MCP server:**
   ```
   /mcp-init
   ```

2. **Add your first tool:**
   ```
   /mcp-add-tool
   ```

3. **Test locally:**
   ```
   /mcp-test
   ```

4. **Get expert guidance:**
   ```
   Ask mcp-architect to help design a weather API MCP server
   ```

## Use Cases

### Data Integration
Create MCP servers that expose databases, APIs, or file systems to AI assistants:
- Database query tools
- API integration resources
- File system access
- Real-time data streams

### Custom Tools
Build domain-specific tools for AI assistants:
- Code analysis tools
- Mathematical computations
- Image processing
- Document generation

### Context Providers
Provide rich context to AI assistants:
- Documentation resources
- Code repositories
- Knowledge bases
- Configuration data

### Workflow Automation
Enable AI-driven workflows:
- Task execution tools
- State management resources
- Event-driven prompts
- Multi-step processes

## Architecture Patterns

### Simple Server
Single-binary stdio server for local use:
- Quick development
- Easy testing
- Perfect for personal tools

### Cloud Service
SSE or HTTP server for remote access:
- Authentication/authorization
- Horizontal scaling
- Load balancing
- Monitoring

### Microservices
Multiple specialized MCP servers:
- Domain separation
- Independent scaling
- Technology diversity
- Fault isolation

## Development Workflow

### 1. Design Phase
Use `mcp-architect` to design your server:
- Define tools, resources, and prompts
- Plan architecture
- Consider security and performance

### 2. Implementation Phase
Use commands to scaffold and implement:
- `/mcp-init` for project setup
- `/mcp-add-tool` for each tool
- `/mcp-add-resource` for resources
- `/mcp-add-prompt` for prompts

### 3. Testing Phase
Validate with comprehensive tests:
- `/mcp-test` for automated testing
- Unit tests for individual components
- Integration tests for full server

### 4. Deployment Phase
Deploy to production:
- `/mcp-deploy` for deployment setup
- CI/CD integration
- Monitoring and observability

## Best Practices

### Code Quality
- Use clippy with pedantic mode
- Format with rustfmt
- Write comprehensive tests
- Document public APIs

### Security
- Validate all inputs
- Implement proper error handling
- Use authentication/authorization
- Audit dependencies with cargo-deny

### Performance
- Use async/await properly
- Implement caching where appropriate
- Profile with cargo-flamegraph
- Optimize hot paths

### Maintainability
- Follow Rust idioms
- Use type-driven design
- Write clear documentation
- Version your API

## Real-World Examples

### File System MCP Server
Provides file system access to AI assistants:
```rust
#[tool(description = "Read file contents")]
async fn read_file(&self, path: String) -> Result<String> {
    tokio::fs::read_to_string(path).await
        .map_err(|e| format!("Failed to read file: {}", e))
}
```

### Weather API MCP Server
Exposes weather data as resources:
```rust
#[resource(uri = "weather://{city}")]
async fn get_weather(&self, city: String) -> Result<WeatherData> {
    self.api_client.fetch_weather(&city).await
}
```

### Code Analysis MCP Server
Provides code analysis tools:
```rust
#[tool(description = "Analyze code complexity")]
async fn analyze_complexity(&self, code: String) -> Result<ComplexityReport> {
    analyze_code_metrics(&code)
}
```

## Troubleshooting

### rmcp crate not found
Ensure you have the correct dependency:
```toml
rmcp = { version = "0.8", features = ["server"] }
```

### Transport connection issues
Check transport configuration:
- stdio: Ensure proper stdin/stdout handling
- SSE: Verify network configuration
- HTTP: Check port availability

### Tool not being called
Verify:
- Tool is properly annotated with `#[tool]`
- Tool is in a `#[tool(tool_box)]` impl block
- Server is correctly listing tools

### Performance issues
Profile and optimize:
```bash
cargo flamegraph --bin your-mcp-server
```

## Contributing

This plugin follows Rust best practices and modern patterns. Contributions should:
- Use Rust 2024 Edition features
- Include comprehensive tests
- Follow clippy pedantic guidelines
- Document all public APIs

## Resources

- [MCP Specification](https://modelcontextprotocol.io)
- [rmcp GitHub](https://github.com/modelcontextprotocol/rust-sdk)
- [rmcp Alternative (4t145)](https://github.com/4t145/rmcp)
- [MCP Building Guide](https://mcpcat.io/guides/building-mcp-server-rust/)
- [Shuttle MCP Tutorials](https://www.shuttle.dev/blog/tags/mcp)

## Version History

- **1.0.0** - Initial release with comprehensive MCP server creation support

---

**Build powerful MCP servers in Rust** 🦀🔧
