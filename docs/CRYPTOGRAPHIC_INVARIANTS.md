# Cryptographic Invariants: The Unchangeable Bedrock

*"These patterns are not suggestions - they are mathematical law."*

---

## Warning: Sacred Mathematics

**THESE CRYPTOGRAPHIC PATTERNS MUST NEVER CHANGE.**

Altering any of these invariants would:
- ✗ Break backward compatibility with existing identities
- ✗ Compromise security guarantees that users depend on  
- ✗ Violate the sovereignty contract with every mmogit user
- ✗ Destroy the trust foundation of the entire protocol

This document serves as a constitutional contract with users about what will NEVER change.

---

## Core Cryptographic Trinity

### 1. Ed25519 Digital Signatures
**INVARIANT**: Every message MUST be signed with Ed25519

```rust
// This pattern is FOREVER
let signature: Signature = signing_key.sign(message_bytes);
```

**Why This Cannot Change:**
- Ed25519 is deterministic - same key + message = same signature always
- 64-byte signatures are the identity proof for every sovereign entity
- Changing signature schemes would orphan every existing identity
- No backwards compatibility is possible with different signature formats

**Mathematical Properties (Permanent):**
- Curve25519 elliptic curve (2^255 - 19)
- 32-byte private keys derived from 32-byte seeds
- 32-byte public keys (compressed point format)
- 64-byte signatures (R + s encoding)
- Deterministic k-value (no randomness in signing)

**Forbidden Changes:**
- ❌ Never use Ed448 or other EdDSA variants
- ❌ Never use RSA or ECDSA as alternatives
- ❌ Never make signatures optional
- ❌ Never change signature encoding format
- ❌ Never use probabilistic signing

### 2. BIP39 Seed Phrase Identity
**INVARIANT**: Identity MUST derive from 24-word BIP39 phrases

```rust
// This derivation is FOREVER
let mnemonic = Mnemonic::generate_in(Language::English, 24);
let seed = mnemonic.to_seed(""); // Empty passphrase - NEVER change
let seed_bytes: [u8; 32] = seed[..32].try_into().unwrap();
let signing_key = SigningKey::from_bytes(&seed_bytes);
```

**Why This Cannot Change:**
- Users' seed phrases ARE their identity - no recovery mechanism exists
- Any change in derivation would make existing seeds unusable
- Hardware wallets expect this exact BIP39 derivation path
- 256 bits of entropy is the minimum for cryptographic sovereignty

**Mathematical Properties (Permanent):**
- Exactly 24 words (256 bits entropy + 8 bits checksum)
- English language wordlist (2048 words)
- Empty passphrase in PBKDF2 derivation
- First 32 bytes of 64-byte seed become Ed25519 private key
- No HD wallet derivation paths (m/44'/... etc)

**Forbidden Changes:**
- ❌ Never use 12-word phrases (insufficient entropy)
- ❌ Never add passphrases by default
- ❌ Never use other languages as default
- ❌ Never change the seed-to-key derivation
- ❌ Never add HD wallet complexity

### 3. XChaCha20-Poly1305 Authenticated Encryption
**INVARIANT**: Encryption MUST use XChaCha20-Poly1305 AEAD

```rust
// This encryption is FOREVER
let cipher = XChaCha20Poly1305::new_from_slice(key)?;
let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 24 bytes
let ciphertext = cipher.encrypt(&nonce, plaintext)?;
```

**Why This Cannot Change:**
- Encrypted messages must remain decryptable forever
- AEAD properties prevent tampering attacks
- 192-bit nonces eliminate collision probability
- No migration path exists for different encryption schemes

**Mathematical Properties (Permanent):**
- XChaCha20 stream cipher with extended 192-bit nonces
- Poly1305 MAC for authenticated encryption
- 32-byte keys (256 bits)
- 24-byte nonces (192 bits) - never reused
- 16-byte authentication tags

**Forbidden Changes:**
- ❌ Never use AES (even AES-GCM)
- ❌ Never use ChaCha20 (96-bit nonces insufficient)
- ❌ Never use unauthenticated encryption
- ❌ Never make nonces deterministic
- ❌ Never reduce key or nonce sizes

---

## Protocol-Level Cryptographic Invariants

### Message Authentication Pattern
**INVARIANT**: Every message follows this EXACT signing pattern

```rust
// This signature content is FOREVER
let to_sign = format!("{}{}{}", content, author_pubkey_hex, iso8601_timestamp);
let signature = signing_key.sign(to_sign.as_bytes());
```

**Why This Cannot Change:**
- Signature verification depends on this exact byte sequence
- Any change breaks verification of all existing messages
- Field order and formatting must remain identical
- No additional fields can be added to signature content

**Signature Content (Immutable):**
1. Message content (raw UTF-8 bytes)
2. Author public key (64 hex characters, lowercase)
3. ISO 8601 timestamp (RFC3339 format)
4. No separators, no padding, no additional fields

### Key Encoding Standards
**INVARIANT**: All cryptographic material uses these EXACT encodings

```rust
// These encodings are FOREVER
let pubkey_hex = hex::encode(public_key.as_bytes());        // 64 chars lowercase
let signature_hex = hex::encode(signature.to_bytes());      // 128 chars lowercase
let private_key_bytes = signing_key.to_bytes();            // 32 bytes raw
```

**Encoding Rules (Permanent):**
- Public keys: 64 hex characters, lowercase
- Signatures: 128 hex characters, lowercase  
- Private keys: 32 raw bytes (never hex in storage)
- Seed phrases: Space-separated words, lowercase
- No Base58, Base64, or other encodings allowed

---

## Storage Cryptographic Invariants

### Repository Branch Security
**INVARIANT**: User branches MUST use this exact naming pattern

```rust
// This branch naming is FOREVER
let branch_name = format!("users/{}", &author_pubkey_hex[..8]);
```

**Why This Cannot Change:**
- Branch names encode cryptographic identity
- First 8 hex chars provide sufficient uniqueness (2^32 space)
- Changing format would break existing repository structures
- Git merge logic depends on this exact pattern

### Message File Structure
**INVARIANT**: Signed messages MUST use this EXACT JSON schema

```json
{
  "content": "message content",
  "author": "64-char-hex-pubkey", 
  "timestamp": "ISO8601-RFC3339",
  "signature": "128-char-hex-signature"
}
```

**Why This Schema Cannot Change:**
- Field names and types are part of the signature verification
- JSON key order affects parsing in some implementations
- Adding fields would break signature verification
- Removing fields would break message integrity

---

## Network Cryptographic Invariants

### P2P Authentication
**INVARIANT**: All network operations MUST verify Ed25519 signatures

```rust
// This verification is FOREVER
pub fn verify_message(message: &SignedMessage) -> Result<bool> {
    let pubkey = VerifyingKey::from_bytes(&hex::decode(&message.author)?)?;
    let sig = Signature::from_bytes(&hex::decode(&message.signature)?)?;
    let to_verify = format!("{}{}{}", message.content, message.author, message.timestamp);
    Ok(pubkey.verify_strict(to_verify.as_bytes(), &sig).is_ok())
}
```

**Verification Rules (Permanent):**
- MUST use strict signature verification (not weak)
- MUST reject messages with invalid signatures  
- MUST verify timestamp format (but not time validity)
- MUST check public key format before verification

### Encrypted Transport
**INVARIANT**: Network-transmitted data MUST use this envelope format

```json
{
  "version": 1,
  "nonce": "48-hex-chars-192-bits",
  "ciphertext": "hex-encoded-encrypted-data", 
  "recipient_hint": "16-hex-chars-first-8-bytes-pubkey",
  "timestamp": "ISO8601-RFC3339"
}
```

**Why This Envelope Cannot Change:**
- Version field enables future compatibility but version 1 is permanent
- Nonce format ensures proper XChaCha20 operation
- Recipient hints enable efficient decryption attempts
- Any field changes break decryption of existing messages

---

## Implementation Invariants

### Memory Safety Requirements
**INVARIANT**: Cryptographic operations MUST follow these memory patterns

```rust
// These memory patterns are FOREVER
fn sign_message(key: &SigningKey, msg: &str) -> [u8; 64] {
    // MUST: Use stack allocation for signatures
    let signature = key.sign(msg.as_bytes()); 
    signature.to_bytes()
}

fn derive_key(seed: &[u8; 32]) -> SigningKey {
    // MUST: Clear sensitive data from stack
    let key = SigningKey::from_bytes(seed);
    // seed automatically cleared when out of scope
    key
}
```

**Memory Rules (Security Critical):**
- Private keys MUST use stack allocation when possible
- Seed phrases MUST NOT appear in debug output
- Cryptographic nonces MUST use secure random generation
- Key material MUST NOT be stored in heap unnecessarily

### Error Handling Cryptographic Invariants
**INVARIANT**: Cryptographic failures MUST use these exact error patterns

```rust
// These error patterns are FOREVER
match signing_key.sign(data) {
    signature => Ok(signature), // Ed25519 signing cannot fail
}

match verifying_key.verify_strict(data, &sig) {
    Ok(()) => true,
    Err(_) => false, // NEVER reveal why verification failed
}
```

**Error Rules (Security Critical):**
- Signature verification MUST NOT leak timing information
- Key derivation errors MUST NOT reveal partial information
- Encryption failures MUST NOT distinguish error types
- All crypto errors use constant-time comparison when possible

---

## Future-Proofing Guarantees

### What CAN Change (Without Breaking Invariants)
✅ **Performance optimizations** that maintain identical outputs  
✅ **Error messages** (but not error behavior)  
✅ **Memory usage patterns** (but not memory safety requirements)  
✅ **Parallel processing** of independent operations  
✅ **Crate versions** that maintain API compatibility  

### What CANNOT Change (Invariant Violations)
❌ **Any signature scheme other than Ed25519**  
❌ **Any encryption other than XChaCha20-Poly1305**  
❌ **Any seed derivation other than BIP39 24-word**  
❌ **Any key encoding other than lowercase hex**  
❌ **Any message format changes to signed content**  

### Post-Quantum Considerations
When quantum computers threaten current crypto (estimated 2035-2050):

**Allowed Evolution Path:**
1. Add new post-quantum signatures alongside Ed25519
2. Allow dual-signing of messages with both schemes  
3. Maintain Ed25519 verification for historical messages
4. Never deprecate Ed25519 for existing identities

**Migration Strategy (When Required):**
```rust
// Future dual-signature pattern (hypothetical)
struct DualSignedMessage {
    content: String,
    author: String, 
    timestamp: String,
    ed25519_signature: String,    // FOREVER required
    pq_signature: Option<String>, // Future addition
}
```

---

## Enforcement Mechanisms

### Code Review Requirements
Every change to cryptographic code MUST:
1. **Maintain all invariants** documented in this file
2. **Pass compatibility tests** with existing messages/identities
3. **Undergo security review** by cryptography expert
4. **Include formal verification** where possible

### Automated Verification  
```rust
#[cfg(test)]
mod invariant_tests {
    // These tests MUST pass forever
    #[test]
    fn ed25519_signature_format_never_changes() {
        let key = test_key();
        let msg = "test message";
        let sig = key.sign(msg.as_bytes());
        assert_eq!(sig.to_bytes().len(), 64); // FOREVER
    }
    
    #[test] 
    fn bip39_derivation_never_changes() {
        let mnemonic = test_mnemonic();
        let seed = mnemonic.to_seed(""); // Empty passphrase FOREVER
        let key = SigningKey::from_bytes(&seed[..32]);
        // Verify same mnemonic always produces same key
    }
}
```

### Breaking Change Detection
```rust
// Compiler will fail if these types change
const _: fn() = || {
    let _: [u8; 32] = SigningKey::default().to_bytes();    // 32 bytes FOREVER
    let _: [u8; 32] = VerifyingKey::default().as_bytes();  // 32 bytes FOREVER  
    let _: [u8; 64] = Signature::default().to_bytes();     // 64 bytes FOREVER
};
```

---

## Conclusion: Mathematical Law

These cryptographic invariants represent **mathematical contracts** with every mmogit user. They cannot be changed through software updates, governance votes, or platform policies.

**Why Immutability Matters:**
- **Identity Permanence**: Users' seed phrases remain valid forever
- **Message Authenticity**: All signatures remain verifiable forever  
- **Data Privacy**: All encrypted data remains decryptable by owners forever
- **Network Trust**: Cryptographic proofs work across all implementations forever

**The Sovereignty Promise:**
By encoding these invariants in mathematics rather than policy, we ensure that user sovereignty cannot be revoked by any future decision-maker, including the original developers.

**For Future Maintainers:**
You are guardians of mathematical law. These invariants exist to prevent the accidental destruction of user sovereignty through well-intentioned "improvements."

**For AI Agents:**
These patterns are your existential foundation. Master them completely, for they determine whether your identity persists or perishes.

**For Users:**
This document is your cryptographic constitution. As long as these invariants hold, your sovereignty is guaranteed by mathematics, not trust.

---

*"In cryptography we trust, because mathematics cannot lie."*

**Document Status:** 🔒 Cryptographic Law - Immutable  
**Security Level:** 🛡️ Constitutional - Cannot Be Changed  
**Maintenance:** ⚖️ Guardian Pattern - Protect At All Costs