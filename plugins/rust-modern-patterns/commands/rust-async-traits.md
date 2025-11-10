---
description: Modernize async trait usage to native async fn in traits
---

You are helping modernize async trait definitions to use native async fn in traits instead of the async-trait crate.

## Your Task

Convert traits using the `async-trait` crate to native async fn in traits, which has been supported since Rust 1.75.

## Background

**Since Rust 1.75 (December 2023):** Async functions in traits are natively supported without requiring the `async-trait` crate.

**When to use async-trait:**
- Supporting Rust versions < 1.75
- Need for `dyn Trait` (dynamic dispatch/object safety)
- Specific edge cases with complex bounds

**When to use native async fn:**
- Rust 1.75 or later ✅
- Static dispatch (generics)
- Modern codebases

## Steps

1. **Check Current Usage**

   Scan for async-trait usage:
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   trait MyTrait {
       async fn method(&self) -> Result<T, E>;
   }
   ```

2. **Verify Rust Version**

   Check Cargo.toml:
   ```toml
   [package]
   rust-version = "1.75"  # Or higher
   ```

   If rust-version < 1.75, ask user if they can upgrade.

3. **Identify Use Cases**

   Categorize each async trait:

   **Can Remove async-trait (most common):**
   - Trait used with generics/static dispatch
   - No `Box<dyn Trait>` usage
   - Rust 1.75+

   **Must Keep async-trait:**
   - Using `dyn Trait` for dynamic dispatch
   - Supporting older Rust versions
   - Object safety required

4. **Convert to Native Async Fn**

   **Before:**
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   pub trait UserRepository: Send + Sync {
       async fn find_user(&self, id: &str) -> Result<User, Error>;
       async fn save_user(&self, user: &User) -> Result<(), Error>;
       async fn delete_user(&self, id: &str) -> Result<(), Error>;
   }

   #[async_trait]
   impl UserRepository for PostgresRepository {
       async fn find_user(&self, id: &str) -> Result<User, Error> {
           sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
               .fetch_one(&self.pool)
               .await
       }

       async fn save_user(&self, user: &User) -> Result<(), Error> {
           sqlx::query!(
               "INSERT INTO users (id, email) VALUES ($1, $2)",
               user.id,
               user.email
           )
           .execute(&self.pool)
           .await?;
           Ok(())
       }

       async fn delete_user(&self, id: &str) -> Result<(), Error> {
           sqlx::query!("DELETE FROM users WHERE id = $1", id)
               .execute(&self.pool)
               .await?;
           Ok(())
       }
   }
   ```

   **After:**
   ```rust
   // No async_trait import needed!

   pub trait UserRepository: Send + Sync {
       async fn find_user(&self, id: &str) -> Result<User, Error>;
       async fn save_user(&self, user: &User) -> Result<(), Error>;
       async fn delete_user(&self, id: &str) -> Result<(), Error>;
   }

   impl UserRepository for PostgresRepository {
       async fn find_user(&self, id: &str) -> Result<User, Error> {
           sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
               .fetch_one(&self.pool)
               .await
       }

       async fn save_user(&self, user: &User) -> Result<(), Error> {
           sqlx::query!(
               "INSERT INTO users (id, email) VALUES ($1, $2)",
               user.id,
               user.email
           )
           .execute(&self.pool)
           .await?;
           Ok(())
       }

       async fn delete_user(&self, id: &str) -> Result<(), Error> {
           sqlx::query!("DELETE FROM users WHERE id = $1", id)
               .execute(&self.pool)
               .await?;
           Ok(())
       }
   }
   ```

5. **Handle Dynamic Dispatch Cases**

   If you need `dyn Trait`, keep async-trait:

   ```rust
   // When you need this:
   let repo: Box<dyn UserRepository> = Box::new(PostgresRepository::new(pool));

   // You MUST use async-trait for object safety:
   use async_trait::async_trait;

   #[async_trait]
   pub trait UserRepository: Send + Sync {
       async fn find_user(&self, id: &str) -> Result<User, Error>;
   }
   ```

   Or redesign to avoid dynamic dispatch:

   ```rust
   // Alternative: Use generics instead
   pub async fn process_users<R: UserRepository>(repo: R) {
       // Works with any UserRepository implementation
       let user = repo.find_user("123").await.unwrap();
   }
   ```

6. **Remove async-trait Dependency**

   Update Cargo.toml:

   **Before:**
   ```toml
   [dependencies]
   async-trait = "0.1"
   ```

   **After:**
   ```toml
   [dependencies]
   # async-trait removed - using native async fn in traits
   ```

   If still needed for some traits:
   ```toml
   [dependencies]
   # Only needed for dyn Trait support
   async-trait = "0.1"  # Optional, only for object-safe traits
   ```

7. **Update Imports**

   Remove unused async-trait imports:

   ```rust
   // Remove this if no longer needed
   use async_trait::async_trait;
   ```

8. **Run Tests**

   Verify everything compiles and works:

   ```bash
   cargo check
   cargo test
   cargo clippy
   ```

9. **Provide Summary**

   ```
   ✅ Modernized async trait usage!

   ## Changes Made:

   ### Converted to Native Async Fn (3 traits):
   - UserRepository (src/domain/user.rs)
   - OrderRepository (src/domain/order.rs)
   - PaymentGateway (src/ports/payment.rs)

   ### Kept async-trait (1 trait):
   - DynamicHandler (src/handlers/mod.rs)
     Reason: Uses Box<dyn Trait> for plugin system

   ## Dependency Updates:
   - Removed async-trait from main dependencies
   - Added as optional for dynamic dispatch cases

   ## Benefits:
   - ✅ Zero-cost abstraction (no boxing overhead)
   - ✅ Simpler code (no macro needed)
   - ✅ Better error messages
   - ✅ Native language feature

   ## Before/After Example:

   Before:
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   trait Repository {
       async fn find(&self, id: &str) -> Result<Item, Error>;
   }

   #[async_trait]
   impl Repository for MyRepo {
       async fn find(&self, id: &str) -> Result<Item, Error> {
           // ...
       }
   }
   ```

   After:
   ```rust
   // No import needed!

   trait Repository {
       async fn find(&self, id: &str) -> Result<Item, Error>;
   }

   impl Repository for MyRepo {
       async fn find(&self, id: &str) -> Result<Item, Error> {
           // ...
       }
   }
   ```

   ## Next Steps:
   1. All tests passing ✅
   2. Consider removing async-trait entirely if unused
   3. Update documentation
   ```

## Key Differences

### Native Async Fn (Rust 1.75+)

**Pros:**
- No external dependency
- Zero-cost abstraction
- Better compiler errors
- Simpler syntax
- Native language feature

**Cons:**
- Cannot use with `dyn Trait` directly
- Requires Rust 1.75+

**Usage:**
```rust
trait MyTrait {
    async fn method(&self) -> Result<T, E>;
}

// Use with generics
fn process<T: MyTrait>(t: T) { }
```

### Async-Trait Crate

**Pros:**
- Works with older Rust
- Supports `dyn Trait`
- Object-safe traits

**Cons:**
- External dependency
- Macro overhead
- Slight performance cost (boxing)

**Usage:**
```rust
use async_trait::async_trait;

#[async_trait]
trait MyTrait {
    async fn method(&self) -> Result<T, E>;
}

// Can use with dyn
let t: Box<dyn MyTrait> = Box::new(impl);
```

## Migration Patterns

### Pattern 1: Simple Repository

```rust
// Before
#[async_trait]
trait Repository {
    async fn get(&self, id: i32) -> Option<Item>;
}

// After
trait Repository {
    async fn get(&self, id: i32) -> Option<Item>;
}
```

### Pattern 2: Generic Service

```rust
// Before
#[async_trait]
trait Service<T> {
    async fn process(&self, item: T) -> Result<(), Error>;
}

// After
trait Service<T> {
    async fn process(&self, item: T) -> Result<(), Error>;
}
```

### Pattern 3: Multiple Async Methods

```rust
// Before
#[async_trait]
trait Complex {
    async fn fetch(&self) -> Result<Data, Error>;
    async fn save(&self, data: Data) -> Result<(), Error>;
    async fn delete(&self, id: i32) -> Result<(), Error>;
}

// After - just remove the macro!
trait Complex {
    async fn fetch(&self) -> Result<Data, Error>;
    async fn save(&self, data: Data) -> Result<(), Error>;
    async fn delete(&self, id: i32) -> Result<(), Error>;
}
```

### Pattern 4: Keep for Dynamic Dispatch

```rust
// When you need this:
struct PluginSystem {
    plugins: Vec<Box<dyn Plugin>>,
}

// Keep async-trait:
#[async_trait]
trait Plugin: Send + Sync {
    async fn execute(&self) -> Result<(), Error>;
}
```

## Important Notes

- Native async fn in traits requires **Rust 1.75+**
- Check MSRV before removing async-trait
- `dyn Trait` requires async-trait (or alternatives)
- Static dispatch (generics) works with native async fn
- Performance is better with native async fn (no boxing)

## Version Requirements

| Feature | Rust Version | Notes |
|---------|-------------|-------|
| Async fn in traits | 1.75.0+ | Native support |
| async-trait crate | Any | Fallback for older versions |
| Return-position impl Trait | 1.75.0+ | Enables async fn |

## After Completion

Ask the user:
1. Did all tests pass?
2. Can we remove async-trait entirely?
3. Are there any dyn Trait use cases remaining?
4. Should we update documentation?
