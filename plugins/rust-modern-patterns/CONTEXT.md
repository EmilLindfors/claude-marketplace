# Rust Modern Patterns Plugin - Context

## Purpose

This plugin helps developers write modern, idiomatic Rust code using the latest features from Rust 2024 Edition and beyond. It provides guidance on migrating to modern patterns and leveraging new language capabilities.

## Rust Version Information

**Current Latest Stable:** Rust 1.91.0 (as of November 2025)
**Edition Introduced:** Rust 2024 (stabilized in 1.85.0)
**Previous Edition:** Rust 2021

**Note:** Rust 2024 Edition features require minimum Rust 1.85.0, but the latest stable version 1.91.0 includes all features plus additional improvements.

### Version Timeline

- **Rust 1.91.0** (Current stable) - Latest stable with all features
- **Rust 1.85.0** (Feb 20, 2025) - Stabilized Rust 2024 Edition
- **Rust 1.84.0** (Jan 9, 2025) - MSRV-aware resolver, new trait solver
- **Rust 1.83.0** (Nov 28, 2024) - Const improvements
- **Rust 1.75.0** (Dec 28, 2023) - Native async fn in traits

## Rust 2024 Edition Features

### 1. Let Chains

**Stabilized:** Rust 1.88.0 (part of 2024 edition)

Let chains allow chaining multiple `let` patterns with `&&` in if/while expressions.

#### Before (Nested):
```rust
// Deeply nested if-let
if let Some(user) = get_user(id) {
    if let Some(profile) = user.profile {
        if profile.is_active {
            if let Some(email) = profile.email {
                send_notification(&email);
            }
        }
    }
}
```

#### After (Let Chains):
```rust
// Flat, readable chain
if let Some(user) = get_user(id)
    && let Some(profile) = user.profile
    && profile.is_active
    && let Some(email) = profile.email
{
    send_notification(&email);
}
```

#### Use Cases:

1. **Multiple Option unwrapping**:
```rust
if let Some(a) = option_a()
    && let Some(b) = option_b()
    && let Some(c) = option_c()
{
    process(a, b, c);
}
```

2. **Result checking with conditions**:
```rust
if let Ok(config) = load_config()
    && config.enabled
    && let Ok(conn) = establish_connection(&config)
{
    use_connection(conn);
}
```

3. **Pattern matching with guards**:
```rust
if let Some(User { age, .. }) = user
    && age >= 18
    && let Some(license) = check_license()
{
    allow_access(license);
}
```

4. **While loops with complex conditions**:
```rust
while let Some(item) = iterator.next()
    && item.is_valid()
    && let Ok(processed) = process_item(item)
{
    results.push(processed);
}
```

### 2. Async Closures

**Stabilized:** Rust 1.85.0 (Rust 2024)

Async closures support `async || {}` syntax and work with `AsyncFn`, `AsyncFnMut`, and `AsyncFnOnce` traits.

**Note on Async Traits:** Since Rust 1.75, async functions in traits are natively supported without the `async-trait` crate. The `async-trait` crate is now only needed for:
- Supporting Rust < 1.75
- Dynamic dispatch with `dyn Trait` (object safety)
- Specific edge cases with complex generic patterns

#### Before:
```rust
let futures: Vec<_> = items
    .iter()
    .map(|item| {
        let item = item.clone();
        async move {
            fetch_data(item).await
        }
    })
    .collect();
```

#### After:
```rust
let futures: Vec<_> = items
    .iter()
    .map(async |item| {
        fetch_data(item).await
    })
    .collect();
```

### Async Functions in Traits (Native - No Macro Needed)

**Stabilized:** Rust 1.75.0 (December 2023)

Since Rust 1.75, async functions in traits are natively supported without the `async-trait` crate.

#### When to Use Native Async Fn:

```rust
// ✅ Modern: No macro needed (Rust 1.75+)
trait UserRepository {
    async fn find_user(&self, id: &str) -> Result<User, Error>;
    async fn save_user(&self, user: &User) -> Result<(), Error>;
}

impl UserRepository for PostgresRepo {
    async fn find_user(&self, id: &str) -> Result<User, Error> {
        // Native async, no macro needed!
        self.db.query(id).await
    }

    async fn save_user(&self, user: &User) -> Result<(), Error> {
        self.db.insert(user).await
    }
}

// Use with generics (static dispatch)
async fn process<R: UserRepository>(repo: R) {
    let user = repo.find_user("123").await.unwrap();
}
```

#### When async-trait is Still Needed:

```rust
// ❌ Native async fn doesn't support dyn Trait
// ✅ Use async-trait for dynamic dispatch

use async_trait::async_trait;

#[async_trait]
trait Plugin: Send + Sync {
    async fn execute(&self) -> Result<(), Error>;
}

// This requires async-trait:
let plugins: Vec<Box<dyn Plugin>> = vec![
    Box::new(PluginA),
    Box::new(PluginB),
];
```

**Summary:**
- **Static dispatch (generics):** Use native async fn ✅
- **Dynamic dispatch (dyn Trait):** Use async-trait crate
- **MSRV < 1.75:** Use async-trait crate
- **Performance critical:** Use native async fn (zero-cost)

#### Use Cases:

1. **Async iteration**:
```rust
async fn process_all<F>(items: Vec<Item>, f: F)
where
    F: AsyncFn(Item) -> Result<(), Error>,
{
    for item in items {
        f(item).await?;
    }
}

// Usage
process_all(items, async |item| {
    validate(&item).await?;
    save(&item).await
}).await?;
```

2. **Async callbacks**:
```rust
struct EventHandler<F>
where
    F: AsyncFnMut(Event) -> Result<(), Error>,
{
    handler: F,
}

impl<F> EventHandler<F>
where
    F: AsyncFnMut(Event) -> Result<(), Error>,
{
    async fn handle(&mut self, event: Event) -> Result<(), Error> {
        (self.handler)(event).await
    }
}
```

3. **Retry with async closure**:
```rust
async fn retry<F, T>(mut f: F, times: usize) -> Result<T, Error>
where
    F: AsyncFnMut() -> Result<T, Error>,
{
    for _ in 0..times {
        match f().await {
            Ok(result) => return Ok(result),
            Err(_) => continue,
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

### 3. Match Ergonomics 2024

**Stabilized:** Rust 1.85.0 (Rust 2024)

Improved match ergonomics with clearer binding semantics.

#### Key Changes:

1. **`mut` binding doesn't force by-value**:

```rust
// Rust 2024
match &data {
    Some(mut x) => {
        // x is &mut T (not T moved)
        x.modify();  // Modifies through reference
    }
    None => {}
}
```

2. **Explicit pattern requirements**:

```rust
// ✅ Allowed: fully explicit pattern
match value {
    &Some(ref mut x) => { /* ... */ }
    _ => {}
}

// ❌ Not allowed in 2024: mixed implicit/explicit
match value {
    Some(ref mut x) => { /* reserved for future */ }
    _ => {}
}
```

#### Use Cases:

1. **Reference matching without moves**:
```rust
fn process(data: &Option<String>) {
    match data {
        Some(mut s) => {
            // s is &mut String, can modify in place
            s.push_str("!");
        }
        None => {}
    }
}
```

2. **Clearer intent with explicit patterns**:
```rust
match &user {
    User { name: ref n, age: ref a } => {
        println!("{} is {}", n, a);
    }
}
```

### 4. Const Improvements

**Ongoing:** Multiple versions (1.83+)

More operations allowed in const contexts.

#### Use Cases:

1. **Reference statics in const**:
```rust
static GLOBAL_CONFIG: Config = Config::default();

const APP_CONFIG: &Config = &GLOBAL_CONFIG;
```

2. **More const fn capabilities**:
```rust
const fn compute_hash(s: &str) -> u64 {
    let mut hash = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(31).wrapping_add(bytes[i] as u64);
        i += 1;
    }
    hash
}

const CACHE_KEY: u64 = compute_hash("my_key");
```

3. **Const panics with messages**:
```rust
const fn validate_size(size: usize) -> usize {
    if size == 0 {
        panic!("Size must be non-zero");
    }
    size
}
```

### 5. Gen Blocks

**Stabilized:** Rust 1.85.0 (Rust 2024)

Generator blocks for creating iterators.

```rust
let fibonacci = gen {
    let (mut a, mut b) = (0, 1);
    loop {
        yield a;
        (a, b) = (b, a + b);
    }
};

for num in fibonacci.take(10) {
    println!("{}", num);
}
```

#### Use Cases:

1. **Custom iterators**:
```rust
fn range_gen(start: i32, end: i32) -> impl Iterator<Item = i32> {
    gen {
        let mut current = start;
        while current < end {
            yield current;
            current += 1;
        }
    }
}
```

2. **Stateful iteration**:
```rust
fn tree_traversal(node: &Node) -> impl Iterator<Item = &Node> {
    gen {
        yield node;
        for child in &node.children {
            for descendant in tree_traversal(child) {
                yield descendant;
            }
        }
    }
}
```

### 6. Never Type `!`

The never type for functions that never return.

```rust
fn exit_with_error(msg: &str) -> ! {
    eprintln!("Fatal error: {}", msg);
    std::process::exit(1)
}

fn validate_or_exit(value: Option<i32>) -> i32 {
    value.unwrap_or_else(|| exit_with_error("Missing value"))
}
```

### 7. MSRV-Aware Resolver

**Stabilized:** Rust 1.84.0

Cargo resolver respects `rust-version` when selecting dependencies.

```toml
[package]
edition = "2024"
rust-version = "1.75"

# Cargo will only select dependency versions compatible with Rust 1.75
```

## Modern Pattern Migrations

### Migration 1: Nested If-Let → Let Chains

**Before:**
```rust
fn process_user(user_id: &str) -> Option<String> {
    if let Some(user) = db.find_user(user_id) {
        if let Some(profile) = user.profile {
            if profile.active {
                return Some(profile.display_name);
            }
        }
    }
    None
}
```

**After:**
```rust
fn process_user(user_id: &str) -> Option<String> {
    if let Some(user) = db.find_user(user_id)
        && let Some(profile) = user.profile
        && profile.active
    {
        Some(profile.display_name)
    } else {
        None
    }
}
```

### Migration 2: Manual Async Wrapper → Async Closure

**Before:**
```rust
let tasks: Vec<_> = ids
    .into_iter()
    .map(|id| {
        tokio::spawn(async move {
            fetch_user(id).await
        })
    })
    .collect();
```

**After:**
```rust
let tasks: Vec<_> = ids
    .into_iter()
    .map(|id| {
        tokio::spawn(async move {
            fetch_user(id).await
        })
    })
    .collect();

// Or with async closure (when fully supported):
async fn map_async<F, T, R>(items: Vec<T>, f: F) -> Vec<R>
where
    F: AsyncFn(T) -> R,
{
    // Implementation
}
```

### Migration 3: Iterator → Gen Block

**Before:**
```rust
struct RangeIter {
    current: i32,
    end: i32,
}

impl Iterator for RangeIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}
```

**After:**
```rust
fn range_iter(start: i32, end: i32) -> impl Iterator<Item = i32> {
    gen {
        let mut current = start;
        while current < end {
            yield current;
            current += 1;
        }
    }
}
```

## Common Anti-Patterns

### ❌ Anti-Pattern 1: Unnecessary Nesting with If-Let

```rust
// BAD: Deep nesting
if let Some(a) = option_a {
    if let Some(b) = option_b {
        if let Some(c) = option_c {
            process(a, b, c);
        }
    }
}

// GOOD: Use let chains
if let Some(a) = option_a
    && let Some(b) = option_b
    && let Some(c) = option_c
{
    process(a, b, c);
}
```

### ❌ Anti-Pattern 2: Clone-Happy Async Closures

```rust
// BAD: Unnecessary clones
let items = vec![1, 2, 3];
let futures: Vec<_> = items
    .iter()
    .map(|item| {
        let item = item.clone();  // Unnecessary with async closures
        async move { process(item).await }
    })
    .collect();

// GOOD: Use async closures (when available)
let futures: Vec<_> = items
    .iter()
    .map(async |item| {
        process(*item).await  // Can borrow directly
    })
    .collect();
```

### ❌ Anti-Pattern 3: Not Using Const When Possible

```rust
// BAD: Runtime computation of constant
fn get_buffer_size() -> usize {
    1024 * 1024
}

static BUFFER: Vec<u8> = Vec::with_capacity(get_buffer_size());

// GOOD: Const computation
const fn buffer_size() -> usize {
    1024 * 1024
}

const BUFFER_SIZE: usize = buffer_size();
static BUFFER: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
```

## Upgrade Checklist

When migrating to Rust 2024:

- [ ] Update `edition = "2024"` in Cargo.toml
- [ ] Set `rust-version` to minimum supported version
- [ ] Run `cargo fix --edition`
- [ ] Convert nested if-let to let chains
- [ ] Replace async wrappers with async closures
- [ ] Update match patterns for new ergonomics
- [ ] Use gen blocks for custom iterators
- [ ] Mark const-eligible functions as const
- [ ] Use `!` for never-returning functions
- [ ] Review and test all changes

## Best Practices

1. **Use Let Chains for Clarity**: Replace nested if-let with flat let chains
2. **Leverage Async Closures**: Simpler async iteration and callbacks
3. **Const by Default**: Make functions const when possible
4. **Gen Blocks for Iterators**: Cleaner than manual Iterator impl
5. **Explicit Match Patterns**: Use ref/mut explicitly in 2024
6. **MSRV Awareness**: Set rust-version in Cargo.toml
7. **Never Type**: Use `!` for functions that don't return
8. **Stay Updated**: Follow Rust release notes for new features

## Resources

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [Let Chains RFC #2497](https://rust-lang.github.io/rfcs/2497-if-let-chains.html)
- [Match Ergonomics RFC #3627](https://rust-lang.github.io/rfcs/3627-match-ergonomics-2024.html)
- [Rust Release Notes](https://doc.rust-lang.org/releases.html)
- [Rust Blog](https://blog.rust-lang.org/)

---

This context helps Claude Code understand and teach modern Rust patterns from the 2024 edition.
