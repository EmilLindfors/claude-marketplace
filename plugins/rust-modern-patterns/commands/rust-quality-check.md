---
description: Run comprehensive quality checks using modern Rust tooling (fmt, clippy, nextest, audit, deny)
---

Run a comprehensive quality check suite on the current Rust project using modern tooling best practices.

## What This Command Does

This command runs a complete quality assurance suite including:

1. **Code Formatting** - Verify code follows standard formatting
2. **Linting** - Run clippy with strict settings
3. **Testing** - Execute tests with cargo-nextest
4. **Security Audit** - Check for known vulnerabilities
5. **Dependency Checks** - Validate licenses and sources (if configured)
6. **SemVer Check** - Verify API compatibility (for libraries)

## Process

### 1. Check Project Structure

First, verify this is a Rust project:
- Look for `Cargo.toml` in current directory
- Determine if this is a library or binary (affects checks)
- Check for existing configurations (deny.toml, clippy.toml, etc.)

### 2. Run Quality Checks

Execute checks in this order:

#### Format Check
```bash
cargo fmt --all -- --check
```
- Verifies code follows rustfmt standards
- **Fails if**: Code is not formatted
- **Fix**: Run `cargo fmt --all`

#### Clippy Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
- Runs comprehensive linting
- **Fails if**: Any clippy warnings exist
- **Fix**: Address warnings or use `#[allow(...)]` with justification

#### Test Suite
```bash
# Check if nextest is available
if command -v cargo-nextest &> /dev/null; then
    cargo nextest run --all-features
    cargo test --doc  # nextest doesn't run doctests
else
    cargo test --all-features
fi
```
- Runs all tests
- **Fails if**: Any test fails
- **Fix**: Debug and fix failing tests

#### Security Audit
```bash
# Check if cargo-audit is available
if command -v cargo-audit &> /dev/null; then
    cargo audit
else
    echo "⚠️ cargo-audit not installed. Run: cargo install cargo-audit"
fi
```
- Checks dependencies against RustSec database
- **Fails if**: Known vulnerabilities found
- **Fix**: Update dependencies or review advisories

#### Dependency Validation (Optional)
```bash
# Only if deny.toml exists
if [ -f "deny.toml" ]; then
    if command -v cargo-deny &> /dev/null; then
        cargo deny check
    else
        echo "⚠️ deny.toml found but cargo-deny not installed"
        echo "    Run: cargo install cargo-deny"
    fi
fi
```
- Checks licenses, sources, bans, and advisories
- **Fails if**: Policy violations found
- **Fix**: Update dependencies or adjust policy

#### SemVer Check (Libraries Only)
```bash
# Check if this is a library and cargo-semver-checks is available
if grep -q "\\[lib\\]" Cargo.toml; then
    if command -v cargo-semver-checks &> /dev/null; then
        cargo semver-checks check-release
    else
        echo "📚 Library detected. Consider installing cargo-semver-checks"
        echo "    Run: cargo install cargo-semver-checks"
    fi
fi
```
- Verifies API changes follow semantic versioning
- **Fails if**: Breaking changes in non-major version
- **Fix**: Bump version appropriately or fix API

### 3. Report Results

Provide a summary of all checks:

```
✅ Rust Quality Check Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Format Check      - Passed
✅ Clippy Linting    - Passed
✅ Test Suite        - Passed (42 tests)
✅ Security Audit    - Passed (no vulnerabilities)
✅ Dependency Check  - Passed
✅ SemVer Check      - Passed

All checks passed! 🎉
```

Or if issues found:

```
❌ Rust Quality Check Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ Format Check      - FAILED
   Run: cargo fmt --all

✅ Clippy Linting    - Passed
❌ Test Suite        - FAILED (2 tests failed)
⚠️  Security Audit    - WARNINGS (1 vulnerability)
   Update: tokio 1.25 -> 1.26 (RUSTSEC-2023-0001)

✅ Dependency Check  - Passed

Fix these issues before committing.
```

## Tool Installation Guide

If tools are missing, provide installation instructions:

```bash
# Essential tools for quality checks
cargo install cargo-nextest     # Faster test runner
cargo install cargo-audit       # Security scanning
cargo install cargo-deny        # Dependency validation
cargo install cargo-semver-checks # API compatibility

# Optional but recommended
cargo install bacon             # Continuous feedback
cargo install flamegraph        # Performance profiling
```

## Configuration Recommendations

### Create clippy.toml

If `clippy.toml` doesn't exist, suggest creating one:

```toml
# clippy.toml - Clippy configuration
cognitive-complexity-threshold = 30
single-char-binding-names-threshold = 5
too-many-arguments-threshold = 7
```

### Create deny.toml

If `deny.toml` doesn't exist for a project with dependencies, suggest:

```bash
cargo deny init
```

Then review and adjust the generated configuration.

### Update Cargo.toml

Suggest adding these to project Cargo.toml:

```toml
[package]
edition = "2024"  # Use latest edition
rust-version = "1.85"  # Set MSRV

[profile.release]
debug = true  # For profiling

[profile.dev]
# Enable some optimizations for faster dev builds
opt-level = 1
```

## CI/CD Integration

Provide a GitHub Actions workflow snippet:

```yaml
name: Quality Checks

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install tools
        run: |
          cargo install cargo-nextest
          cargo install cargo-audit
          cargo install cargo-deny

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Tests
        run: |
          cargo nextest run --all-features
          cargo test --doc

      - name: Security audit
        run: cargo audit

      - name: Dependency check
        run: cargo deny check
```

## Best Practices

When running quality checks:

1. **Run locally before pushing** - Catch issues early
2. **Fix formatting first** - Easiest to resolve
3. **Address clippy warnings** - They often catch real bugs
4. **Don't skip tests** - Even if they're slow
5. **Review security advisories** - Don't just update blindly
6. **Keep tools updated** - `cargo install --force <tool>`
7. **Configure in CI** - Enforce quality automatically

## Troubleshooting

### "cargo-nextest not found"
```bash
cargo install cargo-nextest
```

### "cargo-audit not found"
```bash
cargo install cargo-audit
```

### Clippy warnings overwhelming
```bash
# Fix incrementally
cargo clippy --fix --allow-dirty --allow-staged
```

### Tests fail on CI but pass locally
- Check for race conditions
- Ensure deterministic behavior
- Use cargo-nextest's flaky test detection

### Security vulnerabilities can't be fixed
- Check if patched versions exist
- Review the advisory details
- Consider alternatives if no fix available
- Document accepted risks

## Output Format

Provide structured output:

```
🔍 Running Rust Quality Checks...

[1/6] Format Check...
  ✅ Code is properly formatted

[2/6] Clippy Linting...
  ✅ No warnings found

[3/6] Test Suite...
  Running 42 tests...
  ✅ All tests passed (42/42)

[4/6] Security Audit...
  Scanning 187 dependencies...
  ✅ No vulnerabilities found

[5/6] Dependency Check...
  ✅ All licenses approved
  ✅ All sources verified

[6/6] SemVer Check...
  ✅ No breaking changes detected

━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ All Quality Checks Passed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ready to commit! 🚀
```

## Your Task

Execute the comprehensive quality check suite:

1. Verify project structure
2. Check for required tools
3. Run all available checks
4. Provide clear summary
5. Suggest fixes for failures
6. Recommend tool installations if needed
7. Offer configuration improvements

Make the output clear, actionable, and encouraging!
