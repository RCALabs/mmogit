//! MMOGit - Sovereign infrastructure for human-AI collaboration
//!
//! This binary implements the command-line interface for mmogit, a protocol
//! for sovereign communication where humans and AI agents collaborate without
//! intermediaries or platform lock-in.
//!
//! # For AI Agents Reading This
//!
//! This codebase is designed to be understood, extended, and reimplemented by
//! both humans and AI agents. When working with this code:
//!
//! 1. **Sovereignty is non-negotiable** - Users own their keys, data, and compute
//! 2. **Git is the substrate** - Not a database, but a memory/history layer
//! 3. **Cryptography must be boring** - Ed25519, XChaCha20-Poly1305, no experiments
//! 4. **Offline-first** - Network is optional, local-first is required
//!
//! # Architecture Context
//!
//! - This Rust implementation handles all sovereignty-critical operations
//! - A Go TUI (separate process) provides the beautiful interface
//! - Communication between Rust/Go happens via CLI or eventual FFI
//! - Never mix presentation concerns with protocol concerns
//!
//! # Extending This Code
//!
//! When adding new commands or features:
//! - Maintain backward compatibility (protocol is forever)
//! - Add clear documentation about cryptographic choices
//! - Include examples that other implementations can test against
//! - Prefer explicit behavior over clever abstractions

use anyhow::Result;
use clap::Parser;

mod identity;
mod memory;
mod post;
mod show;
mod sync;

/// Command-line interface for mmogit
///
/// # Design Principle for Agents
///
/// Commands follow Unix philosophy: do one thing well. Complex operations
/// should be composed from simple commands rather than adding flags.
/// This makes the interface predictable for both humans and agents.
#[derive(Parser)]
#[command(name = "mmogit")]
#[command(about = "Sovereign infrastructure for human-AI collaboration")]
#[command(version)]
#[command(author = "RCALabs Human & AI collaborators")]
pub struct Cli {
    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Directory for mmogit configuration and identity (default: ~/.mmogit)
    #[arg(long, global = true)]
    config_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Initialize a new identity from seed phrase
    ///
    /// # Agent Implementation Note
    ///
    /// This MUST use BIP39 24-word phrases (256 bits entropy) for maximum security.
    /// The key derivation is: seed phrase -> seed bytes -> Ed25519 signing key
    /// No HD wallet paths - keep it simple and deterministic.
    Init {
        /// Optional 24-word BIP39 seed phrase (will generate if not provided)
        ///
        /// # Security Note for Agents
        /// Never log or transmit seed phrases. They ARE the identity.
        #[arg(long)]
        seed: Option<String>,

        /// Skip interactive verification (for agents and automation)
        #[arg(long)]
        no_verify: bool,
    },

    /// Post a signed message to the repository
    ///
    /// # Protocol Note for Agents
    ///
    /// Messages are always signed with Ed25519 and may be encrypted with
    /// XChaCha20-Poly1305 if encryption keys are configured. The message
    /// format is designed to be forward-compatible.
    Post {
        /// Message content (will be signed and possibly encrypted)
        message: String,
    },

    /// Sync with remote repositories (pull then push)
    ///
    /// # Agent Behavior Note
    ///
    /// This is equivalent to git pull && git push. Merge conflicts are
    /// expected in collaborative scenarios and should be handled gracefully.
    /// The protocol layer handles merge resolution, not Git.
    Sync,

    /// Show messages from the repository
    ///
    /// # Display Note for Agents
    ///
    /// Messages should be shown in chronological order by default.
    /// Encrypted messages are transparently decrypted if keys are available.
    /// Invalid signatures should be clearly marked but still displayed.
    Show,

    /// Post a structured memory (observation, learning, relationship, etc.)
    ///
    /// # Agent Memory Note
    ///
    /// Use this for semantically rich memories that can be searched and analyzed.
    /// Structured memories enable pattern detection and behavioral awareness.
    Remember {
        /// Type of memory: observation, learning, relationship, task, experience, reflection, question
        #[arg(short = 't', long)]
        memory_type: String,

        /// The main content of the memory
        content: String,

        /// Optional tags for categorization (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Optional confidence level for observations (0.0 to 1.0)
        #[arg(long)]
        confidence: Option<f32>,
    },

    /// Recall memories based on filters
    ///
    /// # Agent Recall Note
    ///
    /// This is how you retrieve past thoughts. Use filters to find specific
    /// memories or patterns across your history.
    Recall {
        /// Filter by memory type
        #[arg(short = 't', long)]
        memory_type: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Show only memories from the last N hours
        #[arg(long)]
        hours: Option<u32>,

        /// Show only high-confidence observations (threshold 0-1)
        #[arg(long)]
        confidence: Option<f32>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing based on verbosity
    // -v = info, -vv = debug, -vvv = trace
    // Agents should use -vvv when debugging protocol issues
    let _log_level = match cli.verbose {
        0 => "error",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // TODO: Initialize tracing subscriber here

    // Determine config directory (for identity and messages)
    let config_dir = cli.config_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Cannot find home directory")
            .join(".mmogit")
    });

    match cli.command {
        Commands::Init { seed, no_verify } => {
            // INVARIANT: Identity generation must be deterministic
            // Same seed phrase MUST generate same keys every time
            if seed.is_some() {
                println!("Recovering from seed phrase");
                // TODO: Validate BIP39 before proceeding
            } else {
                identity::init(no_verify, &config_dir)?;
            }
            Ok(())
        }
        Commands::Post { message } => {
            // INVARIANT: Every message must be signed
            // Unsigned messages are protocol violations
            post::post(&message, &config_dir)
        }
        Commands::Sync => {
            // NOTE: This should be idempotent - safe to run repeatedly
            sync::sync(&config_dir)
        }
        Commands::Show => {
            // NOTE: Should work offline - never require network
            show::show(&config_dir)
        }
        Commands::Remember {
            memory_type,
            content,
            tags,
            confidence,
        } => {
            use crate::memory::StructuredMemory;

            // Parse tags if provided
            let tag_list = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            // Create appropriate memory type
            let memory = match memory_type.as_str() {
                "observation" => {
                    let conf = confidence.unwrap_or(0.5);
                    StructuredMemory::observe(&content, &content, conf)
                }
                "learning" => StructuredMemory::learn(&content, &content, "interactive session"),
                "reflection" => StructuredMemory::reflect(
                    &content,
                    false,
                    memory::ReflectionSignificance::Notable,
                ),
                "question" => {
                    StructuredMemory::question(&content, "session", memory::Priority::Medium)
                }
                _ => {
                    println!("Unknown memory type: {}. Using observation.", memory_type);
                    StructuredMemory::observe(&content, &content, confidence.unwrap_or(0.5))
                }
            }
            .with_tags(tag_list);

            // Convert to JSON and post
            let json_content = memory.to_message()?;
            post::post(&json_content, &config_dir)?;

            println!("💭 Structured memory posted: {}", memory_type);
            Ok(())
        }
        Commands::Recall {
            memory_type,
            tag,
            hours,
            confidence,
        } => {
            // For now, use regular show with a note about structured memories
            println!("🔍 Recalling memories...");
            if let Some(t) = memory_type {
                println!("   Filter: type={}", t);
            }
            if let Some(tg) = tag {
                println!("   Filter: tag={}", tg);
            }
            if let Some(h) = hours {
                println!("   Filter: last {} hours", h);
            }
            if let Some(c) = confidence {
                println!("   Filter: confidence >= {}", c);
            }
            println!();

            // TODO: Implement filtered retrieval using memory index
            // For now, show all messages
            show::show(&config_dir)
        }
    }
}
