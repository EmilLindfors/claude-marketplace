# Rust Modern Patterns Plugin

A comprehensive plugin for using the latest Rust features and patterns from Rust 2024 Edition and beyond.

## Overview

This plugin helps you write modern, idiomatic Rust code using the latest language features, patterns, and best practices. Stay up-to-date with Rust 2024 Edition features including let chains, async closures, improved match ergonomics, and more.

## Current Rust Version

**Latest Stable:** Rust 1.91.0 (as of November 2025)
**Edition:** Rust 2024 Edition (stabilized in 1.85.0)
**Note:** Rust 2024 Edition features are available from Rust 1.85.0 onwards

## Features

### 🚀 Modernization Commands

#### `/rust-modernize`
Analyze and modernize code to use latest Rust features.

**Features**:
- Convert old patterns to Rust 2024 Edition features
- Refactor to use let chains instead of nested if-let
- Update to async closures where appropriate
- Apply modern match ergonomics
- Suggest const improvements

#### `/rust-upgrade-edition`
Upgrade project to Rust 2024 Edition.

**Features**:
- Update Cargo.toml to edition = "2024"
- Check for breaking changes
- Suggest migration paths
- Update deprecated patterns
- Verify MSRV compatibility

#### `/rust-pattern-check`
Check code for opportunities to use modern patterns.

**Features**:
- Identify nested if-let chains
- Find async fn that could use async closures
- Suggest better match patterns
- Identify const-eligible code
- Check for outdated idioms

#### `/rust-async-traits`
Convert async-trait macro usage to native async fn in traits.

**Features**:
- Detect async-trait usage
- Convert to native async fn (Rust 1.75+)
- Identify when async-trait is still needed (dyn Trait)
- Remove unnecessary dependencies
- Improve performance by removing boxing overhead

### 🤖 Modern Rust Expert Agent

A specialized agent (`rust-modern-expert`) for modern Rust patterns.

**Capabilities**:
- Teach latest Rust 2024 features
- Refactor code to modern patterns
- Review code for best practices
- Design using latest features
- Migration guidance to Rust 2024

## Rust 2024 Edition Features

### Let Chains (Stabilized in 1.88)

Chain multiple let patterns with `&&` in if/while expressions:

```rust
// ❌ Old way (nested)
if let Some(user) = get_user() {
    if let Some(email) = user.email {
        if email.contains('@') {
            send_email(&email);
        }
    }
}

// ✅ Modern way (let chains)
if let Some(user) = get_user()
    && let Some(email) = user.email
    && email.contains('@')
{
    send_email(&email);
}
```

### Async Closures

Use async closures with automatic trait bounds:

```rust
// ❌ Old way
let futures: Vec<_> = items
    .iter()
    .map(|item| {
        let item = item.clone();
        async move { process(item).await }
    })
    .collect();

// ✅ Modern way
let futures: Vec<_> = items
    .iter()
    .map(async |item| {
        process(item).await
    })
    .collect();
```

### Async Functions in Traits (Native Support)

**Important:** Since Rust 1.75, async functions in traits are natively supported. The `async-trait` crate is now **optional** and only needed for:
- Supporting older Rust versions (< 1.75)
- Traits that need to be object-safe (dyn Trait)
- Specific edge cases with complex generic bounds

```rust
// ✅ Modern: Native async fn in traits (Rust 1.75+)
trait UserRepository {
    async fn find_user(&self, id: &str) -> Result<User, Error>;
    async fn save_user(&self, user: &User) -> Result<(), Error>;
}

// Implementation
impl UserRepository for PostgresRepo {
    async fn find_user(&self, id: &str) -> Result<User, Error> {
        // Native async, no macro needed!
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
    }

    async fn save_user(&self, user: &User) -> Result<(), Error> {
        // ...
    }
}

// ❌ Only use async-trait when you need dyn Trait
use async_trait::async_trait;

#[async_trait]
trait DynamicRepository {
    async fn fetch(&self) -> Result<Data, Error>;
}

// This is needed for:
let repo: Box<dyn DynamicRepository> = Box::new(my_repo);
```

### Improved Match Ergonomics

Better pattern matching with clearer semantics:

```rust
// ✅ Rust 2024: mut doesn't force by-value binding
match &data {
    Some(mut x) => {
        // x is &mut T, not T
        x.modify();
    }
    None => {}
}
```

### Const Improvements

More capabilities in const contexts:

```rust
// ✅ Reference static items in const contexts
const CONFIG: &Config = &GLOBAL_CONFIG;

// ✅ More const-capable standard library functions
const fn parse_number(s: &str) -> Option<i32> {
    // More operations allowed in const fn
}
```

### MSRV-Aware Resolver

Automatic dependency resolution respecting MSRV:

```toml
[package]
edition = "2024"
rust-version = "1.75"  # MSRV

# Resolver automatically picks compatible versions
```

### Gen Blocks (Rust 2024)

Generator blocks for iterators:

```rust
let fibonacci = gen {
    let (mut a, mut b) = (0, 1);
    loop {
        yield a;
        (a, b) = (b, a + b);
    }
};
```

### Never Type `!`

Proper never type for functions that never return:

```rust
fn exit_program() -> ! {
    std::process::exit(0)
}

fn parse_or_panic(s: &str) -> i32 {
    s.parse().unwrap_or_else(|_| exit_program())
}
```

## Modern Patterns

### Pattern 1: Let Chains for Complex Conditions

```rust
// Checking multiple Option/Result values
if let Ok(config) = load_config()
    && let Some(db_url) = config.database_url
    && let Ok(pool) = connect_to_db(&db_url)
{
    // All conditions met, use pool
}
```

### Pattern 2: Async Closures for Concurrent Operations

```rust
async fn process_items<F>(items: Vec<Item>, processor: F)
where
    F: AsyncFn(Item) -> Result<(), Error>,
{
    for item in items {
        processor(item).await?;
    }
}

// Usage
process_items(items, async |item| {
    validate(&item).await?;
    save_to_db(&item).await
}).await?;
```

### Pattern 3: Modern Match with Deref Patterns

```rust
match &option_box {
    Some(deref!(value)) => {
        // Use value directly
    }
    None => {}
}
```

### Pattern 4: Const Functions for Compile-Time Computation

```rust
const fn compute_table_size(items: usize) -> usize {
    items * std::mem::size_of::<Item>()
}

const TABLE_SIZE: usize = compute_table_size(1000);
static TABLE: [u8; TABLE_SIZE] = [0; TABLE_SIZE];
```

## Best Practices

1. **Use Let Chains**: Reduce nesting with let chains in if/while
2. **Async Closures**: Use async closures for better async ergonomics
3. **Edition 2024**: Upgrade to edition = "2024" for latest features
4. **Const When Possible**: Use const fn for compile-time computation
5. **Match Ergonomics**: Leverage improved match patterns
6. **MSRV Awareness**: Set rust-version in Cargo.toml
7. **Never Type**: Use `!` for functions that never return
8. **Gen Blocks**: Use generators for custom iterators

## Installation

```bash
cp -r plugins/rust-modern-patterns /path/to/your/project/plugins/
```

Register in marketplace.json:
```json
{
  "plugins": [{
    "name": "rust-modern-patterns",
    "source": "./plugins/rust-modern-patterns",
    "description": "Modern Rust patterns for Rust 2024 Edition",
    "version": "1.0.0"
  }]
}
```

## Usage

### Quick Start

1. **Check your code for modernization opportunities**:
   ```
   /rust-pattern-check
   ```

2. **Upgrade to Rust 2024 Edition**:
   ```
   /rust-upgrade-edition
   ```

3. **Modernize specific code**:
   ```
   /rust-modernize
   ```

4. **Get expert guidance**:
   ```
   Ask rust-modern-expert to help modernize my code to use let chains
   ```

## Cargo.toml Configuration

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"  # Use Rust 2024 Edition
rust-version = "1.85"  # Minimum Rust version

[dependencies]
# Your dependencies
```

## Migration Guide

### Upgrading to Rust 2024

1. **Update Cargo.toml**:
   ```toml
   edition = "2024"
   ```

2. **Run cargo fix**:
   ```bash
   cargo fix --edition
   ```

3. **Review warnings**:
   ```bash
   cargo check
   ```

4. **Update patterns**:
   - Convert nested if-let to let chains
   - Use async closures where appropriate
   - Update match patterns for new ergonomics

## Feature Compatibility

| Feature | Minimum Rust Version | Edition |
|---------|---------------------|---------|
| Let Chains | 1.88.0 | 2024 |
| Async Closures | 1.85.0 | 2024 |
| Match Ergonomics | 1.85.0 | 2024 |
| Gen Blocks | 1.85.0 | 2024 |
| MSRV Resolver | 1.84.0 | Any |
| Const Improvements | 1.83.0+ | Any |

## Resources

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [Let Chains RFC](https://rust-lang.github.io/rfcs/2497-if-let-chains.html)
- [Async Closures](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Match Ergonomics 2024](https://rust-lang.github.io/rfcs/3627-match-ergonomics-2024.html)
- [Rust Release Notes](https://doc.rust-lang.org/releases.html)

## Troubleshooting

### Edition 2024 not recognized
- Ensure Rust 1.85.0 or later: `rustup update`
- Check `rustc --version`

### Let chains not working
- Verify edition = "2024" in Cargo.toml
- Ensure Rust 1.88.0 or later

### Async closures not compiling
- Check Rust version (1.85.0+)
- Verify proper trait bounds (AsyncFn/AsyncFnMut)

## Version History

- **1.0.0** - Initial release with Rust 2024 Edition features

---

**Write modern Rust, leverage the latest features** 🦀✨
