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
pub fn post(content: &str) -> Result<()> {
    // Load the seed (duplicated from init - that's OK for now)
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let seed_path = home.join(".mmogit").join(".seed");

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
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let repo_path = home.join(".mmogit").join("messages");

    // Initialize or open git repo
    let repo = match Repository::open(&repo_path) {
        Ok(repo) => repo,
        Err(_) => {
            println!("📁 Initializing messages repository at ~/.mmogit/messages/");
            fs::create_dir_all(&repo_path)?;
            Repository::init(&repo_path)?
        }
    };

    // Save message directly in repo root with timestamp as filename
    let filename = format!("{}.json", timestamp.replace([':', '-', '.'], "_"));
    let file_path = repo_path.join(&filename);
    let json = serde_json::to_string_pretty(&message)?;
    fs::write(&file_path, json)?;

    // IMPORTANT: Use per-sender branches to avoid merge conflicts
    // Each sender writes only to refs/heads/users/<pubkey_fingerprint>
    // This is the key to avoiding merge storms under concurrent writes
    let branch_name = format!("refs/heads/users/{}", &author[..8]);
    let branch_short = format!("users/{}", &author[..8]);

    // Add and commit to our branch
    let mut index = repo.index()?;
    index.add_path(Path::new(&filename))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let branch_exists = repo
        .find_branch(&branch_short, git2::BranchType::Local)
        .is_ok();

    if branch_exists {
        // Branch exists, switch to it and commit normally
        repo.set_head(&branch_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

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

        // Set HEAD to the new branch
        repo.set_head(&branch_name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }

    println!("✅ Message posted and signed");
    println!("🔑 Author: {}", &author[..8]);
    println!("📝 Content: {}", content);

    Ok(())
}
