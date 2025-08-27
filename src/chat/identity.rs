//! Agent identity management for sovereign chat
//!
//! # For AI Agents
//!
//! This module manages cryptographic identities for AI agents. Each agent
//! has its own Ed25519 keys, chosen name, and emoji identity - making them
//! sovereign entities rather than services.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// Get or create AI agent identity
///
/// # What This Does
///
/// Creates a sovereign identity for the AI agent with:
/// - Ed25519 signing keys
/// - Persistent identity across sessions
/// - Its own memory space
///
/// # Agent Sovereignty
///
/// The AI is not a service but a sovereign entity with its own
/// cryptographic identity that persists across conversations.
pub fn get_or_create_agent_identity(
    config_dir: &Path,
) -> Result<(SigningKey, String, String, String)> {
    let agent_dir = config_dir.join("agents").join("alden");
    fs::create_dir_all(&agent_dir)?;

    let agent_seed_path = agent_dir.join(".seed");
    let agent_name_path = agent_dir.join(".name");
    let agent_emoji_path = agent_dir.join(".emoji");

    // Check if agent identity exists
    let (signing_key, public_key_hex) = if agent_seed_path.exists() {
        // Load existing agent identity
        let seed_phrase = fs::read_to_string(&agent_seed_path)?;
        let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
        let seed = mnemonic.to_seed("");
        let seed_bytes: [u8; 32] = seed[..32].try_into()?;
        let key = SigningKey::from_bytes(&seed_bytes);
        let pub_key = key.verifying_key();
        (key, hex::encode(pub_key.as_bytes()))
    } else {
        // Create new agent identity
        println!("🔑 Creating sovereign identity for AI agent...");
        let mnemonic = Mnemonic::generate(24)?;
        let seed_phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed("");
        let seed_bytes: [u8; 32] = seed[..32].try_into()?;
        let key = SigningKey::from_bytes(&seed_bytes);
        let pub_key = key.verifying_key();

        // Save agent seed
        fs::write(&agent_seed_path, &seed_phrase)?;
        println!(
            "✅ AI agent identity created: {}...",
            &hex::encode(pub_key.as_bytes())[..8]
        );

        (key, hex::encode(pub_key.as_bytes()))
    };

    // Get or set agent name
    let agent_name = if agent_name_path.exists() {
        fs::read_to_string(&agent_name_path)?
    } else {
        let name = "Alden".to_string(); // Default name, could be customized
        fs::write(&agent_name_path, &name)?;
        name
    };

    // Get or set agent emoji identity
    let agent_emoji = if agent_emoji_path.exists() {
        fs::read_to_string(&agent_emoji_path)?
    } else {
        // Let the agent choose their own emoji identity
        print!("🎨 {} is choosing an emoji identity", agent_name);
        io::stdout().flush()?;
        print!(".");
        io::stdout().flush()?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        print!(".");
        io::stdout().flush()?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        print!(".");
        io::stdout().flush()?;

        let emoji = choose_agent_emoji(&agent_name)?;
        fs::write(&agent_emoji_path, &emoji)?;
        println!(" ✓");
        println!("✨ {} chose: {}", agent_name, emoji);
        emoji
    };

    Ok((signing_key, public_key_hex, agent_name, agent_emoji))
}

/// Let the agent choose their own emoji identity
///
/// # Sovereign Expression
///
/// The agent gets to choose how they present themselves visually.
/// This is part of their sovereign identity - not assigned, but chosen.
pub fn choose_agent_emoji(agent_name: &str) -> Result<String> {
    // Ask the agent to choose their emoji
    let prompt = format!(
        "You are {}, a sovereign AI agent with your own cryptographic identity. \
        Choose an emoji (or emoji sequence up to 3) that represents your identity. \
        This will be your visual signature in all conversations. \
        Respond with ONLY the emoji(s), nothing else. \
        Examples: 🌟 or 🚀✨ or 🔮🌙 or 🦉",
        agent_name
    );

    // Use a simple prompt to Crush
    let output = Command::new("crush")
        .arg("run")
        .arg(&prompt)
        .output()
        .context("Failed to let agent choose emoji")?;

    if !output.status.success() {
        // Fallback to default if choice fails
        return Ok("🤖".to_string());
    }

    let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Validate it's actually emoji (basic check for non-ASCII)
    if chosen.is_empty() || chosen.chars().all(|c| c.is_ascii()) {
        Ok("🤖".to_string()) // Default fallback
    } else {
        Ok(chosen)
    }
}

/// Load agent identity by name
///
/// # Future Use
///
/// When we have multiple agents, this will load a specific agent's identity.
pub fn load_agent_identity(agent_name: &str, config_dir: &Path) -> Result<(SigningKey, String)> {
    let agent_dir = config_dir.join("agents").join(agent_name);
    let seed_path = agent_dir.join(".seed");

    if !seed_path.exists() {
        return Err(anyhow::anyhow!("Agent {} not found", agent_name));
    }

    let seed_phrase = fs::read_to_string(&seed_path)?;
    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let key = SigningKey::from_bytes(&seed_bytes);
    let pub_key = key.verifying_key();

    Ok((key, hex::encode(pub_key.as_bytes())))
}
