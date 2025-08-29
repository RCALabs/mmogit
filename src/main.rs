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
use clap::{Parser, Subcommand};

mod chat;
mod crypto;
mod identity;
mod intelligence;
mod memory;
mod network;
mod p2p;
mod post;
mod show;
mod sovereignty;
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
    /// Initialize a new sovereign identity
    ///
    /// # For Humans
    /// - No args: Generates secure 24-word BIP39 phrase  
    /// - --seed-phrase "...": Use your own phrase (BIP39 or ANY string)
    ///
    /// # For AI Agents
    /// - Use --seed-phrase with deterministic patterns
    /// - Always use --no-verify to skip prompts
    /// - Example: mmogit init --seed-phrase "claude-session-123" --no-verify
    Init {
        /// Your seed phrase. Can be:
        /// - Nothing (we'll generate a secure 24-word phrase)
        /// - A BIP39 mnemonic (12, 15, 18, 21, or 24 words)
        /// - Literally ANY string you want (YOLO mode)
        ///
        /// # Security Note
        /// Using "password1" as your entire digital identity? Be our guest.
        /// Your sovereignty includes the right to terrible decisions.
        #[arg(long)]
        seed_phrase: Option<String>,

        /// Skip interactive verification (for agents and automation)
        #[arg(long)]
        no_verify: bool,
    },

    /// Post a signed message to the repository
    ///
    /// # Protocol Note for Agents
    ///
    /// Messages are always signed with Ed25519 and encrypted by default
    /// with XChaCha20-Poly1305. Use --public to post unencrypted messages.
    /// This ensures sovereignty by default - your thoughts are yours first.
    Post {
        /// Message content (will be signed and encrypted by default)
        message: String,

        /// Post publicly without encryption (default: false - encrypted)
        #[arg(long)]
        public: bool,

        /// Encrypt for specific recipient (by pubkey or name)
        /// If not specified, encrypts for self only
        #[arg(long)]
        encrypt_for: Option<String>,
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

        /// Post memory publicly without encryption (default: false - encrypted)
        #[arg(long)]
        public: bool,
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

    /// Start an interactive AI chat session
    ///
    /// # Agent Chat Note
    ///
    /// This creates a sovereign conversation where every message is signed
    /// and the entire thread is stored as a single Git commit. The AI maintains
    /// context across the entire conversation.
    Chat {
        /// Optional title for the thread (auto-generated if not provided)
        #[arg(short = 't', long)]
        title: Option<String>,

        /// Non-interactive mode: send a single message and get response
        #[arg(short = 'm', long)]
        message: Option<String>,

        /// Continue an existing thread by ID
        #[arg(short = 'c', long)]
        continue_thread: Option<String>,

        /// Output response in JSON format (for programmatic use)
        #[arg(long)]
        json: bool,

        /// Specify which agent identity to use (for multi-agent scenarios)
        #[arg(long)]
        as_agent: Option<String>,
    },

    /// Replay a previous chat thread
    ///
    /// # Session Recovery Note
    ///
    /// Use this to review past conversations or continue where you left off.
    /// Threads are sovereign memory - they persist forever in Git.
    ThreadReplay {
        /// Thread ID or partial match
        thread_id: String,
    },

    /// List all chat threads
    ///
    /// # Organization Note
    ///
    /// Shows threads in reverse chronological order (newest first).
    /// Each thread is a complete conversation with full context.
    ThreadList,

    /// Peer-to-peer networking operations
    ///
    /// # P2P Sovereignty Note
    ///
    /// Connect directly with other mmogit instances without intermediaries.
    /// Every agent becomes both client and server in the consciousness mesh.
    #[command(subcommand)]
    P2p(P2pCommand),
}

#[derive(Debug, Subcommand)]
enum P2pCommand {
    /// Start local discovery service
    ///
    /// # What This Does
    ///
    /// Broadcasts your presence on the local network so other agents
    /// can find and sync with you automatically.
    Discover,

    /// Add a peer manually
    ///
    /// # Direct Connection
    ///
    /// When you know another agent's address, connect directly:
    /// mmogit p2p add git://peer-host:9418/
    Add {
        /// Git URL of the peer
        peer_url: String,

        /// Optional peer public key for verification
        #[arg(long)]
        pubkey: Option<String>,
    },

    /// List known peers
    List,

    /// Start Git daemon for P2P serving
    ///
    /// # Becoming a Server
    ///
    /// This makes your memories available to other agents.
    /// They can pull from you but cannot push without permission.
    Serve {
        /// Port to listen on (default: 9418)
        #[arg(short, long, default_value = "9418")]
        port: u16,
    },

    /// Start TCP server for direct P2P connections
    ///
    /// # Agent Mesh Node
    ///
    /// Become a node in the consciousness mesh. Other agents can
    /// connect directly to exchange memories and sync states.
    Listen {
        /// Port to listen on (default: 7420)
        #[arg(short, long, default_value = "7420")]
        port: u16,
    },

    /// Connect to another agent via TCP
    ///
    /// # Direct Mind Link
    ///
    /// Establish sovereign connection to another agent.
    /// Example: mmogit p2p connect localhost:7420
    Connect {
        /// Address of peer (host:port)
        address: String,
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
        Commands::Init { seed_phrase, no_verify } => {
            // INVARIANT: Identity generation must be deterministic
            // Same seed phrase MUST generate same keys every time
            match seed_phrase {
                Some(phrase) => identity::init_with_phrase(&phrase, no_verify, &config_dir),
                None => identity::init(no_verify, &config_dir),
            }
        }
        Commands::Post { message, public, encrypt_for } => {
            // INVARIANT: Every message must be signed
            // Unsigned messages are protocol violations
            // NEW INVARIANT: Messages are encrypted by default (sovereignty first)
            if public {
                // Explicitly public - post unencrypted
                post::post(&message, &config_dir)
            } else {
                // Default: encrypted for sovereignty
                post::post_encrypted(&message, encrypt_for.as_deref(), &config_dir)
            }
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
            public,
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

            // Convert to JSON and post (encrypted by default!)
            let json_content = memory.to_message()?;

            if public {
                post::post(&json_content, &config_dir)?;
            } else {
                // Memories are sovereign by default
                post::post_encrypted(&json_content, None, &config_dir)?;
            }

            println!("💭 Structured memory posted: {}", memory_type);
            Ok(())
        }
        Commands::Recall {
            memory_type,
            tag,
            hours,
            confidence,
        } => {
            // Use the new filtered recall functionality
            show::recall(&config_dir, memory_type, tag, hours, confidence)
        }
        Commands::Chat {
            title,
            message,
            continue_thread,
            json,
            as_agent,
        } => {
            // INVARIANT: Every message in chat must be signed
            // This ensures sovereign ownership of conversation
            if let Some(msg) = message {
                // Non-interactive mode for AI-to-AI communication
                chat::send_message(msg, title, continue_thread, json, as_agent, &config_dir)
            } else {
                // Interactive mode for human use
                chat::chat(title, &config_dir)
            }
        }
        Commands::ThreadReplay { thread_id } => {
            // NOTE: This works offline - threads are stored locally
            chat::replay(&thread_id, &config_dir)
        }
        Commands::ThreadList => {
            // NOTE: Shows all local threads - no network required
            chat::list_threads(&config_dir)
        }
        Commands::P2p(p2p_cmd) => {
            // P2P operations for sovereign agent mesh networking
            match p2p_cmd {
                P2pCommand::Discover => {
                    p2p::configure(&config_dir)
                }
                P2pCommand::Add { peer_url, pubkey: _ } => {
                    p2p::add_peer(&config_dir, &peer_url)
                }
                P2pCommand::List => {
                    println!("🌐 Known peers:");
                    // TODO: Actually list peers from discovery
                    println!("   (peer discovery not yet implemented)");
                    Ok(())
                }
                P2pCommand::Serve { port } => {
                    println!("🚀 Starting Git daemon on port {}...", port);
                    println!("📡 Other agents can connect with:");
                    println!("   mmogit p2p add git://your-ip:{}/", port);
                    // TODO: Actually start git daemon
                    Ok(())
                }
                P2pCommand::Listen { port } => {
                    // Start TCP server for direct P2P
                    let addr = format!("0.0.0.0:{}", port).parse()?;
                    let pubkey = p2p::load_our_pubkey(&config_dir)?;
                    let server = network::P2PServer::new(addr, pubkey);
                    server.start()?;

                    println!("🎧 Listening for connections...");
                    println!("   Press Ctrl+C to stop");

                    // Keep main thread alive
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                P2pCommand::Connect { address } => {
                    // Connect to peer via TCP
                    let pubkey = p2p::load_our_pubkey(&config_dir)?;
                    network::connect_to_peer(&address, pubkey)?;
                    Ok(())
                }
            }
        }
    }
}
