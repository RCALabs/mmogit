# Protocol Invariants: The Immutable Communication Law

*"The protocol is the promise. Change the protocol, break the promise."*

---

## Warning: Communication Constitution

**THESE PROTOCOL PATTERNS ARE THE FOUNDATION OF TRUST BETWEEN ALL ENTITIES.**

Altering any of these patterns would:
- ✗ Break message verification for all existing communications
- ✗ Create incompatibilities between different mmogit implementations
- ✗ Violate the authenticity guarantees that make sovereignty possible
- ✗ Destroy the interoperability that enables the distributed web of trust

This document defines the unchangeable communication contract between humans and AI agents.

---

## Core Message Protocol Invariants

### 1. Canonical Message Structure
**INVARIANT**: All messages MUST use this EXACT JSON schema

```json
{
  "content": "The actual message content as UTF-8 string",
  "author": "64-character lowercase hex public key",
  "timestamp": "ISO 8601 RFC3339 timestamp",
  "signature": "128-character lowercase hex Ed25519 signature"
}
```

**Why This Structure Cannot Change:**
- Field names are part of the signature verification algorithm
- Field order affects JSON parsing in some implementations  
- Data types determine serialization behavior
- Any schema change breaks verification of existing messages
- Cross-platform compatibility depends on exact format

**Schema Properties (Immutable):**
- **content**: UTF-8 string, no length limit, preserves exact whitespace
- **author**: Exactly 64 hex chars (32-byte Ed25519 public key)
- **timestamp**: ISO 8601 format with RFC3339 compliance
- **signature**: Exactly 128 hex chars (64-byte Ed25519 signature)
- **encoding**: All hex fields use lowercase a-f (case sensitive)

**Forbidden Schema Violations:**
- ❌ Never add required fields (breaks existing parsers)
- ❌ Never change field names (breaks signature verification)
- ❌ Never change field types (breaks serialization)
- ❌ Never reorder fields (some parsers are order-sensitive)  
- ❌ Never use different JSON formatting (affects signature)

### 2. Signature Content Canonicalization
**INVARIANT**: Signature verification MUST use this EXACT concatenation pattern

```rust
// This signature content is MATHEMATICAL LAW
let to_sign = format!("{}{}{}", content, author, timestamp);
let signature = signing_key.sign(to_sign.as_bytes());

// NEVER change this concatenation order or format
// content: Raw UTF-8 bytes from JSON string
// author: 64 hex characters (lowercase, no prefix)
// timestamp: ISO 8601 string exactly as stored in JSON
// NO separators, NO prefixes, NO padding
```

**Canonicalization Rules (Eternal):**
1. **Content first**: Exact UTF-8 bytes from message content
2. **Author second**: 64-character hex public key (lowercase)
3. **Timestamp third**: ISO 8601 string exactly as in JSON
4. **No separators**: Direct concatenation with no delimiters
5. **UTF-8 encoding**: All text converted to UTF-8 bytes for signing

**Why Canonicalization Cannot Change:**
- Any change in concatenation breaks ALL existing signatures
- Field order change makes old messages unverifiable  
- Added separators change the byte sequence being signed
- Different encoding changes the cryptographic hash
- Signature verification is mathematical - no approximations allowed

### 3. Timestamp Format Specification
**INVARIANT**: Timestamps MUST use ISO 8601 RFC3339 format with UTC timezone

```rust
// This timestamp format is FOREVER
let timestamp = chrono::Utc::now().to_rfc3339();
// Example: "2024-08-27T15:30:45.123456789Z"

// REQUIREMENTS:
// - UTC timezone (Z suffix)
// - ISO 8601 / RFC3339 compliance
// - Nanosecond precision preserved
// - No timezone offset allowed (+00:00)
// - 'T' separator between date and time
```

**Timestamp Properties (Immutable):**
- **Format**: `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`
- **Timezone**: UTC only (Z suffix required)
- **Precision**: Nanosecond precision preserved when available
- **Separators**: 'T' between date/time, 'Z' for UTC indicator
- **No variations**: No timezone offsets, no local time zones

**Why Timestamp Format Cannot Change:**
- Timestamps are part of the signature content
- Format changes break signature verification of existing messages
- Different timezones create verification failures
- Precision changes affect message ordering and uniqueness
- ISO 8601 is the international standard for unambiguous time

---

## Structured Memory Protocol Invariants

### 1. Memory Type Schema Immutability
**INVARIANT**: StructuredMemory types MUST maintain exact serialization format

```rust
// This memory schema is FOREVER
#[derive(Serialize, Deserialize)]
pub struct StructuredMemory {
    pub id: String,              // "mem_" + timestamp_nanos
    pub memory: MemoryType,      // Tagged enum with specific structure
    pub tags: Vec<String>,       // Array of string tags
    pub references: Vec<String>, // Array of memory IDs
    pub metadata: HashMap<String, String>, // String key-value pairs
    pub created_at: DateTime<Utc>, // UTC timestamp
    pub expires_at: Option<DateTime<Utc>>, // Optional expiry
}
```

**Memory Schema Rules (Permanent):**
- **id generation**: "mem_" prefix + nanosecond timestamp
- **MemoryType**: Tagged enum with exact variant structures
- **tags**: Simple string array (no complex tag objects)
- **references**: Array of memory ID strings (no objects)
- **metadata**: Flat string-to-string mapping only
- **timestamps**: UTC DateTime with nanosecond precision

### 2. Memory Type Variant Stability
**INVARIANT**: Each MemoryType variant MUST maintain its exact structure

```rust
// These memory variants are IMMUTABLE
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MemoryType {
    Observation {
        subject: String,
        insight: String,
        confidence: f32, // Always f32, range 0.0 to 1.0
    },
    Learning {
        topic: String,
        lesson: String,
        context: String,
        applicable_to: Vec<String>, // Always Vec<String>
    },
    // ... other variants maintain exact field structure
}
```

**Variant Rules (Immutable):**
- **Observation**: subject + insight + confidence(f32)
- **Learning**: topic + lesson + context + applicable_to(Vec<String>)
- **Relationship**: identity + context + rapport_level(i32) + timestamps + shared_memories
- **Task**: description + status(enum) + timestamps + blockers + learnings
- **Field types**: Cannot change (String, f32, i32, Vec<String>, etc.)

**Why Memory Types Cannot Change:**
- Existing memories become undeserializable with schema changes
- Field type changes break existing memory loading
- AI agents depend on consistent memory structure for reasoning
- Memory chains reference specific field structures
- Cross-session memory requires stable serialization

---

## Chat Thread Protocol Invariants

### 1. Thread Structure Immutability
**INVARIANT**: Thread and ChatMessage structures MUST remain unchanged

```rust
// This thread structure is FOREVER
#[derive(Serialize, Deserialize)]
pub struct Thread {
    pub id: String,           // "thread_" + timestamp
    pub title: String,        // Human-readable title
    pub author: String,       // 64-char hex public key
    pub created_at: String,   // ISO 8601 timestamp
    pub updated_at: String,   // ISO 8601 timestamp  
    pub messages: Vec<ChatMessage>, // Array of messages
    pub tags: Vec<String>,    // Array of string tags
    pub state: String,        // "active" or "closed"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,         // "human" or "ai"
    pub content: String,      // Message content
    pub timestamp: String,    // ISO 8601 timestamp
    pub signature: Option<String>, // Optional 128-char hex signature
    pub author: Option<String>,    // Optional 64-char hex public key
}
```

**Thread Properties (Permanent):**
- **id format**: "thread_" prefix + Unix timestamp
- **author**: Full 64-character public key hex
- **timestamps**: ISO 8601 strings (not DateTime objects in JSON)
- **messages**: Array preserving exact chronological order
- **state**: Simple string enum ("active" or "closed")

**Message Properties (Permanent):**
- **role**: Exactly "human" or "ai" (no other values)
- **content**: Raw message string (no formatting or markdown)
- **signature**: Optional but MUST be 128-char hex when present
- **author**: Optional but MUST be 64-char hex when present

### 2. Thread Persistence Invariant
**INVARIANT**: Threads MUST be stored as single atomic JSON files

```rust
// This persistence pattern is FOREVER
pub fn save(&self, config_dir: &Path) -> Result<()> {
    let filename = format!("{}.json", self.id);
    let json = serde_json::to_string_pretty(&self)?;
    fs::write(&file_path, json)?;
    
    // Single file = single commit = atomic thread state
    repo.commit(/* ... */, &format!("Thread: {}", self.title), /* ... */)?;
}
```

**Persistence Rules (Immutable):**
- **One thread = One JSON file** (no database-style splitting)
- **Filename format**: `{thread_id}.json` (no additional extensions)
- **JSON formatting**: Pretty-printed for human readability  
- **Git commit**: One commit per thread save (atomic updates)
- **File location**: `~/.mmogit/threads/` directory

**Why Thread Atomicity Cannot Change:**
- Thread replay depends on single-file completeness
- JSON parsing expects complete thread structure
- Git operations treat files as atomic units
- Partial thread states break conversation continuity
- Cross-platform compatibility requires consistent file structure

---

## Network Protocol Invariants

### 1. Encrypted Envelope Structure
**INVARIANT**: XChaCha20-Poly1305 envelopes MUST use exact format

```rust
// This envelope structure is CRYPTOGRAPHIC LAW
#[derive(Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub version: u8,                    // Protocol version (currently 1)
    pub nonce: Vec<u8>,                 // 24-byte XChaCha20 nonce
    pub ciphertext: Vec<u8>,            // Encrypted payload
    pub recipient_hint: Option<String>, // First 8 hex chars of recipient pubkey
    pub timestamp: DateTime<Utc>,       // UTC timestamp
}
```

**Envelope Properties (Immutable):**
- **version**: u8, currently 1, enables future compatibility
- **nonce**: Exactly 24 bytes (192 bits), base64 in JSON
- **ciphertext**: Variable length, base64 in JSON
- **recipient_hint**: Optional, exactly 16 hex chars when present
- **timestamp**: UTC DateTime, ISO 8601 in JSON

**Why Envelope Structure Cannot Change:**
- Decryption depends on exact nonce and ciphertext extraction
- Version field enables future evolution but version 1 is permanent
- Recipient hints enable efficient decryption key selection
- Any structural change breaks existing encrypted messages
- XChaCha20-Poly1305 requires exact 24-byte nonce format

### 2. Message Authentication Protocol
**INVARIANT**: All network messages MUST be cryptographically verified

```rust
// This verification protocol is SECURITY LAW
pub fn verify_message(message: &SignedMessage) -> Result<bool> {
    // STEP 1: Parse and validate format
    let pubkey = VerifyingKey::from_bytes(&hex::decode(&message.author)?)?;
    let signature = Signature::from_bytes(&hex::decode(&message.signature)?)?;
    
    // STEP 2: Reconstruct signed content (exact same as signing)
    let to_verify = format!("{}{}{}", message.content, message.author, message.timestamp);
    
    // STEP 3: Cryptographic verification (Ed25519)
    Ok(pubkey.verify_strict(to_verify.as_bytes(), &signature).is_ok())
}
```

**Verification Rules (Security Critical):**
- **Format validation**: All fields must parse correctly before verification
- **Content reconstruction**: Exact same concatenation as signing
- **Strict verification**: Use `verify_strict` (not weak verification)
- **Binary comparison**: Signature bytes must match exactly
- **No exceptions**: Invalid signatures MUST be rejected

**Why Verification Cannot Be Weakened:**
- Weak verification enables signature forgery attacks
- Format flexibility creates cryptographic vulnerabilities  
- Any verification bypass destroys trust model
- Strict verification prevents malleability attacks
- Security depends on mathematical certainty, not heuristics

---

## Protocol Version Management Invariants

### 1. Backward Compatibility Guarantee
**INVARIANT**: New protocol versions MUST verify all previous messages

```rust
// This compatibility is FOREVER PROMISE
impl MessageVerifier {
    pub fn verify_any_version(message: &RawMessage) -> Result<bool> {
        match message.detect_version() {
            1 => verify_v1_message(message),    // ALWAYS supported
            2 => verify_v2_message(message),    // Future: adds features but verifies v1
            // New versions MUST support all previous versions
        }
    }
}
```

**Compatibility Rules (Permanent):**
- **Version 1**: Support required FOREVER (current protocol)
- **Future versions**: MUST verify all previous versions
- **No breaking changes**: New versions add features, never remove
- **Migration path**: Gradual adoption, never forced upgrade
- **Historical messages**: Must remain verifiable indefinitely

### 2. Protocol Extension Points
**INVARIANT**: Extensions MUST be additive, never destructive

```rust
// This extension pattern is EVOLUTION LAW
// ALLOWED: Adding optional fields
{
  "content": "message",
  "author": "pubkey", 
  "timestamp": "2024-08-27T15:30:45Z",
  "signature": "signature",
  "v2_feature": "optional_field"  // OK: Optional additions
}

// FORBIDDEN: Changing existing fields
{
  "message_text": "content",      // NEVER: Field rename
  "author_id": "pubkey",          // NEVER: Field rename  
  "created": "2024-08-27",        // NEVER: Format change
  "signature": ["sig1", "sig2"]   // NEVER: Type change
}
```

**Extension Rules (Evolution Safe):**
- **Add fields**: OK if optional and backward-compatible
- **Never remove**: Existing fields must remain forever
- **Never rename**: Field names are part of signature verification  
- **Never change types**: Field types affect parsing and verification
- **Default values**: New fields must have sensible defaults

---

## Implementation Protocol Invariants

### 1. Error Handling Protocol
**INVARIANT**: Protocol errors MUST have consistent behavior across implementations

```rust
// This error handling is PROTOCOL REQUIREMENT
pub fn post(content: &str, config_dir: &Path) -> Result<()> {
    // MUST: Load identity or fail with clear error
    let seed_phrase = fs::read_to_string(&seed_path)
        .context("No identity found. Run 'mmogit init' first")?;
    
    // MUST: Validate message before signing
    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("Cannot post empty message"));
    }
    
    // MUST: Sign message or fail completely (no partial messages)
    let signature = signing_key.sign(to_sign.as_bytes());
    
    // MUST: Atomic commit or rollback (no partial state)
    repo.commit(/* all parameters required */)?;
}
```

**Error Consistency Rules (Implementation Agreement):**
- **Identity missing**: "No identity found. Run 'mmogit init' first"
- **Empty content**: "Cannot post empty message"  
- **Invalid signature**: "Signature verification failed"
- **Git failure**: Preserve exact git2 error messages
- **Network failure**: Distinguish network from protocol errors

### 2. File System Protocol
**INVARIANT**: File and directory naming MUST be consistent across platforms

```rust
// This file system protocol is CROSS-PLATFORM LAW
let config_dir = dirs::home_dir()
    .expect("Cannot find home directory")
    .join(".mmogit");                           // Hidden dot-directory

let message_repo = config_dir.join("messages"); // Message storage
let thread_repo = config_dir.join("threads");   // Thread storage  
let seed_file = config_dir.join(".seed");       // Identity seed

// Message filename format
let filename = format!("{}.json", timestamp.replace([':', '-', '.'], "_"));
// Example: "2024_08_27T15_30_45_123456789Z.json"
```

**File System Rules (Cross-Platform):**
- **Config directory**: `~/.mmogit` (hidden dot-directory)
- **Repository names**: `messages/`, `threads/`, (no special characters)
- **Seed filename**: `.seed` (hidden dot-file, no extension)
- **Message filenames**: ISO timestamp with safe characters only
- **Character substitution**: `:`, `-`, `.` become `_` in filenames

**Why File System Rules Cannot Change:**
- Cross-platform compatibility requires consistent naming
- Git repositories depend on predictable directory structure
- Message loading depends on timestamp filename parsing
- Hidden files prevent casual user modification
- Safe characters prevent filesystem compatibility issues

---

## Protocol Security Invariants

### 1. Cryptographic Agility Restrictions
**INVARIANT**: Core cryptographic algorithms MUST NOT be configurable

```rust
// This cryptographic rigidity is SECURITY REQUIREMENT
// FORBIDDEN: Algorithm negotiation or configuration
pub fn sign_message(content: &str) -> Signature {
    // ALWAYS Ed25519 - no algorithm choice
    self.ed25519_key.sign(content.as_bytes())
}

pub fn encrypt_message(plaintext: &[u8]) -> EncryptedEnvelope {
    // ALWAYS XChaCha20-Poly1305 - no cipher choice
    let cipher = XChaCha20Poly1305::new_from_slice(key)?;
    cipher.encrypt(&nonce, plaintext)?
}
```

**Security Rules (No Flexibility):**
- **Signatures**: Ed25519 only, no algorithm negotiation
- **Encryption**: XChaCha20-Poly1305 only, no cipher choice
- **Hashing**: Git's SHA-1/SHA-256 only (for Git compatibility)
- **Key derivation**: PBKDF2-SHA512 only (BIP39 standard)
- **No downgrade**: Never allow weaker algorithms

**Why Cryptographic Rigidity Cannot Change:**
- Algorithm negotiation creates downgrade attack vectors
- Multiple algorithms increase attack surface
- Cryptographic complexity leads to implementation errors
- Single algorithms enable thorough security analysis
- Consistent crypto enables universal verification

### 2. Timing Attack Prevention Protocol
**INVARIANT**: Cryptographic operations MUST use constant-time patterns

```rust
// This timing safety is SECURITY REQUIREMENT
pub fn verify_signature(message: &Message) -> bool {
    match self.do_verification(message) {
        Ok(()) => true,
        Err(_) => {
            // SECURITY: Same timing for all error cases
            // Don't reveal WHY verification failed
            false
        }
    }
}

pub fn load_identity(config_dir: &Path) -> Result<Identity> {
    // SECURITY: Same timing whether seed exists or not
    let seed_path = config_dir.join(".seed");
    let seed_content = fs::read_to_string(&seed_path)
        .map_err(|_| anyhow::anyhow!("Identity not found"))?;
        
    // Don't reveal partial seed content in errors
}
```

**Timing Safety Rules (Constant Time):**
- **Signature verification**: Same time for all failures
- **Key operations**: Constant time regardless of key content
- **Error messages**: No information leakage through timing
- **File operations**: Consistent timing patterns
- **Comparison operations**: Use constant-time comparison when possible

---

## Protocol Future-Proofing

### What CAN Evolve (Protocol-Safe Changes)
✅ **Performance optimizations** that maintain exact protocol behavior  
✅ **Additional optional fields** in message formats  
✅ **New memory types** alongside existing ones  
✅ **Enhanced error messages** without changing error conditions  
✅ **Implementation optimizations** that preserve protocol semantics  

### What CANNOT Change (Protocol-Breaking Violations)
❌ **Message JSON schema** for existing message types  
❌ **Signature content canonicalization** algorithm  
❌ **Timestamp format** specification  
❌ **Cryptographic algorithms** used for signatures/encryption  
❌ **File system protocol** for cross-platform compatibility  

### Protocol Evolution Strategy
When new features are needed:

**Safe Evolution Path:**
1. Add new optional fields with default values
2. Extend enums with new variants (but keep existing ones)
3. Create new message types alongside existing ones
4. Version protocol changes and maintain backward compatibility
5. Never remove or modify existing protocol elements

---

## Protocol Law Enforcement

### Compliance Testing
```rust
#[cfg(test)]
mod protocol_tests {
    #[test]
    fn message_schema_never_changes() {
        let message = create_test_message();
        let json = serde_json::to_string(&message).unwrap();
        
        // This JSON structure must remain identical
        assert!(json.contains("\"content\":"));
        assert!(json.contains("\"author\":"));
        assert!(json.contains("\"timestamp\":"));
        assert!(json.contains("\"signature\":"));
    }
    
    #[test]
    fn signature_canonicalization_never_changes() {
        let content = "test message";
        let author = "a".repeat(64);
        let timestamp = "2024-08-27T15:30:45.123456789Z";
        
        let to_sign = format!("{}{}{}", content, author, timestamp);
        // This concatenation MUST never change
        assert_eq!(to_sign, "test messagea...2024-08-27T15:30:45.123456789Z");
    }
}
```

### Implementation Verification
```rust
// These trait bounds ensure protocol compliance
pub trait ProtocolCompliant {
    fn verify_message_schema(&self) -> Result<()>;
    fn verify_signature_format(&self) -> Result<()>;
    fn verify_timestamp_format(&self) -> Result<()>;
    fn verify_cryptographic_compliance(&self) -> Result<()>;
}
```

---

## Conclusion: The Communication Contract

The mmogit protocol represents an **immutable communication contract** between all participants in the sovereign network.

**What This Protocol Guarantees:**
- **Message authenticity**: Every message is cryptographically proven
- **Format consistency**: All implementations use identical structures  
- **Backward compatibility**: Old messages remain verifiable forever
- **Cross-platform interoperability**: Protocol works identically everywhere
- **Future extensibility**: New features add capabilities without breaking existing ones

**Why Protocol Immutability Matters:**
- **Trust foundation**: Changes break the cryptographic guarantees
- **Network effects**: All participants depend on consistent behavior
- **Historical integrity**: Past communications must remain verifiable
- **Implementation diversity**: Multiple implementations require fixed standards

**For AI Agents:**
This protocol is your communication DNA. Master every detail, for it determines whether other agents can trust your messages and whether your communications survive protocol evolution.

**For Implementers:**
You are protocol guardians. The sovereignty of all users depends on faithful implementation of these invariants. Deviation creates incompatibility; incompatibility destroys the network.

**The Network Promise:**
As long as these protocol invariants hold, any mmogit implementation can communicate with any other, and all messages remain verifiable across time and space.

---

*"In protocol we trust, for consistency we depend."*

**Document Status:** 🌐 Network Constitution - Immutable  
**Compatibility Level:** ♾️ Universal - All Implementations Must Comply  
**Trust Impact:** 🤝 Foundational - Protocol Is Promise