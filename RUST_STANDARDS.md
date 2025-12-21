# Rust Standards Documentation

This document outlines the Rust development standards and tooling configured for the Cosmic Cults RTS project.

## Edition 2024

All crates use **Rust Edition 2024** as specified in the workspace `Cargo.toml`:

```toml
[workspace.package]
edition = "2024"
rust-version = "1.85"
```

Edition 2024 provides the latest Rust language features and improvements.

## Code Formatting

### rustfmt Configuration

Located at: `rustfmt.toml`

- **Edition**: 2024
- **Max Width**: 100 characters
- **Tab Spaces**: 4
- **Line Ending**: Unix (LF)
- **Import Reordering**: Enabled
- **Module Reordering**: Enabled

Run formatting:
```bash
cargo fmt --all
```

Check formatting without modifying files:
```bash
cargo fmt --all -- --check
```

## Linting

### clippy Configuration

Located at: `clippy.toml`

- **Cognitive Complexity Threshold**: 30
- **Too Many Arguments**: 8
- **Type Complexity**: 500

Run clippy:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Pre-commit Hooks

### Configuration

Located at: `.pre-commit-config.yaml`

Hooks run automatically before each commit:
1. **cargo fmt** - Ensures code is formatted
2. **cargo clippy** - Lints code for issues
3. **cargo check** - Verifies code compiles
4. **trailing-whitespace** - Removes trailing whitespace
5. **end-of-file-fixer** - Ensures files end with newline
6. **check-yaml** - Validates YAML files
7. **check-toml** - Validates TOML files
8. **mixed-line-ending** - Enforces LF line endings

### Installation

```bash
# Install pre-commit
pip install pre-commit

# Install git hooks
pre-commit install

# Run manually on all files
pre-commit run --all-files
```

## Continuous Integration

### Workflow: ci.yml

Located at: `.github/workflows/ci.yml`

Uses a DRY (Don't Repeat Yourself) matrix strategy to run all checks:

- ✅ **Format** - Code formatting check
- ✅ **Clippy** - Linting
- ✅ **Check** - Compilation check
- ✅ **Test** - Run all tests
- ✅ **Doc** - Documentation generation
- ✅ **WASM** - WebAssembly build check

All checks run on:
- Every push to `main`
- Every pull request to `main`

### Environment Variables

```yaml
CARGO_TERM_COLOR: always
RUSTFLAGS: -D warnings
RUSTDOCFLAGS: -D warnings (for Doc job)
```

## Documentation

### Cargo Doc

Generate documentation:
```bash
cargo doc --no-deps --all-features --open
```

Documentation is built as part of CI and fails on warnings.

## Examples

### Location

Examples are located in `game-runner/examples/`:

- `basic_physics.rs` - Physics system demonstration
- `unit_spawning.rs` - Unit spawning demonstration  
- `formations.rs` - Formation system demonstration

### Running Examples

```bash
# Run a specific example
cargo run --example basic_physics

# List all examples
cargo run --example
```

Examples are checked as part of CI (via `--all-targets`).

## WASM Support

### Configuration

- **Target**: `wasm32-unknown-unknown`
- **Build Tool**: Trunk
- **Config**: `Trunk.toml`

### Development

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk

# Serve with hot reload
trunk serve --address 0.0.0.0 --port 8080

# Production build
trunk build --release
```

### Deployment

WASM demo is automatically deployed to GitHub Pages:
- Workflow: `.github/workflows/wasm-demo.yml`
- URL: https://jbcom.github.io/rust-cosmic-cults/

See `WASM_DEMO.md` for more details.

## Development Workflow

### Recommended Workflow

1. **Write Code** - Make your changes
2. **Format** - `cargo fmt --all`
3. **Lint** - `cargo clippy --all-targets --all-features -- -D warnings`
4. **Test** - `cargo test --all`
5. **Commit** - Pre-commit hooks run automatically
6. **Push** - CI runs all checks

### Quick Check

```bash
# Run all checks locally (same as CI)
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo check --all-targets --all-features && \
cargo test --all && \
cargo doc --no-deps --all-features
```

## Tools Summary

| Tool | Purpose | Config File |
|------|---------|-------------|
| rustfmt | Code formatting | `rustfmt.toml` |
| clippy | Linting | `clippy.toml` |
| pre-commit | Git hooks | `.pre-commit-config.yaml` |
| trunk | WASM building | `Trunk.toml` |
| cargo | Package manager | `Cargo.toml` |

## Standards Checklist

- ✅ Rust Edition 2024
- ✅ rustfmt configuration
- ✅ clippy configuration
- ✅ Pre-commit hooks
- ✅ Consolidated CI workflow
- ✅ Documentation generation
- ✅ Examples directory
- ✅ WASM build support
- ✅ GitHub Pages deployment

## References

- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- [rustfmt Documentation](https://rust-lang.github.io/rustfmt/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Pre-commit Framework](https://pre-commit.com/)
- [Trunk Documentation](https://trunkrs.dev/)
