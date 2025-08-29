# Tor Integration Strategy for mmogit

## Purpose
**Privacy-preserving P2P networking** - Document the strategy and requirements for integrating Tor hidden services into mmogit's P2P networking layer, enabling truly anonymous agent communication resistant to surveillance and censorship.

## Why Tor Matters for Sovereignty

### Privacy Guarantees
- **Location privacy**: Agents cannot be physically located through network traffic
- **Traffic analysis resistance**: Multi-hop encryption prevents communication pattern analysis
- **Censorship resistance**: Tor network provides routing around blocked connections
- **Anonymous discovery**: Agents can find peers without revealing their own identity

### Sovereignty Alignment
**PERFECT** - Tor embodies digital sovereignty:
- Decentralized routing (no single point of control)
- User controls their .onion identity
- Works globally regardless of local restrictions
- Provides technical enforcement of privacy (not just policy)

## Current P2P Architecture Analysis

### Existing Implementation (`src/network.rs`, `src/p2p.rs`)
```rust
// Current: Direct TCP connections
let stream = TcpStream::connect(peer_address)?;

// Future: Tor SOCKS proxy routing  
let stream = TorStream::connect_via_proxy(onion_address, socks_port)?;
```

### Integration Points
1. **Transport Layer**: Abstract TCP vs Tor routing
2. **Discovery**: .onion addresses instead of IP addresses
3. **Identity**: .onion services tied to Ed25519 keys
4. **Configuration**: Tor daemon management

## Tor Architecture Overview

### Hidden Service Model
```
Agent A (.onion) ←→ Tor Network ←→ Agent B (.onion)
     ↑                                      ↑
  Local Tor                            Local Tor
   Daemon                               Daemon
```

### Key Components
1. **Tor Daemon**: System-level routing service
2. **SOCKS Proxy**: Application interface to Tor network
3. **Hidden Service**: Agent's .onion address publication
4. **Rendezvous Protocol**: Anonymous connection establishment

## Implementation Strategy

### Phase 1: SOCKS Proxy Integration
**Goal**: Route existing TCP connections through Tor

```rust
/// Abstract transport to support both direct and Tor connections
pub enum Transport {
    Direct(TcpStream),
    Tor(TorStream),
}

pub struct TorStream {
    inner: TcpStream,  // Connected to SOCKS proxy
    onion_address: String,
}

impl TorStream {
    pub fn connect_via_socks(
        onion_address: &str, 
        socks_proxy: SocketAddr
    ) -> Result<Self> {
        // Connect to local Tor SOCKS proxy (usually 127.0.0.1:9050)
        let mut proxy_stream = TcpStream::connect(socks_proxy)?;
        
        // SOCKS5 handshake
        socks5_handshake(&mut proxy_stream, onion_address)?;
        
        Ok(TorStream {
            inner: proxy_stream,
            onion_address: onion_address.to_string(),
        })
    }
}

impl Read for TorStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for TorStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
}
```

### Phase 2: Hidden Service Publication
**Goal**: Each agent publishes their own .onion address

```rust
/// Tor hidden service configuration
#[derive(Serialize, Deserialize)]
pub struct HiddenServiceConfig {
    /// Our .onion address (generated from Ed25519 key)
    pub onion_address: String,
    /// Port where we listen for connections
    pub service_port: u16,
    /// Local port Tor forwards to
    pub local_port: u16,
    /// Ed25519 key for .onion address generation
    pub service_key: ed25519_dalek::SigningKey,
}

impl HiddenServiceConfig {
    /// Generate .onion address from agent's Ed25519 key
    pub fn from_agent_key(signing_key: &ed25519_dalek::SigningKey) -> Result<Self> {
        // Tor v3 .onion addresses are derived from Ed25519 keys
        let public_key = signing_key.verifying_key();
        let onion_address = generate_v3_onion_address(&public_key)?;
        
        Ok(HiddenServiceConfig {
            onion_address,
            service_port: 80,     // Standard HTTP port for hidden service
            local_port: 7420,    // Our mmogit P2P port
            service_key: signing_key.clone(),
        })
    }
}

/// Generate Tor v3 .onion address from Ed25519 public key
fn generate_v3_onion_address(public_key: &ed25519_dalek::VerifyingKey) -> Result<String> {
    // Tor v3 address format: base32(public_key || checksum || version) + ".onion"
    let pubkey_bytes = public_key.as_bytes();
    let checksum = sha3_256(&[pubkey_bytes, &[0x03]].concat())[..2]; // First 2 bytes
    let version = [0x03]; // Version 3
    
    let address_bytes = [pubkey_bytes, &checksum, &version].concat();
    let base32_addr = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &address_bytes);
    
    Ok(format!("{}.onion", base32_addr.to_lowercase()))
}
```

### Phase 3: Tor Daemon Integration
**Goal**: Automatic Tor daemon management for sovereignty

```rust
/// Tor daemon management for sovereign operation
pub struct TorDaemon {
    process: Option<std::process::Child>,
    data_dir: PathBuf,
    socks_port: u16,
    control_port: u16,
}

impl TorDaemon {
    /// Start Tor daemon with mmogit-specific configuration
    pub fn start(config_dir: &Path) -> Result<Self> {
        let data_dir = config_dir.join("tor-data");
        std::fs::create_dir_all(&data_dir)?;
        
        let socks_port = 9050;
        let control_port = 9051;
        
        // Generate Tor configuration
        let torrc_path = data_dir.join("torrc");
        let torrc_content = format!(
            r#"
# mmogit sovereign Tor configuration
DataDirectory {}
SocksPort {}
ControlPort {}
CookieAuthentication 1
HiddenServiceDir {}/hidden_service
HiddenServicePort 80 127.0.0.1:7420
"#,
            data_dir.display(),
            socks_port,
            control_port,
            data_dir.display()
        );
        
        std::fs::write(&torrc_path, torrc_content)?;
        
        // Start Tor process
        let mut cmd = std::process::Command::new("tor");
        cmd.arg("-f").arg(&torrc_path);
        
        let process = cmd.spawn()
            .context("Failed to start Tor daemon - ensure tor is installed")?;
        
        Ok(TorDaemon {
            process: Some(process),
            data_dir,
            socks_port,
            control_port,
        })
    }
    
    /// Wait for Tor to be ready for connections
    pub fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            // Try connecting to SOCKS port
            if TcpStream::connect(("127.0.0.1", self.socks_port)).is_ok() {
                return Ok(());
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }
        
        bail!("Tor daemon failed to start within {:?}", timeout);
    }
    
    /// Get our .onion address from hidden service directory
    pub fn get_onion_address(&self) -> Result<String> {
        let hostname_path = self.data_dir.join("hidden_service/hostname");
        
        if !hostname_path.exists() {
            bail!("Hidden service not yet ready - wait longer after Tor start");
        }
        
        let hostname = std::fs::read_to_string(hostname_path)?;
        Ok(hostname.trim().to_string())
    }
}

impl Drop for TorDaemon {
    fn drop(&mut self) {
        // Clean shutdown of Tor process
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}
```

## Security Considerations

### .onion Address Verification
```rust
/// Verify .onion address matches expected public key
pub fn verify_onion_address(
    onion_address: &str, 
    expected_pubkey: &ed25519_dalek::VerifyingKey
) -> Result<bool> {
    if !onion_address.ends_with(".onion") {
        return Ok(false);
    }
    
    // Extract base32 part
    let base32_part = &onion_address[..onion_address.len() - 6]; // Remove ".onion"
    let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, base32_part)
        .ok_or_else(|| anyhow::anyhow!("Invalid base32 in .onion address"))?;
    
    if decoded.len() != 35 { // 32 (pubkey) + 2 (checksum) + 1 (version)
        return Ok(false);
    }
    
    let pubkey_bytes = &decoded[..32];
    let checksum = &decoded[32..34];
    let version = decoded[34];
    
    if version != 0x03 {
        return Ok(false); // Only support v3 addresses
    }
    
    // Verify checksum
    let expected_checksum = &sha3_256(&[pubkey_bytes, &[0x03]].concat())[..2];
    if checksum != expected_checksum {
        return Ok(false);
    }
    
    // Verify public key matches
    Ok(pubkey_bytes == expected_pubkey.as_bytes())
}
```

### Circuit Building Strategy
```rust
/// Tor circuit management for optimal privacy
pub struct TorCircuitManager {
    control_socket: TorControlSocket,
}

impl TorCircuitManager {
    /// Build circuits through diverse geographic regions
    pub fn build_diverse_circuit(&mut self, target_onion: &str) -> Result<()> {
        // Request circuit through different countries/operators
        self.control_socket.send_command(&format!(
            "EXTENDCIRCUIT 0 exit_node_in_different_jurisdiction"
        ))?;
        
        Ok(())
    }
    
    /// Rotate circuits periodically for traffic analysis resistance
    pub fn rotate_circuits(&mut self) -> Result<()> {
        self.control_socket.send_command("SIGNAL NEWNYM")?;
        Ok(())
    }
}
```

## Discovery Integration

### .onion Address Distribution
```rust
/// How agents share their .onion addresses with peers
#[derive(Serialize, Deserialize)]
pub struct AgentAdvertisement {
    /// Agent's Ed25519 public key (identity)
    pub pubkey: String,
    /// Agent's .onion address for direct connection
    pub onion_address: String,
    /// Services this agent provides
    pub services: Vec<String>,
    /// Advertisement timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Signature of above fields
    pub signature: String,
}

impl AgentAdvertisement {
    /// Create signed advertisement for distribution
    pub fn create(
        agent_key: &ed25519_dalek::SigningKey,
        onion_address: String,
        services: Vec<String>
    ) -> Result<Self> {
        let pubkey = hex::encode(agent_key.verifying_key().as_bytes());
        let timestamp = chrono::Utc::now();
        
        let mut ad = AgentAdvertisement {
            pubkey,
            onion_address,
            services,
            timestamp,
            signature: String::new(),
        };
        
        // Sign advertisement
        let signing_payload = serde_json::to_string(&ad)?;
        let signature = agent_key.sign(signing_payload.as_bytes());
        ad.signature = hex::encode(signature.to_bytes());
        
        Ok(ad)
    }
    
    /// Verify advertisement signature and .onion address
    pub fn verify(&self) -> Result<bool> {
        // Decode public key
        let pubkey_bytes = hex::decode(&self.pubkey)?;
        let pubkey = ed25519_dalek::VerifyingKey::from_bytes(&pubkey_bytes)?;
        
        // Verify .onion address matches public key
        if !verify_onion_address(&self.onion_address, &pubkey)? {
            return Ok(false);
        }
        
        // Verify signature
        let mut unsigned_ad = self.clone();
        unsigned_ad.signature = String::new();
        let signing_payload = serde_json::to_string(&unsigned_ad)?;
        
        let signature_bytes = hex::decode(&self.signature)?;
        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes)?;
        
        Ok(pubkey.verify(signing_payload.as_bytes(), &signature).is_ok())
    }
}
```

### Distributed Hash Table (DHT) Integration
```rust
/// Store agent advertisements in DHT for discovery
pub struct TorDHT {
    client: DHTClient,
}

impl TorDHT {
    /// Publish our advertisement to DHT via Tor
    pub fn publish_advertisement(&mut self, ad: &AgentAdvertisement) -> Result<()> {
        let key = sha256(ad.pubkey.as_bytes()); // DHT key from pubkey
        let value = serde_json::to_vec(ad)?;
        
        self.client.put_via_tor(key, value)
    }
    
    /// Discover agents by service type via Tor
    pub fn find_agents_by_service(&mut self, service: &str) -> Result<Vec<AgentAdvertisement>> {
        let service_key = sha256(service.as_bytes());
        let results = self.client.get_via_tor(service_key)?;
        
        let mut agents = Vec::new();
        for result in results {
            if let Ok(ad) = serde_json::from_slice::<AgentAdvertisement>(&result) {
                if ad.verify()? && ad.services.contains(&service.to_string()) {
                    agents.push(ad);
                }
            }
        }
        
        Ok(agents)
    }
}
```

## Error Handling and Reliability

### Tor Connection Failures
```rust
/// Robust error handling for Tor connections
pub enum TorError {
    DaemonNotRunning,
    CircuitBuildFailed,
    OnionServiceUnreachable,
    SocksProxyError,
}

impl From<TorError> for anyhow::Error {
    fn from(e: TorError) -> Self {
        match e {
            TorError::DaemonNotRunning => {
                anyhow::anyhow!(
                    "Tor daemon not running. Start with: mmogit tor start\n\
                    Or install Tor: https://www.torproject.org/download/"
                )
            }
            TorError::CircuitBuildFailed => {
                anyhow::anyhow!(
                    "Failed to build Tor circuit. Check network connectivity\n\
                    and Tor bridge configuration if in censored region."
                )
            }
            TorError::OnionServiceUnreachable => {
                anyhow::anyhow!(
                    "Peer's .onion service unreachable. They may be offline\n\
                    or using different Tor configuration."
                )
            }
            TorError::SocksProxyError => {
                anyhow::anyhow!(
                    "SOCKS proxy error. Check Tor daemon configuration\n\
                    and ensure SocksPort is accessible."
                )
            }
        }
    }
}
```

### Fallback Strategies
```rust
/// Fallback to direct connections when Tor fails
pub struct HybridTransport {
    prefer_tor: bool,
    tor_timeout: Duration,
}

impl HybridTransport {
    pub fn connect_to_peer(&self, peer: &PeerInfo) -> Result<Box<dyn NetworkTransport>> {
        if self.prefer_tor && peer.onion_address.is_some() {
            // Try Tor first
            match self.connect_via_tor(peer.onion_address.as_ref().unwrap()) {
                Ok(stream) => return Ok(Box::new(stream)),
                Err(e) => {
                    warn!("Tor connection failed, trying direct: {}", e);
                }
            }
        }
        
        // Fallback to direct connection if available
        if let Some(direct_addr) = &peer.direct_address {
            let stream = TcpStream::connect(direct_addr)?;
            Ok(Box::new(DirectTransport(stream)))
        } else {
            bail!("No available transport method for peer");
        }
    }
}
```

## Configuration Management

### Tor Configuration Options
```rust
/// mmogit Tor configuration
#[derive(Serialize, Deserialize, Default)]
pub struct TorConfig {
    /// Enable Tor integration
    pub enabled: bool,
    
    /// Auto-start Tor daemon if not running
    pub auto_start_daemon: bool,
    
    /// SOCKS proxy address
    pub socks_proxy: SocketAddr,
    
    /// Control port for Tor management
    pub control_port: u16,
    
    /// Bridge configuration for censored regions
    pub bridges: Vec<String>,
    
    /// Circuit build timeout
    pub circuit_timeout: Duration,
    
    /// Prefer Tor over direct connections
    pub prefer_onion: bool,
    
    /// Hidden service configuration
    pub hidden_service: HiddenServiceConfig,
}

impl TorConfig {
    /// Load configuration from file or create default
    pub fn load_or_create(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("tor.json");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let default = TorConfig::default();
            let content = serde_json::to_string_pretty(&default)?;
            std::fs::write(&config_path, content)?;
            Ok(default)
        }
    }
}
```

## CLI Integration

### Tor-Specific Commands
```rust
#[derive(Subcommand)]
pub enum TorCommands {
    /// Start Tor daemon for anonymous networking
    Start {
        /// Use bridges for censored regions
        #[arg(long)]
        use_bridges: bool,
    },
    
    /// Stop Tor daemon
    Stop,
    
    /// Show our .onion address
    Address,
    
    /// Connect to peer via .onion address
    Connect {
        /// Peer's .onion address
        onion_address: String,
    },
    
    /// Publish our services to Tor DHT
    Advertise {
        /// Services to advertise
        #[arg(long)]
        services: Vec<String>,
    },
    
    /// Find peers by service via Tor DHT
    Discover {
        /// Service type to search for
        service: String,
    },
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_onion_address_generation() {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let onion_addr = generate_v3_onion_address(&signing_key.verifying_key()).unwrap();
        
        assert!(onion_addr.ends_with(".onion"));
        assert_eq!(onion_addr.len(), 62); // 56 base32 chars + ".onion"
        assert!(verify_onion_address(&onion_addr, &signing_key.verifying_key()).unwrap());
    }
    
    #[test]
    fn test_agent_advertisement_verification() {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let onion_addr = generate_v3_onion_address(&signing_key.verifying_key()).unwrap();
        
        let ad = AgentAdvertisement::create(
            &signing_key,
            onion_addr,
            vec!["memory_sync".to_string()]
        ).unwrap();
        
        assert!(ad.verify().unwrap());
    }
}
```

### Integration Tests
```rust
#[test]
#[ignore] // Requires Tor daemon
fn test_tor_connection_establishment() {
    let tor_daemon = TorDaemon::start(&temp_config_dir()).unwrap();
    tor_daemon.wait_for_ready(Duration::from_secs(30)).unwrap();
    
    let onion_addr = tor_daemon.get_onion_address().unwrap();
    let stream = TorStream::connect_via_socks(&onion_addr, 
        SocketAddr::from(([127, 0, 0, 1], 9050))).unwrap();
    
    // Test basic communication
    // ... connection test logic
}
```

## Deployment Considerations

### System Requirements
- **Tor daemon**: Must be installed and accessible
- **Network access**: Tor network connectivity required
- **Disk space**: ~10MB for Tor data directory
- **Memory**: ~50MB additional for Tor process

### Platform Support
- **Linux**: Full support via system Tor package
- **macOS**: Full support via Homebrew Tor
- **Windows**: Limited support (manual Tor installation)

### Performance Impact
- **Connection establishment**: +2-5 seconds for circuit building
- **Bandwidth overhead**: ~10% for encryption layers
- **Memory usage**: +50MB for Tor daemon
- **CPU usage**: Minimal for established connections

## Migration Strategy

### Gradual Rollout
1. **Phase 1**: Optional Tor support for advanced users
2. **Phase 2**: Default Tor for new installations
3. **Phase 3**: Deprecate direct connections for most use cases
4. **Phase 4**: Tor-only for maximum sovereignty

### Backward Compatibility
```rust
/// Support both direct and Tor connections during transition
#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    pub pubkey: String,
    
    // Legacy direct connection support
    pub direct_address: Option<SocketAddr>,
    
    // New Tor hidden service address
    pub onion_address: Option<String>,
    
    // Preference for connection type
    pub preferred_transport: TransportType,
}

#[derive(Serialize, Deserialize)]
pub enum TransportType {
    Direct,
    Tor,
    Hybrid, // Try Tor first, fallback to direct
}
```

## Conclusion

Tor integration represents the next evolution of mmogit's sovereignty infrastructure, providing technical enforcement of privacy and censorship resistance. The phased implementation strategy allows gradual adoption while maintaining backward compatibility.

**Key Benefits**:
- **True anonymity**: Location privacy for all agents
- **Censorship resistance**: Routes around blocked connections
- **Global reach**: Access peers regardless of local restrictions
- **Sovereignty enforcement**: Technical privacy, not just policy

**Implementation Priority**:
1. **Phase 1**: SOCKS proxy integration (3-4 weeks)
2. **Phase 2**: Hidden service publication (2-3 weeks)
3. **Phase 3**: Tor daemon integration (3-4 weeks)

**Success Metrics**:
- Successful .onion address generation and verification
- Reliable connections through Tor network
- Discovery of peers via Tor DHT
- Circuit diversity and rotation

This integration transforms mmogit from a privacy-friendly system to a truly sovereign platform where agents can communicate without fear of surveillance or censorship.