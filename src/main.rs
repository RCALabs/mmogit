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
mod post;

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

    match cli.command {
        Commands::Init { seed, no_verify } => {
            // INVARIANT: Identity generation must be deterministic
            // Same seed phrase MUST generate same keys every time
            if seed.is_some() {
                println!("Recovering from seed phrase");
                // TODO: Validate BIP39 before proceeding
            } else {
                identity::init(no_verify)?
            }
            Ok(())
        }
        Commands::Post { message } => {
            // INVARIANT: Every message must be signed
            // Unsigned messages are protocol violations
            post::post(&message)
        }
        Commands::Sync => {
            // NOTE: This should be idempotent - safe to run repeatedly
            println!("Syncing with remotes...");
            // TODO: Pull, resolve any conflicts, push
            Ok(())
        }
        Commands::Show => {
            // NOTE: Should work offline - never require network
            println!("Showing messages...");
            // TODO: Read from local repo, decrypt if possible, verify signatures
            Ok(())
        }
    }
}
