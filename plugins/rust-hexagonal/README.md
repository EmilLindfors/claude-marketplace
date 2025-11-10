# Rust Hexagonal Architecture Plugin

A comprehensive plugin for implementing hexagonal architecture (ports and adapters pattern) in Rust projects.

## Overview

The Rust Hexagonal Architecture plugin helps you design and implement clean, maintainable Rust applications following the hexagonal architecture pattern. This pattern separates your domain logic from external concerns, making your code more testable, flexible, and maintainable.

## Features

### 📐 Architecture Setup Commands

#### `/rust-hex-init`
Initialize a hexagonal architecture project structure for Rust.

**Features**:
- Creates proper directory structure for domain, ports, and adapters
- Sets up initial module files with proper exports
- Generates example port traits and adapters
- Creates Cargo.toml workspace configuration
- Includes best practice comments and documentation

#### `/rust-hex-add-port`
Add a new port (interface) to your hexagonal architecture.

**Features**:
- Interactive prompts for port type (driving/driven) and name
- Generates trait definition with proper documentation
- Creates corresponding adapter stub
- Updates module exports automatically
- Follows Rust naming conventions

#### `/rust-hex-add-adapter`
Add a new adapter implementation for an existing port.

**Features**:
- Lists available ports to implement
- Generates adapter struct and implementation
- Includes common patterns (async, error handling, configuration)
- Creates test module scaffold
- Auto-updates module exports

### 🤖 Hexagonal Architecture Agent

A specialized agent (`rust-hex-architect`) for complex architecture tasks.

**Capabilities**:
- Analyze existing codebase and suggest hexagonal refactoring
- Design port interfaces based on domain requirements
- Generate complete adapter implementations
- Review architecture for proper separation of concerns
- Suggest improvements for testability and maintainability
- Create integration examples between components

## Hexagonal Architecture Concepts

### What is Hexagonal Architecture?

Hexagonal architecture (also known as ports and adapters) is an architectural pattern that aims to create loosely coupled application components. In Rust, this pattern leverages:

- **Traits** for defining ports (interfaces)
- **Structs** for domain logic and adapters
- **Compile-time polymorphism** for zero-cost abstractions
- **Dependency injection** through function parameters or struct fields

### Core Components

1. **Domain** (The Hexagon)
   - Core business logic
   - Independent of external concerns
   - Pure Rust with minimal dependencies

2. **Ports** (Interfaces)
   - **Driving Ports** (Primary): What the domain offers to the outside world
   - **Driven Ports** (Secondary): What the domain needs from the outside world
   - Defined as Rust traits

3. **Adapters** (Implementations)
   - **Driving Adapters**: REST API, CLI, gRPC handlers
   - **Driven Adapters**: Database repositories, HTTP clients, file systems
   - Implement port traits

## Directory Structure

```
your-project/
├── Cargo.toml
└── src/
    ├── domain/
    │   ├── mod.rs
    │   ├── models.rs         # Domain entities
    │   └── services.rs       # Business logic
    ├── ports/
    │   ├── mod.rs
    │   ├── driving.rs        # Primary ports
    │   └── driven.rs         # Secondary ports
    └── adapters/
        ├── mod.rs
        ├── driving/
        │   ├── mod.rs
        │   ├── rest_api.rs   # HTTP adapter
        │   └── cli.rs        # CLI adapter
        └── driven/
            ├── mod.rs
            ├── postgres.rs   # Database adapter
            └── http_client.rs # External API adapter
```

## Installation

The plugin is installed in this repository. To enable it in other projects:

1. **Install the plugin**:
   ```bash
   cp -r plugins/rust-hexagonal /path/to/your/project/plugins/
   ```

2. **Register in marketplace.json**:
   ```json
   {
     "plugins": [
       {
         "name": "rust-hexagonal",
         "source": "./plugins/rust-hexagonal",
         "description": "Hexagonal architecture plugin for Rust",
         "version": "1.0.0"
       }
     ]
   }
   ```

## Usage

### Quick Start

1. **Initialize hexagonal structure**:
   ```
   /rust-hex-init
   ```

2. **Add a driven port** (e.g., user repository):
   ```
   /rust-hex-add-port
   ```
   Select "driven" and name it "UserRepository"

3. **Add an adapter** (e.g., PostgreSQL implementation):
   ```
   /rust-hex-add-adapter
   ```
   Select the port and name the adapter

4. **Use the architect agent** for complex refactoring:
   ```
   Ask the rust-hex-architect agent to help refactor my existing code to use hexagonal architecture
   ```

### Example: User Service

**Domain Service** (`src/domain/user_service.rs`):
```rust
use crate::ports::driven::UserRepository;

pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_user(&self, id: &str) -> Result<User, Error> {
        self.repository.find_by_id(id).await
    }
}
```

**Driven Port** (`src/ports/driven.rs`):
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<User, Error>;
    async fn save(&self, user: &User) -> Result<(), Error>;
}
```

**Adapter** (`src/adapters/driven/postgres_user_repo.rs`):
```rust
pub struct PostgresUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<User, Error> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))
    }
}
```

## Best Practices

1. **Keep domain pure**: No external dependencies in domain code
2. **Use traits for ports**: Define clear contracts with traits
3. **Prefer composition over inheritance**: Rust doesn't have inheritance, use composition
4. **Make adapters swappable**: Easy to replace implementations
5. **Test domain independently**: Mock adapters using test doubles
6. **Use async traits**: For I/O operations, use `#[async_trait]`
7. **Handle errors properly**: Define domain-specific error types
8. **Document ports clearly**: Explain preconditions and postconditions

## Benefits in Rust

- **Zero-cost abstractions**: Traits compile to static dispatch by default
- **Type safety**: Compiler ensures ports are properly implemented
- **Testability**: Easy to create mock implementations
- **Flexibility**: Swap adapters without changing domain code
- **Clear boundaries**: Explicit separation of concerns

## Troubleshooting

### Commands not available
- Verify plugin is registered in marketplace.json
- Restart Claude Code to reload plugins

### Generated code doesn't compile
- Ensure you have required dependencies in Cargo.toml
- Check that async-trait is added if using async
- Verify all imports are correct

## Contributing

To contribute improvements:
1. Make changes to plugin files
2. Test commands and agents
3. Update README and CONTEXT.md
4. Submit improvements

## License

Part of the Claude Code plugin collection.

---

**Clean architecture, powerful Rust** 🦀🔷
