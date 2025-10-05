# Use Ubuntu 22.04 as base for better compatibility
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Set environment variables
ENV RUST_VERSION=1.83.0
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies for pure Rust development
RUN apt-get update && apt-get install -y \
    # Core build tools
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    libssl-dev \
    # Development utilities
    jq \
    unzip \
    zip \
    && rm -rf /var/lib/apt/lists/*

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Install Rust via rustup with latest stable version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Install additional Rust components useful for development
RUN rustup component add rustfmt clippy rust-analyzer

# Install Claude Code CLI
RUN curl -fsSL https://claude.ai/claude-code/install.sh | sh
ENV PATH="/home/vscode/.local/bin:${PATH}"

# Create workspace directory
WORKDIR /workspaces/spotlight-dimmer

# Set up shell configuration
RUN echo 'export PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:${PATH}"' >> /home/vscode/.bashrc
RUN echo 'export PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:${PATH}"' >> /home/vscode/.zshrc

# Verify installations (will show in build logs)
RUN rustc --version && cargo --version && cargo clippy --version && cargo fmt --version
RUN which claude-code || echo "Claude Code installation will be verified in post-create script"