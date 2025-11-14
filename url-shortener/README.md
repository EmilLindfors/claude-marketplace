# URL Shortener

A modern Rust URL shortener library demonstrating best practices in software architecture and type-driven design.

## Features

- **Hexagonal Architecture** - Clean separation of concerns with ports and adapters
- **Type-Driven Design** - Leverage Rust's type system to prevent bugs at compile time
- **Type Safety** - Newtype pattern prevents mixing up identifiers
- **Validated Types** - Domain types guarantee invariants
- **Comprehensive Error Handling** - Using `thiserror` for ergonomic errors
- **Thread-Safe** - Built with `Arc` and `RwLock` for concurrent access
- **Well-Tested** - Extensive unit and integration tests
- **Access Tracking** - Built-in analytics with access counters

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
url_shortener = "0.1.0"
```

Basic usage:

```rust
use url_shortener::{
    service::UrlShortenerService,
    adapters::{InMemoryUrlRepository, RandomIdGenerator},
    domain::OriginalUrl,
};
use std::sync::Arc;

// Set up the service
let repository = Arc::new(InMemoryUrlRepository::new());
let id_generator = Arc::new(RandomIdGenerator::new());
let service = UrlShortenerService::new(repository, id_generator);

// Shorten a URL
let url = OriginalUrl::new("https://example.com/very/long/path".to_string())?;
let shortened = service.shorten_url(url)?;

println!("Short code: {}", shortened.short_code());

// Resolve the short code
let original = service.resolve_short_code(shortened.short_code())?;
println!("Original URL: {}", original);
```

## Architecture

This library follows **Hexagonal Architecture** (also known as Ports and Adapters):

```
┌─────────────────────────────────────────┐
│           Application Core              │
│  ┌─────────────────────────────────┐   │
│  │       Domain Layer              │   │
│  │  - UrlId (newtype)              │   │
│  │  - ShortCode (validated)        │   │
│  │  - OriginalUrl (validated)      │   │
│  │  - ShortenedUrl (aggregate)     │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │       Service Layer             │   │
│  │  - UrlShortenerService          │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │       Ports (Traits)            │   │
│  │  - UrlRepository                │   │
│  │  - IdGenerator                  │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
        ↑                    ↑
        │                    │
┌───────────────┐    ┌───────────────┐
│   Adapters    │    │   Adapters    │
│ InMemoryRepo  │    │ RandomIdGen   │
│ (+ Database)  │    │ (+ UUID, etc) │
└───────────────┘    └───────────────┘
```

### Why Hexagonal Architecture?

1. **Testability** - Easy to mock dependencies with trait objects
2. **Flexibility** - Swap implementations without changing business logic
3. **Maintainability** - Clear separation of concerns
4. **Independence** - Domain logic doesn't depend on external frameworks

## Type-Driven Design

The library uses Rust's type system to encode domain invariants:

### Newtype Pattern

```rust
use url_shortener::domain::{UrlId, ShortCode};

let id = UrlId::new("abc123".to_string());
let code = ShortCode::new("mycode".to_string())?;

// Compile error - can't mix types!
// let x: UrlId = code;
```

### Validated Types

```rust
use url_shortener::domain::ShortCode;

// Short codes are validated at construction
let valid = ShortCode::new("abc123".to_string())?;  // ✅ OK

let too_short = ShortCode::new("abc".to_string());  // ❌ Error
let invalid = ShortCode::new("abc-123".to_string()); // ❌ Error
```

Once you have a `ShortCode`, you're **guaranteed** it's valid!

## Examples

### Custom Short Codes

```rust
let url = OriginalUrl::new("https://example.com".to_string())?;
let code = ShortCode::new("custom".to_string())?;

let shortened = service.shorten_url_with_code(url, code)?;
```

### Access Statistics

```rust
let stats = service.get_statistics(&short_code)?;
println!("Accessed {} times", stats.access_count());
println!("Created at {:?}", stats.created_at());
```

### Delete Short URLs

```rust
service.delete_short_code(&short_code)?;
```

## Extending the Library

Want to add a database backend? Just implement the `UrlRepository` trait:

```rust
use url_shortener::ports::UrlRepository;

struct PostgresRepository {
    pool: PgPool,
}

impl UrlRepository for PostgresRepository {
    fn save(&self, url: ShortenedUrl) -> Result<()> {
        // Your PostgreSQL implementation
    }
    // ... other methods
}
```

Want custom ID generation? Implement `IdGenerator`:

```rust
use url_shortener::ports::IdGenerator;

struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn generate_id(&self) -> UrlId {
        UrlId::new(uuid::Uuid::new_v4().to_string())
    }
    // ...
}
```

## Testing

Run tests:

```bash
cargo test
```

Run with coverage:

```bash
cargo tarpaulin --out Html
```

## License

MIT

## Credits

Built with modern Rust patterns inspired by:
- Domain-Driven Design
- Hexagonal Architecture
- Type-Driven Development
