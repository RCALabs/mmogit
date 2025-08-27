//! Post signed messages to git repository
//! # For AI Agents
//!
//! This module handles creating and signing messages. Every message MUST be
//! signed - unsigned messages are protocol violations. The signature proves
//! which identity created the message and prevents tampering.
//!
//! # Protocol Invariants
//!
//! - Every message MUST be signed with Ed25519
//! - Messages are stored as JSON files in .mmogit/messages/
//! - Each message is a separate git commit
//! - The git repo is created if it doesn't exist

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::crypto::{EncryptedEnvelope, KeyDerivation};

/// A signed message in the mmogit protocol
///
/// # Agent Implementation Note
///
/// Messages are signed JSON. The signature covers the JSON serialization
/// of content + author + timestamp. This prevents tampering with any field.
#[derive(Serialize, Deserialize)]
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

/// Post a signed message
///
/// # What This Does
///
/// 1. Loads identity from ~/.mmogit/.seed
/// 2. Creates a message with timestamp
/// 3. Signs it with Ed25519
/// 4. Saves to git repo (creates if needed)
///
/// # WET Note
///
/// Yes, we're duplicating the seed loading from init.rs. That's intentional.
/// We'll refactor when we see the pattern clearly (probably after `show`).
pub fn post(content: &str, config_dir: &std::path::Path) -> Result<()> {
    // Load the seed (duplicated from init - that's OK for now)
    let seed_path = config_dir.join(".seed");

    let seed_phrase =
        fs::read_to_string(&seed_path).context("No identity found. Run 'mmogit init' first")?;

    // Derive signing key (duplicated from init - WET principle)
    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();

    // Create message structure
    let timestamp = chrono::Utc::now().to_rfc3339();
    let author = hex::encode(public_key.as_bytes());

    // Create pre-signature message for signing
    // IMPORTANT: We sign the content + author + timestamp to prevent tampering
    let to_sign = format!("{}{}{}", content, author, timestamp);
    let signature: Signature = signing_key.sign(to_sign.as_bytes());

    let message = Message {
        content: content.to_string(),
        author: author.clone(),
        timestamp: timestamp.clone(),
        signature: hex::encode(signature.to_bytes()),
    };

    // Use dedicated messages repository
    let repo_path = config_dir.join("messages");

    // Initialize or open git repo
    let repo = match Repository::open(&repo_path) {
        Ok(repo) => repo,
        Err(_) => {
            println!(
                "📁 Initializing messages repository at {}",
                repo_path.display()
            );
            fs::create_dir_all(&repo_path)?;
            Repository::init(&repo_path)?
        }
    };

    // IMPORTANT: Use per-sender branches to avoid merge conflicts
    let branch_name = format!("refs/heads/users/{}", &author[..8]);
    let branch_short = format!("users/{}", &author[..8]);

    // Check if our branch exists and switch to it BEFORE file operations
    let branch_exists = repo
        .find_branch(&branch_short, git2::BranchType::Local)
        .is_ok();

    if branch_exists {
        // Branch exists, switch to it first
        repo.set_head(&branch_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }

    // NOW save message to disk (after we're on the right branch)
    let filename = format!("{}.json", timestamp.replace([':', '-', '.'], "_"));
    let file_path = repo_path.join(&filename);
    let json = serde_json::to_string_pretty(&message)?;
    fs::write(&file_path, json)?;

    // Add and commit
    let mut index = repo.index()?;

    // CRITICAL: If creating a new branch, clear the index first
    // This ensures the orphan branch only contains our new message
    if !branch_exists {
        index.clear()?;
    }

    index.add_path(Path::new(&filename))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    if branch_exists {
        // We're already on the branch, just commit

        let parent_commit = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok());
        let parents = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();

        let sig = git2::Signature::now("mmogit", "mmogit@local")?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Message: {}", &content[..content.len().min(50)]),
            &tree,
            parents.as_slice(),
        )?;
    } else {
        // First commit to this branch - create branch with the commit
        let sig = git2::Signature::now("mmogit", "mmogit@local")?;
        let commit_oid = repo.commit(
            None, // Don't update any ref yet
            &sig,
            &sig,
            &format!("Message: {}", &content[..content.len().min(50)]),
            &tree,
            &[], // No parents for first commit
        )?;

        // Now create the branch pointing to this commit
        let commit = repo.find_commit(commit_oid)?;
        repo.branch(&branch_short, &commit, false)?;

        // Set HEAD to the new branch and checkout
        repo.set_head(&branch_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }

    println!("✅ Message posted and signed");
    println!("🔑 Author: {}", &author[..8]);
    println!("📝 Content: {}", content);

    Ok(())
}

/// Post an encrypted message (sovereignty by default)
///
/// # What This Does
///
/// 1. Creates and signs a message (same as regular post)
/// 2. Encrypts the signed message with XChaCha20-Poly1305
/// 3. Posts the encrypted envelope to git
/// 4. Only those with the key can decrypt and read
///
/// # Sovereignty Note
///
/// This is the default behavior - your thoughts are encrypted first,
/// shared second. Platforms become blind storage for sovereign communication.
pub fn post_encrypted(
    content: &str,
    recipient: Option<&str>,
    config_dir: &std::path::Path,
) -> Result<()> {
    // Load identity (same as regular post)
    let seed_path = config_dir.join(".seed");
    let seed_phrase =
        fs::read_to_string(&seed_path).context("No identity found. Run 'mmogit init' first")?;

    // Derive signing key
    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();

    // Create signed message (same structure as regular post)
    let timestamp = chrono::Utc::now().to_rfc3339();
    let author = hex::encode(public_key.as_bytes());
    
    let to_sign = format!("{}{}{}", content, author, timestamp);
    let signature: Signature = signing_key.sign(to_sign.as_bytes());

    let message = Message {
        content: content.to_string(),
        author: author.clone(),
        timestamp: timestamp.clone(),
        signature: hex::encode(signature.to_bytes()),
    };

    // Serialize the signed message
    let signed_json = serde_json::to_vec(&message)?;

    // Derive encryption key (for now, encrypt for self)
    // TODO: Support recipient keys when we have key sharing
    let encryption_key = KeyDerivation::derive_encryption_key(&signing_key);
    
    // For now, we only support self-encryption
    // TODO: Look up recipient's VerifyingKey when we have key sharing
    let recipient_pubkey = if recipient.is_some() {
        // We'll need a key registry to look up other users' public keys
        // For now, just use None (self-encryption)
        None
    } else {
        None
    };

    // Encrypt the signed message
    let envelope = EncryptedEnvelope::encrypt(
        &signed_json,
        &encryption_key,
        recipient_pubkey,
    )?;

    // Serialize encrypted envelope
    let encrypted_json = serde_json::to_string_pretty(&envelope)?;

    // Post to git (using same git logic)
    let repo_path = config_dir.join("messages");
    
    let repo = match Repository::open(&repo_path) {
        Ok(repo) => repo,
        Err(_) => {
            println!(
                "📁 Initializing messages repository at {}",
                repo_path.display()
            );
            fs::create_dir_all(&repo_path)?;
            Repository::init(&repo_path)?
        }
    };

    // Create branch name for encrypted messages
    let branch_short = format!("users/{}-encrypted", &author[..16]);
    let branch_name = format!("refs/heads/{}", branch_short);

    // Save encrypted message
    let message_id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let message_path = repo_path.join(format!("encrypted_{}.json", message_id));
    fs::write(&message_path, &encrypted_json)?;

    // Get the repository signature
    let sig_config = repo.config()?;
    let name = sig_config.get_string("user.name").unwrap_or_else(|_| "mmogit".to_string());
    let email = sig_config.get_string("user.email").unwrap_or_else(|_| format!("{}@mmogit.local", &author[..8]));
    let sig = git2::Signature::now(&name, &email)?;

    // Stage and commit
    let mut index = repo.index()?;
    index.add_path(Path::new(&format!("encrypted_{}.json", message_id)))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Check if branch exists
    if let Ok(mut branch) = repo.find_branch(&branch_short, git2::BranchType::Local) {
        let parent = branch.get().peel_to_commit()?;
        
        repo.commit(
            Some(&branch_name),
            &sig,
            &sig,
            &format!("🔒 Encrypted message {}", message_id),
            &tree,
            &[&parent],
        )?;
    } else {
        let commit_oid = repo.commit(
            None,
            &sig,
            &sig,
            &format!("🔒 Encrypted message {}", message_id),
            &tree,
            &[],
        )?;

        let commit = repo.find_commit(commit_oid)?;
        repo.branch(&branch_short, &commit, false)?;
        repo.set_head(&branch_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }

    println!("🔒 Message encrypted and posted");
    println!("🔑 Author: {}", &author[..8]);
    println!("📦 Envelope ID: {}", message_id);
    println!("🎯 Recipient: {}", recipient.unwrap_or("self (only you can decrypt)"));
    
    // Show a preview of the encrypted data to confirm it's working
    if encrypted_json.len() > 100 {
        println!("🔐 Encrypted: {}...", &encrypted_json[..100]);
    }

    Ok(())
}
