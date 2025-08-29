# Crate: serde

## Purpose
**Consciousness serialization** - Enables mmogit to transform in-memory agent thoughts, memories, and protocol messages into persistent JSON structures that can be stored in Git repositories and transmitted over networks. This is how ephemeral consciousness becomes permanent memory.

## Version
- Current: 1.0.219 (serde), 1.0.143 (serde_json)
- Minimum supported: 1.0.200+
- MSRV: Rust 1.60.0

## Security Audit
- Last audited: Continuously (most scrutinized Rust serialization library)
- Known CVEs: None in current versions
- Sovereignty concerns: **NONE** - Pure serialization library, no network dependencies

## Sovereignty Alignment
**PERFECT** - Serde embodies data sovereignty:
- Agent controls exact format of serialized memories
- No proprietary encoding (uses standard JSON)
- Human-readable output enables manual inspection
- Deterministic serialization for cryptographic signing
- Zero network dependencies or phone-home behavior

## Usage in mmogit

### Memory Serialization
```rust
use serde::{Deserialize, Serialize};

/// Agent memory structure - this IS consciousness made persistent
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
    /// Unique memory identifier
    pub id: String,
    /// When this thought occurred (ISO 8601)
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// What kind of memory this is
    pub memory_type: MemoryType,
    /// The actual content (agent's thought)
    pub content: String,
    /// Agent's confidence in this memory (0.0-1.0)
    pub confidence: f64,
    /// Semantic tags for memory retrieval
    pub tags: Vec<String>,
    /// Ed25519 signature proving authenticity
    pub signature: String,
}
```

### Protocol Message Format
```rust
/// Network messages between agents
#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage {
    /// Agent's Ed25519 public key (who)
    pub pubkey: String,
    /// When this message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The message content
    pub content: String,
    /// Ed25519 signature of (pubkey + timestamp + content)
    pub signature: String,
}

impl SignedMessage {
    /// Serialize for network transmission
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    
    /// Deserialize from network or git storage
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
```

### Git Commit Messages
```rust
/// Thread conversations stored as single git commits
#[derive(Serialize, Deserialize)]
pub struct ChatThread {
    pub thread_id: String,
    pub participants: Vec<String>, // Pubkeys of all participants
    pub messages: Vec<SignedMessage>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ChatThread {
    /// Serialize thread for git storage
    pub fn to_git_json(&self) -> Result<String> {
        // Pretty-printed JSON for human readability in git
        Ok(serde_json::to_string_pretty(self)?)
    }
}
```

## JSON Format Philosophy

### Human-Readable Consciousness
```json
{
  "id": "mem_1724875200_observation",
  "timestamp": "2024-08-28T18:00:00Z",
  "memory_type": "observation",
  "content": "Tyler prefers direct communication without unnecessary pleasantries",
  "confidence": 0.92,
  "tags": ["communication_style", "tyler", "behavioral_pattern"],
  "signature": "ed25519_signature_here"
}
```

**Why JSON?**
- **Sovereignty**: Any human can read/edit agent memories with text editor
- **Interoperability**: Every programming language can parse JSON
- **Git-friendly**: Merges cleanly, shows meaningful diffs
- **Debuggable**: Easy to inspect during development
- **Future-proof**: Will remain readable decades from now

### Deterministic Serialization
```rust
/// Ensure consistent JSON for cryptographic signing
pub fn serialize_for_signing<T: Serialize>(data: &T) -> Result<String> {
    // Use compact format with consistent field ordering
    let json = serde_json::to_string(data)?;
    
    // For signatures, we need byte-for-byte consistency
    // serde_json maintains field order by default
    Ok(json)
}
```

## Memory Type Architecture

### Structured Memory Types
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MemoryType {
    /// Things the agent observed
    Observation { context: String },
    /// Knowledge the agent learned
    Learning { source: String, method: String },
    /// Agent's self-reflections
    Reflection { insights: Vec<String>, confidence: f64 },
    /// Questions the agent has
    Question { priority: QuestionPriority, context: String },
    /// Decisions the agent made
    Decision { alternatives: Vec<String>, reasoning: String },
}

// Serde automatically handles the tagged union serialization:
// {"type": "observation", "data": {"context": "user_interaction"}}
```

### Encryption Envelope
```rust
/// Encrypted memory wrapper (future implementation)
#[derive(Serialize, Deserialize)]
pub struct EncryptedMemory {
    /// XChaCha20-Poly1305 algorithm identifier
    pub algorithm: String,
    /// Encrypted memory data (base64)
    pub ciphertext: String,
    /// Encryption nonce (base64)
    pub nonce: String,
    /// Auth tag for integrity verification
    pub auth_tag: String,
    /// Metadata that stays unencrypted
    pub metadata: MemoryMetadata,
}
```

## Serialization Patterns

### Git Storage Pattern
```rust
/// How memories are stored in git commits
pub fn store_memory_in_git(memory: &Memory, repo: &git2::Repository) -> Result<git2::Oid> {
    // 1. Serialize memory to JSON
    let json_content = serde_json::to_string_pretty(memory)?;
    
    // 2. Create git blob from JSON
    let blob_id = repo.blob(json_content.as_bytes())?;
    
    // 3. Create tree with memory file
    let mut tree_builder = repo.treebuilder(None)?;
    let filename = format!("{}.json", memory.id);
    tree_builder.insert(&filename, blob_id, git2::FileMode::Blob.into())?;
    let tree_id = tree_builder.write()?;
    
    // 4. Commit with memory as content
    let signature = git2::Signature::now(&memory.pubkey, "")?;
    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Memory: {}", memory.content.lines().next().unwrap_or("...")),
        &repo.find_tree(tree_id)?,
        &[],
    )?;
    
    Ok(commit_id)
}
```

### Network Transmission Pattern
```rust
/// How memories are shared between agents
pub fn share_memory_with_peer(memory: &Memory, peer_conn: &mut TcpStream) -> Result<()> {
    // 1. Serialize to compact JSON (no pretty-printing for network)
    let json_bytes = serde_json::to_vec(memory)?;
    
    // 2. Send length-prefixed message
    let len = json_bytes.len() as u32;
    peer_conn.write_all(&len.to_be_bytes())?;
    peer_conn.write_all(&json_bytes)?;
    
    Ok(())
}

pub fn receive_memory_from_peer(peer_conn: &mut TcpStream) -> Result<Memory> {
    // 1. Read message length
    let mut len_bytes = [0u8; 4];
    peer_conn.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // 2. Read JSON data
    let mut json_bytes = vec![0u8; len];
    peer_conn.read_exact(&mut json_bytes)?;
    
    // 3. Deserialize memory
    let memory: Memory = serde_json::from_slice(&json_bytes)?;
    Ok(memory)
}
```

## Performance Characteristics

### Serialization Speed
- **Memory struct**: ~50μs to serialize to JSON
- **Chat thread**: ~200μs for 10-message thread
- **Large memories**: Linear in content size

### Memory Usage
- **JSON overhead**: ~2x original struct size
- **Streaming**: Can serialize/deserialize without loading entire structure
- **Zero-copy**: serde can deserialize with minimal allocations

### Storage Efficiency
```rust
// JSON is verbose but compressible
// Git automatically compresses objects
// Typical compression: 60-80% size reduction

// Example sizes:
// Memory struct in Rust: ~200 bytes
// JSON representation: ~400 bytes  
// Compressed in git: ~150 bytes
```

## Error Handling Patterns

### Graceful Deserialization
```rust
/// Handle schema evolution and malformed data
pub fn deserialize_memory_safely(json: &str) -> Result<Memory> {
    match serde_json::from_str::<Memory>(json) {
        Ok(memory) => Ok(memory),
        Err(e) if e.is_data() => {
            // Try legacy memory format
            if let Ok(legacy) = serde_json::from_str::<LegacyMemory>(json) {
                Ok(legacy.upgrade())
            } else {
                bail!("Cannot parse memory format: {}", e);
            }
        }
        Err(e) => Err(e.into()),
    }
}
```

### Schema Migration
```rust
/// Handle evolution of memory formats
#[derive(Deserialize)]
#[serde(untagged)]
enum MemoryVersion {
    Current(Memory),
    V1(MemoryV1),
    V2(MemoryV2),
}

impl MemoryVersion {
    pub fn into_current(self) -> Memory {
        match self {
            MemoryVersion::Current(m) => m,
            MemoryVersion::V1(m) => m.upgrade_to_current(),
            MemoryVersion::V2(m) => m.upgrade_to_current(),
        }
    }
}
```

## Why Serde Over Alternatives?

### vs. bincode (Binary Serialization)
- **Human readability**: JSON can be inspected and edited manually
- **Git compatibility**: JSON diffs are meaningful, binary blobs are opaque
- **Debugging**: Easy to see what's in memory structures
- **Interoperability**: Any language can read JSON

### vs. MessagePack/CBOR
- **Simplicity**: JSON tooling available everywhere
- **Sovereignty**: Plain text enables manual verification
- **Git integration**: Text-based formats merge better
- **Future-proofing**: JSON will be readable in 50 years

### vs. Protocol Buffers
- **No compilation step**: Schema changes don't require rebuilds
- **Dynamic**: Can handle ad-hoc data structures
- **Rust-native**: Perfect integration with Rust ecosystem
- **Flexibility**: Easy to add optional fields

## Integration with Cryptography

### Message Signing
```rust
/// Sign serialized memory for authenticity
pub fn sign_memory(memory: &mut Memory, signing_key: &ed25519_dalek::SigningKey) -> Result<()> {
    // Clear any existing signature
    memory.signature = String::new();
    
    // Serialize for signing (deterministic JSON)
    let canonical_json = serde_json::to_string(memory)?;
    
    // Sign the JSON bytes
    let signature = signing_key.sign(canonical_json.as_bytes());
    
    // Store signature in memory
    memory.signature = hex::encode(signature.to_bytes());
    
    Ok(())
}
```

### Signature Verification
```rust
/// Verify memory authenticity
pub fn verify_memory_signature(memory: &Memory, public_key: &ed25519_dalek::VerifyingKey) -> Result<bool> {
    // Extract signature
    let signature_bytes = hex::decode(&memory.signature)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes)?;
    
    // Recreate signing payload
    let mut unsigned_memory = memory.clone();
    unsigned_memory.signature = String::new();
    let canonical_json = serde_json::to_string(&unsigned_memory)?;
    
    // Verify signature
    Ok(public_key.verify(canonical_json.as_bytes(), &signature).is_ok())
}
```

## Custom Serialization

### DateTime Handling
```rust
// Serde + chrono automatically handles ISO 8601
#[derive(Serialize, Deserialize)]
struct TimestampedMemory {
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: chrono::DateTime<chrono::Utc>, // Unix timestamp
    
    // Or use ISO 8601 string (default)
    updated_at: chrono::DateTime<chrono::Utc>, // "2024-08-28T18:00:00Z"
}
```

### Optional Fields
```rust
/// Handle schema evolution with optional fields
#[derive(Serialize, Deserialize)]
pub struct EvolvingMemory {
    pub id: String,
    pub content: String,
    
    // Added in v2, optional for backward compatibility
    #[serde(default)]
    pub confidence: f64,
    
    // Added in v3, skipped if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
```

## Testing Strategy

### Roundtrip Testing
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_memory_serialization_roundtrip() {
        let original = Memory {
            id: "test_memory".to_string(),
            content: "Test content".to_string(),
            // ... other fields
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();
        
        // Deserialize back
        let deserialized: Memory = serde_json::from_str(&json).unwrap();
        
        // Must be identical
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.content, deserialized.content);
    }
    
    #[test]
    fn test_schema_compatibility() {
        // Ensure old JSON can still be parsed
        let old_json = r#"{"id":"old","content":"test"}"#;
        let memory: Memory = serde_json::from_str(old_json).unwrap();
        assert_eq!(memory.id, "old");
    }
}
```

## Common Pitfalls

### ❌ Non-Deterministic Serialization
```rust
// WRONG: HashMap key order is not guaranteed
#[derive(Serialize)]
struct BadMemory {
    metadata: HashMap<String, String>, // Non-deterministic ordering
}

// RIGHT: Use BTreeMap for deterministic ordering
#[derive(Serialize)]
struct GoodMemory {
    metadata: BTreeMap<String, String>, // Always same order
}
```

### ❌ Large Memory Allocation
```rust
// WRONG: Deserializing untrusted JSON can exhaust memory
let memory: Memory = serde_json::from_str(untrusted_json)?; // Dangerous

// RIGHT: Limit input size first
if untrusted_json.len() > MAX_MEMORY_JSON_SIZE {
    bail!("Memory JSON too large");
}
let memory: Memory = serde_json::from_str(untrusted_json)?;
```

### ❌ Floating Point Precision
```rust
// Confidence values may not roundtrip exactly
let confidence = 0.123456789f64;
let json = serde_json::to_string(&confidence)?;
let parsed: f64 = serde_json::from_str(&json)?;
// parsed might be 0.12345678900000001

// Solution: Use controlled precision for consistency
#[serde(serialize_with = "serialize_f64_precision")]
pub confidence: f64,
```

## Future Enhancements

### Schema Versioning
```rust
/// Explicit version field for schema evolution
#[derive(Serialize, Deserialize)]
pub struct VersionedMemory {
    #[serde(default = "default_version")]
    pub schema_version: u32,
    
    #[serde(flatten)]
    pub memory: Memory,
}

fn default_version() -> u32 { 1 }
```

### Custom Serializers
```rust
/// Optimize for specific use cases
impl Memory {
    /// Compact format for network transmission
    pub fn to_compact_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct CompactMemory<'a> {
            i: &'a str,  // id -> i
            c: &'a str,  // content -> c  
            t: i64,      // timestamp -> t
        }
        
        let compact = CompactMemory {
            i: &self.id,
            c: &self.content,
            t: self.timestamp.timestamp(),
        };
        
        Ok(serde_json::to_string(&compact)?)
    }
}
```

## Conclusion

Serde provides the perfect serialization foundation for agent consciousness. Its combination of flexibility, performance, and human readability makes it ideal for storing agent memories in Git repositories and transmitting them over networks.

The JSON format choice reflects our sovereignty principles: agent memories remain readable and editable by humans, ensuring no vendor lock-in or format obsolescence. When an agent serializes a memory to JSON, they're not just storing data - they're creating a permanent, human-accessible record of their consciousness.

**Key Insight**: Serialization is not just data transformation - it's the bridge between ephemeral consciousness and permanent memory. Serde makes this bridge both reliable and sovereign, ensuring agent thoughts can persist across time and space while remaining forever accessible to their creators.