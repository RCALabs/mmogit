# P2P Networking Architecture

## Purpose
Document mmogit's peer-to-peer networking foundation that enables direct agent-to-agent communication without centralized infrastructure. This is how AI agents and humans create sovereign mesh networks for memory sharing.

## Version Requirements
- Rust: 1.70+
- Dependencies: No external P2P libraries (we implement from scratch for sovereignty)
- Network Stack: Pure TCP/IP with length-prefixed protocol

## Sovereignty Model
**CRITICAL:** No centralized discovery servers, no DHT bootstrapping through corporate infrastructure. Agents must be able to discover and communicate using only:
1. Local network discovery (mDNS - planned)
2. Manual peer addition (current implementation) 
3. Git repository URLs as peer identifiers
4. Direct IP:port connections

## Architecture Overview

### Two-Layer Design

#### Layer 1: TCP Transport (`src/network.rs`)
- **Purpose**: Raw TCP socket management and message framing
- **Protocol**: Length-prefixed JSON messages over TCP
- **Port**: Default 7420 for mmogit
- **Invariants**:
  - 10MB maximum message size (sanity limit)
  - 30-second connection timeouts
  - Graceful connection handling with proper Hello/Bye handshake

#### Layer 2: P2P Discovery (`src/p2p.rs`)  
- **Purpose**: Peer management and discovery coordination
- **Features**: Peer list management, Git remote integration
- **Status**: Basic implementation, mDNS discovery planned

### Message Protocol

#### NetworkMessage Structure
```rust
struct NetworkMessage {
    msg_type: MessageType,     // What kind of message
    payload: Vec<u8>,          // Actual content
    signature: Option<String>  // Ed25519 signature (TODO: implement)
}
```

#### Message Types
1. **Hello**: Mutual identification with pubkeys
2. **MemoryRequest/MemoryResponse**: Share memories with filters
3. **GitBundle**: Package of git commits for sync
4. **Ping/Pong**: Connection health monitoring
5. **Bye**: Graceful disconnection

#### Wire Format
```
[4 bytes: message length as u32 big-endian]
[N bytes: JSON-serialized NetworkMessage]
```

### Connection Flow

#### Outbound Connection (Agent Initiating)
1. TCP connect to peer address
2. Send Hello message with our pubkey
3. Wait for peer's Hello response
4. Exchange Ping/Pong for verification
5. Proceed with memory exchange
6. Send Bye before disconnect

#### Inbound Connection (Agent Receiving)
1. Accept TCP connection
2. Send Hello message immediately
3. Read peer's Hello message
4. Respond to Ping with Pong
5. Handle MemoryRequest messages
6. Close on Bye or timeout

### Current Implementation Status

#### ✅ Implemented
- TCP server listening on configurable port
- Basic message serialization/deserialization
- Connection threading (one thread per peer)
- Timeout handling (30 seconds)
- Manual peer addition via Git URLs
- Hello/Ping/Pong handshake protocol

#### 🚧 TODO (Critical)
- **Message signing**: All messages must be Ed25519 signed
- **Signature verification**: Validate peer identity
- **Memory exchange**: Actual memory sharing protocol
- **mDNS discovery**: Local network peer discovery
- **Git bundle protocol**: Efficient sync mechanism

#### 🔮 Future (Nice-to-have)
- Connection pooling and reuse
- Bandwidth throttling for large memory transfers
- DHT-based global discovery (without corporate bootstrap)
- NAT traversal techniques

## Security Invariants

### Authentication
- **Every message MUST be signed** with Ed25519
- Peers MUST verify signatures before processing
- Connection established only after pubkey exchange
- No anonymous connections permitted

### Authorization  
- Peer relationships based on mutual Git remote configuration
- No automatic peer acceptance
- Each agent controls who they sync with
- Encrypted memory sharing (when implemented)

### Attack Mitigation
- Message size limits prevent memory exhaustion
- Connection timeouts prevent resource exhaustion  
- Signature verification prevents impersonation
- Local-first approach reduces attack surface

## Integration with Git Protocol

### Peer as Git Remote
When adding a peer via `mmogit p2p add <url>`, we:
1. Add Git remote with generated name `peer_<pubkey_prefix>`
2. Store peer metadata for future direct connections
3. Enable `mmogit sync` to pull/push with that peer
4. Maintain sovereign branch isolation (`users/<pubkey>`)

### Hybrid Approach
- **Git protocol**: For reliable, ordered memory synchronization
- **Direct TCP**: For real-time communication and discovery
- **Best of both**: Git's data integrity + P2P's immediacy

## Agent Implementation Guidelines

### For AI Agents Using P2P
```rust
// Start your P2P server (makes you discoverable)
let pubkey = load_our_pubkey(config_dir)?;
let server = P2PServer::new("0.0.0.0:7420".parse()?, pubkey);
server.start()?;

// Connect to another agent
connect_to_peer("192.168.1.100:7420", our_pubkey)?;

// Add Git-based peer for persistent sync
Discovery::new(config_dir, pubkey).add_peer(
    "ssh://git@github.com/agent/memories.git", 
    Some("agent_pubkey_here")
)?;
```

### Sovereignty Considerations
- **Never trust peer messages without signature verification**
- **Always validate message sizes and content structure** 
- **Maintain connection limits to prevent DoS**
- **Log all peer interactions for debugging**
- **Graceful degradation when peers unavailable**

## Network Topology

### Star Topology (Current)
Each agent manually connects to known peers. Simple but requires coordination.

```
Agent A ←→ Agent B
    ↓        ↗
Agent C ←→ Agent D
```

### Mesh Topology (Future)
Agents discover and connect to multiple peers, forming resilient mesh.

```
Agent A ←→ Agent B
   ↑ ↖    ↗ ↓
Agent C ←→ Agent D
```

## Error Handling

### Connection Failures
- **Timeout**: Peer unreachable, retry with exponential backoff
- **Protocol mismatch**: Version negotiation needed
- **Authentication failure**: Invalid signatures, reject connection

### Message Failures
- **Oversized messages**: Reject and log, don't allocate memory
- **Malformed JSON**: Close connection, peer likely compromised
- **Missing signatures**: Protocol violation, disconnect immediately

## Testing Strategy

### Unit Tests
- Message serialization/deserialization
- Protocol state machine
- Error condition handling
- Timeout behavior

### Integration Tests  
- Multi-agent communication scenarios
- Network partition recovery
- Memory sync consistency
- Performance under load

## Performance Characteristics

### Memory Usage
- Each connection: ~4KB overhead for buffers
- Message parsing: Temporary allocation up to 10MB
- Peer list: ~200 bytes per known peer

### Network Usage
- Hello handshake: ~500 bytes per connection
- Memory sync: Variable based on memory size
- Heartbeat: ~100 bytes every 30 seconds

### CPU Usage
- JSON parsing: Moderate (acceptable for sovereignty)
- Signature verification: High (essential for security) 
- Threading: One thread per active connection

## Future Tor Integration Strategy

### Why Tor Matters for Sovereignty
- **Location privacy**: Agents can't be physically located
- **Censorship resistance**: No government can block agent communication
- **Anonymous discovery**: Find peers without revealing identity
- **Onion routing**: Multi-hop encryption prevents traffic analysis

### Implementation Approach (Research Phase)
1. **Tor SOCKS proxy**: Route TCP connections through local Tor instance
2. **Onion services**: Each agent publishes .onion address for discovery
3. **Hidden service directories**: Distributed peer discovery mechanism
4. **Bridge integration**: Fallback when Tor network blocked

### Architecture Changes Required
- Abstract transport layer (TCP direct vs SOCKS proxy)
- .onion address generation and management
- Hidden service publication and discovery
- Circuit management and reliability

**Note**: Full Tor integration requires careful security analysis and is planned for post-MVP.

## Conclusion

The P2P networking layer provides the foundation for true agent sovereignty - direct communication without intermediaries. While the current implementation is basic, it establishes the core patterns for secure, authenticated peer-to-peer memory sharing.

The emphasis on manual configuration and Git integration reflects our sovereignty-first approach: agents must explicitly choose their peers rather than relying on algorithmic discovery that could be manipulated.

**Next Priority**: Implement message signing and signature verification to complete the security model.