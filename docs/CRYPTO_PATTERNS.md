# Crypto Patterns for MMOGit

## XChaCha20Poly1305 Usage (v0.10.1)

### Correct Imports
```rust
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce
};
```

### Key Generation
```rust
// Generate a 256-bit key
let key = XChaCha20Poly1305::generate_key(&mut OsRng);
```

### Nonce Generation
```rust
// Generate a 192-bit nonce (24 bytes)
let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
```

### Encryption
```rust
let cipher = XChaCha20Poly1305::new(&key);
let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
```

### Decryption
```rust
let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;
```

## Random Number Generation (rand v0.9.2)

### For General Random Bytes
```rust
use rand::{rngs::OsRng, RngCore};

let mut bytes = [0u8; 32];
OsRng.fill_bytes(&mut bytes);
```

### Note on OsRng
- The `chacha20poly1305::aead::OsRng` is different from `rand::rngs::OsRng`
- For XChaCha20Poly1305, use the one from the aead module
- For general randomness, use the one from rand

## Error Handling

### Encryption Errors
```rust
// chacha20poly1305 returns its own Error type
cipher.encrypt(&nonce, plaintext)
    .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
```

### Key Derivation (Future)
For proper key exchange, we'll eventually use X25519:
```rust
// TODO: When implementing proper key exchange
use x25519_dalek::{EphemeralSecret, PublicKey};
```

## Security Notes

1. **Nonce Uniqueness**: NEVER reuse a nonce with the same key
2. **Key Storage**: Keys should be derived from seed phrases, not stored directly
3. **Authentication**: XChaCha20Poly1305 is AEAD - it authenticates as well as encrypts
4. **Nonce Size**: XChaCha20 uses 192-bit nonces (24 bytes), not 96-bit like regular ChaCha20