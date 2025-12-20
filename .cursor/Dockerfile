# Use official Rust 1.88 image as base
FROM rust:1.88-bookworm

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:$PATH
ENV RUST_BACKTRACE=1
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=30

# Python environment variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV PIP_NO_CACHE_DIR=1
ENV PIP_DISABLE_PIP_VERSION_CHECK=1

# Set timezone to UTC
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install system dependencies and pre-built tools in a single layer
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
    # Python 3.11+ and development packages
    python3.11 \
    python3.11-dev \
    python3.11-venv \
    python3-pip \
    pipx \
    # Python dependencies for various packages
    libffi-dev \
    libxml2-dev \
    libxslt1-dev \
    libjpeg-dev \
    zlib1g-dev \
    libpng-dev \
    # Audio/Video processing (for game assets)
    ffmpeg \
    libavcodec-dev \
    libavformat-dev \
    # Database clients
    postgresql-client \
    redis-tools \
    # Process management
    procps \
    htop \
    # Archive tools
    unzip \
    # Network tools
    net-tools \
    # For downloading pre-built binaries
    jq \
    # For UI automation tests
    xvfb \
    x11-utils \
    scrot \
    && rm -rf /var/lib/apt/lists/*

# Set nightly toolchain as default and add components
RUN rustup default nightly && \
    rustup component add rustfmt clippy rust-analyzer

# Install cargo-binstall for faster installation of pre-built binaries
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install all cargo tools using binstall where possible (much faster than building from source)
# If binstall fails for a tool, fall back to cargo install
RUN cargo binstall --no-confirm --no-symlinks cargo-watch && \
    cargo binstall --no-confirm --no-symlinks cargo-edit && \
    cargo binstall --no-confirm --no-symlinks cargo-audit && \
    cargo binstall --no-confirm --no-symlinks cargo-outdated && \
    (cargo binstall --no-confirm --no-symlinks sqlx-cli || \
     cargo install sqlx-cli --no-default-features --features rustls,postgres)

# Install Python tools globally
RUN python3.11 -m pip install --upgrade pip setuptools wheel && \
    python3.11 -m pip install \
        hatch \
        uv \
        poetry \
        pytest \
        pytest-asyncio \
        pytest-mock \
        pytest-vcr \
        pytest-cov \
        pytest-xdist \
        ruff \
        black \
        mypy \
        tox \
        pre-commit

# Install pipx tools
RUN pipx install hatch && \
    pipx install poetry && \
    pipx install tox && \
    pipx ensurepath

# Create a non-root user with proper permissions and ensure cargo is in standard PATH
RUN useradd -m -s /bin/bash ubuntu && \
    echo 'ubuntu ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers && \
    chown -R ubuntu:ubuntu /usr/local/cargo || true && \
    mkdir -p /home/ubuntu/.cargo && \
    chown -R ubuntu:ubuntu /home/ubuntu/.cargo && \
    ln -sf /usr/local/cargo/bin/cargo /usr/local/bin/cargo && \
    ln -sf /usr/local/cargo/bin/rustc /usr/local/bin/rustc && \
    ln -sf /usr/local/cargo/bin/rustup /usr/local/bin/rustup && \
    ln -sf /usr/bin/python3.11 /usr/local/bin/python3 && \
    ln -sf /usr/bin/python3.11 /usr/local/bin/python

# Switch to the ubuntu user
USER ubuntu
WORKDIR /home/ubuntu

# Create optimized cargo config for the user
RUN mkdir -p /home/ubuntu/.cargo && \
    echo '[build]' > /home/ubuntu/.cargo/config.toml && \
    echo 'target-dir = "/home/ubuntu/.cargo/target"' >> /home/ubuntu/.cargo/config.toml && \
    echo 'jobs = 4' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[net]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'retry = 3' >> /home/ubuntu/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[profile.dev]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'opt-level = 0' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[profile.release]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'lto = "thin"' >> /home/ubuntu/.cargo/config.toml && \
    echo '' >> /home/ubuntu/.cargo/config.toml && \
    echo '[registries.crates-io]' >> /home/ubuntu/.cargo/config.toml && \
    echo 'protocol = "sparse"' >> /home/ubuntu/.cargo/config.toml

# Set up PATH for the ubuntu user and ensure it persists for background agents
ENV PATH="/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:/usr/local/cargo/bin:${PATH}"

# Create .bashrc and .profile with proper PATH to ensure background agents can find cargo and python tools
RUN echo 'export PATH="/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:/usr/local/cargo/bin:$PATH"' >> /home/ubuntu/.bashrc && \
    echo 'export CARGO_HOME=/usr/local/cargo' >> /home/ubuntu/.bashrc && \
    echo 'export RUSTUP_HOME=/usr/local/rustup' >> /home/ubuntu/.bashrc && \
    echo 'export PYTHONUNBUFFERED=1' >> /home/ubuntu/.bashrc && \
    echo 'export PATH="/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:/usr/local/cargo/bin:$PATH"' >> /home/ubuntu/.profile && \
    echo 'export CARGO_HOME=/usr/local/cargo' >> /home/ubuntu/.profile && \
    echo 'export RUSTUP_HOME=/usr/local/rustup' >> /home/ubuntu/.profile && \
    echo 'export PYTHONUNBUFFERED=1' >> /home/ubuntu/.profile

# Pre-create common directories
RUN mkdir -p /home/ubuntu/workspace /home/ubuntu/.cache /home/ubuntu/.local/share/hatch /home/ubuntu/.config

# Create startup script using echo commands with proper PATH setup
RUN echo '#!/bin/bash' > /home/ubuntu/startup.sh && \
    echo '# Ensure PATH is set for background agents' >> /home/ubuntu/startup.sh && \
    echo 'export PATH="/home/ubuntu/.local/bin:/home/ubuntu/.cargo/bin:/usr/local/cargo/bin:$PATH"' >> /home/ubuntu/startup.sh && \
    echo 'export CARGO_HOME=/usr/local/cargo' >> /home/ubuntu/startup.sh && \
    echo 'export RUSTUP_HOME=/usr/local/rustup' >> /home/ubuntu/startup.sh && \
    echo 'export PYTHONUNBUFFERED=1' >> /home/ubuntu/startup.sh && \
    echo 'echo "=== AI RPG Generator Development Container ==="' >> /home/ubuntu/startup.sh && \
    echo 'echo "Rust version: $(rustc --version)"' >> /home/ubuntu/startup.sh && \
    echo 'echo "Cargo version: $(cargo --version)"' >> /home/ubuntu/startup.sh && \
    echo 'echo "Python version: $(python3 --version)"' >> /home/ubuntu/startup.sh && \
    echo 'echo ""' >> /home/ubuntu/startup.sh && \
    echo 'echo "Available tools:"' >> /home/ubuntu/startup.sh && \
    echo 'which cargo-watch >/dev/null 2>&1 && echo "✓ cargo-watch"' >> /home/ubuntu/startup.sh && \
    echo 'which cargo-edit >/dev/null 2>&1 && echo "✓ cargo-edit"' >> /home/ubuntu/startup.sh && \
    echo 'which cargo-audit >/dev/null 2>&1 && echo "✓ cargo-audit"' >> /home/ubuntu/startup.sh && \
    echo 'which cargo-outdated >/dev/null 2>&1 && echo "✓ cargo-outdated"' >> /home/ubuntu/startup.sh && \
    echo 'which sqlx >/dev/null 2>&1 && echo "✓ sqlx-cli"' >> /home/ubuntu/startup.sh && \
    echo 'which hatch >/dev/null 2>&1 && echo "✓ hatch"' >> /home/ubuntu/startup.sh && \
    echo 'which uv >/dev/null 2>&1 && echo "✓ uv"' >> /home/ubuntu/startup.sh && \
    echo 'which pytest >/dev/null 2>&1 && echo "✓ pytest"' >> /home/ubuntu/startup.sh && \
    echo 'which black >/dev/null 2>&1 && echo "✓ black"' >> /home/ubuntu/startup.sh && \
    echo 'which ruff >/dev/null 2>&1 && echo "✓ ruff"' >> /home/ubuntu/startup.sh && \
    echo 'echo ""' >> /home/ubuntu/startup.sh && \
    echo 'echo "Working directory: $(pwd)"' >> /home/ubuntu/startup.sh && \
    echo 'echo "Ready for development!"' >> /home/ubuntu/startup.sh && \
    echo 'exec "$@"' >> /home/ubuntu/startup.sh

# Make startup script executable
RUN chmod +x /home/ubuntu/startup.sh

# Set up virtual display for UI tests
ENV DISPLAY=:99

# Default command optimized for background agents with proper shell setup
CMD ["/bin/bash", "-l"]
