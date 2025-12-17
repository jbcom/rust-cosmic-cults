# Rust Development Environment for jbcom Ecosystem

This directory contains the Cursor cloud agent development environment configuration for Rust projects.

## 🎯 Purpose

Ensures our Rust development environment:
- ✅ Supports Rust Edition 2024 (requires Rust 1.85+)
- ✅ Has all required toolchain components
- ✅ Includes Bevy game engine dependencies
- ✅ Supports WASM compilation
- ✅ Provides secure, efficient development

## 🛠️ Included Tools

### Rust Toolchain
- **Rust 1.85+** - Edition 2024 support
- **rustfmt** - Code formatting
- **clippy** - Linting
- **rust-analyzer** - IDE support
- **wasm32-unknown-unknown** - WASM target

### Cargo Tools
- **cargo-watch** - Auto-rebuild on file changes
- **cargo-edit** - Add/remove/upgrade dependencies
- **cargo-audit** - Security vulnerability scanning
- **cargo-outdated** - Dependency update checker
- **cargo-nextest** - Faster test runner
- **trunk** - WASM application bundler

### System Dependencies
Graphics and audio libraries required for Bevy game engine:
- libudev, libasound2 (input/audio)
- libx11, libxcursor, libxrandr (windowing)
- libwayland, libxkbcommon (Wayland)
- libvulkan, mesa-vulkan-drivers (graphics)

## 🚀 Usage

### Edition 2024 Migration

To upgrade a project to Edition 2024:

```bash
# Update Cargo.toml
edition = "2024"

# Run automatic migration
cargo fix --edition

# Check for issues
cargo clippy
```

### Key Edition 2024 Changes

1. **RPIT (Return Position Impl Trait) lifetime capture** - More precise lifetime inference
2. **Async closures** - Native `async || {}` syntax
3. **`if let` temporary scope changes** - Temporaries drop at end of `if let`
4. **`unsafe extern` blocks** - Explicit unsafe marking required
5. **Reserved syntax** - New reserved tokens for future features

## 📋 Configuration Files

| File | Purpose |
|------|---------|
| `Dockerfile` | Container definition with Rust 1.85+ |
| `environment.json` | Cursor environment configuration |
| `rules/rust.mdc` | Rust coding standards and guidelines |

## 🔄 Compatibility Matrix

| Rust Version | Edition 2021 | Edition 2024 |
|--------------|--------------|--------------|
| 1.82.x       | ✅           | ❌           |
| 1.83.x       | ✅           | ❌           |
| 1.84.x       | ✅           | ❌ (beta)    |
| 1.85+        | ✅           | ✅           |

## 🔍 Troubleshooting

### Edition 2024 Not Available
```
error: the crate `foo` requires edition 2024, but the maximum edition supported by this compiler is 2021
```
**Solution**: Update Rust to 1.85+: `rustup update stable`

### Missing System Dependencies
```
error: could not find system library 'libudev'
```
**Solution**: Install Bevy dependencies (handled automatically in Docker)

### WASM Compilation Fails
```
error: target may not be installed
```
**Solution**: Add WASM target: `rustup target add wasm32-unknown-unknown`

## 📚 References

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [Bevy Setup Guide](https://bevyengine.org/learn/quick-start/getting-started/setup/)
- [Cursor Environment Configuration](https://cursor.com/environment-json-dockerfile.md)
