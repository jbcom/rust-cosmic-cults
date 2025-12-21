#!/bin/bash
# .cursor/setup.sh
# Setup script for AI RPG Generator development environment

set -e

echo "Setting up AI RPG Generator development environment..."

# Install system dependencies (if not in Docker)
if [ ! -f /.dockerenv ]; then
    echo "Installing system dependencies..."
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

# Install Rust nightly if not already installed
if ! rustup toolchain list | grep -q nightly; then
    echo "Installing Rust nightly toolchain..."
    rustup toolchain install nightly
    rustup component add rustfmt clippy rust-analyzer --toolchain nightly
fi

# Set nightly as default for this project
echo "Setting Rust nightly as default for this project..."
rustup override set nightly

# Install cargo tools
echo "Installing cargo tools..."
cargo install cargo-watch || true
cargo install sqlx-cli --no-default-features --features postgres || true

# Build the project to download dependencies
echo "Building project and downloading dependencies..."
cargo build --all

# Run initial checks
echo "Running initial checks..."
cargo +nightly fmt --all -- --check || true
cargo +nightly clippy --all -- -D warnings || true

echo "Setup complete! You can now run:"
echo "  cargo run -p ai_rpg_generator     # Run the main generator"
echo "  cargo test --all                   # Run all tests"
echo "  cargo watch -x check               # Watch mode for development"
