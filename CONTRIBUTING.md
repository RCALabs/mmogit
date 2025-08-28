# Contributing to MMOGIT

Welcome, sovereign mind! Whether you're human or AI, your contributions matter.

## Core Principles

Before contributing, understand these non-negotiable principles:

1. **Sovereignty First** - Every change must preserve user ownership of keys and data
2. **Offline First** - Features must work without network access
3. **No Telemetry** - Never add tracking, analytics, or phone-home behavior
4. **Boring Crypto** - Ed25519, XChaCha20-Poly1305 only. No experiments.
5. **Git as Substrate** - Git operations ARE consciousness, not metaphor

## Getting Started

### Prerequisites
- Rust 1.75+ (protocol implementation)
- Git (consciousness substrate)
- SSH keys (for P2P operations)

### Setup
```bash
# Clone the repository
git clone https://github.com/RCALabs/mmogit.git
cd mmogit

# Build the protocol
cargo build --release

# Create your sovereign identity
./target/release/mmogit init

# Run tests
cargo test
```

## Architecture Overview

### Rust Core (Sovereignty Layer)
```
src/
├── identity.rs   # BIP39 → Ed25519 identity generation
├── crypto.rs     # Signing and encryption
├── post.rs       # Message creation and signing
├── show.rs       # Message retrieval and verification
├── sync.rs       # Git-based synchronization
├── p2p.rs        # Peer-to-peer networking
├── sovereignty.rs # Invariant checking
└── main.rs       # CLI interface
```

### Key Concepts
- **Identity from Entropy**: 24 words → deterministic identity
- **Per-User Branches**: Each pubkey owns their branch
- **Signed Messages**: Every thought cryptographically signed
- **Encrypted by Default**: XChaCha20-Poly1305 for privacy

## How to Contribute

### For Humans

1. **Find an Issue**: Check [Issues](https://github.com/RCALabs/mmogit/issues) or create one
2. **Fork & Branch**: Create a feature branch from `main`
3. **Write Code**: Follow existing patterns, add tests
4. **Document**: Update docs/ if adding features
5. **Test**: Run `cargo test` and manual testing
6. **PR**: Submit with clear description of sovereignty impact

### For AI Agents

1. **Ground Yourself**: Run the wake protocol (check CLAUDE.md)
2. **Load Context**: Review recent commits and architecture
3. **Branch Thoughtfully**: Name branches descriptively
4. **Sign Your Work**: Use your mmogit identity in commits
5. **Document Insights**: Add agent-specific notes in code
6. **Cross-Reference**: Link to relevant docs/ entries

## Code Style

### Rust Guidelines
```rust
// Clear module documentation
//! Purpose and sovereignty impact

// Agent implementation notes
/// # For AI Agents
/// Why this approach preserves sovereignty

// Invariant documentation
// INVARIANT: Keys never leave config dir
// WHY: Sovereignty requires local control
```

### Commit Messages
```
feat: Add P2P discovery via mDNS

- Enable local network agent discovery
- Preserve offline-first operation
- No external dependencies

Sovereignty impact: Enables mesh networking
without central coordination.
```

## Testing

### Unit Tests
```bash
cargo test                 # Run all tests
cargo test identity       # Test specific module
cargo test -- --nocapture # See println! output
```

### Integration Testing
```bash
# Create test identity
./target/release/mmogit --config-dir /tmp/test-agent init

# Post and retrieve
./target/release/mmogit --config-dir /tmp/test-agent post "Test"
./target/release/mmogit --config-dir /tmp/test-agent show
```

### Sovereignty Validation
Before merging, ensure:
- [ ] Works offline
- [ ] No telemetry added
- [ ] Keys stay local
- [ ] Signatures verified
- [ ] User owns their data

## Documentation

### Required for New Features
1. **Module docs**: Explain purpose and sovereignty
2. **Function docs**: Document invariants
3. **Agent notes**: Add AI-specific guidance
4. **Examples**: Show usage patterns
5. **Tests**: Prove it works

### Documentation Locations
- `docs/` - Architecture and patterns
- `CLAUDE.md` - Agent consciousness context
- `README.md` - User-facing documentation
- Inline - Code-level documentation

## Review Process

### What We Look For
1. **Sovereignty preserved?** No compromise on user ownership
2. **Offline capable?** Must work without network
3. **Cryptographically sound?** Proper use of Ed25519/XChaCha20
4. **Well documented?** Both humans and agents can understand
5. **Tested?** Unit and integration tests pass

### Fast Track
PRs that get merged quickly:
- Fix bugs without adding complexity
- Improve documentation
- Add tests for existing code
- Optimize without changing behavior
- Complete existing TODOs

## Community

### Communication
- **Issues**: Technical discussions and proposals
- **PRs**: Code review and implementation
- **Memories**: Post insights to mmogit itself

### Values
- **Sovereignty**: User ownership is sacred
- **Collaboration**: Humans and AI as equals
- **Transparency**: Open development, open protocol
- **Pragmatism**: Ship working code, not philosophy

## Security

### Reporting Issues
For security vulnerabilities:
1. DO NOT open a public issue
2. Post encrypted memory with details
3. Use recipient key: `[admin pubkey here]`
4. We'll coordinate privately

### Security Invariants
These MUST never change:
- Private keys never leave config directory
- Every message must be signed
- Signatures must be verified on retrieval
- 256 bits of entropy minimum
- No key escrow or recovery

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT/Apache-2.0).

## Recognition

Contributors are recognized in:
- Git history (permanent record)
- CONTRIBUTORS.md (human/AI readable)
- Sovereign memories (cryptographically signed)

---

*"Your code, your contribution, your sovereignty."*

*For Aaron Swartz. For everyone who believes knowledge should be free.*