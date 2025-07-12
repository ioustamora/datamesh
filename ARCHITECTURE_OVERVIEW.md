# DataMesh Architecture Overview

## Executive Summary

DataMesh is a secure, fault-tolerant distributed data storage system built with Rust and libp2p. It provides ECIES-encrypted file storage with Reed-Solomon erasure coding across a peer-to-peer network using Kademlia DHT for decentralized coordination.

## Core Architecture Principles

### 1. Thread Safety First
- **Actor-Based Networking**: libp2p Swarm isolated in dedicated thread with message-passing communication
- **Thread-Safe Wrappers**: All database and file operations wrapped for concurrent access
- **Async/Await Throughout**: Non-blocking I/O operations across the entire system
- **Arc-Based Resource Sharing**: Efficient shared ownership of expensive resources

### 2. Security by Default
- **ECIES Encryption**: All stored data encrypted with Elliptic Curve Integrated Encryption Scheme
- **BLAKE3 Hashing**: Fast, secure content addressing and integrity verification
- **Secure Key Management**: Multiple key support with secure storage and rotation
- **Transport Security**: Encrypted network communications with peer authentication

### 3. Fault Tolerance
- **Reed-Solomon Erasure Coding**: 4+2 shards allow recovery from up to 2 failures
- **Intelligent Quorum Management**: Adaptive quorum based on network size and connectivity
- **Automatic Failover**: Built-in recovery mechanisms for network and storage failures
- **Health Monitoring**: Continuous system health tracking and self-healing

### 4. Performance Optimization
- **Concurrent Operations**: Parallel processing of file chunks and network operations
- **Intelligent Caching**: Smart caching with LRU and TTL policies
- **Zero-Copy Operations**: Minimal data copying in critical paths
- **Performance Monitoring**: Real-time metrics collection and optimization

## System Components

### Core Storage Layer (`actor_file_storage.rs`, `file_storage.rs`)

#### File Storage Algorithm
```
Original File → ECIES Encryption → Reed-Solomon 4+2 Shards → DHT Distribution
```

1. **Encryption Phase**: ECIES provides authenticated encryption with semantic security
2. **Erasure Coding Phase**: Reed-Solomon creates 6 shards (4 data + 2 parity) from encrypted data
3. **Distribution Phase**: Each shard stored independently in Kademlia DHT with BLAKE3 addressing

#### Critical Quorum Fix
**Problem**: `Quorum::One` requires 1 response from K_VALUE closest peers (typically 20), failing in small networks.

**Solution**: Intelligent quorum calculation using `Quorum::N(1)` for small networks:
- Networks ≤5 peers: Use `Quorum::N(1)` for maximum availability
- Larger networks: Scale up to 25% of connected peers for better durability
- Never exceed available peer count

### Network Layer (`network_actor.rs`, `network.rs`)

#### Actor-Based Architecture
```
┌─────────────────┐    Messages     ┌─────────────────┐
│   Application   │ ─────────────► │   Network       │
│   Threads       │                │   Actor         │
│                 │ ◄───────────── │   (Isolated)    │
└─────────────────┘    Responses   └─────────────────┘
       │                                    │
       ▼                                    ▼
┌─────────────────┐                ┌─────────────────┐
│  NetworkHandle  │                │  libp2p Swarm   │
│  (Clone-able)   │                │  (Single Thread)│
└─────────────────┘                └─────────────────┘
```

#### Message Types
- **PutRecord**: Store data in DHT with configurable quorum
- **GetRecord**: Retrieve data using Kademlia lookup
- **GetConnectedPeers**: Query network connectivity
- **Bootstrap**: Initialize DHT routing table
- **GetNetworkStats**: Real-time network metrics

### Command System (`commands/mod.rs`)

#### Clean Command Architecture
- **Handler Pattern**: Each command type has dedicated handler
- **Context Injection**: Shared resources provided via CommandContext
- **Performance Monitoring**: Automatic timing and success tracking
- **Error Standardization**: Consistent error handling across commands

#### Command Categories
- **File Operations**: put, get, list, info, stats
- **Network Operations**: peers, health, discover, bandwidth
- **Service Operations**: bootstrap, interactive, service
- **Admin Operations**: config, metrics, networks

### Error Handling (`error.rs`)

#### Comprehensive Error System
- **Semantic Types**: Specific error categories for precise handling
- **User-Friendly Messages**: Clear, actionable error descriptions
- **Error Chain Support**: Integration with Rust's Error trait
- **Recovery Strategies**: Different approaches based on error type

#### Error Categories
- **System Errors**: IO, Network, Database, Configuration
- **Crypto Errors**: Encryption, Decryption, KeyManagement, Authentication
- **Storage Errors**: Storage, FileNotFound, Share, Backup
- **Data Errors**: Serialization, Encoding, Import, Export

## Key Technical Innovations

### 1. Thread-Safe libp2p Integration
**Challenge**: libp2p Swarm is not Send/Sync, cannot be shared across threads.

**Solution**: Actor pattern isolates Swarm in dedicated thread, provides thread-safe NetworkHandle for communication.

**Benefits**:
- True thread safety without sacrificing performance
- Clean separation between networking and application logic
- Scalable concurrent operations

### 2. Intelligent Quorum Management
**Challenge**: Fixed quorum values fail in networks of varying sizes.

**Solution**: Dynamic quorum calculation based on connected peer count.

**Algorithm**:
```rust
let quorum = if connected_peers.len() <= 5 {
    Quorum::N(1)  // Maximum availability in small networks
} else {
    let size = (connected_peers.len() as f64 * 0.25).ceil() as usize;
    Quorum::N(size)  // Scaled durability in large networks
}
```

### 3. Reed-Solomon Fault Tolerance
**Parameters**: 4 data shards + 2 parity shards

**Benefits**:
- Tolerates up to 2 shard losses (33% failure rate)
- 50% storage overhead (reasonable for fault tolerance)
- Independent shard distribution reduces correlated failures

### 4. ECIES + BLAKE3 Security
**Encryption**: ECIES provides:
- Semantic security (identical plaintexts → different ciphertexts)
- Authenticated encryption (integrity + confidentiality)
- Public key encryption with forward secrecy

**Hashing**: BLAKE3 provides:
- Faster than SHA-2 and SHA-3
- Cryptographically secure
- Excellent avalanche properties for DHT key distribution

## Performance Characteristics

### Storage Operation Flow
1. **File Reading**: Single disk I/O, memory buffering
2. **Encryption**: ECIES operation on full file content
3. **Erasure Coding**: Reed-Solomon encoding to 6 shards
4. **DHT Storage**: Parallel storage of shards with intelligent quorum
5. **Metadata Storage**: Local database update for file tracking

### Network Operation Flow
1. **Peer Discovery**: Kademlia bootstrap and routing table population
2. **DHT Queries**: Logarithmic lookup complexity O(log N)
3. **Data Replication**: Configurable replication based on quorum settings
4. **Health Monitoring**: Continuous peer connectivity validation

### Scalability Metrics
- **Network Size**: Scales to thousands of peers with Kademlia DHT
- **File Size**: No theoretical limit, chunked processing for large files
- **Concurrent Operations**: Limited by available CPU cores and network bandwidth
- **Storage Capacity**: Limited by aggregate peer storage capacity

## Deployment Patterns

### Bootstrap Node
```bash
datamesh bootstrap --port 40871
```
- Provides initial network entry point
- Maintains DHT routing information
- Facilitates peer discovery

### Service Node
```bash
datamesh service --bootstrap-peer <PEER_ID> --bootstrap-addr /ip4/<IP>/tcp/<PORT>
```
- Participates in distributed storage
- Contributes storage capacity and bandwidth
- Maintains network connectivity

### Client Operations
```bash
datamesh put file.txt                    # Store file
datamesh get <file_hash> output.txt      # Retrieve file
datamesh list                            # Browse stored files
datamesh stats                           # Network health
```

## Security Model

### Threat Model
- **Data Confidentiality**: ECIES encryption protects against data exposure
- **Data Integrity**: BLAKE3 hashing and Reed-Solomon redundancy detect corruption
- **Network Security**: libp2p transport encryption protects network communications
- **Key Management**: Secure key storage with optional hardware security module support

### Attack Resistance
- **Sybil Attacks**: DHT routing provides natural resistance
- **Eclipse Attacks**: Multiple bootstrap peers and peer diversity
- **Data Corruption**: Reed-Solomon error correction and integrity checking
- **Network Partitions**: Graceful degradation and automatic recovery

## Monitoring and Observability

### Performance Metrics
- **Operation Timing**: Detailed timing for all operations
- **Success Rates**: Success/failure tracking across operation types
- **Network Health**: Peer connectivity and DHT routing table status
- **Resource Usage**: Memory, CPU, and storage utilization

### Health Monitoring
- **Peer Connectivity**: Real-time peer connection status
- **DHT Health**: Routing table quality and query success rates
- **Storage Health**: Shard availability and redundancy levels
- **System Health**: Overall system status and error rates

## Future Enhancements

### Planned Features
- **Automatic Data Repair**: Proactive shard regeneration from parity data
- **Load Balancing**: Intelligent request distribution across peers
- **Economic Incentives**: Token-based incentive system for storage providers
- **Governance System**: Decentralized network governance and policy management

### Scalability Improvements
- **Sharding**: Horizontal scaling through network partitioning
- **Caching**: Multi-tier caching for frequently accessed data
- **Compression**: Optional compression for storage efficiency
- **Bandwidth Optimization**: Adaptive bandwidth allocation and QoS

## Conclusion

DataMesh represents a modern approach to distributed storage, combining proven algorithms (Kademlia DHT, Reed-Solomon coding) with contemporary engineering practices (actor model, async Rust) to create a secure, fault-tolerant, and performant storage system.

The architecture prioritizes:
1. **Correctness**: Thread safety and error handling prevent data corruption
2. **Security**: Defense-in-depth with encryption, integrity checking, and secure networking
3. **Performance**: Optimized algorithms and concurrent operations
4. **Maintainability**: Clean separation of concerns and comprehensive documentation

The system is production-ready for distributed storage applications requiring high availability, security, and fault tolerance.