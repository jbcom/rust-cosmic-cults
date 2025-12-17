# Rust Development Environment for jbcom Ecosystem
# Supports both edition 2021 and 2024 (requires Rust 1.85+)
FROM rust:1.85-bookworm

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:$PATH
ENV RUST_BACKTRACE=1
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=30

# Set timezone to UTC
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install system dependencies
RUN apt-get update -qq && \
    apt-get install -yqq --no-install-recommends \
    # Build essentials
    build-essential \
    pkg-config \
    # SSL/TLS
    libssl-dev \
    ca-certificates \
    # Graphics/Windowing dependencies for Bevy
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
    mesa-vulkan-drivers \
    # Development tools
    curl \
    wget \
    git \
    vim \
    nano \
    sudo \
    jq \
    # For UI tests
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

# Add components
RUN rustup component add rustfmt clippy rust-analyzer

# Install cargo-binstall for faster installation
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install cargo tools via binstall (faster than building)
RUN cargo binstall --no-confirm --no-symlinks \
    cargo-watch \
    cargo-edit \
    cargo-audit \
    cargo-outdated \
    cargo-nextest \
    trunk

# Create non-root user
RUN useradd -m -s /bin/bash ubuntu && \
    echo 'ubuntu ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers && \
    chown -R ubuntu:ubuntu /usr/local/cargo || true && \
    mkdir -p /home/ubuntu/.cargo && \
    chown -R ubuntu:ubuntu /home/ubuntu/.cargo

# Switch to ubuntu user
USER ubuntu
WORKDIR /home/ubuntu

# Create cargo config
RUN mkdir -p /home/ubuntu/.cargo && \
    echo '[build]' > /home/ubuntu/.cargo/config.toml && \
    echo 'target-dir = "/home/ubuntu/.cargo/target"' >> /home/ubuntu/.cargo/config.toml && \
    echo 'jobs = 4' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[net]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'retry = 3' >> /home/ubuntu/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[registries.crates-io]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'protocol = "sparse"' >> /home/ubuntu/.cargo/config.toml

# Set up PATH
ENV PATH="/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:/usr/local/cargo/bin:${PATH}"

# Create startup script
RUN echo '#!/bin/bash' > /home/ubuntu/startup.sh && \
    echo 'echo "=== Rust Development Environment ==="' >> /home/ubuntu/startup.sh && \
    echo 'echo "Rust: $(rustc --version)"' >> /home/ubuntu/startup.sh && \
    echo 'echo "Cargo: $(cargo --version)"' >> /home/ubuntu/startup.sh && \
    echo 'echo "Edition 2024 support: YES (Rust 1.85+)"' >> /home/ubuntu/startup.sh && \
    echo 'echo ""' >> /home/ubuntu/startup.sh && \
    echo 'echo "WASM target: $(rustup target list --installed | grep wasm)"' >> /home/ubuntu/startup.sh && \
    echo 'echo ""' >> /home/ubuntu/startup.sh && \
    echo 'echo "Tools: cargo-watch, cargo-edit, cargo-audit, cargo-outdated, trunk"' >> /home/ubuntu/startup.sh && \
    echo 'exec "$@"' >> /home/ubuntu/startup.sh && \
    chmod +x /home/ubuntu/startup.sh

# Virtual display for UI tests
ENV DISPLAY=:99

CMD ["/bin/bash", "-l"]
