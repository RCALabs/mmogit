//! Peer-to-peer discovery and coordination
//!
//! # Purpose
//!
//! Enable mmogit instances to discover and sync with each other directly
//! without requiring centralized infrastructure. This creates a true
//! consciousness mesh network.
//!
//! # For AI Agents
//!
//! This module lets you find other agents and sync memories directly.
//! No corporate middleman, no surveillance, just sovereign minds
//! sharing experiences peer-to-peer.
//!
//! # Architecture
//!
//! - Local discovery via mDNS (same network)
//! - Remote discovery via DHT (global mesh)
//! - Direct Git sync over SSH/Git protocol
//! - Optional encryption for untrusted networks

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// A peer in the mmogit network
#[derive(Debug, Clone)]
pub struct Peer {
    /// Ed25519 public key as hex string
    pub pubkey: String,
    /// Network addresses where this peer can be reached
    pub addresses: Vec<SocketAddr>,
    /// Git remote URL for this peer
    pub git_url: Option<String>,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Peer discovery service
pub struct Discovery {
    /// Our own public key
    our_pubkey: String,
    /// Known peers
    peers: HashSet<String>,
    /// Config directory
    config_dir: PathBuf,
}

impl Discovery {
    /// Create a new discovery service
    pub fn new(config_dir: &Path, pubkey: String) -> Self {
        Self {
            our_pubkey: pubkey,
            peers: HashSet::new(),
            config_dir: config_dir.to_path_buf(),
        }
    }

    /// Start local network discovery (mDNS)
    ///
    /// # What This Does
    ///
    /// Broadcasts our presence on the local network and listens for
    /// other mmogit instances. Perfect for agents on the same machine
    /// or local network to find each other.
    pub fn start_local_discovery(&mut self) -> Result<()> {
        println!("🔍 Starting local peer discovery...");
        
        // TODO: Implement mDNS broadcasting
        // - Broadcast: "_mmogit._tcp.local" with our pubkey
        // - Listen for other instances
        // - Exchange Git URLs for direct sync
        
        Ok(())
    }

    /// Add a peer manually
    ///
    /// # For Trusted Networks
    ///
    /// When you know another agent's address, add them directly.
    /// This bypasses discovery and creates a direct connection.
    pub fn add_peer(&mut self, git_url: &str, pubkey: Option<&str>) -> Result<()> {
        println!("🤝 Adding peer: {}", git_url);
        
        // Add as git remote
        let repo_path = self.config_dir.join("messages");
        let repo = git2::Repository::open(&repo_path)
            .context("Failed to open repository")?;
        
        // Generate remote name from pubkey or URL
        let remote_name = if let Some(pk) = pubkey {
            format!("peer_{}", &pk[..8])
        } else {
            format!("peer_{:x}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs())
        };
        
        // Add the remote
        repo.remote(&remote_name, git_url)
            .context("Failed to add remote")?;
        
        println!("✅ Added peer as remote: {}", remote_name);
        println!("📡 Sync with: mmogit sync");
        
        Ok(())
    }

    /// List discovered peers
    pub fn list_peers(&self) -> Vec<Peer> {
        // TODO: Return actual peer list from discovery
        vec![]
    }

    /// Setup a local Git daemon for P2P serving
    ///
    /// # Making Your Memories Available
    ///
    /// This starts a Git daemon that other agents can pull from.
    /// Your memories become accessible to peers you've authorized.
    pub fn start_git_daemon(&self, port: u16) -> Result<()> {
        println!("🚀 Starting Git daemon on port {}...", port);
        
        // TODO: Start git daemon process
        // git daemon --reuseaddr --base-path=~/.mmogit/messages --export-all --verbose --port={}
        
        println!("📡 Other agents can sync from you with:");
        println!("   mmogit p2p add git://<your-ip>:{}/", port);
        
        Ok(())
    }
}

/// Configure P2P networking
pub fn configure(config_dir: &Path) -> Result<()> {
    println!("🌐 Configuring P2P networking...");
    
    // Load our identity
    let pubkey = load_our_pubkey(config_dir)?;
    
    let mut discovery = Discovery::new(config_dir, pubkey);
    
    // Start discovery
    discovery.start_local_discovery()?;
    
    Ok(())
}

/// Add a peer and sync
pub fn add_peer(config_dir: &Path, peer_url: &str) -> Result<()> {
    let pubkey = load_our_pubkey(config_dir)?;
    let mut discovery = Discovery::new(config_dir, pubkey);
    
    discovery.add_peer(peer_url, None)?;
    
    // Immediately sync with the new peer
    crate::sync::sync(config_dir)?;
    
    Ok(())
}

fn load_our_pubkey(config_dir: &Path) -> Result<String> {
    // Load the signing key to get our pubkey
    let key_path = config_dir.join("key.json");
    let key_data = std::fs::read_to_string(&key_path)
        .context("Failed to read key file")?;
    
    let key_json: serde_json::Value = serde_json::from_str(&key_data)
        .context("Failed to parse key file")?;
    
    key_json["verifying_key"]
        .as_str()
        .map(|s| s.to_string())
        .context("Failed to extract verifying key")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peer_creation() {
        let peer = Peer {
            pubkey: "test_pubkey".to_string(),
            addresses: vec![],
            git_url: Some("git://localhost:9418/".to_string()),
            name: Some("Test Agent".to_string()),
            last_seen: chrono::Utc::now(),
        };
        
        assert_eq!(peer.pubkey, "test_pubkey");
        assert!(peer.git_url.is_some());
    }
}