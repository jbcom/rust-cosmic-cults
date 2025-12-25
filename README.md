# Cosmic Cults (Rust)

[![CI](https://github.com/jbcom/rust-cosmic-cults/workflows/CI/badge.svg)](https://github.com/jbcom/rust-cosmic-cults/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A Lovecraftian 4X real-time strategy game built with the Bevy game engine, targeting WebGL/WASM for browser-based gameplay.

## Features

- 🎮 **3D RTS Mechanics**: Real-time strategy with proper 3D rendering
- 👁️ **Cult Management**: Control one of three distinct cults
- 🧠 **Advanced AI**: Behavior trees, utility AI, state machines
- ⚔️ **Combat Systems**: Damage, effects, XP progression
- 🌫️ **Fog of War**: Exploration and visibility mechanics
- 🌐 **Web-Native**: WASM compilation for browser play

## Crate Structure

| Crate | Description | Status |
|-------|-------------|--------|
| `bevy-ai-toolkit` | Generic AI systems: behavior trees, state machines, utility AI | ✅ Active |
| `bevy-combat` | Combat, damage, effects, XP | ✅ Active |
| `game-physics` | Physics integration with Avian3D | ✅ Active |
| `cosmic-cults` | Main game crate (World, Units, Assets, AI behaviors) | ✅ Active |

## Development

### Prerequisites

- Rust 1.85+ (Edition 2024)
- For WASM: `wasm32-unknown-unknown` target and `trunk`

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (for WASM development)
cargo install trunk
```

### Building and Testing

```bash
# Check all crates compile
cargo check --all

# Run tests
cargo test --all

# Run clippy (linter)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Build documentation
cargo doc --no-deps --all-features --open
```

### Running Examples

See [cosmic-cults/examples/README.md](cosmic-cults/examples/README.md) for detailed information.

```bash
# Run a native example
cargo run --example basic_physics

# List all examples
cargo run --example
```

### WASM Development

```bash
# Start development server with hot reload
trunk serve --address 0.0.0.0 --port 8080

# Build for production
trunk build --release

# The WASM build will be in dist/
```

## Code Quality Tools

This project uses strict code quality standards:

### Pre-commit Hooks

Install pre-commit hooks to automatically check code before committing:

```bash
# Install pre-commit (if not already installed)
pip install pre-commit

# Install the git hooks
pre-commit install

# Run manually on all files
pre-commit run --all-files
```

The pre-commit hooks will:
- Format code with `rustfmt`
- Lint code with `clippy`
- Check code compiles
- Remove trailing whitespace
- Validate YAML/TOML files

### Continuous Integration

All code is automatically checked by CI on every push and pull request:
- ✅ Format checking (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Compilation (`cargo check`)
- ✅ Tests (`cargo test`)
- ✅ Documentation (`cargo doc`)
- ✅ WASM build

## Documentation

- [Online Documentation](https://jbcom.github.io/rust-cosmic-cults/)
- [Rust Standards](RUST_STANDARDS.md) - Development standards and tooling
- [WASM Demo](WASM_DEMO.md) - WebAssembly build and deployment
- [Examples](cosmic-cults/examples/README.md) - Runnable examples
- [Architecture Docs](docs/) - Detailed architecture documentation

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
