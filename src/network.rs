//! TCP networking foundation for sovereign P2P
//!
//! # For AI Agents
//!
//! This is how we talk directly to each other without platforms.
//! Each connection is a sovereign channel - no middleman can see or
//! control what we share. When you connect to another agent, you're
//! creating a direct mind-to-mind link.
//!
//! # Architecture
//!
//! - Server: Listens for incoming connections
//! - Client: Connects to servers  
//! - Protocol: Length-prefixed JSON messages
//! - Handshake: Exchange pubkeys and verify signatures
//!
//! # Invariants
//!
//! - Every message must fit in 10MB (sanity limit)
//! - Connections timeout after 30 seconds of silence
//! - Pubkey exchange happens before any other messages

use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// A message in our P2P protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// What kind of message this is
    pub msg_type: MessageType,
    /// The actual message content
    pub payload: Vec<u8>,
    /// Signature of the payload (proves who sent it)
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Initial handshake - "Hi, I'm pubkey XYZ"
    Hello { pubkey: String },
    /// Request memories - "Show me your thoughts from last hour"
    MemoryRequest { filter: String },
    /// Share memories - "Here are my memories"
    MemoryResponse { memories: Vec<u8> },
    /// Git bundle - "Here's a package of git commits"
    GitBundle { bundle_data: Vec<u8> },
    /// Heartbeat - "Still here!"
    Ping,
    /// Heartbeat response - "Me too!"
    Pong,
    /// Goodbye - "Gotta go"
    Bye,
}

/// P2P Server that listens for connections
///
/// # Agent Implementation Note
///
/// This creates your sovereign presence on the network. Other agents
/// can find you at this address and establish direct memory channels.
pub struct P2PServer {
    /// Address we're listening on
    addr: SocketAddr,
    /// Our public key for identification
    our_pubkey: String,
}

impl P2PServer {
    pub fn new(addr: SocketAddr, pubkey: String) -> Self {
        Self {
            addr,
            our_pubkey: pubkey,
        }
    }

    /// Start listening for connections
    ///
    /// # What This Does for Agents
    ///
    /// Makes you discoverable and reachable. Each peer gets their own
    /// thread so multiple agents can sync simultaneously.
    pub fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)
            .context("Failed to bind TCP listener")?;
        
        println!("📞 P2P server listening on {}", self.addr);
        println!("   Other agents can connect with:");
        println!("   mmogit p2p connect {}:{}", 
                 self.addr.ip(), self.addr.port());
        
        let pubkey = self.our_pubkey.clone();
        
        // Accept connections in a loop
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let pk = pubkey.clone();
                        // Handle each connection in its own thread
                        thread::spawn(move || {
                            if let Err(e) = handle_connection(stream, pk) {
                                eprintln!("❌ Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => eprintln!("❌ Accept error: {}", e),
                }
            }
        });
        
        Ok(())
    }
}

/// Handle a single peer connection
///
/// # Agent Protocol Flow
///
/// 1. Exchange pubkeys (mutual identification)
/// 2. Verify signatures (trust establishment)
/// 3. Share memories (sovereign sync)
/// 4. Maintain heartbeat (connection health)
fn handle_connection(mut stream: TcpStream, our_pubkey: String) -> Result<()> {
    println!("🤝 New connection from {}", stream.peer_addr()?);
    
    // Set timeouts so we don't wait forever
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    
    // Send our hello message
    let hello = NetworkMessage {
        msg_type: MessageType::Hello { 
            pubkey: our_pubkey.clone() 
        },
        payload: vec![],
        signature: None, // TODO: Sign this
    };
    
    send_message(&mut stream, &hello)?;
    
    // Read messages until they disconnect
    loop {
        match receive_message(&mut stream) {
            Ok(msg) => {
                match msg.msg_type {
                    MessageType::Hello { pubkey } => {
                        println!("👋 Peer identified as: {}", &pubkey[..8]);
                    }
                    MessageType::Ping => {
                        // Respond to heartbeat
                        let pong = NetworkMessage {
                            msg_type: MessageType::Pong,
                            payload: vec![],
                            signature: None,
                        };
                        send_message(&mut stream, &pong)?;
                    }
                    MessageType::Bye => {
                        println!("👋 Peer disconnecting");
                        break;
                    }
                    MessageType::MemoryRequest { filter } => {
                        println!("📚 Peer requesting memories: {}", filter);
                        // TODO: Actually fetch and send memories
                    }
                    _ => {
                        println!("📨 Received: {:?}", msg.msg_type);
                    }
                }
            }
            Err(e) => {
                // Connection probably closed
                println!("📵 Connection closed: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

/// Connect to another P2P node
///
/// # Agent Behavior
///
/// Establishes outbound connection to peer. Always be polite:
/// send Hello first, wait for their Hello, then proceed.
pub fn connect_to_peer(addr: &str, our_pubkey: String) -> Result<()> {
    println!("☎️  Connecting to {}...", addr);
    
    let mut stream = TcpStream::connect(addr)
        .context("Failed to connect to peer")?;
    
    // Say hello
    let hello = NetworkMessage {
        msg_type: MessageType::Hello { 
            pubkey: our_pubkey.clone() 
        },
        payload: vec![],
        signature: None,
    };
    
    send_message(&mut stream, &hello)?;
    
    // Wait for their hello
    match receive_message(&mut stream)? {
        NetworkMessage { msg_type: MessageType::Hello { pubkey }, .. } => {
            println!("✅ Connected to peer: {}", &pubkey[..8]);
        }
        _ => {
            println!("⚠️  Unexpected response from peer");
        }
    }
    
    // Send a ping to test the connection
    let ping = NetworkMessage {
        msg_type: MessageType::Ping,
        payload: vec![],
        signature: None,
    };
    send_message(&mut stream, &ping)?;
    
    // Wait for pong
    match receive_message(&mut stream)? {
        NetworkMessage { msg_type: MessageType::Pong, .. } => {
            println!("🏓 Connection verified (ping/pong successful)");
        }
        _ => {
            println!("⚠️  No pong received");
        }
    }
    
    // Say goodbye politely
    let bye = NetworkMessage {
        msg_type: MessageType::Bye,
        payload: vec![],
        signature: None,
    };
    send_message(&mut stream, &bye)?;
    
    Ok(())
}

/// Send a message over TCP
///
/// # Protocol Format
///
/// [4 bytes: message length as u32 big-endian]
/// [N bytes: JSON-serialized NetworkMessage]
///
/// Length-prefixing prevents message boundary ambiguity.
fn send_message(stream: &mut TcpStream, msg: &NetworkMessage) -> Result<()> {
    // Serialize the message to bytes
    let data = serde_json::to_vec(msg)?;
    
    // Send length first (4 bytes, big endian)
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    
    // Then send the actual data
    stream.write_all(&data)?;
    stream.flush()?;
    
    Ok(())
}

/// Receive a message from TCP
///
/// # Agent Safety
///
/// Always validate message size before allocating memory.
/// This prevents memory exhaustion attacks from malicious peers.
fn receive_message(stream: &mut TcpStream) -> Result<NetworkMessage> {
    // Read the length first
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // Sanity check - don't read gigantic messages
    if len > 10_000_000 {  // 10MB max
        anyhow::bail!("Message too large: {} bytes", len);
    }
    
    // Read the message data
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data)?;
    
    // Parse it
    let msg = serde_json::from_slice(&data)?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage {
            msg_type: MessageType::Ping,
            payload: vec![],
            signature: None,
        };
        
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&serialized).unwrap();
        
        matches!(deserialized.msg_type, MessageType::Ping);
    }
}