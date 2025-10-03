# Use Ubuntu 22.04 as base for better compatibility
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Set environment variables
ENV RUST_VERSION=1.77.2
ENV NODE_VERSION=20.17.0
ENV TAURI_CLI_VERSION=2.0.0
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies required for Tauri and GUI development
RUN apt-get update && apt-get install -y \
    # Core build tools
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    libssl-dev \
    # Tauri dependencies for Linux builds
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # Additional GUI and system libraries
    libxdo-dev \
    libxrandr-dev \
    libxss-dev \
    libgconf-2-4 \
    libxss1 \
    libappindicator1 \
    libnss3 \
    lsb-release \
    # Python for http.server and build scripts
    python3 \
    python3-pip \
    # Development utilities
    jq \
    unzip \
    zip \
    && rm -rf /var/lib/apt/lists/*

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Install Node.js via nvm
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
ENV NVM_DIR="/home/vscode/.nvm"
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION} && nvm use ${NODE_VERSION} && nvm alias default ${NODE_VERSION}
ENV PATH="/home/vscode/.nvm/versions/node/v${NODE_VERSION}/bin:${PATH}"

# Install Tauri CLI
RUN cargo install tauri-cli --version ^${TAURI_CLI_VERSION}

# Install Claude Code CLI
RUN curl -fsSL https://claude.ai/claude-code/install.sh | sh
ENV PATH="/home/vscode/.local/bin:${PATH}"

# Install global npm packages that might be useful
RUN . "$NVM_DIR/nvm.sh" && npm install -g \
    typescript \
    @types/node \
    prettier \
    eslint

# Create workspace directory
WORKDIR /workspaces/spotlight-dimmer

# Set up shell configuration
RUN echo 'export PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:${PATH}"' >> /home/vscode/.bashrc
RUN echo 'export PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:${PATH}"' >> /home/vscode/.zshrc

# Verify installations (will show in build logs)
RUN rustc --version && cargo --version
RUN . "$NVM_DIR/nvm.sh" && node --version && npm --version
RUN which claude-code || echo "Claude Code installation will be verified in post-create script"