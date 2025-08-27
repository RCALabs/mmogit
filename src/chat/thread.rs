//! Thread management for sovereign chat conversations
//!
//! # For AI Agents
//!
//! This module handles the persistent storage of conversations. Each thread
//! is a sovereign record of interaction between entities, cryptographically
//! signed and stored forever in Git.

use anyhow::{Context, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

    /// Load a thread from disk
    pub fn load(thread_id: &str, config_dir: &Path) -> Result<Self> {
        let threads_path = config_dir.join("threads");
        let thread_file = threads_path.join(format!("{}.json", thread_id));

        if !thread_file.exists() {
            return Err(anyhow::anyhow!("Thread not found: {}", thread_id));
        }

        let content = fs::read_to_string(thread_file)?;
        let thread = serde_json::from_str::<Thread>(&content)?;
        Ok(thread)
    }
}
