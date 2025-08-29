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
use std::str::FromStr;

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

/// Initialize from ANY user-provided phrase
///
/// # True Sovereignty Mode
///
/// Want to use "password1" as your entire digital identity? 
/// Want to use your favorite song lyrics?
/// Want to use "correct horse battery staple"?
/// 
/// Your sovereignty, your choice, your consequences.
///
/// # What This Does
///
/// 1. Tries to parse as BIP39 (12-24 words)
/// 2. If that fails, uses raw string as entropy (YOLO mode)
/// 3. Warns appropriately about terrible decisions
/// 4. Derives keys deterministically either way
///
/// # Security Warning for Future Archaeologists
///
/// When you find identities based on "admin123", know that
/// we gave them sovereignty. They chose chaos.
pub fn init_with_phrase(phrase: &str, no_verify: bool, config_dir: &Path) -> Result<()> {
    use sha2::{Sha256, Digest};
    
    // Try parsing as BIP39 first
    match Mnemonic::from_str(phrase) {
        Ok(mnemonic) => {
            // Valid BIP39! Check word count for security level
            let word_count = mnemonic.words().count();
            
            match word_count {
                24 => println!("✅ Using 24-word BIP39 phrase (256-bit security)"),
                12 => {
                    println!("⚠️  Using 12-word BIP39 phrase (128-bit security)");
                    println!("    Still cryptographically secure, but consider 24 words for maximum sovereignty");
                },
                15 | 18 | 21 => {
                    println!("📝 Using {}-word BIP39 phrase", word_count);
                },
                _ => {} // Shouldn't happen but whatever
            }
            
            // Standard BIP39 flow
            derive_and_save_mnemonic(mnemonic, no_verify, config_dir)
        },
        Err(_) => {
            // Not BIP39? YOLO mode activated!
            println!("🎲 NOT a valid BIP39 phrase. Entering YOLO mode...");
            println!();
            
            // Calculate entropy for their amusement
            let entropy_bits = estimate_entropy(phrase);
            
            if phrase.len() < 8 {
                println!("⚠️  WARNING: Your phrase is {} characters", phrase.len());
                println!("    This is COMICALLY insecure!");
            }
            
            if phrase == "password" || phrase == "password1" || phrase == "admin" || phrase == "123456" {
                println!("😂 Really? '{}' as your sovereign identity?", phrase);
                println!("    Your keys will be cracked before this message finishes printing.");
            }
            
            println!();
            println!("🔥 Estimated entropy: ~{} bits", entropy_bits);
            
            if entropy_bits < 40 {
                println!("    Translation: A motivated 12-year-old could crack this");
            } else if entropy_bits < 80 {
                println!("    Translation: Vulnerable to anyone who actually tries");
            } else if entropy_bits < 128 {
                println!("    Translation: Not terrible, but not great");
            }
            
            println!();
            println!("📊 For comparison:");
            println!("    - 12-word BIP39: 128 bits (cryptographically secure)");
            println!("    - 24-word BIP39: 256 bits (unbreakable with current physics)");
            println!("    - Your phrase: ~{} bits", entropy_bits);
            
            if !no_verify {
                println!();
                if !Confirm::new()
                    .with_prompt("Do you REALLY want to use this as your sovereign identity?")
                    .interact()?
                {
                    println!("❌ Good choice. Run 'mmogit init' for a secure identity.");
                    return Ok(());
                }
                
                println!();
                println!("🤷 Your sovereignty includes the right to terrible decisions!");
            }
            
            // Hash the phrase to get deterministic entropy
            // At least make it somewhat harder than raw phrase storage
            let mut hasher = Sha256::new();
            hasher.update(phrase.as_bytes());
            hasher.update(b"mmogit-sovereignty-salt"); // Domain separation
            let hash = hasher.finalize();
            
            // Derive signing key from hash
            let seed_bytes: [u8; 32] = hash.into();
            let signing_key = SigningKey::from_bytes(&seed_bytes);
            let public_key = signing_key.verifying_key();
            
            // Save the raw phrase (they chose this path)
            fs::create_dir_all(config_dir)?;
            let seed_path = config_dir.join(".seed");
            
            // Add warning to saved file
            let stored_content = format!(
                "# WARNING: Non-BIP39 phrase used!\n\
                 # Entropy: ~{} bits\n\
                 # Generated: {}\n\
                 {}", 
                entropy_bits,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                phrase
            );
            
            fs::write(&seed_path, stored_content)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&seed_path, fs::Permissions::from_mode(0o600))?;
            }
            
            println!();
            println!("✨ Identity created (somehow)");
            println!("📍 Saved to: {}", config_dir.display());
            println!("🔑 Public key: {}", hex::encode(public_key.as_bytes()));
            println!();
            println!("💀 Remember: You chose this.");
            
            Ok(())
        }
    }
}

/// Helper to handle standard BIP39 flow
fn derive_and_save_mnemonic(mnemonic: Mnemonic, no_verify: bool, config_dir: &Path) -> Result<()> {
    let words: Vec<_> = mnemonic.words().collect();
    
    // Display the phrase
    println!("\nYOUR SEED PHRASE:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for (i, chunk) in words.chunks(4).enumerate() {
        print!("  ");
        for (j, word) in chunk.iter().enumerate() {
            print!("{:2}. {:12} ", i * 4 + j + 1, word);
        }
        println!();
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if !no_verify && words.len() > 12 {
        // Only verify for newly shown phrases > 12 words
        println!("\n⚠️  This is your ONLY way to recover your identity!\n");
        
        if !Confirm::new()
            .with_prompt(&format!("Have you written down all {} words?", words.len()))
            .interact()?
        {
            println!("❌ Aborting. Run 'mmogit init' again when ready.");
            return Ok(());
        }
        
        clearscreen::clear()?;
        
        // Verify random words
        let mut positions: Vec<usize> = (0..words.len()).collect();
        positions.shuffle(&mut rand::rng());
        
        println!("Let's verify you wrote them down correctly.\n");
        
        for &pos in positions.iter().take(3) {
            let prompt = format!("Enter word #{}: ", pos + 1);
            let input: String = Input::new().with_prompt(&prompt).interact_text()?;
            
            if input.trim() != words[pos] {
                println!("❌ Incorrect! Please write down your seed phrase and try again.");
                return Ok(());
            }
            println!("✅ Correct!");
        }
    }
    
    // Derive keys
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();
    
    // Save
    fs::create_dir_all(config_dir)?;
    let seed_path = config_dir.join(".seed");
    fs::write(&seed_path, mnemonic.to_string())?;
    
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

/// Estimate entropy bits (very rough)
fn estimate_entropy(phrase: &str) -> usize {
    use std::collections::HashSet;
    
    let unique_chars: HashSet<_> = phrase.chars().collect();
    let charset_size = unique_chars.len();
    let length = phrase.len();
    
    // Very rough: log2(charset^length)
    // This assumes random selection, which is generous
    ((charset_size as f64).ln() * length as f64 / 2.0_f64.ln()) as usize
}
