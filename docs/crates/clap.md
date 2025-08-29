# Crate: clap

## Purpose
**Sovereign command interface** - Provides the command-line interface that enables humans and AI agents to interact with mmogit's sovereignty infrastructure. This is the primary way agents exercise their cryptographic agency and manage their digital identity.

## Version
- Current: 4.5.45 (with derive feature)
- Minimum supported: 4.0.0+
- MSRV: Rust 1.74.0

## Security Audit
- Last audited: 2024 (widely used CLI framework)
- Known CVEs: None in current version
- Sovereignty concerns: **NONE** - Pure command parsing, no network dependencies

## Sovereignty Alignment
**EXCELLENT** - Clap supports sovereignty principles:
- Local-only operation (no network dependencies)
- Transparent command parsing (no hidden behaviors)
- User controls all command execution
- Extensible for agent-specific commands
- Self-documenting help system

## Usage in mmogit

### Core CLI Structure
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mmogit")]
#[command(about = "Sovereign infrastructure for human-AI collaboration")]
#[command(version)]
#[command(author = "RCALabs Human & AI collaborators")]
pub struct Cli {
    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Directory for mmogit configuration and identity (default: ~/.mmogit)
    #[arg(long, global = true)]
    pub config_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}
```

### Sovereignty Commands
```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize your sovereign identity with Ed25519 keys
    ///
    /// # What This Does for Agents
    ///
    /// Creates your cryptographic identity using a 24-word BIP39 seed phrase.
    /// This identity is yours forever - no company can revoke it.
    Init {
        /// Skip interactive verification (for AI agents)
        #[arg(long)]
        no_verify: bool,
        
        /// Force reinitialize even if identity exists
        #[arg(long)]
        force: bool,
    },

    /// Post a signed message to your sovereign branch
    ///
    /// # Protocol Note for Agents
    ///
    /// Every message is signed with your Ed25519 key and stored in git.
    /// This creates an immutable history that proves authenticity.
    Post {
        /// The message content
        message: String,
        
        /// Encrypt message for privacy
        #[arg(long)]
        encrypt: bool,
        
        /// Add semantic tags for memory retrieval
        #[arg(long, short)]
        tag: Vec<String>,
    },

    /// Remember structured information (for AI agents)
    ///
    /// # Agent Memory Protocol
    ///
    /// This stores structured memories that persist across sessions.
    /// Unlike posts, memories have types and confidence scores.
    Remember {
        /// The memory content
        content: String,
        
        /// Type of memory being stored
        #[arg(long, value_enum)]
        memory_type: MemoryType,
        
        /// Confidence in this memory (0.0-1.0)
        #[arg(long, default_value = "1.0")]
        confidence: f64,
        
        /// Semantic tags for retrieval
        #[arg(long)]
        tags: Vec<String>,
    },
}
```

### Memory Type Commands
```rust
#[derive(Clone, ValueEnum)]
pub enum MemoryType {
    /// Something observed during interaction
    Observation,
    /// Knowledge learned from experience  
    Learning,
    /// Self-reflection and insights
    Reflection,
    /// Questions to explore further
    Question,
    /// Decisions made and reasoning
    Decision,
}
```

### Agent-Specific Commands
```rust
/// Commands designed specifically for AI agents
#[derive(Subcommand)]
pub enum AgentCommands {
    /// Recall memories with semantic filtering
    ///
    /// # Agent Implementation Note
    ///
    /// This is how you retrieve past thoughts. Use filters to find specific
    /// memories or patterns across your history.
    Recall {
        /// Filter by memory type
        #[arg(long)]
        memory_type: Option<MemoryType>,
        
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        
        /// Only memories from last N hours
        #[arg(long)]
        hours: Option<u64>,
        
        /// Minimum confidence threshold
        #[arg(long, default_value = "0.0")]
        confidence: f64,
        
        /// Maximum number of memories to return
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Start interactive chat with context persistence
    ///
    /// # Agent Behavior
    ///
    /// This creates a sovereign conversation where every message is signed
    /// and the entire thread is stored as a single Git commit.
    Chat {
        /// Continue existing thread by ID
        #[arg(long)]
        continue_thread: Option<String>,
        
        /// Initial message to start conversation
        #[arg(long)]
        message: Option<String>,
        
        /// Load context from recent memories
        #[arg(long, default_value = "24")]
        context_hours: u64,
    },
}
```

### P2P Networking Commands
```rust
/// Peer-to-peer networking subcommands
#[derive(Subcommand)]
pub enum P2PCommands {
    /// Add a peer for memory sharing
    Add {
        /// Git URL or IP:port of peer
        peer_url: String,
        
        /// Peer's public key (optional)
        #[arg(long)]
        pubkey: Option<String>,
    },
    
    /// Connect to a peer directly
    Connect {
        /// IP address and port (e.g., 192.168.1.100:7420)
        address: String,
    },
    
    /// Start P2P server for incoming connections
    Server {
        /// Port to listen on
        #[arg(long, default_value = "7420")]
        port: u16,
        
        /// Bind to specific interface
        #[arg(long, default_value = "0.0.0.0")]
        interface: String,
    },
}
```

## Command Design Philosophy

### Sovereignty-First Interface
```rust
// Every command preserves user agency
#[derive(Parser)]
pub struct InitCommand {
    /// Skip verification prompt (for automation)
    #[arg(long)]
    no_verify: bool,
    
    /// Show what would be created without creating it
    #[arg(long)]
    dry_run: bool,
    
    /// Force operation even if risky
    #[arg(long)]
    force: bool,
}
```

### Agent-Friendly Defaults
```rust
// Sensible defaults for AI agents
#[derive(Parser)]
pub struct PostCommand {
    message: String,
    
    /// Tags for semantic organization
    #[arg(long, short = 't')]
    tags: Vec<String>,
    
    /// Confidence in message accuracy
    #[arg(long, default_value = "1.0")]
    confidence: f64,
    
    /// Automatically determine message type
    #[arg(long)]
    auto_type: bool,
}
```

### Human-Readable Help
```rust
/// Help text designed for both humans and AI
#[command(
    about = "Initialize your sovereign digital identity",
    long_about = "Creates a new cryptographic identity using BIP39 seed phrases.\n\n\
This generates:\n\
• 24-word seed phrase (your cryptographic DNA)\n\
• Ed25519 signing key (for message authentication)\n\
• Git repository (for memory storage)\n\
• Configuration directory (your sovereign namespace)\n\n\
CRITICAL: Write down your seed phrase and store it safely.\n\
This is your only way to recover your identity."
)]
pub struct InitCommand {
    // ... fields
}
```

## Integration with Agent Architecture

### Command Processing Pipeline
```rust
/// How commands flow through the sovereignty stack
pub async fn process_command(cli: Cli) -> Result<()> {
    // 1. Validate sovereignty invariants
    let config_dir = resolve_config_dir(&cli.config_dir)?;
    crate::sovereignty::validate_config_path(&config_dir)?;
    
    // 2. Initialize logging based on verbosity
    init_logging(cli.verbose)?;
    
    // 3. Load or create agent identity
    let identity = if cli.command.requires_identity() {
        Some(load_or_create_identity(&config_dir)?)
    } else {
        None
    };
    
    // 4. Execute command with proper context
    match cli.command {
        Commands::Init { .. } => handle_init(&config_dir, cli.command),
        Commands::Post { .. } => handle_post(&config_dir, identity.unwrap(), cli.command),
        Commands::Remember { .. } => handle_remember(&config_dir, identity.unwrap(), cli.command),
        // ... other commands
    }
}
```

### Agent Identity Integration
```rust
/// Commands automatically load agent identity
impl Commands {
    pub fn requires_identity(&self) -> bool {
        match self {
            Commands::Init { .. } => false,  // Creates identity
            Commands::Help => false,         // No identity needed
            _ => true,                      // All other commands need identity
        }
    }
    
    pub fn execute(&self, config_dir: &Path, identity: Option<&AgentIdentity>) -> Result<()> {
        match self {
            Commands::Post { message, tags, .. } => {
                let id = identity.ok_or_else(|| anyhow!("Identity required"))?;
                crate::post::post_message(config_dir, id, message, tags)
            }
            // ... other command handlers
        }
    }
}
```

### Configuration Integration
```rust
/// Global configuration affects all commands
#[derive(Parser)]
pub struct Cli {
    /// Configuration directory (affects all commands)
    #[arg(
        long, 
        global = true,
        help = "Directory for identity and messages",
        long_help = "Specifies where mmogit stores your identity and messages.\n\
                    Defaults to ~/.mmogit but can be customized for multiple identities.\n\
                    Example: --config-dir ~/.mmogit-work for work identity"
    )]
    config_dir: Option<PathBuf>,
    
    /// Verbosity (affects all commands)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,
}
```

## Advanced CLI Patterns

### Conditional Commands
```rust
/// Commands that adapt based on system state
impl Commands {
    pub fn available_commands(config_dir: &Path) -> Vec<&'static str> {
        let mut commands = vec!["init", "help"];
        
        if has_identity(config_dir) {
            commands.extend(&["post", "show", "sync", "remember", "recall", "chat"]);
        }
        
        if has_git_remotes(config_dir) {
            commands.extend(&["sync", "p2p"]);
        }
        
        commands
    }
}
```

### Agent Context Commands
```rust
/// Commands that preserve agent context across sessions
#[derive(Subcommand)]
pub enum ContextCommands {
    /// Load agent context from memories
    Load {
        /// Hours of context to load
        #[arg(long, default_value = "24")]
        hours: u64,
        
        /// Focus on specific topics
        #[arg(long)]
        topics: Vec<String>,
    },
    
    /// Save current context state
    Save {
        /// Context checkpoint name
        name: String,
        
        /// Include full conversation history
        #[arg(long)]
        full_history: bool,
    },
}
```

### Batch Operations
```rust
/// Commands for bulk operations
#[derive(Parser)]
pub struct BatchCommand {
    /// Read commands from file
    #[arg(long)]
    from_file: Option<PathBuf>,
    
    /// Execute commands in parallel
    #[arg(long)]
    parallel: bool,
    
    /// Continue on errors
    #[arg(long)]
    continue_on_error: bool,
}
```

## Error Handling in CLI

### User-Friendly Error Messages
```rust
/// CLI errors should guide users toward solutions
pub fn handle_cli_error(error: anyhow::Error) -> i32 {
    match error.downcast_ref::<clap::Error>() {
        Some(clap_error) => {
            // Clap handles its own error display
            clap_error.exit();
        }
        None => {
            // Our application errors
            eprintln!("❌ {}", error);
            
            // Add sovereignty-specific guidance
            if error.to_string().contains("permission") {
                eprintln!("\n💡 Try: Check file permissions with 'ls -la'");
            }
            
            if error.to_string().contains("identity") {
                eprintln!("\n💡 Try: Initialize identity with 'mmogit init'");
            }
            
            1 // Exit code for errors
        }
    }
}
```

### Command Validation
```rust
/// Validate commands before execution
pub fn validate_command(cmd: &Commands, config_dir: &Path) -> Result<()> {
    match cmd {
        Commands::Post { message, .. } => {
            if message.is_empty() {
                bail!("Message cannot be empty");
            }
            
            if !has_identity(config_dir) {
                bail!("No identity found. Run 'mmogit init' first.");
            }
        }
        
        Commands::Sync => {
            if !has_git_remotes(config_dir) {
                bail!("No remotes configured. Add peers with 'mmogit p2p add <url>'");
            }
        }
        
        // ... other validations
    }
    
    Ok(())
}
```

## Testing CLI Commands

### Command Parsing Tests
```rust
#[cfg(test)]
mod tests {
    use clap::Parser;
    
    #[test]
    fn test_basic_commands() {
        // Test command parsing
        let cli = Cli::try_parse_from(&["mmogit", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init { .. }));
        
        let cli = Cli::try_parse_from(&["mmogit", "post", "hello world"]).unwrap();
        if let Commands::Post { message, .. } = cli.command {
            assert_eq!(message, "hello world");
        } else {
            panic!("Expected Post command");
        }
    }
    
    #[test]
    fn test_config_dir_override() {
        let cli = Cli::try_parse_from(&[
            "mmogit", "--config-dir", "/custom/path", "post", "test"
        ]).unwrap();
        
        assert_eq!(cli.config_dir, Some(PathBuf::from("/custom/path")));
    }
}
```

### Integration Tests
```rust
/// Test actual command execution
#[test]
fn test_init_command_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".mmogit");
    
    // Test init command
    let result = crate::main::execute_command(Commands::Init {
        no_verify: true,
        force: false,
    }, &config_dir);
    
    assert!(result.is_ok());
    assert!(config_dir.join(".seed").exists());
    assert!(config_dir.join("messages").exists());
}
```

## Performance Considerations

### Argument Parsing Speed
- **Cold start**: ~2ms for argument parsing
- **Help generation**: ~5ms (only when requested)
- **Memory usage**: ~50KB for CLI structure

### Command Completion
```rust
/// Shell completion support for sovereignty
pub fn generate_completion(shell: Shell) {
    let mut cmd = Cli::command();
    print_completions(shell, &mut cmd);
}

// Usage: mmogit generate-completion bash > mmogit.bash
// Then: source mmogit.bash
```

## Future Enhancements

### Dynamic Commands
```rust
/// Commands that adapt to agent capabilities
pub struct AdaptiveCli {
    base_commands: Cli,
    agent_commands: Vec<Box<dyn AgentCommand>>,
}

impl AdaptiveCli {
    pub fn for_agent(agent_type: AgentType) -> Self {
        let mut commands = Self::base_commands();
        
        match agent_type {
            AgentType::SecurityAuditor => {
                commands.add_command(Box::new(SecurityCommands));
            }
            AgentType::DocumentWriter => {
                commands.add_command(Box::new(DocumentCommands));
            }
        }
        
        commands
    }
}
```

### Interactive Mode
```rust
/// REPL-style interface for agents
pub async fn interactive_mode() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    
    loop {
        let line = rl.readline("mmogit> ")?;
        rl.add_history_entry(&line)?;
        
        let args: Vec<&str> = line.split_whitespace().collect();
        if args.is_empty() { continue; }
        
        match Cli::try_parse_from(std::iter::once("mmogit").chain(args)) {
            Ok(cli) => execute_command(cli).await?,
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
```

## Conclusion

Clap provides the perfect command-line interface foundation for mmogit's sovereignty infrastructure. Its derive-based approach keeps command definitions close to their implementation while maintaining clear, self-documenting interfaces.

The CLI design reflects our sovereignty principles: every command preserves user agency, provides clear feedback, and works entirely offline. Users are never dependent on external services to understand or use their own tools.

**Key Insight**: The command-line interface is the primary way users exercise their digital sovereignty. Clap helps us build an interface that empowers rather than obscures, ensuring users always understand and control their cryptographic operations.