#!/bin/bash
# .cursor/setup.sh
# Setup script for Rust development (jbcom ecosystem)

set -e

echo "Setting up Rust development environment..."

# Install system dependencies (if not in Docker)
if [ ! -f /.dockerenv ]; then
    echo "Installing system dependencies for Bevy..."
    sudo apt-get update -qq
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libudev-dev \
        libasound2-dev \
        libx11-dev \
        libxi-dev \
        libgl1-mesa-dev \
        libglu1-mesa-dev \
        libxcursor-dev \
        libxrandr-dev \
        libxinerama-dev \
        libwayland-dev \
        libxkbcommon-dev \
        libvulkan-dev \
        libwayland-cursor0 \
        libwayland-egl1 \
        mesa-vulkan-drivers
fi

# Check Rust version for Edition 2024 support
RUST_VERSION=$(rustc --version | grep -oP '\d+\.\d+' | head -1)
MAJOR=$(echo $RUST_VERSION | cut -d. -f1)
MINOR=$(echo $RUST_VERSION | cut -d. -f2)

echo "Detected Rust version: $RUST_VERSION"

if [ "$MAJOR" -eq 1 ] && [ "$MINOR" -lt 85 ]; then
    echo "⚠️  WARNING: Rust $RUST_VERSION does not support Edition 2024"
    echo "   Edition 2024 requires Rust 1.85+"
    echo "   Using Edition 2021 for compatibility"
    echo ""
    echo "   To upgrade: rustup update stable"
else
    echo "✅ Rust $RUST_VERSION supports Edition 2024"
fi

# Add WASM target if not present
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Installing WASM target..."
    rustup target add wasm32-unknown-unknown
fi

# Add components
rustup component add rustfmt clippy rust-analyzer 2>/dev/null || true

# Install cargo tools (via binstall if available, otherwise cargo install)
install_tool() {
    local tool=$1
    local check_cmd=$2
    
    if command -v cargo-binstall &> /dev/null; then
        cargo binstall --no-confirm --no-symlinks "$tool" 2>/dev/null || \
        cargo install "$tool" 2>/dev/null || true
    else
        cargo install "$tool" 2>/dev/null || true
    fi
}

echo "Installing cargo tools..."
install_tool cargo-watch "cargo watch"
install_tool cargo-edit "cargo add"
install_tool cargo-audit "cargo audit"
install_tool cargo-outdated "cargo outdated"
install_tool cargo-nextest "cargo nextest"

# Build the project
if [ -f Cargo.toml ]; then
    echo "Building project..."
    cargo build --all 2>/dev/null || echo "Build failed - may need dependency updates"
    
    echo ""
    echo "Running checks..."
    cargo fmt --all -- --check 2>/dev/null || echo "Format check failed"
    cargo clippy --all 2>/dev/null || echo "Clippy check failed"
fi

echo ""
echo "Setup complete! Available commands:"
echo "  cargo build          # Build the project"
echo "  cargo test           # Run tests"
echo "  cargo clippy         # Run linter"
echo "  cargo fmt            # Format code"
echo "  cargo watch -x check # Watch mode"
echo "  cargo audit          # Security audit"
echo "  cargo outdated       # Check for updates"
