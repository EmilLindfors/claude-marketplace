---
description: Set up modern Rust development tooling (bacon, nextest, audit, deny, clippy config)
---

Set up a modern Rust development environment with best-in-class tooling for 2025.

## What This Command Does

This command configures your Rust project with:

1. **Essential Development Tools** - Install bacon, cargo-nextest, etc.
2. **Quality Assurance Tools** - Set up clippy, rustfmt, cargo-audit
3. **Security Tools** - Configure cargo-audit and cargo-deny
4. **Configuration Files** - Create clippy.toml, deny.toml, rustfmt.toml
5. **CI/CD Template** - Provide GitHub Actions workflow
6. **Documentation** - Add tool usage guide to project

## Process

### 1. Assess Current Setup

Check what's already installed:

```bash
# Check Rust version
rustc --version
rustup --version

# Check for existing tools
cargo-nextest --version 2>/dev/null
cargo-audit --version 2>/dev/null
cargo-deny --version 2>/dev/null
bacon --version 2>/dev/null
flamegraph --version 2>/dev/null
cargo-semver-checks --version 2>/dev/null
```

### 2. Install Essential Tools

Install missing tools with user confirmation:

```bash
echo "Installing modern Rust tooling..."
echo ""
echo "Essential tools:"
echo "  • bacon           - Background compiler"
echo "  • cargo-nextest   - Fast test runner"
echo "  • cargo-audit     - Security scanner"
echo "  • cargo-deny      - Dependency validator"
echo ""
echo "Optional tools:"
echo "  • flamegraph      - Performance profiler"
echo "  • cargo-semver-checks - API compatibility"
echo "  • cargo-machete   - Unused dependency finder"
echo ""

# Install essentials
cargo install bacon
cargo install cargo-nextest
cargo install cargo-audit
cargo install cargo-deny

# Optionally install others
# cargo install flamegraph
# cargo install cargo-semver-checks
# cargo install cargo-machete
```

### 3. Create Configuration Files

#### clippy.toml

Create `clippy.toml` with sensible defaults:

```toml
# clippy.toml - Clippy linter configuration
# See: https://doc.rust-lang.org/clippy/

# Complexity thresholds
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 7
too-many-lines-threshold = 150
large-error-threshold = 128

# Naming conventions
single-char-binding-names-threshold = 5

# Documentation
missing-docs-in-private-items = false

# Allow some pedantic lints that are too noisy
# Uncomment to allow:
# doc-markdown = "allow"
# module-name-repetitions = "allow"
# missing-errors-doc = "allow"
```

#### rustfmt.toml

Create `rustfmt.toml` for consistent formatting:

```toml
# rustfmt.toml - Rustfmt configuration
# See: https://rust-lang.github.io/rustfmt/

edition = "2024"

# Line length
max_width = 100
hard_tabs = false
tab_spaces = 4

# Imports
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true

# Comments and docs
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true

# Misc
use_field_init_shorthand = true
use_try_shorthand = true
```

#### deny.toml

Initialize cargo-deny configuration:

```bash
cargo deny init
```

Then customize the generated `deny.toml`:

```toml
# deny.toml - Cargo-deny configuration

[advisories]
vulnerability = "deny"
unmaintained = "warn"
unsound = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
# Adjust allowed licenses for your needs
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "BSD-2-Clause",
    "ISC",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "warn"
highlight = "all"

# Ban known problematic crates (customize as needed)
deny = [
    # Example: { name = "openssl", use-instead = "rustls" },
]

[sources]
unknown-registry = "deny"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

#### .cargo/config.toml

Create `.cargo/config.toml` for local development settings:

```toml
# .cargo/config.toml - Cargo configuration

[alias]
# Convenient aliases
check-all = "check --all-targets --all-features"
test-all = "nextest run --all-features"
lint = "clippy --all-targets --all-features -- -D warnings"
quality = "run --bin rust-quality-check"

[build]
# Increase parallel compilation
jobs = 8  # Adjust based on CPU cores

[term]
# Better progress bars
progress.when = "auto"
progress.width = 80
```

### 4. Update Cargo.toml

Suggest updates to the project's `Cargo.toml`:

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"  # Use latest edition
rust-version = "1.85"  # Set minimum Rust version (MSRV)

# Add lints
[lints.rust]
unsafe_code = "forbid"  # Adjust as needed
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Allow some pedantic lints that are too noisy
module_name_repetitions = "allow"
missing_errors_doc = "allow"

[profile.dev]
# Faster iterative compilation
opt-level = 1

[profile.release]
# Enable debug symbols for profiling
debug = true
lto = true
codegen-units = 1
```

### 5. Create Development Scripts

#### scripts/quality.sh

Create a pre-commit script:

```bash
#!/bin/bash
# scripts/quality.sh - Run quality checks

set -e

echo "🔍 Running quality checks..."
echo ""

echo "📝 Formatting..."
cargo fmt --all -- --check

echo "✨ Linting..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Testing..."
if command -v cargo-nextest &> /dev/null; then
    cargo nextest run --all-features
    cargo test --doc
else
    cargo test --all-features
fi

echo "🔒 Security audit..."
cargo audit

echo "📦 Dependency check..."
if [ -f "deny.toml" ]; then
    cargo deny check
fi

echo ""
echo "✅ All checks passed!"
```

Make it executable:
```bash
chmod +x scripts/quality.sh
```

#### scripts/dev.sh

Create a development startup script:

```bash
#!/bin/bash
# scripts/dev.sh - Start development environment

echo "🚀 Starting Rust development environment..."
echo ""

# Start bacon in background
echo "Starting bacon clippy..."
bacon clippy &
BACON_PID=$!

# Trap Ctrl+C to clean up
trap "echo ''; echo 'Shutting down...'; kill $BACON_PID 2>/dev/null; exit" INT TERM

echo ""
echo "✅ Development environment ready!"
echo ""
echo "  📝 Bacon is running clippy in the background"
echo "  🔧 Make changes and see feedback automatically"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Keep script running
wait $BACON_PID
```

Make it executable:
```bash
chmod +x scripts/dev.sh
```

### 6. Create CI/CD Workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Check cargo.toml
        run: cargo check --all-features

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Run tests
        run: cargo nextest run --all-features

      - name: Run doctests
        run: cargo test --doc

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Check dependencies
        run: cargo deny check

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out xml --all-features

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
```

### 7. Create Documentation

Create `DEVELOPMENT.md`:

```markdown
# Development Guide

## Getting Started

### Prerequisites

- Rust 1.85 or later
- Modern Rust tooling (see setup below)

### Setup

Install development tools:

\`\`\`bash
./scripts/setup-tooling.sh  # Or manually install tools
\`\`\`

### Development Workflow

Start the development environment:

\`\`\`bash
./scripts/dev.sh
\`\`\`

This starts bacon for continuous feedback. Make changes and see linting results automatically.

### Before Committing

Run quality checks:

\`\`\`bash
./scripts/quality.sh
# Or use the alias:
cargo quality
\`\`\`

This runs:
- Code formatting check
- Clippy linting
- All tests
- Security audit
- Dependency validation

## Tools

### bacon
Continuous background compilation and linting.

\`\`\`bash
bacon clippy  # Run clippy continuously
bacon test    # Run tests continuously
\`\`\`

### cargo-nextest
Faster test runner with better output.

\`\`\`bash
cargo nextest run           # Run all tests
cargo nextest run test_name # Run specific test
\`\`\`

### cargo-audit
Security vulnerability scanner.

\`\`\`bash
cargo audit        # Check for vulnerabilities
cargo audit fix    # Update dependencies
\`\`\`

### cargo-deny
Dependency validator for licenses, sources, and security.

\`\`\`bash
cargo deny check            # Check all policies
cargo deny check licenses   # Check licenses only
\`\`\`

### flamegraph
Performance profiler.

\`\`\`bash
cargo flamegraph --bin myapp
\`\`\`

## Configuration

- `clippy.toml` - Clippy linting rules
- `rustfmt.toml` - Code formatting rules
- `deny.toml` - Dependency policies
- `.cargo/config.toml` - Cargo aliases and settings

## CI/CD

All checks run automatically in CI:
- Format checking
- Clippy linting
- Test suite
- Security audit
- Dependency validation

See `.github/workflows/ci.yml` for details.
\`\`\`

### 8. Provide Setup Summary

After completion, show summary:

```
✅ Rust Development Tooling Setup Complete!

Installed Tools:
  ✅ bacon            - Background compiler
  ✅ cargo-nextest    - Fast test runner
  ✅ cargo-audit      - Security scanner
  ✅ cargo-deny       - Dependency validator

Created Configurations:
  ✅ clippy.toml      - Linting rules
  ✅ rustfmt.toml     - Formatting rules
  ✅ deny.toml        - Dependency policies
  ✅ .cargo/config.toml - Cargo settings

Created Scripts:
  ✅ scripts/quality.sh - Pre-commit checks
  ✅ scripts/dev.sh     - Development environment

Created Workflows:
  ✅ .github/workflows/ci.yml - CI pipeline

Created Documentation:
  ✅ DEVELOPMENT.md   - Developer guide

Next Steps:
  1. Review and adjust configurations to your needs
  2. Run: ./scripts/dev.sh
  3. Make changes and see instant feedback
  4. Before committing: ./scripts/quality.sh

Happy coding! 🦀✨
```

## Tool Descriptions

Explain each tool's purpose:

- **bacon**: Watches files and runs cargo commands, showing minimal, actionable output
- **cargo-nextest**: Runs tests in parallel with better reporting, 60% faster than cargo test
- **cargo-audit**: Scans dependencies for security vulnerabilities from RustSec database
- **cargo-deny**: Validates licenses, sources, and checks for banned/duplicated dependencies
- **cargo-semver-checks**: Ensures API changes follow semantic versioning (for libraries)
- **flamegraph**: Generates flamegraphs for performance profiling

## Your Task

Set up modern Rust development tooling:

1. Check current tool installation
2. Install missing essential tools
3. Create configuration files
4. Set up development scripts
5. Create CI/CD workflow
6. Generate documentation
7. Provide clear next steps

Make the setup smooth and explain what each tool does!
