# Crate: anyhow

## Purpose
**Sovereign error handling** - Provides ergonomic error propagation that maintains context while keeping error handling simple and transparent. In sovereignty systems, errors must be debuggable by users without vendor tools or hidden diagnostics.

## Version
- Current: 1.0.99
- Minimum supported: 1.0.70+
- MSRV: Rust 1.39.0

## Security Audit
- Last audited: 2023 (widely used, heavily scrutinized)
- Known CVEs: None
- Sovereignty concerns: **NONE** - Pure error handling abstraction, no dependencies

## Sovereignty Alignment
**EXCELLENT** - Anyhow supports sovereignty principles:
- Transparent error chains (no hidden vendor diagnostics)
- Human-readable error messages
- No network dependencies or telemetry
- Source code visibility into all error handling
- Works completely offline

## Usage in mmogit

### Core Error Type
```rust
use anyhow::{Result, Context, bail};

// Every mmogit function returns anyhow::Result
pub fn initialize_agent(config_dir: &Path) -> Result<AgentIdentity> {
    let seed_path = config_dir.join(".seed");
    
    // Context adds sovereignty-friendly error messages
    let mnemonic_str = std::fs::read_to_string(&seed_path)
        .context("Failed to read seed file - check permissions and path")?;
    
    let mnemonic = bip39::Mnemonic::parse(&mnemonic_str)
        .context("Invalid seed phrase - verify backup is correct")?;
    
    Ok(derive_identity(&mnemonic)?)
}
```

### Error Context Chains
```rust
/// Build rich error context for sovereignty debugging
pub fn create_git_repository(config_dir: &Path) -> Result<git2::Repository> {
    let repo_path = config_dir.join("messages");
    
    std::fs::create_dir_all(&repo_path)
        .with_context(|| format!("Cannot create directory: {}", repo_path.display()))?;
    
    git2::Repository::init(&repo_path)
        .with_context(|| format!("Git initialization failed in: {}", repo_path.display()))?
        .ok_or_else(|| anyhow::anyhow!("Git init returned None unexpectedly"))
}
```

### Sovereignty Error Messages
```rust
/// Error messages that help users maintain sovereignty
pub fn validate_sovereignty_invariants(config_dir: &Path, operation: &str) -> Result<()> {
    // Check config directory is within user's home
    let home = dirs::home_dir()
        .context("Cannot determine home directory - check environment")?;
    
    let canonical_config = config_dir.canonicalize()
        .context("Config directory path invalid - check if it exists")?;
    
    let canonical_home = home.canonicalize()
        .context("Home directory not accessible - check permissions")?;
    
    if !canonical_config.starts_with(&canonical_home) {
        bail!(
            "Sovereignty violation in {}: config directory '{}' is outside home directory '{}'.\n\
            This prevents proper isolation and could expose keys to other users.",
            operation,
            canonical_config.display(),
            canonical_home.display()
        );
    }
    
    Ok(())
}
```

## Error Handling Patterns

### Network Operation Errors
```rust
/// P2P connection errors with helpful context
pub fn connect_to_peer(address: &str, our_pubkey: &str) -> Result<PeerConnection> {
    let stream = std::net::TcpStream::connect(address)
        .with_context(|| format!(
            "Cannot connect to peer at {} - check address and network connectivity",
            address
        ))?;
    
    stream.set_read_timeout(Some(Duration::from_secs(30)))
        .context("Failed to set socket read timeout")?;
    
    // Send our identification
    let hello_msg = create_hello_message(our_pubkey)
        .context("Failed to create hello message - check identity")?;
    
    send_message(&stream, &hello_msg)
        .context("Failed to send hello to peer - connection may be broken")?;
    
    Ok(PeerConnection::new(stream))
}
```

### Git Operation Errors
```rust
/// Git errors with sovereignty context
pub fn commit_memory(repo: &git2::Repository, memory: &Memory) -> Result<git2::Oid> {
    let json_content = serde_json::to_string_pretty(memory)
        .context("Failed to serialize memory to JSON")?;
    
    let blob_id = repo.blob(json_content.as_bytes())
        .context("Failed to create git blob - repository may be corrupted")?;
    
    let signature = create_git_signature(&memory.pubkey)
        .context("Failed to create git signature - check identity configuration")?;
    
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Memory: {}", memory.id),
        &create_tree(repo, blob_id)?,
        &[]
    )
    .with_context(|| format!(
        "Failed to commit memory '{}' - check repository state and permissions", 
        memory.id
    ))
}
```

### Identity and Cryptography Errors
```rust
/// Crypto errors with sovereignty guidance
pub fn sign_message(message: &str, config_dir: &Path) -> Result<String> {
    let seed_path = config_dir.join(".seed");
    
    if !seed_path.exists() {
        bail!(
            "No identity found at '{}'. Run 'mmogit init' to create your sovereign identity.",
            seed_path.display()
        );
    }
    
    let mnemonic_str = std::fs::read_to_string(&seed_path)
        .context("Cannot read seed file - check file permissions")?;
    
    let mnemonic = bip39::Mnemonic::parse(&mnemonic_str)
        .context("Seed file corrupted - restore from backup")?;
    
    let signing_key = derive_signing_key(&mnemonic)
        .context("Failed to derive signing key from seed")?;
    
    let signature = signing_key.sign(message.as_bytes());
    Ok(hex::encode(signature.to_bytes()))
}
```

## Why Anyhow Over Alternatives?

### vs. `std::error::Error`
- **Ergonomics**: `?` operator works seamlessly
- **Context**: Rich error chains instead of bare error types
- **Debugging**: Better error messages for sovereignty debugging

### vs. `thiserror`
- **Simplicity**: Don't need custom error types for most cases
- **Flexibility**: Can handle any error type uniformly
- **Rapid development**: Less boilerplate for error handling

### vs. `eyre`
- **Stability**: Anyhow is more mature and stable
- **Ecosystem**: Better integration with existing Rust libraries
- **Simplicity**: Fewer features means fewer places for bugs

### vs. Manual `Result<T, Box<dyn Error>>`
- **Context chains**: Anyhow preserves full error context
- **Ergonomics**: Much less boilerplate
- **Debugging**: Better error formatting and display

## Error Classification for Sovereignty

### User-Fixable Errors
```rust
// Errors that users can resolve themselves
pub fn handle_user_errors() -> Result<()> {
    // File permission errors
    std::fs::File::open("sensitive_file")
        .context("Cannot open file - check permissions with 'ls -la'")?;
    
    // Configuration errors  
    validate_config_file()
        .context("Invalid configuration - check syntax with JSON validator")?;
    
    // Network connectivity
    connect_to_peer("bad_address")
        .context("Cannot connect - check network and peer address")?;
    
    Ok(())
}
```

### System-Level Errors
```rust
// Errors indicating deeper system issues
pub fn handle_system_errors() -> Result<()> {
    // Out of disk space
    create_large_file()
        .context("Insufficient disk space - free up storage or choose different location")?;
    
    // Missing system dependencies
    check_git_installation()
        .context("Git not installed - install with system package manager")?;
    
    // Corrupted data
    verify_repository_integrity()
        .context("Repository corrupted - restore from backup or reinitialize")?;
    
    Ok(())
}
```

### Programming Errors
```rust
// Errors indicating bugs in mmogit itself
pub fn handle_programming_errors() -> Result<()> {
    // Logic errors that shouldn't happen
    if invalid_state() {
        bail!("Internal consistency error - this is a bug, please report");
    }
    
    // Unexpected data formats
    parse_memory_format(&data)
        .context("Unexpected memory format - possible mmogit version mismatch")?;
    
    Ok(())
}
```

## Context Strategies

### Progressive Context Building
```rust
/// Build context as operations nest deeper
pub fn complex_operation() -> Result<()> {
    initialize_system()
        .context("System initialization failed")?;
    
    load_configuration()
        .context("Configuration loading failed during system startup")?;
    
    connect_to_peers()
        .context("Peer connection failed after configuration loaded")?;
    
    start_services()
        .context("Service startup failed with peers connected")?;
    
    Ok(())
}
```

### Conditional Context
```rust
/// Add different context based on conditions
pub fn conditional_context(is_first_run: bool) -> Result<()> {
    let result = load_identity();
    
    if is_first_run {
        result.context("Identity creation failed - check entropy source and disk space")?;
    } else {
        result.context("Identity loading failed - check seed file exists and is readable")?;
    }
    
    Ok(())
}
```

### Sovereignty-Aware Context
```rust
/// Context messages that preserve user agency
pub fn sovereignty_context() -> Result<()> {
    enable_encryption()
        .context(
            "Encryption setup failed. Your data remains unprotected.\n\
            Fix this before syncing with remotes to maintain sovereignty."
        )?;
    
    verify_key_ownership()
        .context(
            "Key verification failed. This may indicate:\n\
            1. Corrupted seed file (restore from backup)\n\
            2. Unauthorized key modification (security issue)\n\
            3. File permission problem (check ownership)"
        )?;
    
    Ok(())
}
```

## Performance Characteristics

### Error Allocation
- **Zero cost when no error**: `Result::Ok` has no overhead
- **Error allocation**: Only when error occurs, not on happy path
- **Context chains**: Small overhead per context added (~100 bytes)

### Memory Usage
```rust
// Anyhow errors are typically small
let error_size = std::mem::size_of::<anyhow::Error>(); // ~24 bytes
let result_size = std::mem::size_of::<Result<(), anyhow::Error>>(); // ~32 bytes

// Context chains add minimal overhead
let with_context = add_context(error);
// ~100 bytes total including error message
```

### Performance vs Alternatives
- **vs std::error**: Identical performance for success cases
- **vs custom errors**: Slightly more allocation on errors
- **vs panic**: Much better performance than unwinding

## Integration Patterns

### With Structured Logging
```rust
use tracing::{error, warn, info};

/// Combine anyhow with structured logging
pub fn logged_operation() -> Result<()> {
    let result = risky_operation();
    
    match result {
        Ok(value) => {
            info!("Operation succeeded: {:?}", value);
            Ok(value)
        }
        Err(e) => {
            error!("Operation failed: {:?}", e);
            // Re-return error for caller to handle
            Err(e.context("Logged operation failed"))
        }
    }
}
```

### With Retry Logic
```rust
/// Combine anyhow with retry patterns
pub fn retry_with_context<F, T>(mut f: F, max_attempts: u32, operation: &str) -> Result<T> 
where
    F: FnMut() -> Result<T>,
{
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_attempts {
                    std::thread::sleep(Duration::from_millis(100 * attempt as u64));
                }
            }
        }
    }
    
    Err(last_error.unwrap())
        .with_context(|| format!("{} failed after {} attempts", operation, max_attempts))
}
```

## Testing Error Handling

### Unit Tests for Errors
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_error_context_chain() {
        let result = failing_operation();
        
        let error = result.unwrap_err();
        let error_chain = format!("{:?}", error);
        
        // Verify context messages are present
        assert!(error_chain.contains("outer context"));
        assert!(error_chain.contains("inner context")); 
        assert!(error_chain.contains("root cause"));
    }
    
    #[test]
    fn test_sovereignty_error_messages() {
        let result = sovereignty_violation();
        
        let error = result.unwrap_err();
        let error_msg = format!("{}", error);
        
        // Verify sovereignty guidance is included
        assert!(error_msg.contains("check permissions"));
        assert!(error_msg.contains("restore from backup"));
    }
}
```

## Common Pitfalls

### ❌ Too Much Context
```rust
// WRONG: Redundant context that doesn't add value
let result = std::fs::read_to_string("file.txt")
    .context("Failed to read file")  // Redundant
    .context("File reading failed")  // Also redundant  
    .context("Could not read file"); // Still redundant
```

### ❌ Generic Context Messages  
```rust
// WRONG: Vague context that doesn't help debugging
let result = complex_operation()
    .context("Operation failed");  // Not helpful

// RIGHT: Specific context with actionable information
let result = complex_operation()
    .context("Failed to initialize P2P networking - check firewall and port 7420");
```

### ❌ Losing Error Information
```rust
// WRONG: Converting errors loses context
let result = detailed_operation()
    .map_err(|_| anyhow::anyhow!("Generic error"))?; // Loses original error

// RIGHT: Preserve original error with context
let result = detailed_operation()
    .context("Generic error occurred in detailed operation")?;
```

## Future Enhancements

### Error Recovery
```rust
/// Automatic error recovery for sovereignty
pub fn self_healing_operation() -> Result<()> {
    match primary_operation() {
        Ok(result) => Ok(result),
        Err(e) if is_recoverable(&e) => {
            warn!("Primary operation failed, attempting recovery: {}", e);
            recovery_operation()
                .with_context(|| format!("Recovery failed after: {}", e))
        }
        Err(e) => Err(e),
    }
}
```

### Structured Error Data
```rust
/// Rich error context for programmatic handling
#[derive(Debug)]
pub struct SovereigntyError {
    pub operation: String,
    pub fix_suggestions: Vec<String>,
    pub sovereignty_impact: SovereigntyImpact,
    pub original_error: anyhow::Error,
}
```

## Conclusion

Anyhow provides exactly the error handling capabilities needed for a sovereignty system. Its emphasis on context chains and human-readable error messages aligns perfectly with our need for transparency and user debuggability.

In sovereignty systems, errors can't be black boxes that require vendor support to understand. Anyhow ensures that every error chain tells a complete story about what went wrong and how users can fix it themselves.

**Key Principle**: In sovereignty systems, error messages are user interfaces. Anyhow helps us build error interfaces that preserve agency rather than requiring dependence on external support.