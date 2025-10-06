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
    # Node.js dependencies (required for Claude Code)
    ca-certificates \
    gnupg \
    # Development utilities
    jq \
    unzip \
    zip \
    tmux \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x (LTS) - required for Claude Code
RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg && \
    echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list && \
    apt-get update && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# Install Claude Code CLI globally as root
# Package: @anthropic-ai/claude-code
# Binary name: claude (NOT claude-code!)
RUN npm install -g @anthropic-ai/claude-code

# Install OpenAI Codex CLI
RUN npm install -g @openai/codex

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Install Rust via rustup with latest stable version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Install additional Rust components useful for development
RUN rustup component add rustfmt clippy rust-analyzer

# Create workspace directory
WORKDIR /workspaces/spotlight-dimmer

# Set up shell configuration
RUN echo 'export PATH="/home/vscode/.cargo/bin:${PATH}"' >> /home/vscode/.bashrc
RUN echo 'export PATH="/home/vscode/.cargo/bin:${PATH}"' >> /home/vscode/.zshrc

# Verify installations (will show in build logs)
RUN rustc --version && cargo --version && cargo clippy --version && cargo fmt --version
RUN claude --version || echo "Claude Code will be verified in post-create script"
