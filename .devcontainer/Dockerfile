FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional system dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        zsh \
        tmux \
        jq \
        ripgrep \
        fd-find \
        bat \
        httpie \
        socat \
        netcat \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install Go for Charm/Crush development
RUN curl -L https://go.dev/dl/go1.21.6.linux-amd64.tar.gz | tar -xz -C /usr/local
ENV PATH="/usr/local/go/bin:${PATH}"

# Install Rust components
RUN rustup component add rustfmt clippy rust-src rust-analyzer

# Install cargo tools for better development
RUN cargo install cargo-watch cargo-edit cargo-expand

# Install Charm tools
RUN go install github.com/charmbracelet/glow@latest \
    && go install github.com/charmbracelet/gum@latest

# Install Crush (when available publicly)
# RUN go install github.com/charmbracelet/crush@latest
# For now, we'll add instructions to install locally

# Create workspace directory
WORKDIR /workspace/mmogit

# Set up Git defaults for sovereignty
RUN git config --global init.defaultBranch main \
    && git config --global user.email "sovereign@mmogit.local" \
    && git config --global user.name "Sovereign Dev"

# Set environment for YOLO development
ENV YOLO_MODE=true
ENV RUST_BACKTRACE=1
ENV MMOGIT_DEV=true

# Create directories for mmogit testing
RUN mkdir -p /home/vscode/.mmogit \
    && mkdir -p /home/vscode/.mmogit-agent \
    && mkdir -p /home/vscode/.mmogit-test

# Switch to vscode user
USER vscode

# Install oh-my-zsh for better terminal experience
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended \
    && echo 'export PATH="/usr/local/go/bin:$HOME/go/bin:$PATH"' >> ~/.zshrc \
    && echo 'alias crush="crush --yolo"' >> ~/.zshrc \
    && echo 'alias mm="./target/release/mmogit"' >> ~/.zshrc \
    && echo 'echo "🚀 MMOGit Sovereign Development Environment"' >> ~/.zshrc \
    && echo 'echo "📝 YOLO Mode Enabled - Build Fast, Own Everything"' >> ~/.zshrc

# Pre-build mmogit (will be overridden by mounted code)
COPY . /tmp/mmogit-prebuild/
RUN cd /tmp/mmogit-prebuild && cargo build --release 2>/dev/null || true

# Final message
CMD ["zsh"]
