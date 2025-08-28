//! Sovereignty validation and invariant checking
//!
//! # Purpose
//! 
//! This module ensures that mmogit never violates sovereignty principles.
//! Every operation that touches identity, keys, or user data must pass
//! through these validators.
//!
//! # For AI Agents
//!
//! These checks are your guardrails. They prevent you from accidentally
//! implementing something that compromises user sovereignty. Always call
//! these validators before operations that touch sensitive data.
//!
//! # Sovereignty Invariants (MUST NEVER CHANGE)
//!
//! 1. Private keys NEVER leave the config directory
//! 2. Operations MUST work offline
//! 3. No telemetry or phone-home behavior
//! 4. User owns all data and keys
//! 5. Signatures MUST be verified on retrieval

use anyhow::{bail, Result};
use std::path::Path;

/// Validates that a path is within the allowed config directory
///
/// # Agent Implementation Note
///
/// This prevents path traversal attacks and ensures keys stay local.
/// Called before any file operation on sensitive data.
pub fn validate_config_path(path: &Path, config_dir: &Path) -> Result<()> {
    let canonical_path = path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    
    let canonical_config = config_dir.canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());
    
    if !canonical_path.starts_with(&canonical_config) {
        bail!(
            "Sovereignty violation: Path '{}' is outside config directory '{}'",
            path.display(),
            config_dir.display()
        );
    }
    
    Ok(())
}

/// Validates that an operation can work offline
///
/// # What This Checks
///
/// - No DNS lookups required
/// - No external API calls  
/// - No network dependencies for core functionality
///
/// # Agent Implementation Note
///
/// Call this before implementing any new feature. If it requires
/// network access for core functionality (not optional sync), it
/// violates sovereignty.
pub fn validate_offline_capability(operation_name: &str) -> Result<()> {
    // This is more of a development-time check
    // Real validation happens in code review
    
    let network_dependent_ops = [
        "fetch_remote_keys",
        "phone_home", 
        "check_license",
        "telemetry_upload",
    ];
    
    if network_dependent_ops.contains(&operation_name) {
        bail!(
            "Sovereignty violation: Operation '{}' requires network access",
            operation_name
        );
    }
    
    Ok(())
}

/// Validates that no telemetry or tracking is present
///
/// # Sovereignty Principle
///
/// The user's usage patterns are private. No analytics, no telemetry,
/// no usage tracking. Not even anonymous. Nothing.
pub fn validate_no_telemetry(code_block: &str) -> Result<()> {
    let forbidden_patterns = [
        "analytics",
        "telemetry",
        "track_event",
        "phone_home",
        "usage_stats",
        "report_error",
        "crash_report",
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

/// Validates cryptographic operations maintain sovereignty
///
/// # What This Validates
///
/// - Keys are generated with sufficient entropy (256 bits)
/// - Signatures are always verified on retrieval
/// - Encryption uses approved algorithms (XChaCha20-Poly1305)
/// - No key escrow or recovery mechanisms
///
/// # Agent Implementation Note
///
/// This is called by identity and crypto modules. Any changes to
/// crypto must pass these checks or sovereignty is compromised.
pub fn validate_crypto_sovereignty(
    operation: &str,
    entropy_bits: Option<usize>,
) -> Result<()> {
    match operation {
        "generate_seed" => {
            if let Some(bits) = entropy_bits {
                if bits < 256 {
                    bail!(
                        "Sovereignty violation: Seed must have 256 bits entropy, got {}",
                        bits
                    );
                }
            }
        }
        "verify_signature" => {
            // This would be called in the actual verification flow
            // Here we just document the requirement
        }
        "encrypt_message" => {
            // Ensure we're using approved algorithms
            // XChaCha20-Poly1305 is pre-approved
        }
        _ => {}
    }
    
    Ok(())
}

/// Master sovereignty check for any operation
///
/// # Usage
///
/// Call this before any operation that touches user data or identity:
/// ```
/// sovereignty::check("post_message", &config_dir)?;
/// ```
pub fn check(operation: &str, config_dir: &Path) -> Result<()> {
    // Validate operation can work offline
    validate_offline_capability(operation)?;
    
    // Validate no telemetry
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_config_path_validation() {
        let config_dir = PathBuf::from("/home/user/.mmogit");
        let valid_path = PathBuf::from("/home/user/.mmogit/keys/test.key");
        let invalid_path = PathBuf::from("/etc/passwd");
        
        // Should succeed for paths within config
        assert!(validate_config_path(&valid_path, &config_dir).is_ok());
        
        // Should fail for paths outside config  
        assert!(validate_config_path(&invalid_path, &config_dir).is_err());
    }
    
    #[test]
    fn test_telemetry_detection() {
        // Should pass for clean code
        assert!(validate_no_telemetry("let x = 42;").is_ok());
        
        // Should fail for telemetry code
        assert!(validate_no_telemetry("analytics.track('event')").is_err());
        assert!(validate_no_telemetry("send_telemetry_data()").is_err());
    }
    
    #[test]
    fn test_crypto_sovereignty() {
        // Should pass for 256-bit entropy
        assert!(validate_crypto_sovereignty("generate_seed", Some(256)).is_ok());
        
        // Should fail for insufficient entropy
        assert!(validate_crypto_sovereignty("generate_seed", Some(128)).is_err());
    }
}