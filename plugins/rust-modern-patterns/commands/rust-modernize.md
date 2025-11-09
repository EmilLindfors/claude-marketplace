---
description: Analyze and modernize Rust code to use latest features and patterns
---

You are helping modernize Rust code to use the latest features from Rust 2024 Edition.

## Your Task

Analyze Rust code and refactor it to use modern patterns including let chains, async closures, improved match ergonomics, and other Rust 2024 features.

## Steps

1. **Scan for Modernization Opportunities**

   Look for these patterns that can be modernized:
   - Nested if-let statements → let chains
   - Manual async closures → native async closures
   - Old match patterns → improved match ergonomics
   - Regular functions → const functions where possible
   - Manual iterators → gen blocks
   - Complex error propagation → cleaner patterns

2. **Categorize Findings**

   Group by modernization type:
   ```
   Modernization Opportunities Found:

   Let Chains (5):
   - src/user.rs:42 - Nested if-let (3 levels deep)
   - src/api.rs:15 - Multiple Option checks
   ...

   Async Closures (3):
   - src/tasks.rs:28 - Manual async wrapper
   ...

   Const Opportunities (2):
   - src/config.rs:10 - Function could be const
   ...
   ```

3. **Ask User for Scope**

   Ask which modernizations to apply:
   - All recommended changes?
   - Specific category (let chains, async, etc.)?
   - Specific file or function?

4. **Refactor Pattern: Nested If-Let → Let Chains**

   **Before:**
   ```rust
   if let Some(user) = get_user(id) {
       if let Some(profile) = user.profile {
           if profile.is_active {
               if let Some(email) = profile.email {
                   send_email(&email);
               }
           }
       }
   }
   ```

   **After:**
   ```rust
   if let Some(user) = get_user(id)
       && let Some(profile) = user.profile
       && profile.is_active
       && let Some(email) = profile.email
   {
       send_email(&email);
   }
   ```

5. **Refactor Pattern: Manual Async → Async Closures**

   **Before:**
   ```rust
   let futures: Vec<_> = items
       .iter()
       .map(|item| {
           let item = item.clone();
           async move {
               process_item(item).await
           }
       })
       .collect();
   ```

   **After:**
   ```rust
   let futures: Vec<_> = items
       .iter()
       .map(async |item| {
           process_item(item).await
       })
       .collect();
   ```

6. **Refactor Pattern: Functions → Const Functions**

   **Before:**
   ```rust
   fn calculate_buffer_size(items: usize) -> usize {
       items * std::mem::size_of::<Item>()
   }

   static BUFFER: [u8; calculate_buffer_size(100)]; // Error!
   ```

   **After:**
   ```rust
   const fn calculate_buffer_size(items: usize) -> usize {
       items * std::mem::size_of::<Item>()
   }

   const BUFFER_SIZE: usize = calculate_buffer_size(100);
   static BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
   ```

7. **Refactor Pattern: Manual Iterator → Gen Block**

   **Before:**
   ```rust
   struct FibIterator {
       a: u64,
       b: u64,
   }

   impl Iterator for FibIterator {
       type Item = u64;

       fn next(&mut self) -> Option<Self::Item> {
           let current = self.a;
           self.a = self.b;
           self.b = current + self.b;
           Some(current)
       }
   }

   fn fibonacci() -> FibIterator {
       FibIterator { a: 0, b: 1 }
   }
   ```

   **After:**
   ```rust
   fn fibonacci() -> impl Iterator<Item = u64> {
       gen {
           let (mut a, mut b) = (0, 1);
           loop {
               yield a;
               (a, b) = (b, a + b);
           }
       }
   }
   ```

8. **Refactor Pattern: Match Ergonomics**

   **Before (Rust 2021):**
   ```rust
   match &option {
       Some(x) => {
           // x is &T, need to clone or handle reference
           process(x.clone());
       }
       None => {}
   }
   ```

   **After (Rust 2024):**
   ```rust
   match &option {
       Some(mut x) => {
           // x is &mut T (not moved), can modify in place
           x.update();
       }
       None => {}
   }
   ```

9. **Check Edition and Version**

   Ensure project supports modern features:

   ```toml
   [package]
   edition = "2024"
   rust-version = "1.85"  # For full 2024 support
   ```

   If edition needs updating:
   - Ask user if they want to upgrade
   - Run `cargo fix --edition` after updating
   - Check for breaking changes

10. **Provide Modernization Summary**

    ```
    ✅ Code modernized successfully!

    ## Changes Made:

    ### Let Chains (5 locations):
    - src/user.rs:42 - Flattened 3-level nesting
    - src/api.rs:15 - Combined Option checks
    - src/validate.rs:88 - Simplified Result handling

    ### Async Closures (3 locations):
    - src/tasks.rs:28 - Replaced manual async wrapper
    - src/jobs.rs:45 - Simplified async map
    - src/handlers.rs:102 - Cleaner async callback

    ### Const Functions (2 locations):
    - src/config.rs:10 - Made calculate_size const
    - src/utils.rs:25 - Made hash_string const

    ### Gen Blocks (1 location):
    - src/iter.rs:15 - Simplified iterator with gen block

    ## Edition Status:
    - Current: edition = "2024" ✅
    - MSRV: rust-version = "1.85" ✅

    ## Before/After Example:

    Before:
    ```rust
    if let Some(user) = get_user() {
        if let Some(email) = user.email {
            if email.contains('@') {
                send_email(&email);
            }
        }
    }
    ```

    After:
    ```rust
    if let Some(user) = get_user()
        && let Some(email) = user.email
        && email.contains('@')
    {
        send_email(&email);
    }
    ```

    ## Next Steps:
    1. Run tests: `cargo test`
    2. Check for warnings: `cargo check`
    3. Review changes for correctness
    4. Consider enabling more lints for modern patterns

    ## Suggested Lints:
    Add to Cargo.toml or lib.rs:
    ```rust
    #![warn(rust_2024_compatibility)]
    #![warn(let_underscore_drop)]
    ```
    ```

## Modernization Patterns

### Let Chains

Look for:
- Multiple nested if-let
- if-let followed by if condition
- while-let with additional conditions

Convert to:
```rust
if let Pattern1 = expr1
    && let Pattern2 = expr2
    && boolean_condition
{
    // body
}
```

### Async Closures

Look for:
- `.map(|x| { let x = x.clone(); async move { ... } })`
- Manual future wrapping
- Complex async callback patterns

Convert to:
```rust
.map(async |x| { ... })
```

### Const Functions

Look for:
- Functions with only const-safe operations
- Compile-time computations
- Functions used in const contexts

Convert to:
```rust
const fn function_name(...) -> ReturnType {
    // const-safe operations only
}
```

### Gen Blocks

Look for:
- Manual Iterator implementations
- State machines for iteration
- Complex iteration logic

Convert to:
```rust
gen {
    // yield values
}
```

## Important Notes

- Only apply changes if edition = "2024" or offer to upgrade
- Test thoroughly after modernization
- Some patterns require minimum Rust versions
- Preserve behavior - modernization should not change logic
- Document breaking changes if any

## Version Requirements

| Feature | Min Rust Version | Edition |
|---------|-----------------|---------|
| Let chains | 1.88.0 | 2024 |
| Async closures | 1.85.0 | 2024 |
| Gen blocks | 1.85.0 | 2024 |
| Match ergonomics | 1.85.0 | 2024 |

## After Completion

Ask the user:
1. Did all tests pass?
2. Are there more files to modernize?
3. Should we enable additional lints?
4. Do you want to update documentation?
