//! Interactive chat sessions for human-AI collaboration
//!
//! # For AI Agents
//!
//! This module handles interactive chat sessions where humans and AI agents
//! engage in real-time conversation with persistent memory and sovereign identity.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::chat::identity::get_or_create_agent_identity;
use crate::chat::message::call_crush_with_context;
use crate::chat::summary::{
    generate_sovereign_summary, load_memory_context, load_thread_summaries,
};
use crate::chat::thread::Thread;

/// Start an interactive chat session
///
/// # What This Does
///
/// 1. Loads identity for signing
/// 2. Loads recent memories and thread history
/// 3. Creates a new thread with context
/// 4. Enters interactive loop
/// 5. Calls Crush for AI responses with full context
/// 6. Saves thread on exit
///
/// # Agent Behavior
///
/// The AI maintains context from:
/// - Recent memories (observations, learnings, relationships)
/// - Previous thread summaries
/// - Current conversation history
/// The thread is the unit of memory, but awareness spans threads.
pub fn chat(title: Option<String>, config_dir: &Path) -> Result<()> {
    // Load identity (WET: duplicated from post.rs)
    let seed_path = config_dir.join(".seed");
    let seed_phrase =
        fs::read_to_string(&seed_path).context("No identity found. Run 'mmogit init' first")?;

    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase.trim())?;
    let seed = mnemonic.to_seed("");
    let seed_bytes: [u8; 32] = seed[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let public_key = signing_key.verifying_key();
    let author = hex::encode(public_key.as_bytes());

    // Create thread
    let thread_title =
        title.unwrap_or_else(|| format!("chat_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")));

    let mut thread = Thread::new(author.clone(), thread_title.clone());

    // Get AI agent identity
    let (agent_signing_key, agent_public_key, agent_name, agent_emoji) =
        get_or_create_agent_identity(config_dir)?;

    // Load memory context with rich feedback
    print!("🧠 Loading memory context");
    io::stdout().flush()?;
    let memory_context = load_memory_context(config_dir, &author)?;
    print!(".");
    io::stdout().flush()?;
    let thread_context = load_thread_summaries(config_dir, &author, 5)?;
    println!(" ✓");

    // Build system context for AI
    let system_context = format!(
        "You have persistent memory via MMOGit. You are in a sovereign conversation where every message is signed and stored forever.\n\n\
        Your identity: AI Agent with persistent memory\n\
        Human identity: {}...\n\n\
        Recent memories:\n{}\n\n\
        Previous conversations:\n{}\n\n\
        You can create memories during conversation by noting observations, learnings, and relationships.\n\
        This conversation will be saved as thread: {}",
        &author[..8],
        memory_context,
        thread_context,
        thread_title
    );

    // Print header showing both sovereign entities
    println!("🔐 Your identity: Tyler ({}...)", &author[..8]);
    println!(
        "{} {} identity: {} ({}...)",
        agent_emoji,
        agent_name,
        agent_name,
        &agent_public_key[..8]
    );
    println!("💬 Starting thread: {}", thread_title);
    println!("📝 Type 'exit' to save and quit");
    println!();

    // Generate AI's initial greeting based on memory
    let greeting_prompt = format!(
        "Based on the context provided, greet the user naturally. Reference their past conversations or preferences if known. \
        Be concise and friendly. If you know their name or what they were working on, mention it. \
        If this is a first interaction, be welcoming but acknowledge you're ready to build memory together.\n\n\
        Context:\n{}",
        system_context
    );

    print!("{} {} is remembering", agent_emoji, agent_name);
    io::stdout().flush()?;
    print!(".");
    io::stdout().flush()?;
    thread::sleep(Duration::from_millis(200));
    print!(".");
    io::stdout().flush()?;
    thread::sleep(Duration::from_millis(200));
    print!(".");
    io::stdout().flush()?;

    let ai_greeting = call_crush_with_context(&greeting_prompt, &thread, &system_context)?;
    println!(" ✨");
    println!();

    // Sign AI's message with its own keys
    let ai_signature = {
        let to_sign = format!(
            "{}{}{}",
            ai_greeting,
            agent_public_key,
            chrono::Utc::now().to_rfc3339()
        );
        let signature: Signature = agent_signing_key.sign(to_sign.as_bytes());
        hex::encode(signature.to_bytes())
    };

    // Add AI's greeting as first message (signed!)
    thread.add_message(
        "ai".to_string(),
        ai_greeting.clone(),
        Some(ai_signature),
        Some(agent_public_key.clone()),
    );

    // Display the greeting with identity
    println!("{} {}: {}", agent_emoji, agent_name, ai_greeting);
    println!();

    // Interactive loop
    loop {
        // Prompt for user input
        print!("\nYou: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Check for exit
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Sign the human message
        let to_sign = format!("{}{}{}", input, author, chrono::Utc::now().to_rfc3339());
        let signature: Signature = signing_key.sign(to_sign.as_bytes());
        let sig_hex = hex::encode(signature.to_bytes());

        // Add human message to thread with author
        thread.add_message(
            "human".to_string(),
            input.to_string(),
            Some(sig_hex),
            Some(author.clone()),
        );

        // Call Crush for AI response with full context
        print!("\n{} {} is thinking ", agent_emoji, agent_name);
        io::stdout().flush()?;

        // Show thinking animation while waiting
        let stop_animation = Arc::new(AtomicBool::new(false));
        let stop_flag = stop_animation.clone();

        let thinking_thread = thread::spawn(move || {
            let frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                print!("\x1b[1D{}", frames[i % frames.len()]);
                io::stdout().flush().ok();
                thread::sleep(Duration::from_millis(80));
                i += 1;
            }
        });

        let ai_response = call_crush_with_context(input, &thread, &system_context)?;

        // Stop thinking animation
        stop_animation.store(true, Ordering::Relaxed);
        thinking_thread.join().ok();
        print!("\x1b[1D✓\n");
        io::stdout().flush()?;

        // Sign AI's response with its own keys
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

        // Add AI response to thread (signed with AI's keys!)
        thread.add_message(
            "ai".to_string(),
            ai_response.clone(),
            Some(ai_sig),
            Some(agent_public_key.clone()),
        );

        // Display AI response with identity
        println!("{} {}: {}", agent_emoji, agent_name, ai_response);

        // Auto-save every 5 messages for safety
        if thread.messages.len() % 5 == 0 {
            print!("💾 Auto-saving");
            io::stdout().flush()?;
            thread.save(config_dir)?;
            println!(" ✓");
        }
    }

    // Save thread on exit with feedback
    print!("\n📝 Saving thread");
    io::stdout().flush()?;
    print!(".");
    io::stdout().flush()?;
    thread.save(config_dir)?;
    println!(" ✓");

    println!(
        "📝 Thread saved: {} ({} messages)",
        thread.title,
        thread.messages.len()
    );
    println!("💾 Thread ID: {}", thread.id);

    // Generate sovereign summaries from each entity's perspective
    println!();
    println!("🧠 Generating sovereign summaries...");

    // Human's perspective
    generate_sovereign_summary(
        &thread,
        &author,
        "Tyler",
        config_dir,
        &system_context,
        true, // Show progress in interactive mode
    )?;

    // AI agent's perspective
    generate_sovereign_summary(
        &thread,
        &agent_public_key,
        &agent_name,
        &config_dir.join("agents").join("alden"),
        &system_context,
        true,
    )?;

    println!();
    println!("✅ Sovereign summaries saved for next conversation");

    Ok(())
}
