# DataMesh - Distributed Data Storage Network

A secure, fault-tolerant distributed data storage system with advanced monitoring, governance, and web interface capabilities. Built with Rust, libp2p, and cryptographic best practices. 

![Last Updated](https://img.shields.io/badge/Last%20Updated-January%202025-blue)
![Status](https://img.shields.io/badge/Status-Production%20Ready-green)
![Security](https://img.shields.io/badge/Security-Hardened-red)
![License](https://img.shields.io/badge/License-MIT-orange)
![Architecture](https://img.shields.io/badge/Architecture-Modular-purple)
![Monitoring](https://img.shields.io/badge/Monitoring-Advanced-yellow)

## 🚀 Features

### Core Storage & Security
- **🔒 Secure**: ECIES encryption ensures only you can access your files
- **🛡️ Fault-tolerant**: Reed-Solomon erasure coding survives peer failures
- **⚡ Fast**: BLAKE3 hashing and optimized networking for high performance
- **🌐 Distributed**: Kademlia DHT for decentralized peer-to-peer storage
- **🔧 Comprehensive**: 47 CLI commands covering all distributed storage needs

### Advanced Network Management
- **⚖️ Intelligent Load Balancing**: Adaptive load distribution with auto-scaling
- **🛡️ Advanced Failover**: Circuit breakers and automatic recovery systems
- **🚀 Performance Optimization**: ML-based performance tuning and recommendations
- **📊 Real-time Monitoring**: Advanced monitoring system with predictive analytics
- **🔍 System Health**: Automated health scoring and optimization recommendations

### Web Interface & API
- **🌐 Modern Web UI**: Vue.js-based web interface with real-time updates
- **🔗 REST API**: Comprehensive RESTful API for integration
- **👥 Multi-user Support**: User authentication and role-based access
- **📱 Responsive Design**: Mobile-friendly interface with drag-and-drop uploads
- **🔔 Real-time Notifications**: WebSocket-based live updates

### Network & Governance
- **🏛️ Network Governance**: Bootstrap node administration and user quotas
- **💰 Economic Model**: Token-based incentives and comprehensive billing system
- **🗳️ Democratic Voting**: Community governance for network decisions
- **💳 Billing & Subscriptions**: Multi-tier subscription management with usage tracking
- **🔐 User Management**: Account tiers, quotas, and payment processing

### File Management & Operations
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

# Test advanced systems
./target/release/datamesh advanced --comprehensive
```

## 💻 Installation

### Prerequisites
- Rust and Cargo (1.68.0 or newer)
- OpenSSL development libraries
- Git

### Building from Source
```bash
# Clone the repository
git clone https://github.com/ioustamora/datamesh.git
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

#### Performance & Config (5 commands)
```bash
optimize, benchmark, config, metrics, networks
```

#### Advanced Systems (1 command)
```bash
advanced
```

## 🏗️ Architecture

### Core Data Flow
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

### System Architecture
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Interface │────│   REST API      │────│   Core Engine   │
│   (Vue.js)      │    │   (Axum)        │    │   (Rust)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Dashboard     │────│   User Auth     │────│   P2P Network   │
│   & Analytics   │    │   & Governance  │    │   (libp2p)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Monitoring    │────│   Quota & Billing│────│   File Storage  │
│   System        │    │   Management    │    │   & Encryption  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔧 Technology Stack

### Backend Core
- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[libp2p](https://libp2p.io/)** - Peer-to-peer networking framework
- **[Kademlia DHT](https://en.wikipedia.org/wiki/Kademlia)** - Distributed hash table for peer discovery
- **[Axum](https://github.com/tokio-rs/axum)** - Modern async web framework for REST API
- **[SQLite](https://sqlite.org/)** - Embedded database for metadata storage
- **[RocksDB](https://rocksdb.org/)** - High-performance key-value store for time-series data

### Cryptography & Security
- **[ECIES](https://en.wikipedia.org/wiki/Integrated_Encryption_Scheme)** - Elliptic curve encryption for file security
- **[Reed-Solomon](https://en.wikipedia.org/wiki/Reed%E2%80%93Solomon_error_correction)** - Erasure coding for fault tolerance
- **[BLAKE3](https://github.com/BLAKE3-team/BLAKE3)** - Cryptographic hashing for integrity
- **[JWT](https://jwt.io/)** - JSON Web Tokens for authentication

### Frontend & Web Interface
- **[Vue.js 3](https://vuejs.org/)** - Progressive JavaScript framework
- **[Element Plus](https://element-plus.org/)** - Vue.js UI component library
- **[Pinia](https://pinia.vuejs.org/)** - State management for Vue.js
- **[Chart.js](https://www.chartjs.org/)** - Interactive charts for analytics
- **[Vite](https://vitejs.dev/)** - Fast build tool and dev server

### Monitoring & Analytics
- **[Prometheus](https://prometheus.io/)** - Time-series metrics collection
- **[Grafana](https://grafana.com/)** - Data visualization and dashboards
- **[Tokio](https://tokio.rs/)** - Async runtime for concurrent operations
- **[Serde](https://serde.rs/)** - Serialization framework for data processing

## 🛠️ Development Status

### ✅ **Complete and Production Ready:**

#### Core Infrastructure
- ✅ **Core Storage**: File encryption, chunking, distributed storage with Reed-Solomon (4+2)
- ✅ **Network Layer**: P2P networking, Kademlia DHT, peer discovery and routing
- ✅ **CLI Interface**: 47 comprehensive commands for all operations
- ✅ **Security**: ECIES encryption, BLAKE3 hashing, secure transport
- ✅ **Testing**: Comprehensive test suites covering all functionality

#### Advanced Features
- ✅ **File Management**: Sync, backup/restore, duplication, renaming with file watching
- ✅ **Search & Discovery**: Advanced search, recent files, popularity tracking
- ✅ **Batch Operations**: Parallel bulk upload/download/tagging operations
- ✅ **Health & Maintenance**: Automatic repair, cleanup, storage optimization
- ✅ **Performance**: Real-time metrics, benchmarking, bandwidth testing
- ✅ **Import/Export**: Archive integration with standard formats

#### Web Interface & API
- ✅ **Modern Web UI**: Vue.js-based interface with drag-and-drop uploads
- ✅ **REST API**: Comprehensive RESTful API with authentication
- ✅ **Real-time Updates**: WebSocket integration for live notifications
- ✅ **Responsive Design**: Mobile-friendly interface with dark/light themes
- ✅ **User Management**: Authentication, profiles, and settings

#### Monitoring & Analytics
- ✅ **Advanced Monitoring**: ML-based system with predictive analytics
- ✅ **Time-series Database**: High-performance historical data storage
- ✅ **Intelligent Alerting**: Anomaly detection with escalation management
- ✅ **Analytics Engine**: Performance insights and optimization recommendations
- ✅ **Interactive Dashboard**: Real-time visualization with customizable widgets

#### Governance & Economics
- ✅ **Network Governance**: Bootstrap node administration framework
- ✅ **User Quotas**: Fair usage policies with tiered access levels
- ✅ **Economic Model**: Token-based incentives and billing integration
- ✅ **Democratic Voting**: Community governance mechanisms
- ✅ **Multi-region Support**: Geographic distribution and compliance

#### Advanced Network Systems
- ✅ **Load Balancing**: Intelligent load distribution with multiple strategies
- ✅ **Auto-scaling**: Dynamic scaling based on performance metrics
- ✅ **Failover Management**: Circuit breakers and automatic recovery systems
- ✅ **Performance Optimization**: ML-based performance tuning and recommendations
- ✅ **Billing System**: Comprehensive billing with subscription management
- ✅ **Advanced Testing**: Comprehensive test suite for all advanced systems

### 🔄 **In Progress:**
- Enhanced caching system with intelligent prefetching
- Advanced fault injection testing
- Integration with cloud storage providers

### 📋 **Future Enhancements:**
- Docker deployment and Kubernetes operators
- Advanced compression and deduplication
- Enterprise SSO integration
- Multi-tenancy and permission systems

## 📝 Code Documentation

The codebase is organized into well-defined modules with comprehensive functionality:

### Core Infrastructure
- **main.rs**: Entry point handling all 47 CLI commands
- **cli.rs**: Complete command-line interface with clap parsing
- **file_storage.rs**: Core file operations (chunking, encryption, storage, retrieval)
- **network.rs**: P2P networking using libp2p and Kademlia DHT
- **key_manager.rs**: Cryptographic key management and ECIES operations
- **database.rs**: SQLite metadata storage with comprehensive functionality

### Advanced Features
- **file_manager.rs**: File watching, sync operations, directory management
- **batch_operations.rs**: Parallel bulk operations (put/get/tag)
- **health_manager.rs**: Health monitoring, repair, cleanup, and benchmarking
- **network_diagnostics.rs**: Network topology analysis and peer discovery
- **performance.rs**: Real-time metrics collection and analysis
- **smart_cache.rs**: Intelligent caching with ML-based prefetching

### Advanced Network Systems
- **load_balancer.rs**: Intelligent load balancing with auto-scaling
- **failover.rs**: Advanced failover management with circuit breakers
- **performance_optimizer.rs**: ML-based performance optimization
- **billing_system.rs**: Comprehensive billing and subscription management
- **datamesh_core.rs**: Unified integration layer for all systems
- **advanced_commands.rs**: Testing and management commands

### Web Interface & API
- **api_server.rs**: REST API server with authentication and rate limiting
- **web-interface/**: Vue.js frontend with modern UI components
  - **src/views/**: Main application views (Dashboard, Analytics, FileManager, etc.)
  - **src/components/**: Reusable UI components
  - **src/services/**: API integration and WebSocket handling
  - **src/store/**: Pinia state management modules

### Monitoring & Analytics
- **monitoring/mod.rs**: Advanced monitoring system core
- **monitoring/metrics.rs**: Comprehensive metrics collection
- **monitoring/time_series.rs**: High-performance time-series database
- **monitoring/alerts.rs**: Intelligent alerting with ML-based detection
- **monitoring/analytics.rs**: Analytics engine with predictive insights
- **monitoring/dashboard.rs**: Real-time dashboard with customizable widgets

### Governance & Economics
- **governance.rs**: Network governance and voting mechanisms
- **governance_service.rs**: Governance API and management
- **quota_service.rs**: User quotas and fair usage enforcement
- **economics.rs**: Token economics and incentive systems
- **audit_logger.rs**: Comprehensive audit logging

### Supporting Infrastructure
- **interactive.rs**: Interactive console and service mode implementations
- **ui.rs**: User interface components and progress indicators
- **config.rs**: Configuration file handling and network presets
- **error_handling.rs**: Comprehensive error types and recovery
- **logging.rs**: Structured logging with multiple levels
- **presets.rs**: Network configuration presets and templates
- **resilience.rs**: Network resilience and fault tolerance

### Build & Test Infrastructure
- **Cargo.toml**: Production dependencies with advanced features
- **examples/**: Comprehensive testing infrastructure with cluster tests
- **tests/**: Integration tests for all modules
- **scripts/**: Build verification and deployment scripts

Each module follows Rust best practices with extensive documentation. Generate docs with:
```bash
cargo doc --no-deps --open
```

### Web Interface Structure
```
web-interface/
├── src/
│   ├── views/          # Main application pages
│   │   ├── Dashboard.vue
│   │   ├── Analytics.vue
│   │   ├── FileManager.vue
│   │   ├── Governance.vue
│   │   └── auth/
│   ├── components/     # Reusable UI components
│   │   ├── common/
│   │   ├── dashboard/
│   │   ├── files/
│   │   └── governance/
│   ├── services/       # API integration
│   │   └── api.js
│   ├── store/          # State management
│   │   ├── auth.js
│   │   ├── files.js
│   │   └── governance.js
│   └── utils/          # Utility functions
├── package.json
└── vite.config.js
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
