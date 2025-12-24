# Examples

This directory contains runnable examples demonstrating various features of the Cosmic Cults RTS game engine.

## Running Examples

To run an example, use:

```bash
cargo run --example <example_name>
```

## Available Examples

### `basic_physics`
Demonstrates the physics system with moving entities and collision detection.
- Shows spatial grid integration
- Demonstrates collision detection
- Interactive: Press SPACE to spawn entities

```bash
cargo run --example basic_physics
```

### `unit_spawning`
Demonstrates unit spawning and basic unit behavior.
- Shows unit initialization
- Demonstrates unit templates
- Visual unit representation

```bash
cargo run --example unit_spawning
```

### `formations`
Demonstrates unit formations and group movement.
- Shows different formation types (Line, Column, Box, Wedge)
- Interactive formation switching with number keys
- Group movement with arrow keys

```bash
cargo run --example formations
```

## WASM Examples

To build examples for WASM, use:

```bash
# Build for WASM
cargo build --example <example_name> --target wasm32-unknown-unknown

# Or use trunk for a full dev server
trunk serve examples/<example_name>.html
```

## Requirements

- Rust 1.85+ (Edition 2024)
- Bevy 0.17+
- For native: Standard Rust toolchain
- For WASM: `wasm32-unknown-unknown` target and `trunk`
