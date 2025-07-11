/// Network Performance Benchmarks for DataMesh
///
/// Comprehensive benchmarking of network operations including P2P communication,
/// DHT operations, data transfer, and protocol overhead measurements.
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;

// DataMesh network modules
use datamesh::concurrent_chunks::{ConcurrentChunkConfig, ConcurrentChunkManager};
use datamesh::network::{MyBehaviour, NetworkHandle};
use datamesh::network_diagnostics::NetworkDiagnostics;

fn benchmark_dht_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("dht_operations");

    // Simulate DHT key generation and operations
    let data_sizes = vec![32, 64, 128, 256, 512, 1024];

    for size in data_sizes {
        let data = vec![0u8; size];

        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark key generation from data
        group.bench_with_input(
            BenchmarkId::new("key_generation", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let hash = blake3::hash(black_box(data));
                    let key = libp2p::kad::RecordKey::new(&hash.as_bytes());
                    black_box(key);
                });
            },
        );

        // Benchmark record creation
        group.bench_with_input(
            BenchmarkId::new("record_creation", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let hash = blake3::hash(data);
                    let key = libp2p::kad::RecordKey::new(&hash.as_bytes());
                    let record = libp2p::kad::Record::new(key, black_box(data.clone()));
                    black_box(record);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_multiaddr_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiaddr_operations");

    let addresses = vec![
        "/ip4/127.0.0.1/tcp/40871",
        "/ip4/192.168.1.100/tcp/40872",
        "/ip6/::1/tcp/40873",
        "/dns4/example.com/tcp/40874",
        "/dns6/example.com/tcp/40875",
        "/ip4/127.0.0.1/tcp/40876/p2p/12D3KooWGBdz4YeEtLK1Eqth5ZvRrQF6Q1VxRQqUVJJ7VHdHf8m1",
    ];

    for addr_str in &addresses {
        group.bench_with_input(
            BenchmarkId::new("parse", addr_str.len()),
            addr_str,
            |b, addr_str| {
                b.iter(|| {
                    let addr: Result<libp2p::Multiaddr, _> = black_box(addr_str).parse();
                    black_box(addr);
                });
            },
        );

        // Benchmark address validation
        if let Ok(addr) = addr_str.parse::<libp2p::Multiaddr>() {
            group.bench_with_input(
                BenchmarkId::new("validate", addr_str.len()),
                &addr,
                |b, addr| {
                    b.iter(|| {
                        let protocols: Vec<_> = black_box(addr).iter().collect();
                        black_box(protocols);
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_peer_id_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_id_operations");

    // Generate test keypairs
    let keypairs: Vec<_> = (0..10)
        .map(|_| libp2p::identity::Keypair::generate_ed25519())
        .collect();

    group.bench_function("keypair_generation", |b| {
        b.iter(|| {
            let keypair = libp2p::identity::Keypair::generate_ed25519();
            black_box(keypair);
        });
    });

    group.bench_function("peer_id_from_keypair", |b| {
        b.iter_with_setup(
            || libp2p::identity::Keypair::generate_ed25519(),
            |keypair| {
                let peer_id = libp2p::PeerId::from(keypair.public());
                black_box(peer_id);
            },
        );
    });

    let peer_ids: Vec<_> = keypairs
        .iter()
        .map(|kp| libp2p::PeerId::from(kp.public()))
        .collect();

    group.bench_function("peer_id_to_string", |b| {
        b.iter_with_setup(
            || &peer_ids[fastrand::usize(..peer_ids.len())],
            |peer_id| {
                let s = peer_id.to_string();
                black_box(s);
            },
        );
    });

    group.bench_function("peer_id_from_string", |b| {
        let peer_id_strings: Vec<_> = peer_ids.iter().map(|pid| pid.to_string()).collect();

        b.iter_with_setup(
            || &peer_id_strings[fastrand::usize(..peer_id_strings.len())],
            |s| {
                let parsed: Result<libp2p::PeerId, _> = s.parse();
                black_box(parsed);
            },
        );
    });

    group.finish();
}

fn benchmark_data_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_serialization");

    use chrono::Local;
    use datamesh::database::FileEntry;

    // Create test data structures
    let file_entry = FileEntry {
        id: 1,
        name: "benchmark_test_file".to_string(),
        file_key: "abc123def456ghi789".to_string(),
        original_filename: "test_document.pdf".to_string(),
        file_size: 1024 * 1024, // 1MB
        upload_time: Local::now(),
        tags: vec![
            "benchmark".to_string(),
            "test".to_string(),
            "performance".to_string(),
        ],
        public_key_hex: "0123456789abcdef".to_string(),
        chunks_total: 6,
        chunks_healthy: 6,
    };

    // JSON serialization benchmarks
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&file_entry)).unwrap();
            black_box(json);
        });
    });

    let json_str = serde_json::to_string(&file_entry).unwrap();
    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let entry: FileEntry = serde_json::from_str(black_box(&json_str)).unwrap();
            black_box(entry);
        });
    });

    // Bincode serialization benchmarks
    group.bench_function("bincode_serialize", |b| {
        b.iter(|| {
            let encoded = bincode::serialize(black_box(&file_entry)).unwrap();
            black_box(encoded);
        });
    });

    let bincode_data = bincode::serialize(&file_entry).unwrap();
    group.bench_function("bincode_deserialize", |b| {
        b.iter(|| {
            let entry: FileEntry = bincode::deserialize(black_box(&bincode_data)).unwrap();
            black_box(entry);
        });
    });

    // TOML serialization benchmarks
    group.bench_function("toml_serialize", |b| {
        b.iter(|| {
            let toml = toml::to_string(black_box(&file_entry)).unwrap();
            black_box(toml);
        });
    });

    let toml_str = toml::to_string(&file_entry).unwrap();
    group.bench_function("toml_deserialize", |b| {
        b.iter(|| {
            let entry: FileEntry = toml::from_str(black_box(&toml_str)).unwrap();
            black_box(entry);
        });
    });

    group.finish();
}

fn benchmark_network_message_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_processing");

    use datamesh::network_actor::{NetworkMessage, NetworkStats};
    use libp2p::kad::{Record, RecordKey};
    use tokio::sync::oneshot;

    let data_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    for size in data_sizes {
        let data = vec![0u8; size];
        let key = RecordKey::new(&blake3::hash(&data).as_bytes());
        let record = Record::new(key.clone(), data.clone());

        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark message creation
        group.bench_with_input(
            BenchmarkId::new("put_message_creation", size),
            &record,
            |b, record| {
                b.iter(|| {
                    let (tx, _rx) = oneshot::channel();
                    let msg = NetworkMessage::PutRecord {
                        record: black_box(record.clone()),
                        response: tx,
                    };
                    black_box(msg);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("get_message_creation", size),
            &key,
            |b, key| {
                b.iter(|| {
                    let (tx, _rx) = oneshot::channel();
                    let msg = NetworkMessage::GetRecord {
                        key: black_box(key.clone()),
                        response: tx,
                    };
                    black_box(msg);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_chunk_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_chunks");

    group.bench_function("chunk_config_creation", |b| {
        b.iter(|| {
            let config = ConcurrentChunkConfig {
                max_concurrent_uploads: 4,
                max_concurrent_downloads: 8,
                upload_timeout_seconds: 30,
                download_timeout_seconds: 60,
                retry_failed_chunks: 3,
                chunk_size: 65536,
            };
            black_box(config);
        });
    });

    group.bench_function("chunk_manager_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = ConcurrentChunkConfig::default();
            let manager = ConcurrentChunkManager::new(config);
            black_box(manager);
        });
    });

    group.finish();
}

fn benchmark_network_diagnostics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("network_diagnostics");

    group.bench_function("diagnostics_creation", |b| {
        b.iter(|| {
            let diagnostics = NetworkDiagnostics::new();
            black_box(diagnostics);
        });
    });

    // Benchmark latency calculation
    let latencies = vec![10, 25, 50, 100, 200, 500]; // milliseconds

    group.bench_function("latency_calculation", |b| {
        b.iter(|| {
            let total: u64 = black_box(&latencies).iter().sum();
            let avg = total as f64 / latencies.len() as f64;
            black_box(avg);
        });
    });

    // Benchmark bandwidth calculation
    let data_points = vec![
        (1024, 100),     // 1KB in 100ms
        (10240, 250),    // 10KB in 250ms
        (102400, 500),   // 100KB in 500ms
        (1048576, 2000), // 1MB in 2s
    ];

    group.bench_function("bandwidth_calculation", |b| {
        b.iter(|| {
            let bandwidths: Vec<f64> = black_box(&data_points)
                .iter()
                .map(|(bytes, ms)| (*bytes as f64 / *ms as f64) * 1000.0) // bytes per second
                .collect();
            black_box(bandwidths);
        });
    });

    group.finish();
}

fn benchmark_protocol_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_overhead");

    // Measure overhead of different protocol layers
    let payload_sizes = vec![64, 256, 1024, 4096, 16384];

    for size in payload_sizes {
        let payload = vec![0u8; size];

        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark raw data copying (baseline)
        group.bench_with_input(
            BenchmarkId::new("raw_copy", size),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let copy = black_box(payload).clone();
                    black_box(copy);
                });
            },
        );

        // Benchmark with Record wrapper
        group.bench_with_input(
            BenchmarkId::new("record_wrap", size),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let key = RecordKey::new(&blake3::hash(payload).as_bytes());
                    let record = Record::new(key, black_box(payload).clone());
                    black_box(record);
                });
            },
        );

        // Benchmark with network message wrapper
        group.bench_with_input(
            BenchmarkId::new("message_wrap", size),
            &payload,
            |b, payload| {
                use datamesh::network_actor::NetworkMessage;
                use tokio::sync::oneshot;

                b.iter(|| {
                    let key = RecordKey::new(&blake3::hash(payload).as_bytes());
                    let record = Record::new(key, payload.clone());
                    let (tx, _rx) = oneshot::channel();
                    let msg = NetworkMessage::PutRecord {
                        record: black_box(record),
                        response: tx,
                    };
                    black_box(msg);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_connection_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_management");

    // Benchmark address parsing and validation
    let bootstrap_addresses = vec![
        "/ip4/127.0.0.1/tcp/40871",
        "/ip4/192.168.1.100/tcp/40872",
        "/ip4/10.0.0.1/tcp/40873",
        "/dns4/bootstrap1.example.com/tcp/40874",
        "/dns4/bootstrap2.example.com/tcp/40875",
    ];

    group.bench_function("parse_bootstrap_addresses", |b| {
        b.iter(|| {
            let addrs: Result<Vec<libp2p::Multiaddr>, _> = black_box(&bootstrap_addresses)
                .iter()
                .map(|s| s.parse())
                .collect();
            black_box(addrs);
        });
    });

    // Benchmark peer list management
    let peer_ids: Vec<libp2p::PeerId> = (0..100)
        .map(|_| {
            let keypair = libp2p::identity::Keypair::generate_ed25519();
            libp2p::PeerId::from(keypair.public())
        })
        .collect();

    group.bench_function("peer_list_operations", |b| {
        b.iter(|| {
            let mut peer_set = std::collections::HashSet::new();
            for peer_id in black_box(&peer_ids) {
                peer_set.insert(*peer_id);
            }

            // Simulate peer operations
            let random_peer = &peer_ids[fastrand::usize(..peer_ids.len())];
            let contains = peer_set.contains(random_peer);
            black_box((peer_set, contains));
        });
    });

    group.finish();
}

fn benchmark_event_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("event_handling");

    use datamesh::network::MyBehaviourEvent;
    use libp2p::kad::Event as KademliaEvent;
    use libp2p::kad::{GetRecordOk, QueryResult, Record, RecordKey};

    // Create sample events
    let test_data = vec![0u8; 1024];
    let key = RecordKey::new(&blake3::hash(&test_data).as_bytes());
    let record = Record::new(key.clone(), test_data);

    let events = vec![MyBehaviourEvent::Kademlia(
        KademliaEvent::OutboundQueryProgressed {
            id: libp2p::kad::QueryId::new(),
            result: QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(record.clone()))),
            stats: libp2p::kad::ProgressStep {
                count: 1,
                duration: Duration::from_millis(100),
            },
            step: libp2p::kad::ProgressStep {
                count: 1,
                duration: Duration::from_millis(100),
            },
        },
    )];

    group.bench_function("event_processing", |b| {
        b.iter(|| {
            for event in black_box(&events) {
                match event {
                    MyBehaviourEvent::Kademlia(kad_event) => match kad_event {
                        KademliaEvent::OutboundQueryProgressed { result, .. } => match result {
                            QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(record))) => {
                                let data_len = record.value.len();
                                black_box(data_len);
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                }
            }
        });
    });

    group.finish();
}

// Configure benchmark groups
criterion_group!(
    name = network_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        benchmark_dht_operations,
        benchmark_multiaddr_parsing,
        benchmark_peer_id_operations,
        benchmark_data_serialization,
        benchmark_network_message_processing,
        benchmark_concurrent_chunk_operations,
        benchmark_network_diagnostics,
        benchmark_protocol_overhead,
        benchmark_connection_management,
        benchmark_event_handling
);

criterion_main!(network_benches);
