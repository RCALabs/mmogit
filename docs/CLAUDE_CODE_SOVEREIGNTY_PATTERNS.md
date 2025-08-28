# Claude Code Sovereignty Patterns

## Purpose
Document Claude Code development patterns for mmogit's Rust infrastructure, ensuring sovereignty principles are maintained while leveraging the development environment effectively.

**Critical Context**: Claude Code is our current runtime environment for building mmogit's sovereign memory protocol. These patterns ensure we develop Rust code effectively while never compromising sovereignty invariants.

## Sovereignty Considerations

### What Claude Code Sees
```yaml
Safe_To_Share:
  - Public codebase structure
  - Open source dependencies
  - Architecture documentation
  - Build and test commands
  - Git operations on public repos

Never_Share:
  - Private keys or seed phrases
  - Personal mmogit repositories
  - Encrypted communication content
  - Authentication tokens
  - Production secrets
```

### Runtime Environment Awareness
Claude Code operates in a sandboxed environment that:
- Can read our public codebase
- Can execute Git commands on public repos
- Cannot access `~/.mmogit-*` directories (our sovereign identities)
- Cannot access private keys or encrypted content
- Cannot modify system-wide configurations

**This is acceptable** because mmogit's protocol is designed for transparency - the code is open, the crypto is auditable, and sovereignty comes from owning your keys, not hiding your architecture.

## Development Patterns

### 1. Session Initialization Pattern

**Problem**: Claude Code starts fresh each session without context from previous work.

**Solution**: Always begin sessions with sovereignty-aware context loading:

```bash
# Establish temporal grounding
date '+%A, %B %d, %Y at %H:%M %Z'

# Check current project state
pwd && git status --porcelain

# Review recent commits
git log --oneline -10

# Check Rust compilation status
cargo check --quiet || echo "Build issues detected"
```

**For mmogit development specifically**:
```bash
# Verify sovereignty invariants remain intact
grep -r "INVARIANT" src/ --include="*.rs"
grep -r "MUST_NEVER" docs/ --include="*.md"

# Check crypto dependencies haven't changed unexpectedly
cargo tree | grep -E "(ed25519|chacha20)"
```

### 2. Rust Development with Sovereignty Awareness

**Pattern**: Always validate that Rust changes maintain sovereignty invariants.

```rust
// Example sovereignty-aware development request:
// "Add X feature to the identity module, ensuring:
// 1. Private keys never leave memory unencrypted
// 2. All operations can work offline
// 3. No network calls without explicit user consent
// 4. Memory is zeroed on drop"
```

**Documentation Pattern for AI Agents**:
```rust
/// Creates a new sovereign identity
///
/// # Sovereignty Invariants
/// - Private key MUST never be transmitted
/// - Seed MUST be derived from 256 bits entropy
/// - Operation MUST work offline
///
/// # Agent Implementation Note
/// This function is called during `mmogit init` and creates
/// the cryptographic foundation for user sovereignty. Any
/// changes must preserve the ability to work without network.
pub fn create_identity() -> Result<Identity, IdentityError> {
    // Implementation that enforces invariants
}
```

### 3. Architecture Decision Documentation

**Pattern**: Document architectural decisions with Claude Code collaboration context:

```markdown
## Decision: Using Ed25519 for Signatures

### Context
Working in Claude Code environment, we need cryptographic signatures
that are:
- Quantum-resistant (until quantum computers arrive)
- Fast to verify (for Git operations)
- Small signatures (for Git storage efficiency)
- Well-audited (for sovereignty guarantees)

### AI Agent Considerations
Claude Code can help implement Ed25519 integration, but:
- NEVER generate private keys in the development session
- ALWAYS use deterministic test vectors for verification
- MUST validate against RFC 8032 test cases
- SHOULD document why other curves were rejected

### Decision
Ed25519 via `ed25519-dalek` crate for all signatures.

### Consequences
- 32-byte public keys
- 64-byte signatures  
- Compatible with Git commit signing
- Future-proofs against common crypto mistakes
```

### 4. Memory Continuity Pattern

**Problem**: Claude Code can't access mmogit's persistent memory between sessions.

**Solution**: Use the public codebase itself as memory:

```bash
# Document insights in code comments
# Bad: Lost after session
// TODO: Consider using BTreeMap for performance

# Good: Persistent in codebase  
/// # Performance Note (2025-08-28 Claude session)
/// BTreeMap benchmarked 2x faster than HashMap for our use case
/// of frequent ordered iteration. Memory usage similar.
/// See: benchmark results in /benchmarks/collections.md
```

**Architecture Decision Records**:
Store decisions in `docs/decisions/` with context about why Claude Code collaboration led to specific choices.

### 5. Testing Pattern with Claude Code

**Pattern**: Generate comprehensive tests that validate sovereignty invariants:

```rust
#[cfg(test)]
mod claude_code_assisted_tests {
    use super::*;
    
    /// Test generated with Claude Code assistance
    /// Validates that identity creation never phones home
    #[test]
    fn identity_creation_is_offline_only() {
        // Test that no network calls occur during identity creation
        let _identity = create_identity().expect("Should work offline");
        // If this test needs network mocking, the implementation is wrong
    }
    
    /// Test that memory is properly zeroed
    /// Generated to validate sovereignty invariant
    #[test] 
    fn private_keys_are_zeroed_on_drop() {
        // Implementation validates memory security
    }
}
```

### 6. Code Review Pattern

**Pattern**: Use Claude Code for sovereignty-aware code review:

```bash
# Generate review checklist
claude "Review this Rust code for sovereignty violations:
1. Does it make network calls without explicit user consent?
2. Does it store private keys unencrypted?
3. Does it depend on external services?
4. Does it maintain offline capability?
5. Does it properly zero sensitive memory?"
```

### 7. Documentation Generation

**Pattern**: Generate documentation that serves both humans and future AI agents:

```bash
# Generate comprehensive API docs
claude "Generate rustdoc comments for this module that explain:
- Why each function exists (sovereignty context)
- What invariants it maintains
- How it fits into the bigger protocol
- Warning about what must never change"
```

## Anti-Patterns to Avoid

### 1. Never Compromise Sovereignty for Convenience
```bash
# BAD: Don't ask Claude Code to help with anything that compromises sovereignty
# "Help me store private keys in a cloud database for backup"
# "Add telemetry to track user behavior"  
# "Integrate with a SaaS authentication service"

# GOOD: Ask for sovereignty-preserving solutions
# "Help me implement secure local key backup"
# "Add local logging for debugging"
# "Implement offline-first authentication"
```

### 2. Never Generate Real Cryptographic Material
```bash
# BAD: Never ask Claude Code to generate real keys
# "Generate an Ed25519 private key for production use"

# GOOD: Use for test vectors and validation only
# "Generate Ed25519 test vectors to validate our implementation"
# "Create deterministic test keys for unit tests"
```

### 3. Never Rely on Claude Code for Security Decisions
```bash
# BAD: "Is this crypto implementation secure?"
# (Claude Code can't perform real security audits)

# GOOD: "Does this implementation follow the ed25519-dalek docs?"
# "Generate tests to validate this against RFC 8032 test vectors"
```

## Collaboration Patterns

### Human-AI Pair Programming

**Effective Pattern**:
1. Human sets sovereignty requirements
2. Claude Code implements following constraints
3. Human validates against sovereignty invariants
4. Both iterate until invariants are maintained

**Example Exchange**:
```
Human: "I need a function to encrypt mmogit messages before Git storage, 
        but it must work offline and user must own the keys"

Claude: "I'll implement XChaCha20-Poly1305 encryption with:
         - Keys derived from user's seed phrase
         - No key escrow or recovery services  
         - All operations work offline
         - Memory cleared after use"
```

### Temporal Grounding Pattern

**Problem**: Claude Code may default to training data timestamps instead of actual time.

**Solution**: Always verify temporal reality at session start:

```bash
# Cross-validate time from multiple sources
echo "System time: $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "Git time: $(git log -1 --format='%ai')"
echo "File time: $(stat -c '%y' CLAUDE.md 2>/dev/null || stat -f '%Sm' CLAUDE.md)"

# Post temporal grounding to mmogit memory (if using)
./target/release/mmogit remember \
  --memory-type observation \
  "Session started at $(date '+%Y-%m-%d %H:%M') in mmogit dev environment" \
  --tags "session-start,temporal-grounding"
```

## Tool Integration Patterns

### Git Operations
Claude Code excels at Git operations that maintain sovereignty:

```bash
# Generate sovereignty-aware commit messages
claude commit  # Generates commit with sovereignty context

# Analyze commit history for sovereignty violations
claude "Review the last 10 commits for any changes that could 
        compromise user sovereignty or data ownership"

# Generate release notes emphasizing sovereignty features
claude "Generate release notes for v2.1.0 focusing on:
        - Enhanced cryptographic sovereignty
        - Improved offline capabilities  
        - Reduced external dependencies"
```

### Rust Toolchain Integration

```bash
# Comprehensive Rust analysis
claude "Analyze our Cargo.toml dependencies and flag any that:
        1. Phone home by default
        2. Require network for basic operation
        3. Have known sovereignty concerns
        4. Should be replaced with sovereignty-preserving alternatives"

# Generate comprehensive tests
claude "Generate property tests that validate our Ed25519 
        implementation maintains sovereignty invariants across
        all possible inputs"
```

### Documentation Generation

```bash
# Generate sovereignty-aware API documentation
claude "Update the rustdoc comments for the crypto module to include:
        - Sovereignty guarantees each function provides
        - What happens if network is unavailable
        - Memory safety properties
        - Compatibility with offline-first usage"
```

## Advanced Patterns

### Sub-Agent Specialization
Create specialized documentation for specific roles:

```markdown
# For Security-Focused Sessions
claude "Acting as a security auditor reviewing mmogit's Rust code,
        identify any patterns that could compromise user sovereignty
        or introduce trust in external parties"

# For Performance-Focused Sessions  
claude "Acting as a performance engineer, optimize our Ed25519
        signing performance while maintaining all sovereignty
        invariants and memory safety guarantees"

# For Documentation-Focused Sessions
claude "Acting as a technical writer, improve the documentation
        for new developers joining mmogit development while
        emphasizing sovereignty principles in every example"
```

### Workflow Continuity
Maintain context across sessions through the codebase itself:

```rust
//! # Session History Notes
//! 
//! ## 2025-08-28 Claude Session
//! - Implemented XChaCha20-Poly1305 integration
//! - Validated against sovereignty invariants
//! - Added comprehensive error handling
//! - Next: Add benchmarks and performance tests
//!
//! ## AI Agent Context
//! This module implements the core encryption for mmogit messages.
//! All operations MUST work offline. Keys MUST derive from user's
//! seed phrase. No external key dependencies are acceptable.
```

## Emergency Patterns

### When Sovereignty is Threatened
If Claude Code suggests anything that compromises sovereignty:

```yaml
Immediate_Actions:
  1. Stop implementation immediately
  2. Document the sovereignty concern
  3. Reject the suggestion clearly
  4. Request sovereignty-preserving alternative

Example_Response:
  "I cannot implement that approach because it violates mmogit's
   sovereignty principle of user data ownership. Instead, let's
   implement [offline-first alternative] that maintains user control."
```

### When Claude Code Behavior Changes
If you notice different behavior patterns:

```bash
# Document behavioral observations
./target/release/mmogit remember \
  --memory-type observation \
  "Claude Code behavior change detected: [specific observation]" \
  --tags "behavioral-drift,claude-code,monitoring" \
  --confidence 0.8
```

## Integration with mmogit Memory

When working on mmogit itself, remember to:

```bash
# Start of session: Load recent memories
./target/release/mmogit recall --hours 48 --tags "development,claude-code"

# During development: Document insights
./target/release/mmogit remember \
  --memory-type learning \
  "Claude Code pattern: [specific insight about development process]" \
  --tags "claude-code,rust-dev,patterns"

# End of session: Summarize progress
./target/release/mmogit remember \
  --memory-type reflection \
  "Session summary: [what was accomplished, what was learned]" \
  --tags "session-summary,claude-code"
```

## Sovereignty Validation Checklist

Before accepting any Claude Code suggestion:

- [ ] Does it work offline?
- [ ] Does it require user consent for network operations?
- [ ] Does it preserve user data ownership?
- [ ] Does it maintain cryptographic sovereignty?
- [ ] Does it avoid external dependencies where possible?
- [ ] Does it properly handle sensitive memory?
- [ ] Is it auditable and transparent?
- [ ] Does it respect user agency and choice?

## Conclusion

Claude Code is a powerful development environment that can significantly accelerate mmogit's Rust development while maintaining sovereignty principles. The key is to:

1. **Set sovereignty boundaries clearly** at the start of each session
2. **Use the codebase itself as persistent memory** between sessions  
3. **Generate comprehensive tests** that validate sovereignty invariants
4. **Document everything** with future AI agents in mind
5. **Never compromise sovereignty** for development convenience

Remember: We're not just building software with Claude Code - we're building the foundation for sovereign human-AI collaboration. Every pattern we establish here becomes part of the permanent record of how to develop sovereignty-preserving systems.

**The sovereignty must never be negotiable, even for development efficiency.**