---
description: Build Rust Lambda function for AWS deployment with optimizations
---

You are helping the user build their Rust Lambda function for AWS deployment.

## Your Task

Guide the user through building their Lambda function with appropriate optimizations:

1. **Verify project setup**:
   - Check that Cargo.toml has release profile optimizations
   - Verify lambda_runtime dependency is present
   - Confirm project compiles: `cargo check`

2. **Choose architecture**:
   Ask the user which architecture to target:
   - **x86_64** (default): Compatible with most existing infrastructure
   - **ARM64** (Graviton2): 20% better price/performance, often faster cold starts
   - **Both**: Build for both architectures

3. **Build command**:

   **For x86_64**:
   ```bash
   cargo lambda build --release
   ```

   **For ARM64** (recommended):
   ```bash
   cargo lambda build --release --arm64
   ```

   **For both**:
   ```bash
   cargo lambda build --release
   cargo lambda build --release --arm64
   ```

   **With zip output** (for manual deployment):
   ```bash
   cargo lambda build --release --output-format zip
   ```

4. **Verify build**:
   - Check binary size: `ls -lh target/lambda/*/bootstrap`
   - Typical sizes:
     - Small function: 1-3 MB
     - With AWS SDK: 5-10 MB
     - Large dependencies: 10-20 MB
   - If too large, suggest optimizations (see below)

5. **Build output location**:
   - x86_64: `target/lambda/<function-name>/bootstrap`
   - ARM64: `target/lambda/<function-name>/bootstrap` (when building with --arm64)
   - Zip: `target/lambda/<function-name>.zip`

## Release Profile Optimization

Ensure Cargo.toml has optimal release profile:

```toml
[profile.release]
opt-level = 'z'     # Optimize for size (or 3 for speed)
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization (slower compile)
strip = true        # Remove debug symbols
panic = 'abort'     # Smaller panic handling
```

### Optimization Tradeoffs

**For smaller binary (faster cold start)**:
```toml
opt-level = 'z'
```

**For faster execution**:
```toml
opt-level = 3
```

## Size Optimization Tips

If the binary is too large:

1. **Check dependencies**:
   ```bash
   cargo tree
   ```
   Look for unnecessary or duplicate dependencies

2. **Use feature flags**:
   ```toml
   # Only enable needed features
   tokio = { version = "1", features = ["macros", "rt"] }
   # Instead of:
   # tokio = { version = "1", features = ["full"] }
   ```

3. **Audit with cargo-bloat**:
   ```bash
   cargo install cargo-bloat
   cargo bloat --release -n 20
   ```

4. **Consider lighter alternatives**:
   - Use `ureq` instead of `reqwest` for simple HTTP
   - Use `rustls` instead of `native-tls`
   - Minimize AWS SDK crates

5. **Remove unused code**:
   - Ensure `strip = true` in profile
   - Use `cargo-unused-features` to find unused features

## Cross-Compilation Requirements

cargo-lambda uses Zig for cross-compilation. If you encounter issues:

1. **Install Zig**:
   ```bash
   # macOS
   brew install zig

   # Linux (download from ziglang.org)
   wget https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz
   tar xf zig-linux-x86_64-0.11.0.tar.xz
   export PATH=$PATH:$PWD/zig-linux-x86_64-0.11.0
   ```

2. **Verify Zig**:
   ```bash
   zig version
   ```

## Build Flags

Additional useful flags:

```bash
# Build specific binary in workspace
cargo lambda build --release --bin my-function

# Build all binaries
cargo lambda build --release --all

# Build with compiler flags
cargo lambda build --release -- -C target-cpu=native

# Verbose output
cargo lambda build --release --verbose
```

## Testing the Build

After building, test locally:

```bash
# Start local Lambda runtime
cargo lambda watch

# In another terminal, invoke the function
cargo lambda invoke --data-ascii '{"test": "data"}'
```

## Build Performance

Speed up builds:

1. **Use sccache**:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

2. **Parallel compilation** (already enabled by default)

3. **Incremental compilation** (for development):
   ```toml
   [profile.dev]
   incremental = true
   ```

## Architecture Decision Guide

**Choose x86_64 when**:
- Need compatibility with existing x86 infrastructure
- Using dependencies that don't support ARM64
- Already have x86 configuration/scripts

**Choose ARM64 when**:
- Want better price/performance (20% savings)
- Need faster execution
- Want potentially faster cold starts
- Starting fresh project (recommended)

**Build both when**:
- Want to test both architectures
- Supporting multiple deployment targets
- Migrating from x86 to ARM

## Common Build Issues

### Issue: "Zig not found"
**Solution**: Install Zig (see above)

### Issue: "Cannot find -lssl"
**Solution**: Install OpenSSL development files
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl
```

### Issue: "Binary too large" (>50MB uncompressed)
**Solution**:
- Review dependencies with `cargo tree`
- Enable all size optimizations
- Consider splitting into multiple functions

### Issue: Build succeeds but Lambda fails
**Solution**:
- Ensure building for correct architecture
- Test locally with `cargo lambda watch`
- Check CloudWatch logs for specific errors

## Next Steps

After successful build:
1. Test locally: `/lambda-invoke` or `cargo lambda watch`
2. Deploy: Use `/lambda-deploy`
3. Set up CI/CD: Use `/lambda-github-actions`

Report the build results including:
- Binary size
- Architecture
- Build time
- Any warnings or suggestions
