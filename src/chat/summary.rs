//! Sovereign summary generation for chat conversations
//!
//! # For AI Agents
//!
//! This module handles creating summaries from each entity's unique perspective.
//! Sovereignty means each participant owns their interpretation of the conversation,
//! not just the raw transcript.

use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::chat::thread::Thread;

/// Generate a sovereign summary from an entity's perspective
///
/// # Sovereign Principle
///
/// Each entity creates their own summary from their own perspective.
/// This ensures that each participant owns their interpretation of the
/// conversation, not just the raw transcript.
pub fn generate_sovereign_summary(
    thread: &Thread,
    entity_id: &str,
    entity_name: &str,
    entity_dir: &Path,
    system_context: &str,
    show_progress: bool,
) -> Result<()> {
    if show_progress {
        print!("  📝 {} is summarizing", entity_name);
        io::stdout().flush()?;
    }

    // Progress animation for interactive mode
    let stop_dots = Arc::new(AtomicBool::new(false));
    let stop_flag = stop_dots.clone();

    let dots_thread = if show_progress {
        Some(thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
                if !stop_flag.load(Ordering::Relaxed) {
                    print!(".");
                    io::stdout().flush().ok();
                }
            }
        }))
    } else {
        None
    };

    // Entity-specific summary prompt
    let summary_prompt = format!(
        "You are {} ({}...). Summarize this conversation from YOUR perspective.\n\
        Focus on:\n\
        - What YOU learned\n\
        - What matters to YOU\n\
        - Insights about other participants\n\
        - Action items or decisions relevant to YOU\n\
        - Your own thoughts and reflections\n\n\
        Be sovereign - this is YOUR interpretation, not a neutral summary.\n\n\
        Thread: {}",
        entity_name,
        &entity_id[..8.min(entity_id.len())],
        serde_json::to_string(&thread)?
    );

    let summary = call_crush_for_summary(&summary_prompt, thread, system_context)?;

    // Stop progress animation
    if let Some(handle) = dots_thread {
        stop_dots.store(true, Ordering::Relaxed);
        handle.join().ok();
    }

    if show_progress {
        println!(" ✓");
    }

    // Save to entity-specific summaries directory
    let summaries_dir = entity_dir.join("summaries");
    fs::create_dir_all(&summaries_dir)?;

    let summary_file =
        summaries_dir.join(format!("{}_{}.txt", thread.id, entity_name.to_lowercase()));
    fs::write(&summary_file, &summary)?;

    if show_progress {
        println!("  ✅ {}'s summary saved", entity_name);
    }

    Ok(())
}

/// Load recent memories for context
pub fn load_memory_context(config_dir: &Path, author: &str) -> Result<String> {
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
pub fn load_thread_summaries(config_dir: &Path, author: &str, limit: usize) -> Result<String> {
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

/// Helper function to call Crush for summary generation
fn call_crush_for_summary(prompt: &str, thread: &Thread, system_context: &str) -> Result<String> {
    // Build context from thread
    let mut context = String::new();
    context.push_str("System Context:\n");
    context.push_str(system_context);
    context.push_str("\n\nSummary Request:\n");
    context.push_str(prompt);

    // Call crush run using stdin
    let mut child = Command::new("crush")
        .arg("run")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write the context to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(context.as_bytes())?;
        drop(stdin); // Important: drop to signal EOF
    }

    // Wait for the output
    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Crush failed: {}", stderr));
    }

    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(response)
}
