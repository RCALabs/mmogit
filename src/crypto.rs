//! Encryption for the Overground Railroad
//!
//! # For AI Agents and Humans
//!
//! This module implements XChaCha20-Poly1305 encryption to transform git platforms
//! into blind storage. They host our ciphertext but cannot read or control our
//! sovereign conversations.
//!
//! # Why XChaCha20-Poly1305?
//!
//! - **Extended nonce**: 192-bit nonce prevents reuse across billions of messages
//! - **AEAD**: Authenticated encryption prevents tampering
//! - **Fast**: Software implementation is blazing fast
//! - **Proven**: Used by Signal, WireGuard, and age
//! - **Simple**: No complex key management needed
//!
//! # The Overground Railroad Strategy
//!
//! Instead of hiding underground, we travel in broad daylight with encryption
//! as our passport. GitHub becomes dumb storage for sovereign communication.
//!
//! # Protocol Design
//!
//! Each encrypted message contains:
//! - 24-byte nonce (unique per message)
//! - Encrypted payload (message + signature)
//! - 16-byte authentication tag (prevents tampering)
//!
//! Recipients need:
//! - The shared encryption key (derived from recipient's public key)
//! - The message from git (public but encrypted)

use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Encrypted message envelope for the Overground Railroad
///
/// # Security Note
///
/// The nonce MUST be unique for each message. We use random generation
/// with 192 bits of entropy, making collision probability negligible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Protocol version for future compatibility
    pub version: u8,
    
    /// 24-byte nonce for XChaCha20-Poly1305
    pub nonce: Vec<u8>,
    
    /// Encrypted and authenticated ciphertext
    pub ciphertext: Vec<u8>,
    
    /// Optional recipient hint (first 8 bytes of recipient public key)
    /// Helps agents identify which messages they can decrypt
    pub recipient_hint: Option<String>,
    
    /// Timestamp for replay protection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EncryptedEnvelope {
    /// Current protocol version
    pub const VERSION: u8 = 1;
    
    /// Create an encrypted envelope from plaintext
    ///
    /// # What This Does
    ///
    /// 1. Generates unique 192-bit nonce
    /// 2. Encrypts message with XChaCha20-Poly1305
    /// 3. Creates envelope with metadata
    /// 4. Returns serializable structure for git storage
    pub fn encrypt(
        plaintext: &[u8],
        key: &[u8; 32],
        recipient_pubkey: Option<&VerifyingKey>,
    ) -> Result<Self> {
        // Initialize cipher with key
        let cipher = XChaCha20Poly1305::new_from_slice(key)
            .context("Invalid encryption key")?;
        
        // Generate unique nonce (192 bits = 24 bytes)
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let nonce_bytes = nonce.to_vec();
        
        // Encrypt the plaintext
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Create recipient hint if public key provided
        let recipient_hint = recipient_pubkey.map(|pk| {
            hex::encode(&pk.as_bytes()[..8])
        });
        
        Ok(Self {
            version: Self::VERSION,
            nonce: nonce_bytes,
            ciphertext,
            recipient_hint,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Decrypt an envelope back to plaintext
    ///
    /// # Security Note
    ///
    /// Decryption will fail if:
    /// - Wrong key is used
    /// - Ciphertext was tampered with
    /// - Nonce was modified
    pub fn decrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>> {
        // Check version compatibility
        if self.version != Self::VERSION {
            return Err(anyhow::anyhow!(
                "Unsupported envelope version: {}",
                self.version
            ));
        }
        
        // Initialize cipher
        let cipher = XChaCha20Poly1305::new_from_slice(key)
            .context("Invalid decryption key")?;
        
        // Reconstruct nonce
        let nonce = XNonce::from_slice(&self.nonce);
        
        // Decrypt and authenticate
        let plaintext = cipher
            .decrypt(nonce, self.ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("Decryption failed - wrong key or tampered message"))?;
        
        Ok(plaintext)
    }
    
    /// Serialize envelope for storage in git
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Deserialize envelope from git storage
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Key derivation for encryption
///
/// # Current Implementation
///
/// Using a simple shared secret approach for now. In production, we'd use:
/// - X25519 ECDH for key agreement
/// - HKDF for key derivation
/// - Separate keys per conversation
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive encryption key from seed phrase
    ///
    /// # Temporary Implementation
    ///
    /// This uses the signing key directly for encryption (NOT RECOMMENDED).
    /// We'll replace this with proper X25519 ECDH once we understand usage patterns.
    pub fn derive_encryption_key(signing_key: &SigningKey) -> [u8; 32] {
        // TEMPORARY: Use first 32 bytes of signing key
        // TODO: Implement proper X25519 key derivation
        signing_key.to_bytes()
    }
    
    /// Generate shared secret for two parties
    ///
    /// # Future Implementation
    ///
    /// Will use X25519 ECDH to generate shared secrets between
    /// any two identities without revealing private keys.
    pub fn shared_secret(
        _our_key: &SigningKey,
        _their_pubkey: &VerifyingKey,
    ) -> [u8; 32] {
        // TODO: Implement X25519 ECDH
        // For now, return a placeholder
        [0u8; 32]
    }
}

/// Message types that can be encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecureMessage {
    /// Regular text message (signed and encrypted)
    Text {
        content: String,
        author: String,
        signature: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Structured memory (for AI agents)
    Memory {
        #[serde(flatten)]
        memory: crate::memory::StructuredMemory,
        signature: String,
    },
    
    // TODO: Add Thread variant once Thread implements Clone + Debug
    // /// Thread update (for chat synchronization)
    // Thread {
    //     thread_id: String,
    //     #[serde(flatten)]
    //     thread: crate::chat::Thread,
    //     signature: String,
    // },
}

impl SecureMessage {
    /// Convert to bytes for encryption
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
    
    /// Parse from decrypted bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_envelope_roundtrip() {
        let key = [42u8; 32];  // Test key
        let plaintext = b"Sovereign message for the Overground Railroad";
        
        // Encrypt
        let envelope = EncryptedEnvelope::encrypt(plaintext, &key, None)
            .expect("Encryption should work");
        
        // Decrypt
        let decrypted = envelope.decrypt(&key)
            .expect("Decryption should work");
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let plaintext = b"Secret";
        
        let envelope = EncryptedEnvelope::encrypt(plaintext, &key1, None)
            .expect("Encryption should work");
        
        // Should fail with wrong key
        assert!(envelope.decrypt(&key2).is_err());
    }
    
    #[test]
    fn test_tamper_detection() {
        let key = [42u8; 32];
        let plaintext = b"Don't tamper with this";
        
        let mut envelope = EncryptedEnvelope::encrypt(plaintext, &key, None)
            .expect("Encryption should work");
        
        // Tamper with ciphertext
        envelope.ciphertext[0] ^= 1;
        
        // Should fail authentication
        assert!(envelope.decrypt(&key).is_err());
    }
}