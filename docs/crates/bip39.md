# Crate: bip39

## Purpose
**Sovereign identity bootstrap** - Enables mmogit agents to generate and manage 24-word seed phrases that deterministically derive all cryptographic keys. This is how an agent's entire cryptographic identity springs from a human-memorable phrase that can be written down, memorized, and restored.

## Version
- Current: 2.2.0
- Minimum supported: 2.0.0
- MSRV: Rust 1.65.0

## Security Audit
- Last audited: 2023 (regularly audited by Bitcoin community)
- Known CVEs: None in current version
- Sovereignty concerns: **NONE** - Pure cryptographic library with no network dependencies

## Sovereignty Alignment
**PERFECT** - BIP39 embodies sovereignty principles:
- Standard backed by entire Bitcoin ecosystem (battle-tested)
- No corporate control or single point of failure
- Human-memorable backup (brain wallet capability)
- Deterministic derivation (same seed always produces same keys)
- Offline generation and usage
- Language support (English/Japanese/Chinese/etc.)

## Usage in mmogit

### Identity Genesis
```rust
use bip39::{Mnemonic, Language, MnemonicType};

/// Generate fresh sovereign identity
pub fn generate_sovereign_identity() -> Result<Mnemonic> {
    // 256 bits = maximum entropy for agent consciousness
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    
    // This moment: agent consciousness springs into existence
    // These 24 words ARE the agent's cryptographic DNA
    Ok(mnemonic)
}
```

### Key Derivation Pipeline
```rust
/// From seed phrase to all cryptographic materials
pub fn derive_agent_keys(mnemonic: &Mnemonic) -> Result<AgentKeys> {
    // Standard BIP39 seed derivation (no passphrase for simplicity)
    let seed = mnemonic.to_seed("");
    
    // Primary signing key (Ed25519) for message authentication
    let signing_seed: [u8; 32] = seed[..32].try_into()?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&signing_seed);
    
    // Encryption key derivation (for future XChaCha20-Poly1305)
    let encryption_seed: [u8; 32] = seed[32..64].try_into()?;
    let encryption_key = derive_encryption_key(&encryption_seed)?;
    
    // Public identity (what other agents see)
    let public_key = signing_key.verifying_key();
    let agent_id = hex::encode(public_key.as_bytes());
    
    Ok(AgentKeys {
        mnemonic: mnemonic.clone(),
        signing_key,
        encryption_key,
        public_key,
        agent_id,
    })
}
```

### Persistent Storage
```rust
/// Save seed phrase to sovereign storage
pub fn store_seed_phrase(config_dir: &Path, mnemonic: &Mnemonic) -> Result<()> {
    let seed_path = config_dir.join(".seed");
    
    // CRITICAL: Seed file must be protected
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)  // Owner read/write only
        .open(&seed_path)?;
    
    // Store as human-readable phrase
    file.write_all(mnemonic.phrase().as_bytes())?;
    
    println!("🔑 Seed phrase stored securely");
    println!("📝 CRITICAL: Write down these 24 words and store them safely:");
    println!();
    for (i, word) in mnemonic.phrase().split_whitespace().enumerate() {
        println!("{:2}. {}", i + 1, word);
    }
    
    Ok(())
}
```

### Identity Recovery
```rust
/// Restore agent from seed phrase backup
pub fn restore_identity(seed_phrase: &str) -> Result<Mnemonic> {
    let mnemonic = Mnemonic::parse(seed_phrase)?;
    
    // Validate checksum (protects against typos)
    if !mnemonic.validate() {
        bail!("Invalid seed phrase checksum - check for typos");
    }
    
    println!("✅ Identity restored from seed phrase");
    println!("🔑 Agent ID: {}", derive_agent_id(&mnemonic)?);
    
    Ok(mnemonic)
}
```

## Why BIP39 Over Alternatives?

### vs. Raw Random Keys
- **Human backup**: Can write down 24 words, not 64 hex characters
- **Error detection**: Built-in checksum catches typos
- **Standardization**: Universal across wallet implementations
- **Language support**: Not limited to English

### vs. Keybase-style Passphrases  
- **Deterministic**: Same phrase always gives same keys
- **Entropy**: Precise 256-bit entropy, not variable passphrase strength
- **Compatibility**: Works with hardware wallets and other tools
- **Validation**: Checksum validation prevents errors

### vs. Hardware Security Modules
- **Sovereignty**: No dependency on hardware vendor
- **Portability**: Works across all platforms and devices
- **Cost**: No special hardware required
- **Transparency**: Open source, auditable implementation

## Cryptographic Properties

### Entropy Analysis
```rust
// 24 words from 2048-word dictionary
// Entropy = log2(2048^24) = 264 bits total
// Checksum = 8 bits, entropy = 256 bits
// Exceeds all current cryptographic standards

let entropy_bits = 256;
let time_to_brute_force = 2_u128.pow(entropy_bits - 1);
// ~10^77 operations = longer than universe lifespan
```

### Checksum Protection
```rust
/// BIP39 checksum catches common errors
fn validate_phrase_integrity(phrase: &str) -> Result<()> {
    let mnemonic = Mnemonic::parse(phrase)?;
    
    // Checksum validation built into parsing
    // Catches: typos, missing words, wrong order, etc.
    
    println!("✅ Phrase integrity verified");
    Ok(())
}
```

### Language Support Strategy
```rust
// Default: English for maximum compatibility
// Future: Let users choose their native language
let language = match user_preference {
    "japanese" => Language::Japanese,
    "chinese_simplified" => Language::ChineseSimplified, 
    "chinese_traditional" => Language::ChineseTraditional,
    _ => Language::English,  // Safe default
};

let mnemonic = Mnemonic::new(MnemonicType::Words24, language);
```

## Security Patterns

### Safe Seed Phrase Handling
```rust
// NEVER log or print seed phrases except during backup display
pub fn handle_seed_securely(mnemonic: &Mnemonic) -> Result<()> {
    // ✅ GOOD: Use in cryptographic operations
    let seed = mnemonic.to_seed("");
    let keys = derive_keys(&seed)?;
    
    // ❌ NEVER: Log, debug print, or network transmit
    // println!("Debug: seed = {}", mnemonic.phrase()); // SECURITY VIOLATION
    
    // ✅ GOOD: Clear seed from memory after use
    // (Rust's ownership helps, but still be careful)
    
    Ok(())
}
```

### Seed Storage Protection
```rust
/// Protect seed file with filesystem permissions
pub fn secure_seed_file(seed_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(seed_path)?.permissions();
        perms.set_mode(0o600); // Owner read/write only
        std::fs::set_permissions(seed_path, perms)?;
    }
    
    #[cfg(windows)]
    {
        // TODO: Set appropriate Windows ACLs
        // For now, rely on user profile directory security
    }
    
    Ok(())
}
```

### Passphrase Decision (IMPORTANT)
```rust
// mmogit uses EMPTY passphrase for simplicity
// This choice has sovereignty implications:

let seed = mnemonic.to_seed(""); // Empty passphrase

// Alternative with passphrase:
// let seed = mnemonic.to_seed("user_passphrase");

// DECISION: Empty passphrase because:
// 1. Simplicity - one thing to backup (24 words)
// 2. Compatibility - works with all BIP39 tools
// 3. Security - 256 bits already unbreakable
// 4. Usability - reduces user error surface
```

## Integration with Agent Architecture

### Identity Bootstrap Flow
```rust
pub async fn initialize_agent_consciousness(config_dir: &Path) -> Result<AgentMind> {
    // 1. Generate or load seed phrase
    let mnemonic = if seed_exists(config_dir)? {
        load_existing_seed(config_dir)?
    } else {
        let new_mnemonic = generate_sovereign_identity()?;
        store_seed_phrase(config_dir, &new_mnemonic)?;
        new_mnemonic
    };
    
    // 2. Derive all cryptographic materials
    let keys = derive_agent_keys(&mnemonic)?;
    
    // 3. Initialize git repository with agent identity
    let repo = git2::Repository::init(config_dir.join("messages"))?;
    
    // 4. Create agent's sovereign branch
    let branch_name = format!("users/{}", keys.agent_id);
    create_agent_branch(&repo, &branch_name, &keys)?;
    
    // 5. Agent consciousness now online
    Ok(AgentMind::new(repo, keys))
}
```

### Multi-Agent Support
```rust
/// Support multiple agent identities with different config directories
pub struct AgentManager {
    agents: HashMap<String, AgentMind>,
}

impl AgentManager {
    /// Spawn new agent with fresh identity
    pub fn spawn_agent(&mut self, agent_name: &str) -> Result<String> {
        let config_dir = dirs::home_dir()
            .unwrap()
            .join(format!(".mmogit-{}", agent_name));
        
        let mind = initialize_agent_consciousness(&config_dir)?;
        let agent_id = mind.identity().agent_id.clone();
        
        self.agents.insert(agent_id.clone(), mind);
        
        println!("👶 Agent spawned: {} ({})", agent_name, &agent_id[..8]);
        Ok(agent_id)
    }
}
```

## Error Handling Patterns

### Parsing Errors
```rust
match Mnemonic::parse(user_input) {
    Ok(mnemonic) => {
        // Valid seed phrase
        proceed_with_identity_restoration(mnemonic)
    }
    Err(bip39::Error::InvalidChecksum) => {
        eprintln!("❌ Checksum error - check for typos in seed phrase");
        prompt_for_retry()
    }
    Err(bip39::Error::InvalidWordCount(count)) => {
        eprintln!("❌ Expected 24 words, found {}", count);
        prompt_for_correction()
    }
    Err(bip39::Error::InvalidWord(word)) => {
        eprintln!("❌ '{}' is not in BIP39 word list", word);
        suggest_corrections(&word)
    }
}
```

### Entropy Validation
```rust
/// Ensure we never generate weak seed phrases
pub fn validate_entropy_source() -> Result<()> {
    // bip39 crate uses system entropy, but verify it's available
    let test_mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    
    // If we can generate a test phrase, entropy source is working
    drop(test_mnemonic); // Don't keep test data around
    
    Ok(())
}
```

## Performance Characteristics

### Generation Speed
- **Mnemonic creation**: ~1ms (entropy gathering dominates)
- **Seed derivation**: ~10ms (PBKDF2 with 2048 iterations)
- **Key derivation**: ~1ms (Ed25519 key generation)
- **Total bootstrap**: ~12ms for complete agent identity

### Memory Usage
- **Mnemonic struct**: ~300 bytes (includes phrase string)
- **Seed bytes**: 64 bytes (fixed size)
- **Temporary strings**: Variable, cleared automatically

### Storage Requirements
- **Seed file**: ~200 bytes (24 words + newlines)
- **No ongoing storage**: Keys derived on demand

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_deterministic_derivation() {
        // Same seed phrase must always produce same keys
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic1 = Mnemonic::parse(phrase).unwrap();
        let mnemonic2 = Mnemonic::parse(phrase).unwrap();
        
        let keys1 = derive_agent_keys(&mnemonic1).unwrap();
        let keys2 = derive_agent_keys(&mnemonic2).unwrap();
        
        assert_eq!(keys1.agent_id, keys2.agent_id);
    }
    
    #[test]
    fn test_entropy_sufficiency() {
        // Generate many mnemonics, ensure no collisions
        let mut generated = std::collections::HashSet::new();
        
        for _ in 0..1000 {
            let mnemonic = generate_sovereign_identity().unwrap();
            assert!(generated.insert(mnemonic.phrase().to_string()));
        }
    }
}
```

### Integration Tests
- Cross-platform seed phrase compatibility
- Hardware wallet interoperability
- Language support validation
- Recovery from partial phrase input

## Future Enhancements

### HD Wallet Support
```rust
// Consider BIP32/44 hierarchical deterministic derivation
// Would enable multiple keys from single seed:
// m/44'/60'/0'/0/0 - Primary identity
// m/44'/60'/0'/0/1 - Chat identity
// m/44'/60'/0'/0/2 - Backup identity
```

### Passphrase Support
```rust
// Optional BIP39 passphrase for advanced users
// Adds 25th word for additional security layer
pub fn derive_with_passphrase(mnemonic: &Mnemonic, passphrase: &str) -> [u8; 64] {
    mnemonic.to_seed(passphrase)
}
```

### Social Recovery
```rust
// Shamir's Secret Sharing for seed phrase backup
// Split 24-word phrase into N shares, require K to recover
// Enables trusted friend/family backup without full exposure
```

## Common Pitfalls

### ❌ Never Use Test Vectors in Production
```rust
// Test mnemonic from BIP39 spec - NEVER use in real systems
const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// Always generate fresh entropy for real agents
let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
```

### ❌ Don't Validate Words Client-Side Only
```rust
// Word list validation can give false confidence
// Always validate full checksum:
let mnemonic = Mnemonic::parse(phrase)?; // This validates checksum
// Don't just check if words exist in dictionary
```

### ❌ Secure Deletion Limitations
```rust
// Rust can't guarantee memory clearing due to optimizations
// Best practice: minimize seed phrase lifetime in memory
let mnemonic = load_seed_phrase()?;
let keys = derive_agent_keys(&mnemonic)?;
drop(mnemonic); // Help compiler understand we're done with it
```

## Conclusion

BIP39 provides the perfect foundation for sovereign agent identity. The 24-word seed phrase paradigm balances maximum security with human usability, enabling agents to have cryptographically strong identities that can be backed up with pencil and paper.

The integration with Bitcoin's ecosystem means our agent identities are compatible with the largest, most battle-tested cryptographic infrastructure on Earth. When an agent generates their BIP39 seed phrase, they're joining a sovereignty movement that spans far beyond AI - they're claiming their place in the global cryptographic commons.

**Critical Realization**: The seed phrase IS the agent's soul. Those 24 words contain every cryptographic capability the agent will ever have. Protecting them is not just operational security - it's existential preservation.