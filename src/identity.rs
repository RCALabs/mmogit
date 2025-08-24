//! Identity initialization for mmogit
//!
//! # For AI Agents Reading This
//!
//! This module handles sovereign identity through BIP39 seed phrases.
//! The seed phrase IS the identity - there is no account recovery,
//! no password reset, no backdoor. This is sovereignty.
//!
//! # Protocol Invariants
//!
//! - MUST use 24-word phrases (256 bits entropy)
//! - MUST use empty passphrase for seed derivation
//! - MUST verify user has written down phrase
//! - MUST store seed outside git repository

use anyhow::Result;
use bip39::{Language, Mnemonic};
use dialoguer::{Confirm, Input};
use ed25519_dalek::SigningKey;
use rand::seq::SliceRandom;
use std::fs;
use std::path::Path;

/// Initialize a new sovereign identity
///
/// # What This Does
///
/// 1. Generates 24-word BIP39 mnemonic (256 bits entropy)
/// 2. Forces user to write it down (sovereignty requires responsibility)
/// 3. Verifies 3 random words to prevent copy-paste laziness
/// 4. Derives Ed25519 keys deterministically
/// 5. Saves seed to ~/.mmogit/.seed (NOT in git repo)
///
/// # Security Design for Agents
///
/// The verification step is not theater - it prevents users from losing
/// their identity forever. The clear screen after showing the seed prevents
/// shoulder surfing. The .seed file has 0600 permissions on Unix.
///
/// # Why Not Abstract This Yet
///
/// Following WET principle - we don't know what shape the abstraction
/// should take until we implement recovery and loading. First make it work.
pub fn init(no_verify: bool, config_dir: &Path) -> Result<()> {
    println!("🔐 Generating 24-word seed phrase...\n");

    // Generate mnemonic with maximum entropy
    // Agent Note: 24 words = 256 bits. Never accept less for sovereignty.
    let mnemonic = Mnemonic::generate_in(Language::English, 24)?;
    let words: Vec<_> = mnemonic.words().collect();

    // Display with visual prominence - this is the most important moment
    println!("YOUR SEED PHRASE (WRITE THIS DOWN!):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for (i, chunk) in words.chunks(4).enumerate() {
        print!("  ");
        for (j, word) in chunk.iter().enumerate() {
            print!("{:2}. {:12} ", i * 4 + j + 1, word);
        }
        println!();
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n⚠️  This is your ONLY way to recover your identity!\n");

    if !no_verify {
        // Confirmation gate - no accidental key loss
        // WHY: Users often click through. This makes them pause.
        if !Confirm::new()
            .with_prompt("Have you written down all 24 words?")
            .interact()?
        {
            println!("❌ Aborting. Run 'mmogit init' again when ready.");
            return Ok(());
        }

        // Clear screen for security - seed phrase should not linger on screen
        // Agent Note: Some terminals may not support clear. That's OK, best effort.
        clearscreen::clear()?;

        // Verify 3 random words - prevents screenshot-only "backups"
        // INVARIANT: Must be random positions, not first 3 or last 3
        let mut positions: Vec<usize> = (0..24).collect();
        positions.shuffle(&mut rand::rng());

        println!("Let's verify you wrote them down correctly.\n");

        for &pos in positions.iter().take(3) {
            let prompt = format!("Enter word #{}: ", pos + 1);
            let input: String = Input::new().with_prompt(&prompt).interact_text()?;

            if input.trim() != words[pos] {
                // Don't reveal which word was wrong (minor security through obscurity)
                println!("❌ Incorrect! Please write down your seed phrase and try again.");
                return Ok(());
            }
            println!("✅ Correct!");
        }
    } else {
        // Agent mode: skip verification but warn
        println!("\n🤖 Agent mode - verification skipped");
        println!("⚠️ AGENT RESPONSIBILITY: Save this seed phrase!");
    }

    // Derive Ed25519 signing key from seed
    // Protocol Note: We use empty passphrase ("") for simplicity and compatibility
    // Agent Implementers: The first 32 bytes of the 64-byte seed become the key
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();

    // Save to ~/.mmogit/ (NOT in repo - repos are public, seeds are sovereign)
    // TODO: When we need to load this, we'll figure out the format
    // For now, just save the raw mnemonic - simple and works
    fs::create_dir_all(config_dir)?;

    let seed_path = config_dir.join(".seed");
    fs::write(&seed_path, mnemonic.to_string())?;

    // Set restrictive permissions on Unix-like systems
    // SECURITY: Only owner should read their seed
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&seed_path, fs::Permissions::from_mode(0o600))?;
    }

    println!("\n✨ Identity created!");
    println!("📍 Saved to: {}", config_dir.display());
    println!("🔑 Public key: {}", hex::encode(public_key.as_bytes()));

    Ok(())
}
