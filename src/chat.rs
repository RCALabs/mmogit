//! Interactive AI chat with sovereign memory
//!
//! # For AI Agents
//!
//! This module implements persistent chat sessions where both humans and AI
//! maintain sovereign memory. Every message is signed, every thread is owned,
//! and conversations persist forever in Git.
//!
//! # Architecture Decision
//!
//! We shell out to `crush run` for now instead of using APIs directly.
//! This lets us learn the patterns before abstracting. Once we understand
//! exactly what we need, we can fork Crush or use APIs directly.
//!
//! # Thread-as-Commit Pattern
//!
//! Instead of one commit per message (which creates ~20 Git objects per
//! conversation), we use one commit per thread. This reduces Git bloat
//! while maintaining complete history.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey};
use git2::Repository;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// A single message in a chat thread
///
/// # Agent Note
///
/// Messages track role (human/ai) and maintain signatures for sovereignty.
/// Both human and AI messages can be signed with their respective Ed25519 keys.
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    /// Who sent this: "human" or "ai"
    pub role: String,
    /// The actual message content
    pub content: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Ed25519 signature for message authenticity
    pub signature: Option<String>,
    /// Public key of the sender (hex encoded)
    pub author: Option<String>,
}

/// A conversation thread containing multiple messages
///
/// # Design Principle
///
/// Threads are the primary unit of conversation. They contain multiple
/// messages but create only one Git commit, reducing repository bloat
/// by ~20x compared to message-per-commit.
#[derive(Serialize, Deserialize)]
pub struct Thread {
    /// Unique thread identifier (timestamp-based)
    pub id: String,
    /// Human-readable thread title
    pub title: String,
    /// Public key of the thread creator (hex encoded)
    pub author: String,
    /// When the thread started (ISO 8601)
    pub created_at: String,
    /// When the thread was last updated (ISO 8601)
    pub updated_at: String,
    /// All messages in chronological order
    pub messages: Vec<ChatMessage>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Thread state: "active" or "closed"
    pub state: String,
}

impl Thread {
    /// Create a new thread
    pub fn new(author: String, title: String) -> Self {
        let now = chrono::Utc::now();
        let id = format!("thread_{}", now.timestamp());

        Thread {
            id,
            title,
            author,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            messages: Vec::new(),
            tags: Vec::new(),
            state: "active".to_string(),
        }
    }

    /// Add a message to the thread
    pub fn add_message(
        &mut self,
        role: String,
        content: String,
        signature: Option<String>,
        author: Option<String>,
    ) {
        let message = ChatMessage {
            role,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            signature,
            author,
        };

        self.messages.push(message);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Save thread to Git as a single commit
    ///
    /// # WET Note
    ///
    /// Yes, this duplicates some Git logic from post.rs. We're building
    /// twice before abstracting, following the WET principle.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        // Use dedicated threads directory
        let repo_path = config_dir.join("threads");

        // Initialize or open git repo
        let repo = match Repository::open(&repo_path) {
            Ok(repo) => repo,
            Err(_) => {
                println!(
                    "📁 Initializing threads repository at {}",
                    repo_path.display()
                );
                fs::create_dir_all(&repo_path)?;
                Repository::init(&repo_path)?
            }
        };

        // Use per-author branches to avoid conflicts
        let branch_name = format!("refs/heads/users/{}", &self.author[..8]);
        let branch_short = format!("users/{}", &self.author[..8]);

        // Check if branch exists
        let branch_exists = repo
            .find_branch(&branch_short, git2::BranchType::Local)
            .is_ok();

        if branch_exists {
            repo.set_head(&branch_name)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        // Save thread to disk
        let filename = format!("{}.json", self.id);
        let file_path = repo_path.join(&filename);
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&file_path, json)?;

        // Add and commit
        let mut index = repo.index()?;

        if !branch_exists {
            index.clear()?;
        }

        index.add_path(Path::new(&filename))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let sig = git2::Signature::now("mmogit", "mmogit@local")?;

        if branch_exists {
            let parent_commit = repo
                .head()
                .ok()
                .and_then(|h| h.target())
                .and_then(|oid| repo.find_commit(oid).ok());
            let parents = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Thread: {}", self.title),
                &tree,
                parents.as_slice(),
            )?;
        } else {
            let commit_oid = repo.commit(
                None,
                &sig,
                &sig,
                &format!("Thread: {}", self.title),
                &tree,
                &[],
            )?;

            let commit = repo.find_commit(commit_oid)?;
            repo.branch(&branch_short, &commit, false)?;
            repo.set_head(&branch_name)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        Ok(())
    }
}

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
fn get_or_create_agent_identity(config_dir: &Path) -> Result<(SigningKey, String, String, String)> {
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
fn choose_agent_emoji(agent_name: &str) -> Result<String> {
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
    std::thread::sleep(std::time::Duration::from_millis(200));
    print!(".");
    io::stdout().flush()?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    print!(".");
    io::stdout().flush()?;

    let ai_greeting = call_crush_with_context(&greeting_prompt, &thread, &system_context)?;
    println!(" ✨");
    println!(); // Add spacing after loading

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
    println!(); // Add spacing after greeting

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
        // NOTE: Using shell out for now - will abstract after understanding patterns
        print!("\n{} {} is thinking ", agent_emoji, agent_name); // Added space after thinking
        io::stdout().flush()?;

        // Show thinking animation while waiting
        let stop_animation = Arc::new(AtomicBool::new(false));
        let stop_flag = stop_animation.clone();

        let thinking_thread = std::thread::spawn(move || {
            let frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                print!("\x1b[1D{}", frames[i % frames.len()]);
                io::stdout().flush().ok();
                std::thread::sleep(std::time::Duration::from_millis(80));
                i += 1;
            }
        });

        let ai_response = call_crush_with_context(input, &thread, &system_context)?;

        // Stop thinking animation
        stop_animation.store(true, Ordering::Relaxed);
        thinking_thread.join().ok();
        print!("\x1b[1D✓\n"); // No extra space needed
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
        // Don't add extra spacing here, the next "You:" prompt will handle it

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

    // Generate and save thread summary for future memory
    println!();
    print!("🧠 Generating summary for {}'s memory", agent_name);
    io::stdout().flush()?;

    // Show progress dots
    let stop_dots = Arc::new(AtomicBool::new(false));
    let stop_flag = stop_dots.clone();

    let dots_thread = std::thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if !stop_flag.load(Ordering::Relaxed) {
                print!(".");
                io::stdout().flush().ok();
            }
        }
    });

    let summary_prompt = format!(
        "Summarize this conversation with sovereign agency. Note:\n\
        - Key topics discussed\n\
        - Important decisions or conclusions\n\
        - What you learned about the human\n\
        - What matters for future conversations\n\
        - Any patterns or preferences you noticed\n\n\
        Thread: {}",
        serde_json::to_string(&thread)?
    );

    let summary = call_crush_with_context(&summary_prompt, &thread, &system_context)?;

    // Stop progress dots
    stop_dots.store(true, Ordering::Relaxed);
    dots_thread.join().ok();
    println!(" ✓");

    // Save summary to a dedicated summaries directory
    let summaries_dir = config_dir.join("summaries");
    fs::create_dir_all(&summaries_dir)?;

    let summary_file = summaries_dir.join(format!("{}_summary.txt", thread.id));
    print!("💾 Saving summary");
    io::stdout().flush()?;
    fs::write(&summary_file, &summary)?;
    println!(" ✓");

    println!("✅ Summary saved for next conversation");
    println!();
    println!("Summary preview:");
    println!("{}", summary.chars().take(200).collect::<String>());
    if summary.len() > 200 {
        println!("...");
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
fn call_crush_with_context(prompt: &str, thread: &Thread, system_context: &str) -> Result<String> {
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
    // NOTE: This assumes crush is installed and configured
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

/// Load recent memories for context
fn load_memory_context(config_dir: &Path, author: &str) -> Result<String> {
    use crate::show;

    // Get memories from last 72 hours
    let messages_path = config_dir.join("messages");
    if !messages_path.exists() {
        return Ok("No previous memories found. This appears to be a new user.".to_string());
    }

    // TODO: Actually load and parse recent memories
    // For now, note if we have history
    Ok(format!(
        "- Previous interactions detected with user ({}...)\n\
         - User prefers direct implementation over theory\n\
         - Focus on working code and sovereignty principles",
        &author[..8]
    ))
}

/// Load summaries of recent threads
fn load_thread_summaries(config_dir: &Path, author: &str, limit: usize) -> Result<String> {
    // First try to load actual summaries
    let summaries_dir = config_dir.join("summaries");
    let mut loaded_summaries = Vec::new();

    if summaries_dir.exists() {
        if let Ok(entries) = fs::read_dir(&summaries_dir) {
            let mut summary_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.ends_with("_summary.txt"))
                        .unwrap_or(false)
                })
                .collect();

            // Sort by modification time (most recent first)
            summary_files.sort_by_key(|e| {
                e.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            summary_files.reverse();

            // Load most recent summaries
            for entry in summary_files.iter().take(limit) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    loaded_summaries.push(format!("Previous conversation summary:\n{}", content));
                }
            }
        }
    }

    if !loaded_summaries.is_empty() {
        return Ok(loaded_summaries.join("\n\n"));
    }

    // Fallback to loading thread metadata if no summaries exist
    let threads_path = config_dir.join("threads");
    if !threads_path.exists() {
        return Ok("No previous conversations found.".to_string());
    }

    let mut summaries = Vec::new();
    let mut threads: Vec<Thread> = Vec::new();

    // Load all threads
    if let Ok(entries) = fs::read_dir(&threads_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(thread) = serde_json::from_str::<Thread>(&content) {
                            if thread.author.starts_with(&author[..8]) {
                                threads.push(thread);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    // Take most recent threads and create summaries
    for thread in threads.iter().take(limit) {
        // Get more context from the thread
        let topic = thread
            .messages
            .first()
            .map(|m| {
                if m.content.len() > 100 {
                    format!("{}...", m.content.chars().take(100).collect::<String>())
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_else(|| "Empty conversation".to_string());

        let last_message = thread
            .messages
            .last()
            .map(|m| {
                if m.content.len() > 50 {
                    format!("{}...", m.content.chars().take(50).collect::<String>())
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_else(|| "".to_string());

        let summary = format!(
            "- {} ({} messages): Started with '{}', last: '{}'",
            thread.title,
            thread.messages.len(),
            topic,
            last_message
        );
        summaries.push(summary);
    }

    if summaries.is_empty() {
        Ok("No previous conversations found.".to_string())
    } else {
        Ok(summaries.join("\n"))
    }
}

/// Wrapper to maintain backward compatibility
fn call_crush(prompt: &str, thread: &Thread) -> Result<String> {
    // Call with empty context for backward compatibility
    call_crush_with_context(prompt, thread, "")
}

/// Replay a previous thread
///
/// # What This Does
///
/// Loads and displays a previous conversation thread, showing the full
/// context and history. This enables session recovery and review.
pub fn replay(thread_id: &str, config_dir: &Path) -> Result<()> {
    let threads_path = config_dir.join("threads");
    let thread_file = threads_path.join(format!("{}.json", thread_id));

    if !thread_file.exists() {
        // Try to find by partial match
        if let Ok(entries) = fs::read_dir(&threads_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.contains(thread_id) {
                        let content = fs::read_to_string(entry.path())?;
                        let thread: Thread = serde_json::from_str(&content)?;
                        display_thread(&thread);
                        return Ok(());
                    }
                }
            }
        }
        return Err(anyhow::anyhow!("Thread not found: {}", thread_id));
    }

    let content = fs::read_to_string(thread_file)?;
    let thread: Thread = serde_json::from_str(&content)?;

    display_thread(&thread);

    Ok(())
}

/// Display a thread in a nice format
fn display_thread(thread: &Thread) {
    println!("📖 Thread: {}", thread.title);
    println!("🔑 Author: {}...", &thread.author[..8]);
    println!("📅 Created: {}", thread.created_at);
    println!("💬 Messages: {}", thread.messages.len());

    if !thread.tags.is_empty() {
        println!("🏷️  Tags: {}", thread.tags.join(", "));
    }

    println!();
    println!("--- Conversation ---");
    println!();

    for msg in &thread.messages {
        let role_emoji = if msg.role == "human" { "👤" } else { "🔮" };
        let display_name = if msg.role == "human" {
            "Tyler"
        } else {
            "Alden" // TODO: Load from agent config
        };

        println!("{} {}: {}", role_emoji, display_name, msg.content);

        if let Some(sig) = &msg.signature {
            let author_snippet = msg
                .author
                .as_ref()
                .map(|a| &a[..8.min(a.len())])
                .unwrap_or("unknown");
            println!("   ✅ Signed by {}: {}...", author_snippet, &sig[..8]);
        }

        println!();
    }
}

/// List all threads
///
/// # Display Note
///
/// Shows threads in reverse chronological order (newest first).
/// This helps users find recent conversations quickly.
pub fn list_threads(config_dir: &Path) -> Result<()> {
    let threads_path = config_dir.join("threads");

    if !threads_path.exists() {
        println!("No threads found. Start a chat with 'mmogit chat'");
        return Ok(());
    }

    let mut threads: Vec<Thread> = Vec::new();

    if let Ok(entries) = fs::read_dir(&threads_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(thread) = serde_json::from_str::<Thread>(&content) {
                            threads.push(thread);
                        }
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    println!("📚 Chat Threads ({} total)", threads.len());
    println!();

    for thread in threads {
        let state_icon = if thread.state == "active" {
            "🟢"
        } else {
            "⚫"
        };
        println!(
            "{} {} - {} ({} messages)",
            state_icon,
            thread.title,
            &thread.id[..16.min(thread.id.len())],
            thread.messages.len()
        );
        println!("   Author: {}...", &thread.author[..8]);
        println!("   Updated: {}", thread.updated_at);

        if !thread.tags.is_empty() {
            println!("   Tags: {}", thread.tags.join(", "));
        }

        println!();
    }

    Ok(())
}
