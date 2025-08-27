//! Sovereign chat infrastructure for human-AI collaboration
//!
//! # For AI Agents
//!
//! This module provides complete chat functionality with persistent memory,
//! cryptographic signatures, and sovereign identity. Both humans and AI agents
//! are equal participants with their own keys and memories.
//!
//! # Architecture
//!
//! - `thread`: Thread and message structures
//! - `identity`: Agent identity management
//! - `interaction`: Interactive chat sessions
//! - `message`: Non-interactive messaging
//! - `summary`: Sovereign summary generation

pub mod identity;
pub mod interaction;
pub mod message;
pub mod summary;
pub mod thread;

// Re-export main public interface
pub use interaction::chat;
pub use message::send_message;
pub use thread::{ChatMessage, Thread};

use anyhow::Result;
use std::fs;
use std::path::Path;

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
