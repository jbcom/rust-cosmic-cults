# WASM Demo Showcase

This directory contains configuration for building and deploying the Cosmic Cults RTS game as a WebAssembly application.

## Building for WASM

### Prerequisites

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (WASM bundler)
cargo install trunk
```

### Development Build

For local development with hot reload:

```bash
trunk serve --address 0.0.0.0 --port 8080
```

Then open your browser to `http://localhost:8080`

### Production Build

For an optimized production build:

```bash
trunk build --release
```

The built files will be in the `dist/` directory.

## Deployment

The WASM build is automatically deployed to GitHub Pages on every push to main.

See the game live at: https://jbcom.github.io/rust-cosmic-cults/

## Features

The WASM build includes:
- Full 3D RTS gameplay
- Physics simulation with Avian3D
- Unit management and formations
- AI systems
- Combat mechanics

## Technical Details

- **Target**: `wasm32-unknown-unknown`
- **Engine**: Bevy 0.17
- **Physics**: Avian3D 0.4
- **Build Tool**: Trunk
- **Optimization**: wasm-opt with `-O3`

## Performance

The game is optimized for browser performance:
- Efficient ECS architecture with Bevy
- Spatial partitioning for physics
- Optimized WASM binaries
- Target: 60 FPS with 300+ entities

## Browser Compatibility

Tested on:
- Chrome/Edge 90+
- Firefox 90+
- Safari 15+

Requires WebGL 2.0 support.
