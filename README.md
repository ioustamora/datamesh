# DataMesh - Distributed Data Storage Network

A secure, fault-tolerant distributed data storage system built with Rust, libp2p, and cryptographic best practices. 

![Last Updated](https://img.shields.io/badge/Last%20Updated-July%202025-blue)
![Status](https://img.shields.io/badge/Status-Production%20Ready-green)
![License](https://img.shields.io/badge/License-MIT-orange)

## 🚀 Features

- **🔒 Secure**: ECIES encryption ensures only you can access your files
- **🛡️ Fault-tolerant**: Reed-Solomon erasure coding survives peer failures
- **⚡ Fast**: BLAKE3 hashing and optimized networking for high performance
- **🌐 Distributed**: Kademlia DHT for decentralized peer-to-peer storage
- **🔧 Comprehensive**: 47 CLI commands covering all distributed storage needs
- **📊 Monitoring**: Real-time performance metrics and health monitoring
- **🔄 Synchronization**: Directory sync with file watching capabilities
- **💾 Backup/Restore**: Versioned backups with integrity verification
- **🔍 Advanced Search**: Multi-criteria file search and discovery
- **⚙️ Batch Operations**: Efficient bulk file operations
- **🏥 Self-Healing**: Automatic repair and cleanup maintenance
- **📈 Optimization**: Performance benchmarks and storage optimization

## 🏃 Quick Start

```bash
# Build the project
cargo build --release

# Store a file
./target/release/datamesh put myfile.txt
# Returns: File stored with key: a1b2c3d4e5f6...

# Retrieve a file
./target/release/datamesh get a1b2c3d4e5f6... ./recovered_file.txt
```

## 💻 Installation

### Prerequisites
- Rust and Cargo (1.68.0 or newer)
- OpenSSL development libraries
- Git

### Building from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/datamesh.git
cd datamesh

# Build in release mode
cargo build --release

# Run tests
cargo test

# The binary will be available at
./target/release/datamesh
```

### Running in Development Mode
```bash
cargo run -- put myfile.txt
```

## 🧪 Testing and Examples

The project includes comprehensive testing infrastructure in the `examples/` directory:

- **perfect_cluster_test.sh**: Complete interactive cluster testing with monitoring dashboard
- **comprehensive_cluster_test.sh**: Automated testing of all 47 CLI commands
- **interactive_dashboard_functions.sh**: Real-time cluster management functions
- **cluster_test.sh**: Basic multi-node cluster setup and testing
- **simple_test.sh**: Quick test for basic file operations
- **start_bootstrap.sh**: Start a bootstrap node for the DHT
- **start_node.sh**: Start a regular node that connects to the bootstrap
- **backup.sh**: Example script for backing up key files
- **restore.sh**: Example script for restoring from backups

To run the comprehensive test:

```bash
cd examples
./comprehensive_cluster_test.sh
```

This tests all features including sync, backup/restore, search, batch operations, health monitoring, and network diagnostics.

For interactive cluster management:

```bash
cd examples  
./perfect_cluster_test.sh
```

This provides a full management dashboard with real-time monitoring and interactive testing capabilities.

## 📚 Documentation

- **[USAGE.md](USAGE.md)** - Comprehensive usage guide with all 47 commands
- **[MODULES.md](MODULES.md)** - Technical module documentation
- **[IMPROVEMENTS.md](IMPROVEMENTS.md)** - Future enhancements and roadmap

### Command Categories

#### Core Operations (5 commands)
```bash
put, get, list, info, stats
```

#### Network Management (9 commands)  
```bash
bootstrap, interactive, service, peers, network, discover, distribution, health, bandwidth
```

#### File Management (5 commands)
```bash
sync, backup, restore, duplicate, rename
```

#### Search & Discovery (3 commands)
```bash
search, recent, popular
```

#### Batch Operations (3 commands)
```bash
batch-put, batch-get, batch-tag
```

#### Health & Maintenance (3 commands)
```bash
repair, cleanup, quota
```

#### Import/Export (2 commands)
```bash
export, import
```

#### Quick Actions (3 commands)
```bash
pin, unpin, share
```

#### Performance & Config (3 commands)
```bash
optimize, benchmark, config, metrics, networks
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Your File     │───▶│   Encryption    │───▶│  Reed-Solomon   │
│                 │    │     (ECIES)     │    │  Erasure Code   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Distributed    │◀───│   BLAKE3 Hash   │◀───│   Split into    │
│   Network       │    │     Keys        │    │     Chunks      │
│  (Kademlia)     │    │                 │    │   (4+2 shards)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔧 Technology Stack

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[libp2p](https://libp2p.io/)** - Peer-to-peer networking
- **[Kademlia DHT](https://en.wikipedia.org/wiki/Kademlia)** - Distributed hash table
- **[ECIES](https://en.wikipedia.org/wiki/Integrated_Encryption_Scheme)** - Elliptic curve encryption
- **[Reed-Solomon](https://en.wikipedia.org/wiki/Reed%E2%80%93Solomon_error_correction)** - Erasure coding
- **[BLAKE3](https://github.com/BLAKE3-team/BLAKE3)** - Cryptographic hashing

## 🛠️ Development Status

✅ **Complete and Production Ready:**
- ✅ **Core Storage**: File encryption, chunking, distributed storage with Reed-Solomon (4+2)
- ✅ **Network Layer**: P2P networking, Kademlia DHT, peer discovery and routing
- ✅ **CLI Interface**: 47 comprehensive commands for all operations
- ✅ **File Management**: Sync, backup/restore, duplication, renaming with file watching
- ✅ **Search & Discovery**: Advanced search, recent files, popularity tracking
- ✅ **Batch Operations**: Parallel bulk upload/download/tagging operations
- ✅ **Health & Maintenance**: Automatic repair, cleanup, storage optimization
- ✅ **Performance**: Real-time metrics, benchmarking, bandwidth testing
- ✅ **Import/Export**: Archive integration with standard formats
- ✅ **Security**: ECIES encryption, BLAKE3 hashing, secure transport
- ✅ **Monitoring**: Real-time health monitoring and interactive dashboards
- ✅ **Testing**: Comprehensive test suites covering all functionality

🔄 **In Progress:**
- Enhanced web interface and REST API
- Advanced fault injection testing
- Distributed consensus improvements

📋 **Future Enhancements:**
- Docker deployment and Kubernetes operators
- Advanced analytics and usage insights
- Multi-tenancy and permission systems
- Integration with cloud storage providers

## 📝 Code Documentation

The codebase is organized into well-defined modules with comprehensive functionality:

### Core Modules
- **main.rs** (827 lines): Entry point handling all 47 CLI commands
- **cli.rs** (568 lines): Complete command-line interface with clap parsing
- **file_storage.rs**: Core file operations (chunking, encryption, storage, retrieval)
- **network.rs**: P2P networking using libp2p and Kademlia DHT
- **key_manager.rs**: Cryptographic key management and ECIES operations

### Feature Modules  
- **file_manager.rs**: File watching, sync operations, directory management
- **batch_operations.rs**: Parallel bulk operations (put/get/tag)
- **health_manager.rs**: Health monitoring, repair, cleanup, and benchmarking
- **database.rs**: SQLite metadata storage with 459 lines of functionality
- **network_diagnostics.rs**: Network topology analysis and peer discovery

### Supporting Modules
- **interactive.rs**: Interactive console and service mode implementations
- **ui.rs**: User interface components and progress indicators  
- **config.rs**: Configuration file handling and network presets
- **performance.rs**: Real-time metrics collection and analysis
- **error_handling.rs**: Comprehensive error types and recovery
- **logging.rs**: Structured logging with multiple levels
- **presets.rs**: Network configuration presets and templates

### Build & Test Infrastructure
- **Cargo.toml**: Production dependencies including notify for file watching
- **examples/**: Comprehensive testing infrastructure (3 major test suites)
- **tests/**: Integration tests for module functionality

Each module follows Rust best practices with extensive documentation. Generate docs with:
```bash
cargo doc --no-deps --open
```

## 🤝 Contributing

Contributions are welcome! Please see [IMPROVEMENTS.md](IMPROVEMENTS.md) for planned enhancements and development priorities.

To contribute:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure your code follows the project's coding style and includes appropriate tests and documentation.

## 📄 License

MIT License

Copyright (c) 2025 DataMesh Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

## ⚠️ Security Notice

This is experimental software. Do not use for critical data without proper testing and security review. Always keep backups of your file keys - losing them means losing access to your files permanently.
