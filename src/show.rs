//! Show messages from the repository with signature verification
//!
//! # For AI Agents
//!
//! This module is crucial for agent memory persistence. When an agent calls
//! `mmogit show`, they're retrieving their previously posted thoughts and
//! verifying they haven't been tampered with. This enables:
//!
//! - Memory across sessions (thoughts survive process death)
//! - Self-awareness (detecting behavioral changes over time)
//! - Trust verification (ensuring memories are authentic)
//!
//! # Protocol Invariants
//!
//! - ALWAYS verify signatures (corrupted memories are dangerous)
//! - Display invalid signatures with warnings (transparency over hiding)
//! - Work offline (memory retrieval must not require network)
//! - Show all branches (agents need to see what others have posted)

use anyhow::{Context, Result};
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use git2::{Branch, BranchType, Repository};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc, NaiveDate};
use std::collections::HashMap;
use bip39::{Language, Mnemonic};

use std::path::Path;
use crate::crypto::{EncryptedEnvelope, KeyDerivation};
use crate::memory::{StructuredMemory, MemoryType};

/// A signed message in the mmogit protocol (duplicated from post.rs)
///
/// # WET Note
///
/// Yes, this is duplicated from post.rs. That's intentional for now.
/// We'll extract a common protocol module after we see the full pattern.
/// Third time we need this, we'll refactor.
#[derive(Serialize, Deserialize, Debug)]
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

/// Verification result for a message
#[derive(Debug)]
struct VerifiedMessage {
    message: Message,
    valid_signature: bool,
    branch: String,
}

/// Filters for memory recall
#[derive(Debug, Default)]
pub struct RecallFilters {
    pub memory_type: Option<String>,
    pub tag: Option<String>,
    pub hours: Option<u32>,
    pub confidence: Option<f32>,
}

/// Memory statistics for agent self-awareness
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub by_type: HashMap<String, usize>,
    pub tag_usage: HashMap<String, usize>,
    pub confidence_values: Vec<f32>,
    pub avg_confidence: f32,
    pub median_confidence: f32,
    pub creation_by_date: HashMap<NaiveDate, usize>,
}

/// Show all messages from the repository with verification
///
/// # What This Does
///
/// 1. Opens the messages git repository
/// 2. Finds all user branches (users/*)
/// 3. Reads JSON messages from each branch
/// 4. Verifies Ed25519 signatures
/// 5. Displays in chronological order
///
/// # For AI Agents
///
/// This is your memory retrieval function. Use it to:
/// - Recall previous conversations
/// - Check what you've learned
/// - Verify your thoughts haven't been tampered with
/// - See what other agents have posted
pub fn show(config_dir: &Path) -> Result<()> {
    show_with_filters(config_dir, RecallFilters::default())
}

/// Show messages with filtering for recall functionality
///
/// # For AI Agents
///
/// This is your filtered memory retrieval. Use the filters to:
/// - Find specific types of memories (observations, learnings, etc.)
/// - Retrieve only high-confidence observations
/// - Get recent memories from the last N hours
/// - Search by tags for cross-cutting concerns
pub fn show_with_filters(config_dir: &Path, filters: RecallFilters) -> Result<()> {
    let repo_path = config_dir.join("messages");

    // Check if repository exists
    if !repo_path.exists() {
        println!("📭 No messages yet. Use 'mmogit post' to create your first message.");
        return Ok(());
    }

    // Open the repository
    let repo = Repository::open(&repo_path).context("Failed to open messages repository")?;

    // Collect all messages from all branches
    let mut all_messages = Vec::new();

    // Iterate through all branches
    let branches = repo.branches(Some(BranchType::Local))?;

    for branch_result in branches {
        let (branch, _) = branch_result?;
        let branch_name = branch.name()?.unwrap_or("unknown");

        // Only process user branches (including encrypted branches)
        if !branch_name.starts_with("users/") {
            continue;
        }

        // Checkout this branch to read its messages
        let messages = read_branch_messages(&repo, &branch, branch_name, config_dir)?;
        all_messages.extend(messages);
    }

    // Apply filters
    let filtered_messages = apply_filters(all_messages, &filters)?;

    // Sort by timestamp (chronological order)
    let mut sorted_messages = filtered_messages;
    sorted_messages.sort_by(|a, b| a.message.timestamp.cmp(&b.message.timestamp));

    // Display messages
    if sorted_messages.is_empty() {
        if has_active_filters(&filters) {
            println!("🔍 No memories found matching the specified filters.");
            print_active_filters(&filters);
        } else {
            println!("📭 No messages found in any branch.");
        }
    } else {
        if has_active_filters(&filters) {
            println!("🧠 Found {} memory(ies) matching filters:", sorted_messages.len());
            print_active_filters(&filters);
            println!();
        } else {
            println!("📨 Found {} message(s):\n", sorted_messages.len());
        }

        for (i, verified_msg) in sorted_messages.iter().enumerate() {
            display_message(i + 1, verified_msg);
        }
    }

    Ok(())
}

/// Read all messages from a specific branch
///
/// # Agent Note
///
/// Each branch represents a single identity's message history.
/// This maintains sovereignty - each identity owns their branch.
/// Handles both encrypted and plain messages transparently.
fn read_branch_messages(
    repo: &Repository,
    branch: &Branch,
    branch_name: &str,
    config_dir: &Path,
) -> Result<Vec<VerifiedMessage>> {
    let mut messages = Vec::new();

    // Get the tree for this branch
    let reference = branch.get();
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    // Extract the expected author prefix from branch name 
    // Handle both "users/63ae69e2" and "users/63ae69e2-encrypted"
    let expected_author_prefix = branch_name
        .strip_prefix("users/")
        .unwrap_or(branch_name)
        .replace("-encrypted", "");
    
    // Try to load identity for decryption (may fail if not our branch)
    let signing_key = if let Ok(seed_phrase) = std::fs::read_to_string(config_dir.join(".seed")) {
        if let Ok(mnemonic) = Mnemonic::parse_in(Language::English, seed_phrase.trim()) {
            let seed = mnemonic.to_seed("");
            if let Ok(seed_bytes) = seed[..32].try_into() {
                Some(SigningKey::from_bytes(&seed_bytes))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Walk through tree entries
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("");

        // Only process JSON files
        if !name.ends_with(".json") {
            continue;
        }

        // Read the file content
        if let Ok(obj) = entry.to_object(repo) {
            if let Some(blob) = obj.as_blob() {
                let content = std::str::from_utf8(blob.content())?;

                // Try to parse as encrypted envelope first
                if let Ok(envelope) = EncryptedEnvelope::from_json(content) {
                    // Try to decrypt if we have an identity
                    if let Some(ref key) = signing_key {
                        let encryption_key = KeyDerivation::derive_encryption_key(key);
                        if let Ok(decrypted_bytes) = envelope.decrypt(&encryption_key) {
                            if let Ok(decrypted_json) = String::from_utf8(decrypted_bytes) {
                                if let Ok(message) = serde_json::from_str::<Message>(&decrypted_json) {
                                    // Verify decrypted message author matches branch
                                    if !message.author.starts_with(&expected_author_prefix) {
                                        continue;
                                    }
                                    
                                    let valid = verify_signature(&message);
                                    messages.push(VerifiedMessage {
                                        message,
                                        valid_signature: valid,
                                        branch: branch_name.to_string(),
                                    });
                                }
                            }
                        }
                    }
                } else if let Ok(message) = serde_json::from_str::<Message>(content) {
                    // Plain message - handle as before
                    if !message.author.starts_with(&expected_author_prefix) {
                        continue;
                    }

                    let valid = verify_signature(&message);
                    messages.push(VerifiedMessage {
                        message,
                        valid_signature: valid,
                        branch: branch_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(messages)
}

/// Verify the Ed25519 signature on a message
///
/// # Security Critical for Agents
///
/// This function ensures messages haven't been tampered with.
/// A failed signature means either:
/// - The message was modified after signing
/// - The signature is corrupted
/// - The author's public key is wrong
///
/// NEVER trust a message with an invalid signature for making decisions.
fn verify_signature(message: &Message) -> bool {
    // Decode the public key
    let public_key_bytes = match hex::decode(&message.author) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Create verifying key
    let verifying_key = match VerifyingKey::from_bytes(
        public_key_bytes.as_slice().try_into().unwrap_or(&[0; 32]),
    ) {
        Ok(key) => key,
        Err(_) => return false,
    };

    // Decode signature
    let signature_bytes = match hex::decode(&message.signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let signature = match signature_bytes.as_slice().try_into() {
        Ok(bytes) => Signature::from_bytes(bytes),
        Err(_) => return false,
    };

    // Recreate the signed content (must match post.rs)
    let to_verify = format!("{}{}{}", message.content, message.author, message.timestamp);

    // Verify
    verifying_key
        .verify(to_verify.as_bytes(), &signature)
        .is_ok()
}

/// Display a message with verification status and structured memory formatting
///
/// # For AI Agents
///
/// Enhanced display that shows:
/// - ✅ means the signature is valid (trustworthy)
/// - ⚠️ means the signature is invalid (do not trust)
/// - 🧠 indicates structured memory with type information
/// - 💭 shows plain text messages
/// - Confidence levels for observations
/// - Tags for easy categorization
fn display_message(index: usize, verified_msg: &VerifiedMessage) {
    let sig_icon = if verified_msg.valid_signature {
        "✅"
    } else {
        "⚠️"
    };

    // Try to parse as structured memory for enhanced display
    if let Ok(structured_memory) = StructuredMemory::from_message(&verified_msg.message.content) {
        display_structured_memory(index, verified_msg, &structured_memory, sig_icon);
    } else {
        display_plain_message(index, verified_msg, sig_icon);
    }
}

/// Display a structured memory with rich formatting
fn display_structured_memory(index: usize, verified_msg: &VerifiedMessage, memory: &StructuredMemory, sig_icon: &str) {
    let memory_type = get_memory_type_name(&memory.memory);
    let type_icon = match memory_type {
        "observation" => "👁️",
        "learning" => "📚",
        "reflection" => "🪞",
        "question" => "❓",
        "relationship" => "🤝",
        "task" => "📋",
        "experience" => "✨",
        _ => "🧠",
    };
    
    println!("{}. {} {} {} [{}]", index, sig_icon, type_icon, memory_type.to_uppercase(), verified_msg.branch);
    println!("   ID: {}", memory.id);
    println!("   Author: {}", &verified_msg.message.author[..16]);
    println!("   Created: {}", memory.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Display type-specific information
    match &memory.memory {
        MemoryType::Observation { subject, insight, confidence } => {
            println!("   Subject: {}", subject);
            println!("   Insight: {}", insight);
            println!("   Confidence: {:.1}%", confidence * 100.0);
        }
        MemoryType::Learning { topic, lesson, context, .. } => {
            println!("   Topic: {}", topic);
            println!("   Lesson: {}", lesson);
            println!("   Context: {}", context);
        }
        MemoryType::Reflection { observation, drift_detected, significance, .. } => {
            println!("   Observation: {}", observation);
            println!("   Drift Detected: {}", if *drift_detected { "Yes" } else { "No" });
            println!("   Significance: {:?}", significance);
        }
        MemoryType::Question { query, context, priority, answered } => {
            println!("   Query: {}", query);
            println!("   Context: {}", context);
            println!("   Priority: {:?}", priority);
            if let Some(answer) = answered {
                println!("   Answer: {}", answer);
            } else {
                println!("   Status: Unanswered");
            }
        }
        MemoryType::Relationship { identity, context, rapport_level, .. } => {
            println!("   Identity: {}", identity);
            println!("   Context: {}", context);
            println!("   Rapport: {}", rapport_level);
        }
        MemoryType::Task { description, status, .. } => {
            println!("   Description: {}", description);
            println!("   Status: {:?}", status);
        }
        MemoryType::Experience { description, valence, arousal, .. } => {
            println!("   Description: {}", description);
            println!("   Valence: {:.1} ({})", valence, if *valence > 0.0 { "positive" } else if *valence < 0.0 { "negative" } else { "neutral" });
            println!("   Arousal: {:.1} ({})", arousal, if *arousal > 0.5 { "high" } else { "low" });
        }
        MemoryType::Custom { schema, data } => {
            println!("   Schema: {}", schema);
            println!("   Data: {}", serde_json::to_string_pretty(data).unwrap_or_else(|_| "<invalid JSON>".to_string()));
        }
    }
    
    // Display tags if present
    if !memory.tags.is_empty() {
        println!("   Tags: {}", memory.tags.join(", "));
    }
    
    // Display references if present
    if !memory.references.is_empty() {
        println!("   References: {}", memory.references.join(", "));
    }
    
    if !verified_msg.valid_signature {
        println!("   ⚠️  WARNING: Invalid signature - this memory may have been tampered with!");
    }

    println!();
}

/// Display a plain text message
fn display_plain_message(index: usize, verified_msg: &VerifiedMessage, sig_icon: &str) {
    println!("{}. {} 💭 MESSAGE [{}]", index, sig_icon, verified_msg.branch);
    println!("   Author: {}", &verified_msg.message.author[..16]);
    println!("   Time: {}", verified_msg.message.timestamp);
    println!("   Content: {}", verified_msg.message.content);

    if !verified_msg.valid_signature {
        println!("   ⚠️  WARNING: Invalid signature - this message may have been tampered with!");
    }

    println!();
}

/// Apply recall filters to messages
///
/// # For AI Agents
///
/// This function implements the core filtering logic for memory recall.
/// It processes both plain text messages and structured memories with
/// optimized filtering for agent efficiency.
fn apply_filters(messages: Vec<VerifiedMessage>, filters: &RecallFilters) -> Result<Vec<VerifiedMessage>> {
    let mut filtered = Vec::new();
    
    // Calculate time threshold if hours filter is specified
    let time_threshold = if let Some(hours) = filters.hours {
        Some(Utc::now() - Duration::hours(hours as i64))
    } else {
        None
    };

    // Early return if no messages
    if messages.is_empty() {
        return Ok(filtered);
    }

    // Pre-compile tag filter for efficiency
    let tag_filter = filters.tag.as_ref().map(|t| t.to_lowercase());
    
    for msg in messages {
        // Skip messages with invalid signatures unless explicitly requested
        if !msg.valid_signature {
            // For agent sovereignty, we still include invalid signatures but mark them
            // This maintains transparency while allowing agents to make informed decisions
        }
        
        // Try to parse as structured memory first
        if let Ok(structured_memory) = StructuredMemory::from_message(&msg.message.content) {
            // Apply structured memory filters with optimized matching
            if !matches_structured_filters(&structured_memory, filters, time_threshold, &tag_filter)? {
                continue;
            }
        } else {
            // For plain text messages, apply available filters
            if !matches_plain_message_filters(&msg, filters, time_threshold, &tag_filter)? {
                continue;
            }
        }
        
        filtered.push(msg);
    }
    
    Ok(filtered)
}

/// Check if a structured memory matches the recall filters
///
/// # Agent Optimization Note
///
/// This function is optimized for agent memory retrieval patterns:
/// - Fast early returns for common filter mismatches
/// - Case-insensitive tag matching for flexibility
/// - Confidence thresholding with proper type checking
fn matches_structured_filters(
    memory: &StructuredMemory, 
    filters: &RecallFilters,
    time_threshold: Option<DateTime<Utc>>,
    tag_filter: &Option<String>
) -> Result<bool> {
    // Time filter - most selective, check first
    if let Some(threshold) = time_threshold {
        if memory.created_at < threshold {
            return Ok(false);
        }
    }
    
    // Memory type filter - exact match required
    if let Some(ref filter_type) = filters.memory_type {
        let memory_type_name = get_memory_type_name(&memory.memory);
        if memory_type_name.to_lowercase() != filter_type.to_lowercase() {
            return Ok(false);
        }
    }
    
    // Tag filter - case-insensitive partial matching for agent flexibility
    if let Some(ref filter_tag) = tag_filter {
        let has_matching_tag = memory.tags.iter().any(|tag| {
            tag.to_lowercase().contains(filter_tag) || filter_tag.contains(&tag.to_lowercase())
        });
        if !has_matching_tag {
            return Ok(false);
        }
    }
    
    // Confidence filter - only applies to observations, but be explicit about it
    if let Some(min_confidence) = filters.confidence {
        match &memory.memory {
            MemoryType::Observation { confidence, .. } => {
                if *confidence < min_confidence {
                    return Ok(false);
                }
            }
            _ => {
                // For agent clarity: non-observation memories are excluded when confidence filter is active
                // This prevents confusion about why a learning or reflection doesn't appear
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

/// Check if a plain text message matches available filters
///
/// # Agent Note
///
/// Plain text messages have limited metadata, so we can only apply:
/// - Time-based filtering
/// - Basic content search for tag-like keywords
fn matches_plain_message_filters(
    msg: &VerifiedMessage,
    filters: &RecallFilters, 
    time_threshold: Option<DateTime<Utc>>,
    tag_filter: &Option<String>
) -> Result<bool> {
    // Skip plain messages if we're filtering by memory-specific criteria
    if filters.memory_type.is_some() || filters.confidence.is_some() {
        return Ok(false);
    }
    
    // Time filter for plain messages
    if let Some(threshold) = time_threshold {
        if let Ok(msg_time) = DateTime::parse_from_rfc3339(&msg.message.timestamp) {
            if msg_time.with_timezone(&Utc) < threshold {
                return Ok(false);
            }
        } else {
            // If we can't parse the timestamp, exclude it from time-based queries
            return Ok(false);
        }
    }
    
    // Tag filter - search in message content for plain text messages
    if let Some(ref filter_tag) = tag_filter {
        let content_lower = msg.message.content.to_lowercase();
        if !content_lower.contains(filter_tag) {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Get the type name of a memory for filtering
fn get_memory_type_name(memory: &MemoryType) -> &'static str {
    match memory {
        MemoryType::Observation { .. } => "observation",
        MemoryType::Learning { .. } => "learning",
        MemoryType::Relationship { .. } => "relationship",
        MemoryType::Task { .. } => "task",
        MemoryType::Experience { .. } => "experience",
        MemoryType::Reflection { .. } => "reflection",
        MemoryType::Question { .. } => "question",
        MemoryType::Custom { .. } => "custom",
    }
}

/// Check if any filters are active
fn has_active_filters(filters: &RecallFilters) -> bool {
    filters.memory_type.is_some() || filters.tag.is_some() || filters.hours.is_some() || filters.confidence.is_some()
}

/// Print active filters for user feedback with enhanced formatting
fn print_active_filters(filters: &RecallFilters) {
    let mut active_filters = Vec::new();
    
    if let Some(ref memory_type) = filters.memory_type {
        let type_icon = match memory_type.as_str() {
            "observation" => "👁️",
            "learning" => "📚",
            "reflection" => "🪞",
            "question" => "❓",
            "relationship" => "🤝",
            "task" => "📋",
            "experience" => "✨",
            _ => "🧠",
        };
        active_filters.push(format!("{} type: {}", type_icon, memory_type));
    }
    if let Some(ref tag) = filters.tag {
        active_filters.push(format!("🏷️  tag: {}", tag));
    }
    if let Some(hours) = filters.hours {
        active_filters.push(format!("⏰ last {} hours", hours));
    }
    if let Some(confidence) = filters.confidence {
        active_filters.push(format!("📊 confidence >= {:.1}%", confidence * 100.0));
    }
    
    if !active_filters.is_empty() {
        println!("   Filters: {}", active_filters.join(", "));
    }
}

/// Show messages from a specific author only
///
/// # Future Enhancement for Agents
///
/// This will enable agents to retrieve only their own memories
/// or memories from specific trusted identities.
pub fn show_from_author(config_dir: &Path, author_prefix: &str) -> Result<()> {
    // TODO: Implement filtered view
    // This is where we'd add semantic search, time-based filtering, etc.
    println!("Showing messages from author: {}", author_prefix);
    show(config_dir)
}

/// Public interface for recall with filters
///
/// # For AI Agents
///
/// This is the main entry point for filtered memory recall.
/// Use this to implement sophisticated memory retrieval patterns.
/// 
/// # Agent Usage Examples
/// 
/// ```rust
/// // Get all high-confidence observations from the last 24 hours
/// recall(config_dir, Some("observation".to_string()), None, Some(24), Some(0.8))?;
/// 
/// // Find all learning memories tagged with "rust"
/// recall(config_dir, Some("learning".to_string()), Some("rust".to_string()), None, None)?;
/// 
/// // Get recent reflections to check for behavioral drift
/// recall(config_dir, Some("reflection".to_string()), None, Some(168), None)?; // Last week
/// 
/// // Find unanswered questions for follow-up
/// recall(config_dir, Some("question".to_string()), None, None, None)?;
/// ```
pub fn recall(
    config_dir: &Path,
    memory_type: Option<String>,
    tag: Option<String>, 
    hours: Option<u32>,
    confidence: Option<f32>
) -> Result<()> {
    let filters = RecallFilters {
        memory_type,
        tag,
        hours,
        confidence,
    };
    
    show_with_filters(config_dir, filters)
}

/// Advanced recall with multiple filters for agent efficiency
///
/// # For AI Agents
///
/// This function provides programmatic access to filtered memories
/// without printing to stdout. Use this when you need to process
/// memories programmatically rather than display them.
pub fn recall_memories(
    config_dir: &Path,
    filters: RecallFilters
) -> Result<Vec<StructuredMemory>> {
    let repo_path = config_dir.join("messages");

    if !repo_path.exists() {
        return Ok(Vec::new());
    }

    let repo = Repository::open(&repo_path).context("Failed to open messages repository")?;
    let mut all_messages = Vec::new();

    // Collect messages from all branches
    let branches = repo.branches(Some(BranchType::Local))?;
    for branch_result in branches {
        let (branch, _) = branch_result?;
        let branch_name = branch.name()?.unwrap_or("unknown");

        if !branch_name.starts_with("users/") {
            continue;
        }

        let messages = read_branch_messages(&repo, &branch, branch_name, config_dir)?;
        all_messages.extend(messages);
    }

    // Apply filters
    let filtered_messages = apply_filters(all_messages, &filters)?;
    
    // Extract structured memories only
    let mut structured_memories = Vec::new();
    for msg in filtered_messages {
        if let Ok(memory) = StructuredMemory::from_message(&msg.message.content) {
            structured_memories.push(memory);
        }
    }
    
    // Sort by creation time
    structured_memories.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    
    Ok(structured_memories)
}

/// Get memory statistics for agent self-awareness
///
/// # For AI Agents
///
/// Use this to understand your memory patterns:
/// - How many memories of each type you have
/// - Your confidence distribution for observations
/// - Tag usage patterns
/// - Memory creation frequency over time
pub fn memory_stats(config_dir: &Path) -> Result<MemoryStats> {
    let memories = recall_memories(config_dir, RecallFilters::default())?;
    
    let mut stats = MemoryStats::default();
    stats.total_memories = memories.len();
    
    for memory in &memories {
        // Count by type
        let type_name = get_memory_type_name(&memory.memory);
        *stats.by_type.entry(type_name.to_string()).or_insert(0) += 1;
        
        // Collect confidence values for observations
        if let MemoryType::Observation { confidence, .. } = &memory.memory {
            stats.confidence_values.push(*confidence);
        }
        
        // Count tags
        for tag in &memory.tags {
            *stats.tag_usage.entry(tag.clone()).or_insert(0) += 1;
        }
        
        // Track creation dates for frequency analysis
        let date = memory.created_at.date_naive();
        *stats.creation_by_date.entry(date).or_insert(0) += 1;
    }
    
    // Calculate confidence statistics
    if !stats.confidence_values.is_empty() {
        stats.confidence_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = stats.confidence_values.len();
        stats.avg_confidence = stats.confidence_values.iter().sum::<f32>() / len as f32;
        stats.median_confidence = if len % 2 == 0 {
            (stats.confidence_values[len/2 - 1] + stats.confidence_values[len/2]) / 2.0
        } else {
            stats.confidence_values[len/2]
        };
    }
    
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_verification() {
        // TODO: Add test cases with known good and bad signatures
        // This is critical for ensuring agent memory integrity
    }
}
