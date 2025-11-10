---
description: Upgrade Rust project to Rust 2024 Edition
---

You are helping upgrade a Rust project to Rust 2024 Edition.

## Your Task

Guide the user through upgrading their project from an older edition (2015, 2018, or 2021) to Rust 2024 Edition, handling breaking changes and migration steps.

## Steps

1. **Check Current Status**

   Read Cargo.toml to determine:
   - Current edition
   - Current rust-version (MSRV) if set
   - Project structure (workspace or single crate)

   ```rust
   Current Status:
   - Edition: 2021
   - MSRV: Not set
   - Type: Single crate
   ```

2. **Verify Rust Version**

   Check that Rust toolchain is recent enough:

   ```bash
   rustc --version
   ```

   Required: Rust 1.85.0 or later for full Rust 2024 support.

   If version is too old:
   ```
   ⚠️ Rust version is too old. Rust 2024 Edition requires 1.85.0 or later.

   Please update:
   ```bash
   rustup update stable
   ```
   ```

3. **Create Backup**

   Suggest creating a git commit or backup:
   ```
   💡 Recommendation: Commit current changes before upgrading

   ```bash
   git add .
   git commit -m "Pre-2024 edition upgrade snapshot"
   ```
   ```

4. **Update Cargo.toml**

   For single crate:
   ```toml
   [package]
   name = "my-project"
   version = "0.1.0"
   edition = "2024"  # Updated from 2021
   rust-version = "1.85"  # Add MSRV
   ```

   For workspace:
   ```toml
   [workspace]
   members = ["crate1", "crate2"]

   [workspace.package]
   edition = "2024"
   rust-version = "1.85"

   # Then each crate can inherit:
   [package]
   name = "crate1"
   version = "0.1.0"
   edition.workspace = true
   rust-version.workspace = true
   ```

5. **Run cargo fix**

   Automatically fix edition-related issues:

   ```bash
   cargo fix --edition
   ```

   This will:
   - Fix deprecated patterns
   - Update syntax where needed
   - Add compatibility shims

6. **Check for Warnings**

   Review compiler warnings:

   ```bash
   cargo check --all-targets
   ```

   Common warnings:
   - Match ergonomics changes
   - Binding mode changes
   - Reserved syntax warnings

7. **Update Match Patterns (Rust 2024)**

   **Breaking Change:** mut binding behavior changed

   Before (2021):
   ```rust
   match &option {
       Some(mut x) => {
           // x is T (moved)
       }
       None => {}
   }
   ```

   After (2024):
   ```rust
   match &option {
       Some(mut x) => {
           // x is &mut T (not moved)
       }
       None => {}
   }
   ```

   If you need the old behavior:
   ```rust
   match option {  // Match on value, not reference
       Some(mut x) => {
           // x is T (moved)
       }
       None => {}
   }
   ```

8. **Update Reserved Patterns**

   Rust 2024 reserves some pattern combinations for future use:

   ```rust
   // ❌ Not allowed in 2024 (mixed ergonomics)
   match value {
       Some(ref x) => {}  // Error if not fully explicit
       _ => {}
   }

   // ✅ Allowed (fully explicit)
   match value {
       &Some(ref x) => {}
       _ => {}
   }

   // ✅ Or use ergonomics fully
   match &value {
       Some(x) => {}  // x is &T
       _ => {}
   }
   ```

9. **Run Tests**

   Verify everything still works:

   ```bash
   cargo test --all-targets
   ```

   If tests fail:
   - Check for match pattern changes
   - Look for binding mode issues
   - Review compiler errors carefully

10. **Update Dependencies**

    Check if dependencies support Rust 2024:

    ```bash
    cargo update
    cargo check
    ```

    The MSRV-aware resolver (Rust 1.84+) will automatically select compatible versions based on your rust-version.

11. **Enable Modern Features**

    Now you can use Rust 2024 features:

    ```rust
    // Let chains
    if let Some(x) = opt1
        && let Some(y) = opt2
    {
        // ...
    }

    // Async closures
    items.iter().map(async |item| {
        process(item).await
    })

    // Gen blocks
    let iter = gen {
        yield 1;
        yield 2;
    };
    ```

12. **Update Lints**

    Add modern lints to catch issues:

    ```rust
    // In lib.rs or main.rs
    #![warn(rust_2024_compatibility)]
    #![warn(let_underscore_drop)]
    #![warn(unused_qualifications)]
    ```

13. **Provide Upgrade Summary**

    ```
    ✅ Successfully upgraded to Rust 2024 Edition!

    ## Changes Made:

    ### Cargo.toml Updates:
    - Edition: 2021 → 2024
    - Added rust-version = "1.85"

    ### Code Changes:
    - Applied automatic fixes via cargo fix
    - Updated 3 match patterns for new ergonomics
    - Fixed 2 reserved pattern warnings

    ### Tests:
    - All tests passing ✅

    ## New Features Available:

    1. **Let Chains** - Flatten nested if-let
    2. **Async Closures** - Native async || {} syntax
    3. **Gen Blocks** - Simplified iterators
    4. **Improved Match Ergonomics** - Clearer semantics
    5. **MSRV-Aware Resolver** - Automatic compatible versions

    ## Next Steps:

    1. Use `/rust-modernize` to apply modern patterns
    2. Review new edition guide: https://doc.rust-lang.org/edition-guide/rust-2024/
    3. Update CI/CD to use Rust 1.85+
    4. Consider modernizing code patterns

    ## Migration Guide:
    See the [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/) for details.
    ```

## Breaking Changes Checklist

When upgrading to Rust 2024, be aware of:

- [ ] **Match ergonomics** - mut bindings work differently
- [ ] **Reserved patterns** - Some patterns reserved for future
- [ ] **Temporary scopes in let chains** - Different drop order
- [ ] **Impl trait captures** - More lifetime capture rules

## Workspace Upgrade

For workspaces with multiple crates:

1. **Update workspace root**:
   ```toml
   [workspace.package]
   edition = "2024"
   rust-version = "1.85"
   ```

2. **Update each crate**:
   ```toml
   [package]
   edition.workspace = true
   rust-version.workspace = true
   ```

3. **Run cargo fix for each crate**:
   ```bash
   cargo fix --edition --workspace
   ```

## Troubleshooting

### cargo fix fails

If `cargo fix --edition` fails:
1. Fix compilation errors first: `cargo check`
2. Resolve dependency issues
3. Try fixing one crate at a time
4. Check for proc-macro compatibility

### Tests fail after upgrade

Common issues:
1. **Match pattern changes** - Check mut bindings
2. **Drop order changes** - Let chains have different scoping
3. **Lifetime changes** - Impl trait captures more lifetimes

### Dependencies incompatible

If dependencies don't support Rust 2024:
1. Check for updates: `cargo update`
2. MSRV resolver should pick compatible versions
3. File issues with dependency maintainers
4. Consider alternatives if critical

## Version Requirements

- **Minimum Rust:** 1.85.0 for full Rust 2024 Edition
- **Let chains:** 1.88.0
- **MSRV resolver:** 1.84.0 (recommended before upgrade)

## Edition Comparison

| Feature | 2021 | 2024 |
|---------|------|------|
| Let chains | ❌ | ✅ |
| Async closures | ❌ | ✅ |
| Gen blocks | ❌ | ✅ |
| Match ergonomics | Old | Improved |
| MSRV resolver | ❌ | ✅ |

## After Completion

Ask the user:
1. Did the upgrade complete successfully?
2. Are all tests passing?
3. Should we modernize the code to use new features?
4. Do you want to update CI/CD configuration?
