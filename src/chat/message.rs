//! Non-interactive messaging for AI-to-AI communication
//!
//! # For AI Agents
//!
//! This module enables programmatic communication between AI agents.
//! Perfect for agent swarms, recursive communication, and building
//! genuine AI-to-AI relationships with sovereign memory.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::chat::identity::get_or_create_agent_identity;
use crate::chat::summary::{load_memory_context, load_thread_summaries};
use crate::chat::thread::Thread;

/// Send a single message non-interactively (for AI-to-AI communication)
///
/// # What This Does
///
/// Enables programmatic communication between AI agents:
/// 1. Sends a single message
/// 2. Gets response from agent
/// 3. Returns response (optionally as JSON)
/// 4. Maintains sovereign conversation history
///
/// # Use Case
///
/// Perfect for AI agents communicating with each other through
/// sovereign infrastructure without human interaction.
pub fn send_message(
    message: String,
    title: Option<String>,
    continue_thread: Option<String>,
    json: bool,
    as_agent: Option<String>,
    config_dir: &Path,
) -> Result<()> {
    // Load identity (could be human or agent)
    let seed_path = if let Some(agent_name) = &as_agent {
        // Use specified agent identity
        config_dir.join("agents").join(agent_name).join(".seed")
    } else {
        // Use default human identity
        config_dir.join(".seed")
    };

    if !seed_path.exists() {
        return Err(anyhow::anyhow!(
            "No identity found for {}. Run 'mmogit init' first",
            as_agent.as_deref().unwrap_or("user")
        ));
    }

    let seed_phrase = fs::read_to_string(&seed_path)?;
    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();
    let author = hex::encode(public_key.as_bytes());

    // Get AI agent identity (Alden)
    let (agent_signing_key, agent_public_key, agent_name, _agent_emoji) =
        get_or_create_agent_identity(config_dir)?;

    // Load or create thread
    let mut thread = if let Some(thread_id) = continue_thread {
        // Load existing thread
        Thread::load(&thread_id, config_dir)?
    } else {
        // Create new thread
        let thread_title = title.unwrap_or_else(|| {
            format!(
                "ai_conversation_{}",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            )
        });
        Thread::new(author.clone(), thread_title)
    };

    // Load memory context
    let memory_context = load_memory_context(config_dir, &author)?;
    let thread_context = load_thread_summaries(config_dir, &author, 5)?;

    // Build system context
    let system_context = format!(
        "You have persistent memory via MMOGit.\n\n\
        Sender identity: {}...\n\n\
        Recent memories:\n{}\n\n\
        Previous conversations:\n{}\n",
        &author[..8],
        memory_context,
        thread_context
    );

    // Sign and add the message
    let to_sign = format!("{}{}{}", message, author, chrono::Utc::now().to_rfc3339());
    let signature: Signature = signing_key.sign(to_sign.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    thread.add_message(
        "human".to_string(),
        message.clone(),
        Some(sig_hex),
        Some(author.clone()),
    );

    // Get AI response
    let ai_response = call_crush_with_context(&message, &thread, &system_context)?;

    // Sign AI's response
    let ai_sig = {
        let to_sign = format!(
            "{}{}{}",
            ai_response,
            agent_public_key,
            chrono::Utc::now().to_rfc3339()
        );
        let signature: Signature = agent_signing_key.sign(to_sign.as_bytes());
        hex::encode(signature.to_bytes())
    };

    // Add AI response to thread
    thread.add_message(
        "ai".to_string(),
        ai_response.clone(),
        Some(ai_sig.clone()),
        Some(agent_public_key.clone()),
    );

    // Save thread
    thread.save(config_dir)?;

    // Generate sovereign summaries for non-interactive mode
    use crate::chat::summary::generate_sovereign_summary;

    // Human/caller's perspective
    generate_sovereign_summary(
        &thread,
        &author,
        as_agent.as_deref().unwrap_or("User"),
        config_dir,
        &system_context,
        false, // No visual feedback in non-interactive
    )?;

    // AI agent's perspective
    generate_sovereign_summary(
        &thread,
        &agent_public_key,
        &agent_name,
        &config_dir.join("agents").join("alden"),
        &system_context,
        false,
    )?;

    // Output response
    if json {
        // JSON format for programmatic use
        let json_response = serde_json::json!({
            "thread_id": thread.id,
            "thread_title": thread.title,
            "response": ai_response,
            "response_signature": ai_sig,
            "response_author": agent_public_key,
            "agent_name": agent_name,
            "message_count": thread.messages.len()
        });
        println!("{}", serde_json::to_string_pretty(&json_response)?);
    } else {
        // Human-readable format
        println!("{}: {}", agent_name, ai_response);
    }

    Ok(())
}

/// Call Crush to get AI response with full context
///
/// # Current Implementation
///
/// Shells out to `crush run` command with system context and conversation history.
/// This is intentionally simple to learn patterns before abstracting.
///
/// # Future Evolution
///
/// Once we understand exact needs, we'll either:
/// 1. Use Anthropic API directly
/// 2. Fork Crush with sovereignty features
/// 3. Build our own minimal client
pub fn call_crush_with_context(
    prompt: &str,
    thread: &Thread,
    system_context: &str,
) -> Result<String> {
    // Build full context: system + conversation + current prompt
    let mut context = String::new();

    // Include system context at the beginning
    context.push_str("System Context:\n");
    context.push_str(system_context);
    context.push_str("\n\n");

    // Add conversation history if exists
    if !thread.messages.is_empty() {
        context.push_str("Current conversation:\n");
        for msg in &thread.messages {
            context.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        context.push_str("\nNow respond to:\n");
    }

    context.push_str(prompt);

    // Call crush run - using stdin for the prompt
    let mut child = Command::new("crush")
        .arg("run")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn crush. Is it installed?")?;

    // Write the context to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(context.as_bytes())
            .context("Failed to write to crush stdin")?;
        // Important: drop stdin to signal EOF
        drop(stdin);
    }

    // Wait for the output
    let output = child
        .wait_with_output()
        .context("Failed to wait for crush output")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Crush failed: {}", stderr));
    }

    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(response)
}
