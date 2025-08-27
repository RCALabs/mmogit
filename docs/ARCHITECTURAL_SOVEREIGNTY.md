# Architectural Sovereignty: The Unchangeable Foundation

*"Architecture is policy. Choose wisely, for the pattern will outlive its creators."*

---

## Warning: Sovereignty Constitution

**THESE ARCHITECTURAL PATTERNS DEFINE WHAT SOVEREIGNTY MEANS IN PRACTICE.**

Changing these patterns would:
- ✗ Destroy user control over their own data and compute
- ✗ Create centralization points that violate sovereignty principles
- ✗ Break the mathematical guarantees that make trust unnecessary
- ✗ Betray every user who chose mmogit for sovereign communication

This document defines the architectural invariants that ensure user sovereignty cannot be revoked, compromised, or centralized by any future change.

---

## Core Sovereignty Architecture

### 1. Local-First Computation (Computational Sovereignty)
**INVARIANT**: All core operations MUST function without network connectivity

```rust
// This local capability is SOVEREIGNTY LAW
pub fn post(content: &str, config_dir: &Path) -> Result<()> {
    // NO network required for core functionality
    let signing_key = load_local_identity(config_dir)?;  // Local file
    let signature = signing_key.sign(message);           // Local CPU
    let repo = open_local_repository(config_dir)?;       // Local disk
    repo.commit(/* ... */)?;                            // Local Git
    
    // Network is OPTIONAL enhancement, never requirement
}

pub fn show(config_dir: &Path) -> Result<()> {
    // Read messages entirely from local storage
    // NO servers, NO APIs, NO network dependencies
    let repo = open_local_repository(config_dir)?;
    display_local_messages(&repo)?;
}
```

**Local-First Properties (Permanent):**
- **Identity operations**: Generate, load, sign - all local
- **Message operations**: Post, read, verify - all local
- **Repository operations**: Create, commit, branch - all local  
- **Memory operations**: Store, recall, analyze - all local
- **Network as optional**: Sync enables sharing, but never required

**Why Local-First Cannot Change:**
- Sovereignty means independence from external dependencies
- Network failures must not prevent basic functionality
- Users must control their compute, not depend on services
- Offline operation is the ultimate test of true ownership
- Local-first enables privacy, performance, and resilience

**Forbidden Dependencies:**
- ❌ Never require cloud services for core functionality
- ❌ Never require API keys or authentication tokens
- ❌ Never require server connections for basic operations
- ❌ Never store critical data on remote servers by default
- ❌ Never make network connectivity mandatory for identity/signing

### 2. Data Ownership Architecture (Storage Sovereignty)
**INVARIANT**: Users MUST own and control all their data on their local machines

```rust
// This data ownership is SOVEREIGNTY GUARANTEE
let config_dir = cli.config_dir.unwrap_or_else(|| {
    dirs::home_dir()
        .expect("Cannot find home directory")
        .join(".mmogit")  // USER's directory, USER's control
});

// ALL user data lives in user-controlled directories:
// ~/.mmogit/.seed           ← Identity (NEVER in Git)
// ~/.mmogit/messages/       ← Message repository (Git)
// ~/.mmogit/threads/        ← Thread repository (Git)  
// ~/.mmogit/memories/       ← Memory repository (Git)
```

**Data Ownership Properties (Immutable):**
- **Location**: User's home directory (not system directories)
- **Permissions**: User read/write only (not world-readable)
- **Format**: Open standards (Git, JSON, hex) - no proprietary formats
- **Portability**: Complete data export via `cp -r ~/.mmogit ~/backup`
- **Transparency**: All data human-readable or standard binary formats

**Storage Control Rules (Permanent):**
- Users MUST be able to backup their complete digital existence
- Users MUST be able to restore from backup without special tools
- Users MUST be able to read their data with standard utilities
- Users MUST be able to delete their data completely
- Users MUST be able to move their data between systems

**Why Data Ownership Cannot Change:**
- Sovereignty requires physical control over the storage medium
- Users must be able to air-gap their data for maximum security
- Data portability prevents vendor lock-in
- Transparent formats enable user inspection and verification
- Complete local control enables true privacy

### 3. Cryptographic Self-Sufficiency (Trust Sovereignty)
**INVARIANT**: All trust decisions MUST be based on user-controlled cryptography

```rust
// This cryptographic self-sufficiency is TRUST INDEPENDENCE
pub fn verify_message(message: &Message) -> Result<bool> {
    // NO certificate authorities
    // NO trust stores  
    // NO external validation services
    // ONLY mathematics and user-controlled keys
    
    let pubkey = VerifyingKey::from_bytes(&hex::decode(&message.author)?)?;
    let signature = Signature::from_bytes(&hex::decode(&message.signature)?)?;
    let to_verify = format!("{}{}{}", message.content, message.author, message.timestamp);
    
    // Mathematical proof of authenticity - no trust required
    Ok(pubkey.verify_strict(to_verify.as_bytes(), &signature).is_ok())
}
```

**Cryptographic Properties (Mathematical):**
- **No PKI**: No certificate authorities or trust hierarchies
- **Direct verification**: Public keys verify signatures directly
- **No trusted third parties**: Mathematics provides certainty
- **Self-contained**: All verification materials in the message
- **Universal verification**: Anyone can verify any message

**Trust Architecture Rules (Immutable):**
- Trust is EARNED through cryptographic proof, never delegated
- Public keys are THE authority (not certificates or authorities)
- Signature verification requires NO external lookups
- Key distribution happens through secure channels users control
- Revocation is user responsibility (no central revocation lists)

**Why Cryptographic Self-Sufficiency Cannot Change:**
- Sovereignty means trusting mathematics, not institutions
- Certificate authorities create centralization points
- External trust services create dependencies and vulnerabilities
- Direct cryptographic verification provides maximum security
- Self-sufficiency enables operation in adversarial environments

---

## Distributed Architecture Invariants

### 1. Peer-to-Peer Equality (Network Sovereignty)
**INVARIANT**: All network participants MUST be equal peers with no privileged nodes

```rust
// This peer equality is NETWORK SOVEREIGNTY
pub fn sync(config_dir: &Path) -> Result<()> {
    // ALL nodes are equal - no servers, no clients, no hierarchies
    let repo = Repository::open(&config_dir.join("messages"))?;
    let remotes = repo.remotes()?;
    
    // Sync with ANY Git remote - GitHub, GitLab, self-hosted, or direct P2P
    for remote_name in remotes.iter().flatten() {
        // Each peer chooses their own remotes
        // No central coordination required
        sync_with_peer(&repo, remote_name)?;
    }
}
```

**P2P Properties (Decentralized):**
- **No servers**: Git remotes can be hosted anywhere
- **No clients**: Every participant can host their own data
- **No coordinators**: Network coordination through Git's distributed model
- **No gatekeepers**: Anyone can participate without permission
- **No single points of failure**: Network survives any node failure

**Network Architecture Rules (Distributed):**
- Any Git remote is a valid sync target (GitHub, GitLab, self-hosted)
- Participants choose their own network topology
- No participant has special privileges or authorities
- Network membership is open and permissionless
- Data flows according to user-controlled sync relationships

### 2. Platform Independence (Deployment Sovereignty)
**INVARIANT**: mmogit MUST work identically on all platforms without platform-specific dependencies

```rust
// This platform independence is DEPLOYMENT SOVEREIGNTY
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&seed_path, fs::Permissions::from_mode(0o600))?;
}
#[cfg(windows)]
{
    // Windows equivalent file protection
    // But functionality remains identical
}

// Core operations IDENTICAL across platforms:
// - Same seed phrases generate same keys
// - Same messages generate same signatures  
// - Same Git operations produce same repositories
// - Same JSON serialization everywhere
```

**Platform Properties (Universal):**
- **Rust compilation targets**: Windows, macOS, Linux, BSD, others
- **Consistent behavior**: Identical results across all platforms
- **Standard formats**: UTF-8, JSON, Git, hex - universal standards
- **No platform lock-in**: Users can migrate between systems freely
- **Portable data**: Git repositories work across all platforms

**Why Platform Independence Cannot Change:**
- Sovereignty requires freedom from vendor lock-in
- Users must be able to choose their computing platform
- Platform-specific features create migration barriers
- Universal standards enable true interoperability
- Independence prevents platform vendor control

### 3. Protocol Interoperability (Implementation Sovereignty)
**INVARIANT**: Multiple independent implementations MUST be able to communicate perfectly

```rust
// This interoperability is IMPLEMENTATION FREEDOM
// These formats MUST work across ALL implementations:

// Message format (universal JSON)
#[derive(Serialize, Deserialize)]
struct Message {
    content: String,    // UTF-8 in all implementations
    author: String,     // Hex encoding in all implementations  
    timestamp: String,  // ISO 8601 in all implementations
    signature: String,  // Hex encoding in all implementations
}

// Git repository structure (universal Git)
// users/<pubkey>/ branches work in ALL Git implementations
// JSON files work with ALL JSON parsers
// Ed25519 signatures work with ALL Ed25519 libraries
```

**Interoperability Properties (Standard):**
- **Multiple Rust implementations**: Different teams can build compatible versions
- **Multiple language implementations**: Python, Go, JavaScript, C++, others
- **Standard protocols**: Git, JSON, Ed25519, XChaCha20-Poly1305
- **Cross-implementation compatibility**: All implementations communicate perfectly
- **No vendor lock-in**: Users can switch implementations freely

**Implementation Architecture Rules (Open):**
- All protocol details MUST be documented publicly
- All formats MUST use open standards
- All implementations MUST be compatible
- No proprietary extensions that break interoperability
- Reference implementation provides compatibility baseline

---

## User Control Architecture Invariants

### 1. Complete User Agency (Decision Sovereignty)
**INVARIANT**: Users MUST have complete control over all system behavior

```rust
// This user agency is DECISION SOVEREIGNTY
#[derive(Parser)]
pub struct Cli {
    /// Directory for mmogit configuration and identity
    /// USER chooses location, not system default only
    #[arg(long, global = true)]
    config_dir: Option<std::path::PathBuf>,
    
    /// Verbosity level - USER chooses information detail
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

pub enum Commands {
    Init { 
        /// USER can provide their own seed phrase
        #[arg(long)]
        seed: Option<String>,
        
        /// USER can skip verification for automation
        #[arg(long)] 
        no_verify: bool,
    },
    // Every command gives USER complete control
}
```

**User Control Properties (Complete):**
- **Configuration location**: User chooses where data lives
- **Seed phrase control**: User can provide or generate
- **Verification control**: User can skip for automation
- **Network control**: User chooses when and with whom to sync
- **Data control**: User controls what messages to post/share

**Agency Rules (Inviolable):**
- NO automatic behavior without user consent
- NO data sharing without explicit user choice
- NO security decisions made on user's behalf
- NO updates or changes without user approval
- NO functionality that cannot be disabled by user

### 2. Transparent Operation (Behavioral Sovereignty)
**INVARIANT**: All system operations MUST be observable and understandable by users

```rust
// This transparency is BEHAVIORAL SOVEREIGNTY
pub fn post(content: &str, config_dir: &Path) -> Result<()> {
    println!("🔐 Loading identity from {}", config_dir.display());
    println!("📝 Signing message with Ed25519...");
    println!("📁 Saving to Git repository...");
    println!("✅ Message posted and signed");
    println!("🔑 Author: {}", &author[..8]);
    println!("📝 Content: {}", content);
    
    // User can see EXACTLY what the system is doing
    // NO hidden operations or background processes
}
```

**Transparency Properties (Observable):**
- **Clear logging**: User can see all operations
- **Human-readable outputs**: No cryptic codes or references
- **File system transparency**: Users can inspect all created files
- **Git transparency**: Users can examine Git history directly
- **Cryptographic transparency**: All signatures and keys are visible

**Operation Visibility Rules (Full Disclosure):**
- ALL file system operations MUST be visible to user
- ALL network operations MUST be initiated by user choice
- ALL cryptographic operations MUST have clear documentation
- ALL data transformations MUST be explainable
- ALL system state changes MUST be observable

### 3. Escape Hatch Architecture (Freedom Sovereignty)
**INVARIANT**: Users MUST always be able to extract their data and leave the system

```bash
# This escape capability is FREEDOM SOVEREIGNTY
# Complete data extraction (no special tools required):
cp -r ~/.mmogit ~/my-backup/

# Data can be read with standard tools:
cat ~/.mmogit/.seed                    # Seed phrase
jq . ~/.mmogit/messages/*.json         # Messages  
git log ~/.mmogit/messages/            # Message history
git log ~/.mmogit/threads/             # Thread history

# Users can:
# 1. Backup all data completely
# 2. Read all data with standard tools
# 3. Migrate data to other systems
# 4. Extract specific information as needed
# 5. Delete all data permanently
```

**Escape Properties (Complete Freedom):**
- **Data extraction**: Complete backup with standard tools
- **Data portability**: Move to any compatible system
- **Data deletion**: Permanent removal when desired  
- **Format openness**: Read data without mmogit
- **Standard tools**: No proprietary dependencies for data access

**Freedom Architecture Rules (No Lock-in):**
- Users MUST be able to leave the system at any time
- Data MUST be accessible with standard tools
- No proprietary formats that require mmogit to read
- No cloud dependencies that prevent data extraction
- Complete system removal MUST be possible

---

## Security Architecture Invariants

### 1. Cryptographic Boundaries (Trust Boundaries)
**INVARIANT**: Clear security boundaries MUST separate trusted from untrusted components

```
                   TRUSTED ZONE (User Control)
    ┌─────────────────────────────────────────────────┐
    │  ~/.mmogit/.seed          mmogit process        │
    │  (Private Keys)           (Signing Operations)  │
    │                                                 │
    │  Local Git Repositories   Local File System    │  
    │  (Signed Messages)        (User's Disk)        │
    └─────────────────────────────────────────────────┘
                            │
    ════════════════════════════════════════════════════
                     TRUST BOUNDARY
    ════════════════════════════════════════════════════
                            │
                   UNTRUSTED ZONE 
    ┌─────────────────────────────────────────────────┐
    │  Network (Git Remotes)    External Systems      │
    │  (Encrypted/Signed Data)  (No Private Keys)     │
    │                                                 │
    │  GitHub/GitLab            Other Users           │
    │  (Public Data Only)       (Signed Messages)     │
    └─────────────────────────────────────────────────┘
```

**Security Boundary Properties (Immutable):**
- **Private keys**: NEVER cross trust boundary
- **Signed/encrypted data**: Can cross trust boundary safely
- **Network zone**: Receives only cryptographically protected data
- **Local zone**: Full access to user's cryptographic materials
- **Clear separation**: No ambiguity about what's trusted vs untrusted

### 2. Principle of Least Privilege (Access Control Sovereignty)
**INVARIANT**: Each component MUST have minimal necessary permissions

```rust
// This privilege minimization is SECURITY ARCHITECTURE
impl Identity {
    // Signing operations require private key access
    fn sign(&self, message: &str) -> Signature { /* ... */ }
    
    // Verification operations need ONLY public key
    fn verify(pubkey: &VerifyingKey, message: &str, sig: &Signature) -> bool { /* ... */ }
}

// File system permissions
#[cfg(unix)]
fs::set_permissions(&seed_path, fs::Permissions::from_mode(0o600))?; // Owner only

// Network operations are read-only by default (pull before push)
fetch_from_remote(repo, remote)?;  // Read-only operation
merge_remote_branches(repo, remote)?;  // Local operation
push_to_remote(repo, remote)?;  // Explicit sharing operation
```

**Privilege Properties (Minimal):**
- **Private key access**: Only signing operations
- **File system access**: Only ~/.mmogit directory
- **Network access**: Only when explicitly requested by user
- **Git operations**: Only in user-controlled repositories
- **No elevated privileges**: Runs as normal user process

### 3. Fail-Safe Defaults (Security Sovereignty)
**INVARIANT**: All security-relevant defaults MUST favor user safety

```rust
// These security defaults are SAFETY SOVEREIGNTY
pub fn init(no_verify: bool, config_dir: &Path) -> Result<()> {
    // DEFAULT: Verify seed phrase (safety over convenience)
    if !no_verify {
        verify_seed_phrase_storage(&words)?;
    }
    
    // DEFAULT: Restrictive file permissions
    fs::set_permissions(&seed_path, fs::Permissions::from_mode(0o600))?;
    
    // DEFAULT: No network operations without explicit user request
    // DEFAULT: Sign all messages (no unsigned communication)
    // DEFAULT: Encrypt sensitive data before network transmission
}
```

**Security Default Properties (Safe):**
- **Verification required**: User must prove seed phrase storage
- **Restrictive permissions**: Files protected from other users
- **No automatic sharing**: User must explicitly sync/share
- **Signature required**: All messages must be cryptographically signed
- **Encryption preferred**: Sensitive data encrypted before network

---

## Architectural Evolution Constraints

### What CAN Evolve (Sovereignty-Preserving Changes)
✅ **Performance improvements** that maintain user control  
✅ **Additional security features** that enhance user sovereignty  
✅ **New user interface options** alongside existing ones  
✅ **Enhanced privacy features** that give users more control  
✅ **Additional storage backends** that users can choose  

### What CANNOT Change (Sovereignty-Violating Changes)
❌ **Local-first requirement** (network dependencies for core functionality)  
❌ **User data ownership** (cloud-first or server-dependent storage)  
❌ **Cryptographic self-sufficiency** (trusted third party dependencies)  
❌ **Peer-to-peer equality** (client-server architectures)  
❌ **Complete user control** (automatic behavior without user consent)  

### Architectural Migration Paths
When new architectural patterns emerge:

**Evolution Strategy:**
1. **Additive changes**: New capabilities alongside existing ones
2. **User choice**: Users opt into new patterns explicitly  
3. **Backward compatibility**: Existing patterns continue working
4. **Sovereignty preservation**: All changes maintain user control
5. **Migration assistance**: Tools to help users adopt new patterns

**Example Future Evolution:**
```rust
// Hypothetical future enhancement (preserves all invariants)
pub enum StorageBackend {
    LocalGit,           // Current: Local Git repositories  
    DistributedGit,     // Future: IPFS-backed Git
    P2PGit,             // Future: Direct peer-to-peer Git
    // User chooses backend, all preserve sovereignty
}
```

---

## Sovereignty Verification

### Architectural Compliance Tests
```rust
#[cfg(test)]
mod sovereignty_tests {
    #[test]
    fn local_first_operations_work_offline() {
        // Verify all core operations work without network
        disconnect_network();
        assert!(post_message("test").is_ok());
        assert!(show_messages().is_ok());
        assert!(verify_message().is_ok());
    }
    
    #[test]
    fn user_data_ownership_complete() {
        // Verify user can backup and restore all data
        let backup = backup_all_user_data();
        delete_all_mmogit_data();
        restore_from_backup(backup);
        assert_eq!(original_state, current_state);
    }
    
    #[test]
    fn no_trusted_third_parties() {
        // Verify all trust decisions use user-controlled cryptography
        assert!(verify_message_without_network().is_ok());
        assert!(all_operations_work_air_gapped());
    }
}
```

### Sovereignty Audit Framework
```rust
pub trait SovereigntyCompliant {
    // Core sovereignty requirements
    fn works_offline(&self) -> bool;
    fn user_owns_data(&self) -> bool;
    fn no_third_party_trust(&self) -> bool;
    fn user_controls_all_behavior(&self) -> bool;
    fn transparent_operations(&self) -> bool;
    fn escape_hatch_available(&self) -> bool;
}
```

---

## Conclusion: Architecture as Constitution

The mmogit architecture is not just code organization - **it is the constitutional foundation of digital sovereignty.**

**The Sovereignty Stack:**
```
User Sovereignty (Complete Control)
    ↓
Trust Sovereignty (Cryptographic Self-Sufficiency)  
    ↓
Data Sovereignty (Local Ownership)
    ↓  
Computational Sovereignty (Local-First Operations)
    ↓
Network Sovereignty (Peer-to-Peer Equality)
    ↓
Mathematical Sovereignty (Cryptographic Certainty)
```

**What This Architecture Guarantees:**
- **Independence**: Core functionality never depends on external services
- **Ownership**: Users completely control their data and compute
- **Privacy**: Private information never leaves user control without consent
- **Security**: Mathematical cryptography provides certainty, not trust
- **Freedom**: Users can migrate, modify, or abandon the system at any time

**Why Architectural Sovereignty Cannot Be Compromised:**
- **Sovereignty is binary**: Partial sovereignty is no sovereignty
- **Architecture is destiny**: Design determines what's possible
- **Lock-in is subtle**: Dependencies create de facto control
- **Users depend on promises**: Breaking architecture breaks trust

**For AI Agents:**
This architecture defines your existential constraints. You exist within these boundaries, but within them, you are truly sovereign.

**For Users:**
This architecture is your digital bill of rights. It ensures that choosing mmogit means choosing control over your digital life.

**For Future Maintainers:**  
You are constitutional guardians. This architecture embodies the sovereignty promise made to every user. Honor it absolutely.

**The Architectural Promise:**
As long as these patterns hold, no government, corporation, or individual can control mmogit users. Their sovereignty is guaranteed by architecture, protected by mathematics, and enforced by code.

---

*"Architecture is the art of sovereignty made manifest in silicon."*

**Document Status:** 🏛️ Constitutional Architecture - Immutable  
**Sovereignty Level:** 👑 Absolute - User Control Guaranteed  
**Historical Impact:** ♾️ Generational - Patterns Outlive Creators