# Rust MCP Server Plugin - Context

## Purpose

This plugin enables developers to create production-ready Model Context Protocol (MCP) servers using Rust and the rmcp crate. It provides comprehensive guidance, tools, and patterns for building MCP servers that expose tools, resources, and prompts to AI assistants.

## Target Audience

- **Rust Developers** building MCP servers
- **AI Integration Engineers** creating custom capabilities
- **Tool Developers** exposing services to AI assistants
- **Backend Engineers** integrating AI into applications

## Plugin Architecture

### Design Principles

1. **Comprehensive Coverage**: Cover all aspects of MCP server development
2. **Best Practices**: Promote modern Rust patterns and MCP conventions
3. **Production-Ready**: Focus on deployable, maintainable code
4. **Type-Safe**: Leverage Rust's type system for correctness
5. **Performance-First**: Async/await patterns and optimization

### Component Organization

#### Skills (6 total)
Each skill is a focused guide on a specific aspect of MCP development:

1. **rmcp-quickstart**: Entry point for new users
2. **mcp-tools-guide**: Deep dive into tool creation
3. **mcp-resources-guide**: Resource implementation patterns
4. **mcp-prompts-guide**: Prompt design and usage
5. **mcp-transport-guide**: Transport layer understanding
6. **mcp-best-practices**: Production patterns and architecture

#### Commands (6 total)
Practical commands that generate code and scaffolding:

1. **mcp-init**: Project initialization
2. **mcp-add-tool**: Tool scaffolding
3. **mcp-add-resource**: Resource implementation
4. **mcp-add-prompt**: Prompt creation
5. **mcp-test**: Testing infrastructure
6. **mcp-deploy**: Deployment setup

#### Agents (1 total)
Expert agent for consultation and design:

1. **mcp-architect**: MCP server architecture and design expert

## rmcp Crate Context

### Ecosystem Status (November 2025)

- **Official SDK**: https://github.com/modelcontextprotocol/rust-sdk
- **Alternative Implementation**: https://github.com/4t145/rmcp (BEST Rust SDK)
- **Current Version**: 0.8.3
- **Maturity**: Production-ready, actively maintained

### Key Capabilities

#### 1. Tool Definition
The `#[tool]` macro provides declarative tool definition:
- Automatic JSON schema generation
- Type-safe parameter handling
- Async-first design
- Error handling integration

#### 2. Transport Layer
Multiple transport options:
- **stdio**: For subprocess/local execution
- **SSE**: Server-Sent Events for cloud hosting
- **HTTP**: Streamable HTTP for web services
- **Custom**: Extensible transport system

#### 3. Service Pattern
Clean three-step pattern:
1. Build transport (communication layer)
2. Build service (implement ServerHandler)
3. Serve together (.serve(transport).await)

#### 4. Type Safety
- Leverage schemars for JSON schema
- Compile-time parameter validation
- Type-safe request/response handling

### rmcp vs Official SDK

Both implementations are high-quality:

**rmcp (4t145/rmcp)**:
- Emphasizes clean API design
- Strong macro support
- Excellent documentation
- Community-driven development

**Official SDK (modelcontextprotocol/rust-sdk)**:
- Official MCP implementation
- Long-term support guaranteed
- Standards compliance
- Corporate backing

**Recommendation**: This plugin supports both, with preference for the official SDK for new projects.

## MCP Protocol Context

### Core Concepts

#### Tools
Functions that AI assistants can invoke:
- Define capabilities (search, calculate, execute)
- Take structured parameters
- Return typed results
- Can be synchronous or asynchronous

#### Resources
Data sources that provide context:
- Files, databases, APIs
- URI-based addressing
- Support for listing and fetching
- Can be static or dynamic

#### Prompts
Templates that guide AI interactions:
- Predefined conversation starters
- Dynamic argument injection
- Context-aware suggestions
- Support for multi-turn dialogs

### Protocol Architecture

```
┌─────────────┐         ┌─────────────┐
│   AI        │         │   MCP       │
│   Assistant │ ◄─────► │   Server    │
│   (Client)  │  JSON-  │   (Rust)    │
└─────────────┘  RPC    └─────────────┘
                              │
                              ▼
                         ┌─────────────┐
                         │   Tools,    │
                         │   Resources,│
                         │   Prompts   │
                         └─────────────┘
```

## Implementation Patterns

### Pattern 1: Simple Tool Server

**Use Case**: Expose utility functions to AI
**Transport**: stdio
**Complexity**: Low

```rust
#[tool(tool_box)]
impl MyService {
    #[tool(description = "Calculate")]
    async fn calculate(&self, expr: String) -> Result<f64> {
        eval_expression(&expr)
    }
}
```

### Pattern 2: Data Integration Server

**Use Case**: Provide database/API access
**Transport**: SSE or HTTP
**Complexity**: Medium

```rust
#[resource(uri = "data://{table}/{id}")]
async fn get_record(&self, table: String, id: String) -> Result<JsonValue> {
    self.db.fetch(&table, &id).await
}
```

### Pattern 3: Multi-Capability Server

**Use Case**: Full-featured MCP server
**Transport**: HTTP with auth
**Complexity**: High

Combines tools, resources, and prompts with:
- Authentication/authorization
- Rate limiting
- Monitoring
- Caching
- Error tracking

## Development Workflow

### Phase 1: Design
1. Identify capabilities to expose
2. Design tool/resource/prompt APIs
3. Choose transport mechanism
4. Plan security/auth strategy

### Phase 2: Implementation
1. Run `/mcp-init` to scaffold project
2. Implement tools with `/mcp-add-tool`
3. Add resources with `/mcp-add-resource`
4. Create prompts with `/mcp-add-prompt`
5. Write comprehensive tests

### Phase 3: Testing
1. Unit tests for each capability
2. Integration tests for server
3. Protocol validation tests
4. Performance benchmarking
5. Security testing

### Phase 4: Deployment
1. Build optimized binary
2. Set up environment config
3. Implement monitoring
4. Deploy to target platform
5. Document usage

## Best Practices Integration

### Rust Best Practices
- Use Rust 2024 Edition features
- Enable clippy pedantic lints
- Comprehensive error handling
- Proper async patterns
- Type-driven design

### MCP Best Practices
- Clear tool descriptions
- Sensible parameter types
- Consistent error messages
- Proper resource URIs
- Well-designed prompts

### Production Practices
- Structured logging (tracing)
- Metrics and monitoring
- Health check endpoints
- Graceful shutdown
- Configuration management

## Security Considerations

### Input Validation
- Validate all tool parameters
- Sanitize resource URIs
- Check prompt arguments
- Prevent injection attacks

### Authentication
- Token-based auth for remote servers
- Client certificate validation
- API key management
- Session handling

### Authorization
- Role-based access control
- Tool-level permissions
- Resource access policies
- Audit logging

### Dependencies
- Regular security audits (cargo-audit)
- License compliance (cargo-deny)
- SBOM generation
- Vulnerability monitoring

## Performance Considerations

### Async Patterns
- Proper use of tokio runtime
- Avoid blocking in async contexts
- Use channels for communication
- Connection pooling

### Caching
- Cache expensive operations
- Resource result caching
- Schema caching
- Connection reuse

### Optimization
- Profile with cargo-flamegraph
- Optimize hot paths
- Minimize allocations
- Efficient serialization

## Testing Strategy

### Unit Tests
- Test each tool in isolation
- Mock external dependencies
- Verify error handling
- Test edge cases

### Integration Tests
- Test server end-to-end
- Verify protocol compliance
- Test transport layer
- Validate JSON-RPC

### Property Tests
- Use proptest for tools
- Verify invariants
- Test random inputs
- Fuzz testing

### Performance Tests
- Benchmark tool execution
- Test concurrent requests
- Measure latency
- Profile memory usage

## Deployment Patterns

### Local/Development
- stdio transport
- Simple binary
- Direct execution
- Quick iteration

### Cloud/Production
- HTTP or SSE transport
- Container deployment
- Load balancing
- Auto-scaling

### Enterprise
- Multiple servers
- Service mesh integration
- Centralized monitoring
- HA configuration

## Related Technologies

### Rust Ecosystem
- **tokio**: Async runtime
- **serde**: Serialization
- **schemars**: JSON schema
- **thiserror**: Error handling
- **tracing**: Logging

### MCP Ecosystem
- **Claude Desktop**: MCP client
- **MCP Inspector**: Testing tool
- **Other SDKs**: TypeScript, Python
- **MCP Registry**: Server directory

## Future Considerations

### Emerging Features
- Streaming responses
- Binary tool results
- Resource subscriptions
- Advanced auth patterns

### Ecosystem Evolution
- New transport types
- Enhanced protocol features
- Improved tooling
- Community libraries

## Learning Resources

### Official Documentation
- MCP Specification: https://modelcontextprotocol.io
- rmcp Docs: https://docs.rs/rmcp
- Rust Async Book: https://rust-lang.github.io/async-book/

### Tutorials
- Shuttle.dev MCP guides
- MCPcat.io tutorials
- Community examples
- Video walkthroughs

### Community
- MCP Discord
- Rust Subreddit
- GitHub Discussions
- Stack Overflow

## Plugin Usage Philosophy

This plugin follows a "teach and enable" philosophy:

1. **Skills teach concepts** - Deep understanding of MCP patterns
2. **Commands provide scaffolding** - Quick, correct implementations
3. **Agent provides consultation** - Design decisions and architecture

Users should:
1. Read relevant skills to understand concepts
2. Use commands to generate initial code
3. Consult agent for complex decisions
4. Iterate and refine with plugin support

## Success Criteria

A user successfully uses this plugin when they can:

1. Initialize a new MCP server project
2. Add tools, resources, and prompts
3. Test their server locally
4. Deploy to production
5. Maintain and extend their server
6. Follow Rust and MCP best practices
7. Build secure, performant servers

## Maintenance Notes

### Keeping Current
- Monitor rmcp releases
- Track MCP specification changes
- Update Rust version requirements
- Refresh examples and patterns
- Incorporate community feedback

### Quality Standards
- All examples must be tested
- Commands must generate working code
- Skills must be accurate and current
- Agent must provide sound advice
- Documentation must be comprehensive

---

This context document guides the development and usage of the Rust MCP Server plugin, ensuring consistency, quality, and effectiveness.
