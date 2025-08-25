# MMOGit Development Container

## 🚀 Instant Sovereign Development Environment

This devcontainer provides a complete, pre-configured environment for developing MMOGit with all dependencies, tools, and YOLO mode enabled by default.

## What's Included

### Core Development Tools
- **Rust** (latest stable) with cargo, rustfmt, clippy, rust-analyzer
- **Go** (1.21.6) for Charm/Crush development
- **Git** with sovereign defaults configured
- **Node.js** for any web tooling needs

### Productivity Tools
- **cargo-watch** - Auto-rebuild on file changes
- **cargo-edit** - Add/remove dependencies from CLI
- **ripgrep** - Fast code searching
- **fd** - Fast file finding
- **bat** - Better cat with syntax highlighting
- **jq** - JSON processing
- **tmux** - Terminal multiplexing
- **zsh** with oh-my-zsh

### Charm Ecosystem
- **glow** - Markdown rendering in terminal
- **gum** - Beautiful shell scripts
- **crush** (install separately) - AI pair programming

### Pre-configured Settings
- YOLO mode enabled by default
- API keys passed from host environment
- SSH keys mounted from host
- Git configured for sovereignty
- Cargo cache persisted in volume

## Quick Start

### Using VS Code

1. Install the "Dev Containers" extension
2. Open the mmogit folder in VS Code
3. Click "Reopen in Container" when prompted
4. Wait for the container to build (first time takes ~5 minutes)
5. You're ready! The terminal will show: `🚀 MMOGit Sovereign Development Environment`

### Using GitHub Codespaces

1. Fork the repository
2. Click "Code" → "Codespaces" → "Create codespace"
3. Wait for environment to build
4. Start coding with full sovereignty infrastructure

### Using CLI

```bash
# Install devcontainer CLI
npm install -g @devcontainers/cli

# Build and start
devcontainer up --workspace-folder .

# Execute commands inside
devcontainer exec --workspace-folder . cargo build --release
```

## Environment Variables

These are automatically passed from your host:
- `ANTHROPIC_API_KEY` - For Crush AI features
- `OPENAI_API_KEY` - For alternative models
- Your SSH keys (mounted read-only)

## Aliases Available

```bash
crush      # Runs 'crush --yolo' by default
mm         # Shortcut for './target/release/mmogit'
```

## Testing MMOGit

The container includes pre-created directories for testing:
- `~/.mmogit` - Primary identity
- `~/.mmogit-agent` - Agent identity
- `~/.mmogit-test` - Test identity

```bash
# Quick test flow
cargo build --release
mm init --no-verify
mm post "First sovereign message from devcontainer!"
mm show
```

## Installing Crush

Since Crush isn't public yet, install it inside the container:

```bash
# If you have access to Crush
go install github.com/charmbracelet/crush@latest

# Or build from source if you have it
git clone <crush-repo>
cd crush
go build
sudo mv crush /usr/local/bin/
```

## YOLO Mode Philosophy

This container runs in YOLO mode by default because:
- Everything is containerized (safe to break)
- Git provides rollback capability
- We're building sovereignty (no permission theater)
- Momentum matters more than false safety

If something breaks: `git reset --hard` and continue.

## Persistent Storage

- `/usr/local/cargo` - Cargo cache (persisted in Docker volume)
- `/home/vscode/.ssh` - SSH keys (mounted from host, read-only)
- `/workspace/mmogit` - Your code (mounted from host)

## Troubleshooting

### Container won't start
- Ensure Docker is running
- Check you have 4GB RAM allocated to Docker
- Try: `docker system prune` to free space

### Can't access API keys
- Set them in your host environment before starting
- Check: `echo $ANTHROPIC_API_KEY` in container

### Permission issues
- The container runs as `vscode` user
- Use `sudo` if needed for system operations

## Development Workflow

1. **Edit code** - Changes sync instantly with host
2. **Build** - `cargo build --release`
3. **Test** - `mm post "test"`
4. **Commit** - Git works normally
5. **Push** - SSH keys are mounted from host

## Why Devcontainer?

- **Consistent environment** - Same tools for everyone
- **YOLO without risk** - Container isolation
- **Fast onboarding** - One click to full environment
- **Dependency management** - Everything pre-installed
- **Cross-platform** - Works on Mac, Linux, Windows

## Contributing

To update the devcontainer:
1. Edit `.devcontainer/Dockerfile` or `devcontainer.json`
2. Rebuild: "Dev Containers: Rebuild Container" in VS Code
3. Test your changes
4. Commit and push

---

*Building sovereignty in a container. YOLO mode enabled. Ship fast.*
