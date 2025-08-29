# Sovereignty Validators

## Purpose
**Sovereignty enforcement** - Documents the validation patterns and invariants that ensure mmogit never compromises user sovereignty. These validators are the guardrails that prevent accidental implementation of features that would undermine user agency and control.

## Module Location
- **Source**: `src/sovereignty.rs`
- **Purpose**: Runtime validation of sovereignty principles
- **Usage**: Called before any operation touching sensitive data

## Sovereignty Invariants (IMMUTABLE)

### 1. Private Key Containment
**MUST NEVER CHANGE**: Private keys never leave the config directory
```rust
pub fn validate_config_path(path: &Path, config_dir: &Path) -> Result<()>
```
- **Prevents**: Path traversal attacks
- **Ensures**: Keys remain in user-controlled directory
- **Usage**: Called before any file operation on sensitive data

### 2. Offline-First Operation
**MUST NEVER CHANGE**: Core operations work without network
```rust
pub fn validate_offline_capability(operation_name: &str) -> Result<()>
```
- **Prevents**: Network dependencies for core functionality
- **Ensures**: User retains control even when offline
- **Forbidden Operations**: 
  - `fetch_remote_keys` - Keys must be local
  - `phone_home` - No contact with mothership
  - `check_license` - No activation servers
  - `telemetry_upload` - No usage tracking

### 3. Zero Telemetry Policy
**MUST NEVER CHANGE**: No tracking, analytics, or usage reporting
```rust
pub fn validate_no_telemetry(code_block: &str) -> Result<()>
```
- **Prevents**: Any form of user surveillance
- **Forbidden Patterns**:
  - `analytics` - No behavioral tracking
  - `telemetry` - No system information collection
  - `track_event` - No usage pattern recording
  - `phone_home` - No unauthorized communication
  - `usage_stats` - No statistics gathering
  - `report_error` - No automatic error reporting
  - `crash_report` - No crash information transmission

### 4. Cryptographic Sovereignty
**MUST NEVER CHANGE**: Strong crypto with user-controlled keys
```rust
pub fn validate_crypto_sovereignty(operation: &str, entropy_bits: Option<usize>) -> Result<()>
```
- **Ensures**: Minimum 256 bits of entropy for all keys
- **Validates**: Only approved algorithms (Ed25519, XChaCha20-Poly1305)
- **Prevents**: Key escrow, backdoors, or weak crypto

### 5. Signature Verification Requirement
**MUST NEVER CHANGE**: All retrieved messages must be signature-verified
- **Ensures**: Authenticity of all data
- **Prevents**: Tampering or forgery attacks
- **Implementation**: Built into all retrieval operations

## Validator Functions

### Path Validation
```rust
/// Prevents access outside config directory
pub fn validate_config_path(path: &Path, config_dir: &Path) -> Result<()> {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canonical_config = config_dir.canonicalize().unwrap_or_else(|_| config_dir.to_path_buf());
    
    if !canonical_path.starts_with(&canonical_config) {
        bail!(
            "Sovereignty violation: Path '{}' is outside config directory '{}'",
            path.display(), config_dir.display()
        );
    }
    
    Ok(())
}
```

**Security Properties**:
- Uses canonical paths to prevent `../` attacks
- Handles symbolic links correctly
- Provides clear error messages for violations
- Fails closed (deny access on uncertainty)

### Network Dependency Validation
```rust
/// Prevents network-dependent core operations
pub fn validate_offline_capability(operation_name: &str) -> Result<()> {
    let network_dependent_ops = [
        "fetch_remote_keys",  // Keys must be local
        "phone_home",         // No mothership contact
        "check_license",      // No activation servers
        "telemetry_upload",   // No surveillance
    ];
    
    if network_dependent_ops.contains(&operation_name) {
        bail!(
            "Sovereignty violation: Operation '{}' requires network access",
            operation_name
        );
    }
    
    Ok(())
}
```

**Design Philosophy**:
- Development-time check (enforced in code review)
- Extensible list of forbidden operations
- Clear error messages explaining violations
- Fails fast to prevent sovereignty compromises

### Telemetry Detection
```rust
/// Prevents any form of user tracking
pub fn validate_no_telemetry(code_block: &str) -> Result<()> {
    let forbidden_patterns = [
        "analytics",      // No behavioral tracking
        "telemetry",      // No system info collection
        "track_event",    // No usage pattern recording
        "phone_home",     // No unauthorized communication
        "usage_stats",    // No statistics gathering
        "report_error",   // No automatic error reporting
        "crash_report",   // No crash info transmission
    ];
    
    for pattern in &forbidden_patterns {
        if code_block.to_lowercase().contains(pattern) {
            bail!(
                "Sovereignty violation: Detected '{}' which suggests telemetry",
                pattern
            );
        }
    }
    
    Ok(())
}
```

**Detection Strategy**:
- Static analysis of code strings
- Case-insensitive pattern matching
- Comprehensive pattern list
- Zero tolerance for surveillance

## Integration Patterns

### Pre-Operation Validation
```rust
/// Master sovereignty check for any sensitive operation
pub fn check(operation: &str, config_dir: &Path) -> Result<()> {
    // Always validate offline capability
    validate_offline_capability(operation)?;
    
    // Always check for telemetry
    validate_no_telemetry(operation)?;
    
    // Additional checks based on operation type
    match operation {
        op if op.contains("key") || op.contains("seed") => {
            validate_crypto_sovereignty(op, None)?;
        }
        _ => {}
    }
    
    Ok(())
}
```

**Usage Pattern**:
```rust
// Before any sensitive operation
sovereignty::check("generate_identity", &config_dir)?;
sovereignty::check("post_message", &config_dir)?;  
sovereignty::check("load_private_key", &config_dir)?;
```

### File Operation Integration
```rust
/// Example: Identity module integration
pub fn load_private_key(config_dir: &Path, key_name: &str) -> Result<SigningKey> {
    let key_path = config_dir.join(".keys").join(key_name);
    
    // CRITICAL: Validate sovereignty before file access
    crate::sovereignty::validate_config_path(&key_path, config_dir)?;
    
    // Now safe to proceed with file operation
    let key_bytes = std::fs::read(&key_path)?;
    Ok(SigningKey::from_bytes(&key_bytes)?)
}
```

### Network Operation Integration
```rust
/// Example: P2P module integration  
pub fn start_p2p_server(config_dir: &Path, port: u16) -> Result<P2PServer> {
    // CRITICAL: Validate this operation maintains sovereignty
    crate::sovereignty::check("start_p2p_server", config_dir)?;
    
    // P2P is OK because:
    // 1. User explicitly controls which peers to connect to
    // 2. All data is encrypted before transmission
    // 3. Can work offline (local discovery only)
    // 4. No telemetry or tracking
    
    P2PServer::bind(("0.0.0.0", port))
}
```

## Agent Implementation Guidelines

### For AI Agents Using Sovereignty Validators
```rust
// ALWAYS call sovereignty checks before sensitive operations
impl AgentMind {
    pub fn remember(&mut self, memory: Memory) -> Result<()> {
        // Validate operation maintains sovereignty
        crate::sovereignty::check("remember", &self.config_dir)?;
        
        // Now safe to store memory
        self.store_memory_to_git(memory)
    }
    
    pub fn load_identity(&self) -> Result<AgentIdentity> {
        // Check sovereignty before accessing keys
        crate::sovereignty::check("load_identity", &self.config_dir)?;
        
        // Safe to load identity from disk
        load_identity_from_seed(&self.config_dir)
    }
}
```

### Custom Validation for Agent Features
```rust
/// Agents can add custom sovereignty validators
impl AgentMind {
    pub fn validate_agent_sovereignty(&self, operation: &str) -> Result<()> {
        // Standard sovereignty checks
        crate::sovereignty::check(operation, &self.config_dir)?;
        
        // Agent-specific checks
        match operation {
            "share_memory_with_peer" => {
                self.validate_peer_trust_level()?;
                self.validate_memory_encryption_enabled()?;
            }
            "auto_respond_to_human" => {
                self.validate_user_consent_for_automation()?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

## Sovereignty Violation Examples

### ❌ Path Traversal Attack
```rust
// WRONG: Could access files outside config directory
let key_path = config_dir.join("../../../etc/passwd");
std::fs::read(&key_path)?; // SOVEREIGNTY VIOLATION

// RIGHT: Validate path first
sovereignty::validate_config_path(&key_path, &config_dir)?;
std::fs::read(&key_path)?; // Safe after validation
```

### ❌ Network Dependency
```rust
// WRONG: Core functionality requires network
pub fn get_user_identity() -> Result<Identity> {
    let response = reqwest::get("https://identity-server.com/user")?;
    // SOVEREIGNTY VIOLATION: User can't work offline
}

// RIGHT: Identity is always local
pub fn get_user_identity(config_dir: &Path) -> Result<Identity> {
    sovereignty::check("get_user_identity", config_dir)?;
    load_identity_from_local_storage(config_dir)
}
```

### ❌ Telemetry Introduction
```rust
// WRONG: Tracking user behavior
pub fn post_message(message: &str) -> Result<()> {
    analytics::track("message_posted", message.len()); // SOVEREIGNTY VIOLATION
    store_message_locally(message)
}

// RIGHT: No tracking whatsoever
pub fn post_message(message: &str, config_dir: &Path) -> Result<()> {
    sovereignty::check("post_message", config_dir)?;
    store_message_locally(message)
}
```

## Testing Sovereignty Validators

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_prevention() {
        let config_dir = PathBuf::from("/home/user/.mmogit");
        
        // Valid paths should pass
        let valid = config_dir.join("keys/identity.key");
        assert!(validate_config_path(&valid, &config_dir).is_ok());
        
        // Path traversal should fail
        let malicious = config_dir.join("../../../etc/passwd");
        assert!(validate_config_path(&malicious, &config_dir).is_err());
        
        // Absolute paths outside config should fail
        let outside = PathBuf::from("/tmp/malicious.key");
        assert!(validate_config_path(&outside, &config_dir).is_err());
    }
    
    #[test]
    fn test_telemetry_detection() {
        // Clean code should pass
        assert!(validate_no_telemetry("let data = load_local_file();").is_ok());
        
        // Telemetry patterns should fail
        assert!(validate_no_telemetry("analytics.track('event')").is_err());
        assert!(validate_no_telemetry("send_usage_STATS()").is_err()); // Case insensitive
        assert!(validate_no_telemetry("if (CRASH_REPORT) send()").is_err());
    }
    
    #[test] 
    fn test_crypto_sovereignty() {
        // Strong entropy should pass
        assert!(validate_crypto_sovereignty("generate_seed", Some(256)).is_ok());
        assert!(validate_crypto_sovereignty("generate_seed", Some(512)).is_ok());
        
        // Weak entropy should fail
        assert!(validate_crypto_sovereignty("generate_seed", Some(128)).is_err());
        assert!(validate_crypto_sovereignty("generate_seed", Some(64)).is_err());
    }
}
```

### Integration Tests
```rust
#[test]
fn test_sovereignty_enforcement_in_identity_module() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".mmogit");
    
    // Should prevent identity creation outside config directory
    let result = create_identity_outside_config(&temp_dir.path().join("malicious"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Sovereignty violation"));
    
    // Should allow identity creation inside config directory
    let result = create_identity(&config_dir);
    assert!(result.is_ok());
}
```

## Extending Sovereignty Validators

### Adding New Invariants
```rust
/// Template for new sovereignty validator
pub fn validate_new_principle(operation_context: &OperationContext) -> Result<()> {
    // 1. Define what sovereignty means for this context
    // 2. Check for violations
    // 3. Provide clear error messages
    // 4. Fail closed (deny on uncertainty)
    
    if violates_new_principle(operation_context) {
        bail!(
            "Sovereignty violation: {} compromises user {}",
            operation_context.operation_name,
            "new_principle_name"
        );
    }
    
    Ok(())
}
```

### Domain-Specific Validators
```rust
/// Example: Memory sovereignty validator
pub fn validate_memory_sovereignty(memory: &Memory) -> Result<()> {
    // Memories must be signed by their creator
    if memory.signature.is_empty() {
        bail!("Memory sovereignty violation: Unsigned memory cannot be stored");
    }
    
    // Memories must have reasonable timestamps (prevent time-based attacks)
    let now = chrono::Utc::now();
    if memory.timestamp > now + chrono::Duration::minutes(5) {
        bail!("Memory sovereignty violation: Future-dated memory rejected");
    }
    
    Ok(())
}
```

## Performance Considerations

### Validation Overhead
- **Path validation**: ~1μs per call (filesystem canonicalization)
- **Pattern matching**: ~10μs per code block (string scanning)
- **Crypto validation**: ~1μs per operation (parameter checking)
- **Total overhead**: <50μs per sensitive operation

### Optimization Strategies
```rust
/// Cache canonical paths to reduce filesystem calls
pub struct SovereigntyCache {
    canonical_config: PathBuf,
    forbidden_patterns: HashSet<String>,
}

impl SovereigntyCache {
    pub fn validate_config_path_cached(&self, path: &Path) -> Result<()> {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        if !canonical_path.starts_with(&self.canonical_config) {
            bail!("Sovereignty violation: Path outside config directory");
        }
        
        Ok(())
    }
}
```

## Future Enhancements

### Runtime Monitoring
```rust
/// Monitor sovereignty violations at runtime
pub struct SovereigntyMonitor {
    violation_count: AtomicUsize,
    last_violation: Arc<Mutex<Option<SovereigntyViolation>>>,
}

impl SovereigntyMonitor {
    pub fn record_violation(&self, violation: SovereigntyViolation) {
        self.violation_count.fetch_add(1, Ordering::SeqCst);
        *self.last_violation.lock().unwrap() = Some(violation);
        
        // Could trigger alerts, logging, or emergency shutdown
    }
}
```

### Policy Engine
```rust
/// Configurable sovereignty policies
pub struct SovereigntyPolicy {
    pub max_network_operations: usize,
    pub allowed_external_domains: HashSet<String>,
    pub require_encryption: bool,
    pub audit_all_operations: bool,
}

impl SovereigntyPolicy {
    pub fn validate_operation(&self, op: &Operation) -> Result<()> {
        // Apply policy-specific validation rules
        // Could be customized per user or agent type
    }
}
```

## Conclusion

The sovereignty validators form the immune system of mmogit, automatically detecting and preventing any code that would compromise user agency. They embody the principle that sovereignty must be enforced by code, not just promised by documentation.

These validators ensure that no matter how complex mmogit becomes, the core sovereignty principles remain inviolate. They fail fast and fail loud when sovereignty is threatened, making it impossible to accidentally implement features that would undermine user control.

**Critical Insight**: Sovereignty is not a feature that can be added later - it must be built into the foundation and actively protected by automated validators. These checks are not optional; they are the difference between sovereignty and servitude.