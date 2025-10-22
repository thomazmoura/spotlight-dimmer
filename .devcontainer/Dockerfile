# Use Ubuntu 22.04 as base for better compatibility
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Set environment variables
ENV DOTNET_VERSION=10.0
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies for .NET development
RUN apt-get update && apt-get install -y \
    # Core build tools
    curl \
    wget \
    git \
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

# Install .NET SDK
RUN wget https://dot.net/v1/dotnet-install.sh -O dotnet-install.sh && \
    chmod +x dotnet-install.sh && \
    ./dotnet-install.sh --channel 10.0 --install-dir /usr/share/dotnet && \
    ln -s /usr/share/dotnet/dotnet /usr/bin/dotnet && \
    rm dotnet-install.sh

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Create workspace directory
WORKDIR /workspaces/spotlight-dimmer

# Verify installations (will show in build logs)
RUN dotnet --version
RUN claude --version || echo "Claude Code will be verified in post-create script"
