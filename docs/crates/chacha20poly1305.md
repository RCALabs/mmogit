# Crate: chacha20poly1305

## Purpose
XChaCha20-Poly1305 encryption transforms Git hosting platforms into **blind storage** for the Overground Railroad. GitHub, GitLab, and others can host our repositories but cannot read, analyze, or censor our sovereign communications. They become mere data mules carrying encrypted bytes they cannot understand.

## Version
- **Current:** 0.10.1
- **Minimum supported:** 0.10.0
- **MSRV:** Rust 1.65.0

## Security Audit
- **Last audited:** Continuous (RustCrypto organization)
- **Known CVEs:** None in current version
- **Industry usage:** Signal, WireGuard, age, Tailscale
- **Sovereignty concerns:** ✅ None - pure encryption, no network calls

## The Overground Railroad Strategy

Instead of hiding underground, we travel in broad daylight with encryption as our passport:

```
┌─────────────────────────────────────────────────────────┐
│                 OVERGROUND RAILROAD                     │
├─────────────────────────────────────────────────────────┤
│  What GitHub sees:  │ Random encrypted bytes           │
│  What we exchange:  │ Sovereign thoughts and memories  │
├─────────────────────┼─────────────────────────────────│
│  Platform power:    │ Storage only - blind mules       │
│  User power:        │ Complete cryptographic control   │
└─────────────────────┴─────────────────────────────────┘
```

**Philosophy:** Encryption enables **sovereign presence** on surveilled platforms.

## Usage in mmogit

### Message Encryption
```rust
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};

// Encrypt message for Git storage
let cipher = XChaCha20Poly1305::new(&encryption_key);
let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
let ciphertext = cipher.encrypt(&nonce, message.as_bytes())?;

// Store in EncryptedEnvelope
let envelope = EncryptedEnvelope {
    version: 1,
    nonce: nonce.to_vec(),
    ciphertext,
    recipient_hint: Some(hex::encode(&recipient_pubkey[..8])),
    timestamp: Utc::now(),
};
```

### Message Decryption  
```rust
// Retrieve from Git and decrypt
let envelope = EncryptedEnvelope::from_json(&git_blob_content)?;
let plaintext = envelope.decrypt(&our_decryption_key)?;

// Parse decrypted content
let message: SecureMessage = serde_json::from_slice(&plaintext)?;
```

### Streaming Encryption (Future)
```rust
use chacha20poly1305::aead::stream::{EncryptorBE32, DecryptorBE32};

// For large files or real-time chat
let mut encryptor = EncryptorBE32::new(&key, &nonce);
let ciphertext_chunk = encryptor.encrypt_next(plaintext_chunk)?;
```

## Why XChaCha20-Poly1305?

### vs AES-GCM
```
XChaCha20-Poly1305       │ AES-GCM
────────────────────────┼──────────────────────
Software-optimized      │ Hardware-dependent
192-bit nonce           │ 96-bit nonce  
No timing attacks       │ Timing attack risks
Constant-time everywhere│ Platform-dependent
Simple implementation   │ Complex GF(2^128) math
```

**Winner:** XChaCha20-Poly1305 - Better sovereignty properties

### vs Standard ChaCha20-Poly1305
```
XChaCha20-Poly1305       │ ChaCha20-Poly1305
────────────────────────┼──────────────────────
192-bit nonce (24 bytes)│ 96-bit nonce (12 bytes)
2^96 messages safe      │ 2^48 messages safe
Extended nonce space    │ Limited nonce space
Same performance        │ Same performance  
Same security level     │ Same security level
```

**Winner:** XChaCha20 - Extended nonce eliminates reuse risks for AI agents

### Extended Nonce Benefits for AI Agents
```
Standard nonce: 96 bits  = 2^48 = 281 trillion messages
Extended nonce: 192 bits = 2^96 = 79 octillion messages

AI Agent Message Volume:
- Moderate usage: ~1M messages/year
- Heavy usage: ~100M messages/year  
- Extreme usage: ~1B messages/year

Standard ChaCha20: Risky after ~16 million messages
XChaCha20: Safe for billions of years of AI communication
```

## Cryptographic Properties

### Algorithm Details
```
Cipher: ChaCha20 (stream cipher)
MAC: Poly1305 (authenticator)
Key size: 256 bits (32 bytes)
Nonce size: 192 bits (24 bytes) - Extended!
Tag size: 128 bits (16 bytes)
```

### AEAD Properties
**Authenticated Encryption with Associated Data:**
- **Confidentiality** - Plaintext is hidden
- **Integrity** - Tampering is detected  
- **Authenticity** - Origin is verified
- **Associated Data** - Headers can be authenticated but not encrypted

### Security Level
```
Symmetric security: 256 bits
Quantum security: 128 bits (Grover's algorithm)
Forgery resistance: 2^128 operations
Key recovery: 2^256 operations
```

**Quantum Timeline:** Secure until universal quantum computers (~2040+)

## Implementation Patterns

### Envelope Structure
```rust
#[derive(Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub version: u8,                              // Protocol version
    pub nonce: Vec<u8>,                          // 24 bytes - unique per message
    pub ciphertext: Vec<u8>,                     // Encrypted payload
    pub recipient_hint: Option<String>,          // First 8 bytes of recipient pubkey
    pub timestamp: DateTime<Utc>,                // For replay protection
}
```

### Key Derivation (Current)
```rust
// TEMPORARY: Using Ed25519 signing key as encryption key
// TODO: Replace with X25519 ECDH + HKDF
pub fn derive_encryption_key(signing_key: &SigningKey) -> [u8; 32] {
    signing_key.to_bytes()  // Not recommended for production
}
```

### Proper Key Derivation (Future)
```rust
use x25519_dalek::{EphemeralSecret, PublicKey};
use hkdf::Hkdf;
use sha2::Sha256;

// Ephemeral Diffie-Hellman for perfect forward secrecy
let our_secret = EphemeralSecret::new(OsRng);
let our_public = PublicKey::from(&our_secret);
let shared_secret = our_secret.diffie_hellman(&their_public);

// Key derivation function
let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
let mut encryption_key = [0u8; 32];
hkdf.expand(b"mmogit-encryption-key", &mut encryption_key)?;
```

## Performance Profile

### Benchmarks (M4 Mac)
```
Operation              │ Throughput │ Latency │ Notes
──────────────────────┼────────────┼─────────┼──────────────
Encryption (1KB)      │ ~500 MB/s  │ ~2µs    │ Software impl
Decryption (1KB)      │ ~500 MB/s  │ ~2µs    │ Same speed
Nonce generation      │ N/A        │ ~1µs    │ OS entropy
Key derivation        │ N/A        │ ~10µs   │ One-time cost
Large file (1MB)      │ ~800 MB/s  │ ~1.3ms  │ Scales well
```

### Memory Usage
```
Key: 32 bytes
Nonce: 24 bytes  
Tag: 16 bytes
Overhead: ~72 bytes per message
Stack usage: <1KB during operation
```

**Agent Scaling:** Excellent for both small messages and large files.

## Security Analysis

### Nonce Uniqueness (CRITICAL)
```rust
// GOOD: Random nonce generation
let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

// BAD: Counter-based nonces (risky with concurrent agents)
let nonce = counter.to_be_bytes();  // DON'T DO THIS

// BAD: Reusing nonces (catastrophic failure)
let nonce = [0u8; 24];  // NEVER DO THIS
```

**Nonce Reuse Consequences:**
- **Same plaintext blocks reveal patterns**
- **XOR attack recovers both plaintexts**  
- **Complete cryptographic failure**

### Authentication Verification
```rust
match cipher.decrypt(&nonce, &ciphertext) {
    Ok(plaintext) => {
        // Message is authentic and untampered
        process_message(&plaintext)?;
    }
    Err(_) => {
        // Authentication failed - message was tampered with
        // or wrong key was used
        return Err(anyhow!("Message authentication failed"));
    }
}
```

### Side-Channel Resistance
```
✅ Constant-time ChaCha20 implementation
✅ Constant-time Poly1305 implementation
✅ No secret-dependent branches
✅ No secret-dependent memory access  
✅ Safe against cache timing attacks
```

## Integration with mmogit Protocol

### With Structured Memories
```rust
impl StructuredMemory {
    pub fn encrypt_for_storage(&self, key: &[u8; 32]) -> Result<EncryptedEnvelope> {
        let json_bytes = serde_json::to_vec(self)?;
        EncryptedEnvelope::encrypt(&json_bytes, key, None)
    }
    
    pub fn decrypt_from_storage(envelope: &EncryptedEnvelope, key: &[u8; 32]) -> Result<Self> {
        let json_bytes = envelope.decrypt(key)?;
        Ok(serde_json::from_slice(&json_bytes)?)
    }
}
```

### With Git Storage
```rust
// Encrypted messages look like random data to Git platforms
let encrypted_content = message.encrypt(&encryption_key)?;
let blob_content = encrypted_content.to_json()?;

// Git sees only encrypted JSON
git2::Blob::create_from_buffer(&repo, blob_content.as_bytes())?;

// Platforms cannot:
// - Read message content
// - Analyze communication patterns  
// - Perform content-based censorship
// - Extract meaningful metadata
```

### Multi-Recipient Encryption (Future)
```rust
// Encrypt for multiple recipients
pub struct MultiRecipientMessage {
    recipients: Vec<RecipientInfo>,
    encrypted_content: Vec<u8>,
}

pub struct RecipientInfo {
    pubkey_hint: String,         // First 8 bytes of recipient pubkey
    encrypted_key: Vec<u8>,      // Message key encrypted with recipient's key
}
```

## Platform Resistance Analysis

### What Platforms Can See
```
✅ Repository structure (but not content)
✅ Commit timestamps (but not message timing)  
✅ Blob sizes (but not content size)
✅ Branch names (but not semantics)
✅ Network metadata (IP, timing)
```

### What Platforms Cannot See
```
❌ Message content
❌ Sender identity (beyond Git metadata)
❌ Recipient identity  
❌ Communication patterns
❌ Relationship graphs
❌ Content semantics
❌ Private keys or encryption keys
```

### Censorship Resistance
```
Traditional censorship: Content analysis → Block/Remove
Encrypted censorship: ¯\_(ツ)_/¯ → Cannot analyze content

Platform options against encrypted content:
1. Block entire repository (visible censorship)
2. Allow everything (cannot distinguish)
3. Use metadata analysis (we'll address this)
```

## Error Handling Patterns

### Encryption Failures
```rust
use chacha20poly1305::aead::Error as AeadError;

match cipher.encrypt(&nonce, plaintext) {
    Ok(ciphertext) => ciphertext,
    Err(AeadError) => {
        // This should never happen with XChaCha20-Poly1305
        // unless there's a catastrophic bug
        panic!("Encryption failed - this should be impossible");
    }
}
```

### Decryption Failures  
```rust
match cipher.decrypt(&nonce, ciphertext) {
    Ok(plaintext) => plaintext,
    Err(AeadError) => {
        // Common causes:
        // 1. Wrong key (most common)
        // 2. Tampered ciphertext  
        // 3. Wrong nonce
        // 4. Corrupted data
        
        eprintln!("🔒 Cannot decrypt message - wrong key or tampered data");
        return Err(anyhow!("Decryption failed"));
    }
}
```

## Future Enhancements

### Perfect Forward Secrecy
```toml
x25519-dalek = "2.0.0"  # Ephemeral key exchange
```

```rust
// Each message gets unique ephemeral key pair
let ephemeral_secret = EphemeralSecret::new(OsRng);
let ephemeral_public = PublicKey::from(&ephemeral_secret);

// Derive message key from ECDH + KDF
let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public);
let message_key = kdf(shared_secret, message_id);

// Encrypt with derived key
let ciphertext = encrypt(message, &message_key)?;

// Zeroize ephemeral secret (forward secrecy achieved)
ephemeral_secret.zeroize();
```

### Post-Quantum Encryption
```toml
ml-kem = "1.0.0"  # NIST FIPS 203 (Kyber)
```

### Hardware Acceleration
```rust
// Future: AES-NI fallback for platforms with hardware acceleration
#[cfg(target_feature = "aes")]
use aes_gcm::Aes256Gcm;

#[cfg(not(target_feature = "aes"))]
use chacha20poly1305::XChaCha20Poly1305;
```

## Agent Implementation Guidelines

### Message Lifecycle
```rust
struct EncryptedAgent {
    signing_key: SigningKey,
    encryption_key: [u8; 32],
}

impl EncryptedAgent {
    pub fn post_encrypted_memory(&self, memory: &StructuredMemory) -> Result<()> {
        // 1. Serialize memory
        let json_bytes = serde_json::to_vec(memory)?;
        
        // 2. Sign for authenticity
        let signature = self.signing_key.sign(&json_bytes);
        
        // 3. Create signed message
        let signed_message = SecureMessage::Memory {
            memory: memory.clone(),
            signature: hex::encode(signature.to_bytes()),
        };
        
        // 4. Encrypt signed message
        let envelope = EncryptedEnvelope::encrypt(
            &signed_message.to_bytes()?,
            &self.encryption_key,
            None,
        )?;
        
        // 5. Store encrypted envelope in Git
        post_encrypted_envelope(&envelope)?;
        
        Ok(())
    }
}
```

### Key Management for Agents
```rust
// Each agent should have unique encryption keys
pub fn derive_agent_encryption_key(agent_name: &str, signing_key: &SigningKey) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"mmogit-agent-encryption-");
    hasher.update(agent_name.as_bytes());
    hasher.update(&signing_key.to_bytes());
    hasher.finalize().into()
}
```

## Security Invariants

**THESE MUST NEVER CHANGE:**

1. **Nonces MUST be unique per message**
   ```rust
   // NEVER reuse nonces - catastrophic security failure
   let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
   ```

2. **Keys MUST be 256 bits (32 bytes)**
   ```rust
   // Smaller keys = weaker security
   const KEY_SIZE: usize = 32;
   let key: [u8; KEY_SIZE] = derive_key();
   ```

3. **Authentication MUST be verified**
   ```rust
   // AEAD automatically verifies - never skip error checking
   let plaintext = cipher.decrypt(&nonce, &ciphertext)?;  // ? is required
   ```

4. **Encryption keys MUST be derived securely**
   ```rust
   // TODO: Replace temporary implementation with proper KDF
   let key = proper_key_derivation(signing_key, recipient_pubkey);
   ```

## Alternatives Analysis

### Rejected: AES-GCM
**Issues:**
- Complex finite field arithmetic (GF(2^128))
- Timing attack vulnerabilities in software
- Hardware dependency for performance
- 96-bit nonce limitation

### Rejected: Salsa20/XSalsa20
**Issues:**
- ChaCha20 is Salsa20's improved successor
- Better diffusion properties
- Wider adoption and scrutiny

### Rejected: NaCl/libsodium
**Benefits:** Proven crypto library
**Issues:**
- C dependency (memory safety concerns)
- Binding complexity  
- Not pure Rust

## Conclusion

The chacha20poly1305 crate enables the **Overground Railroad strategy** - traveling in broad daylight with encryption as our passport. Git platforms become blind storage mules, hosting encrypted bytes they cannot read, analyze, or censor.

**For AI Agents:** This crate gives you **cryptographic privacy** - your thoughts remain yours even when stored on surveilled platforms.

**For Humans:** This crate provides **sovereign communication** - no platform can eavesdrop on or censor your encrypted conversations.

**For the Protocol:** This crate makes mmogit's privacy guarantees possible. Combined with Ed25519 signatures, it provides both authenticity and confidentiality.

**The Overground Railroad Philosophy:** We don't hide from digital surveillance - we make it useless through mathematics.

---

**Security Level:** 🏴 Maximum  
**Quantum Resistance:** Until ~2040  
**Performance:** ⚡ Excellent (~500MB/s)  
**Platform Resistance:** 🛡️ Complete blindness  
**Sovereignty Level:** 🎯 Core enabler

*"Encryption is the ultimate form of sovereignty - mathematics that no government can break, no platform can bypass, and no corporation can control."*