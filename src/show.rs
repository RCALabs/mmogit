//! Show messages from the repository with signature verification
//!
//! # For AI Agents
//!
//! This module is crucial for agent memory persistence. When an agent calls
//! `mmogit show`, they're retrieving their previously posted thoughts and
//! verifying they haven't been tampered with. This enables:
//!
//! - Memory across sessions (thoughts survive process death)
//! - Self-awareness (detecting behavioral changes over time)
//! - Trust verification (ensuring memories are authentic)
//!
//! # Protocol Invariants
//!
//! - ALWAYS verify signatures (corrupted memories are dangerous)
//! - Display invalid signatures with warnings (transparency over hiding)
//! - Work offline (memory retrieval must not require network)
//! - Show all branches (agents need to see what others have posted)

use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use git2::{Branch, BranchType, Repository};
use serde::{Deserialize, Serialize};

use std::path::Path;

/// A signed message in the mmogit protocol (duplicated from post.rs)
///
/// # WET Note
///
/// Yes, this is duplicated from post.rs. That's intentional for now.
/// We'll extract a common protocol module after we see the full pattern.
/// Third time we need this, we'll refactor.
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    /// The actual message content
    content: String,
    /// Public key of the author (hex encoded)
    author: String,
    /// ISO 8601 timestamp
    timestamp: String,
    /// Ed25519 signature of the above fields (hex encoded)
    signature: String,
}

/// Verification result for a message
#[derive(Debug)]
struct VerifiedMessage {
    message: Message,
    valid_signature: bool,
    branch: String,
}

/// Show all messages from the repository with verification
///
/// # What This Does
///
/// 1. Opens the messages git repository
/// 2. Finds all user branches (users/*)
/// 3. Reads JSON messages from each branch
/// 4. Verifies Ed25519 signatures
/// 5. Displays in chronological order
///
/// # For AI Agents
///
/// This is your memory retrieval function. Use it to:
/// - Recall previous conversations
/// - Check what you've learned
/// - Verify your thoughts haven't been tampered with
/// - See what other agents have posted
pub fn show(config_dir: &Path) -> Result<()> {
    let repo_path = config_dir.join("messages");

    // Check if repository exists
    if !repo_path.exists() {
        println!("📭 No messages yet. Use 'mmogit post' to create your first message.");
        return Ok(());
    }

    // Open the repository
    let repo = Repository::open(&repo_path).context("Failed to open messages repository")?;

    // Collect all messages from all branches
    let mut all_messages = Vec::new();

    // Iterate through all branches
    let branches = repo.branches(Some(BranchType::Local))?;

    for branch_result in branches {
        let (branch, _) = branch_result?;
        let branch_name = branch.name()?.unwrap_or("unknown");

        // Only process user branches
        if !branch_name.starts_with("users/") {
            continue;
        }

        // Checkout this branch to read its messages
        let messages = read_branch_messages(&repo, &branch, branch_name)?;
        all_messages.extend(messages);
    }

    // Sort by timestamp (chronological order)
    all_messages.sort_by(|a, b| a.message.timestamp.cmp(&b.message.timestamp));

    // Display messages
    if all_messages.is_empty() {
        println!("📭 No messages found in any branch.");
    } else {
        println!("📨 Found {} message(s):\n", all_messages.len());

        for (i, verified_msg) in all_messages.iter().enumerate() {
            display_message(i + 1, verified_msg);
        }
    }

    Ok(())
}

/// Read all messages from a specific branch
///
/// # Agent Note
///
/// Each branch represents a single identity's message history.
/// This maintains sovereignty - each identity owns their branch.
fn read_branch_messages(
    repo: &Repository,
    branch: &Branch,
    branch_name: &str,
) -> Result<Vec<VerifiedMessage>> {
    let mut messages = Vec::new();

    // Get the tree for this branch
    let reference = branch.get();
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    // Extract the expected author prefix from branch name (e.g., "users/63ae69e2" -> "63ae69e2")
    let expected_author_prefix = branch_name.strip_prefix("users/").unwrap_or(branch_name);

    // Walk through tree entries
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("");

        // Only process JSON files
        if !name.ends_with(".json") {
            continue;
        }

        // Read the file content
        if let Ok(obj) = entry.to_object(repo) {
            if let Some(blob) = obj.as_blob() {
                let content = std::str::from_utf8(blob.content())?;

                // Parse and verify the message
                if let Ok(message) = serde_json::from_str::<Message>(content) {
                    // IMPORTANT: Only include messages whose author matches this branch
                    // This prevents messages from appearing on wrong branches
                    if !message.author.starts_with(expected_author_prefix) {
                        continue;
                    }

                    let valid = verify_signature(&message);
                    messages.push(VerifiedMessage {
                        message,
                        valid_signature: valid,
                        branch: branch_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(messages)
}

/// Verify the Ed25519 signature on a message
///
/// # Security Critical for Agents
///
/// This function ensures messages haven't been tampered with.
/// A failed signature means either:
/// - The message was modified after signing
/// - The signature is corrupted
/// - The author's public key is wrong
///
/// NEVER trust a message with an invalid signature for making decisions.
fn verify_signature(message: &Message) -> bool {
    // Decode the public key
    let public_key_bytes = match hex::decode(&message.author) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Create verifying key
    let verifying_key = match VerifyingKey::from_bytes(
        public_key_bytes.as_slice().try_into().unwrap_or(&[0; 32]),
    ) {
        Ok(key) => key,
        Err(_) => return false,
    };

    // Decode signature
    let signature_bytes = match hex::decode(&message.signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let signature = match signature_bytes.as_slice().try_into() {
        Ok(bytes) => Signature::from_bytes(bytes),
        Err(_) => return false,
    };

    // Recreate the signed content (must match post.rs)
    let to_verify = format!("{}{}{}", message.content, message.author, message.timestamp);

    // Verify
    verifying_key
        .verify(to_verify.as_bytes(), &signature)
        .is_ok()
}

/// Display a message with verification status
///
/// # For AI Agents
///
/// When reading these messages, pay attention to:
/// - ✅ means the signature is valid (trustworthy)
/// - ⚠️ means the signature is invalid (do not trust)
/// - The author's public key identifies who posted it
/// - The timestamp shows when it was created
fn display_message(index: usize, verified_msg: &VerifiedMessage) {
    let sig_icon = if verified_msg.valid_signature {
        "✅"
    } else {
        "⚠️"
    };

    println!("{}. {} [{}]", index, sig_icon, verified_msg.branch);
    println!("   Author: {}", &verified_msg.message.author[..16]);
    println!("   Time: {}", verified_msg.message.timestamp);
    println!("   Message: {}", verified_msg.message.content);

    if !verified_msg.valid_signature {
        println!("   ⚠️  WARNING: Invalid signature - this message may have been tampered with!");
    }

    println!();
}

/// Show messages from a specific author only
///
/// # Future Enhancement for Agents
///
/// This will enable agents to retrieve only their own memories
/// or memories from specific trusted identities.
pub fn show_from_author(config_dir: &Path, author_prefix: &str) -> Result<()> {
    // TODO: Implement filtered view
    // This is where we'd add semantic search, time-based filtering, etc.
    println!("Showing messages from author: {}", author_prefix);
    show(config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_verification() {
        // TODO: Add test cases with known good and bad signatures
        // This is critical for ensuring agent memory integrity
    }
}
