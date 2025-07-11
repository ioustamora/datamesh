/// Storage Performance Benchmarks for DataMesh
///
/// Comprehensive benchmarking of storage operations including file I/O,
/// database operations, caching, and data processing pipelines.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

// DataMesh storage modules
use datamesh::database::{DatabaseManager, FileEntry};
use datamesh::thread_safe_database::ThreadSafeDatabaseManager;
use datamesh::smart_cache::{SmartCacheManager, CacheConfig};
use datamesh::file_storage;

fn benchmark_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_operations");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("benchmark.db");
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Benchmark database initialization
    group.bench_function("database_creation", |b| {
        b.iter_with_setup(
            || {
                let temp_dir = TempDir::new().unwrap();
                temp_dir.path().join("bench_init.db")
            },
            |db_path| {
                let db = DatabaseManager::new(black_box(&db_path)).unwrap();
                black_box(db);
            },
        );
    });
    
    // Benchmark file insertion
    let upload_time = chrono::Local::now();
    let tags = vec!["benchmark".to_string(), "test".to_string()];
    
    group.bench_function("file_insertion", |b| {
        let mut counter = 0;
        b.iter(|| {
            let name = format!("benchmark_file_{}", counter);
            let key = format!("key_{}", counter);
            counter += 1;
            
            let id = db.store_file(
                black_box(&name),
                black_box(&key),
                "benchmark.txt",
                1024,
                upload_time,
                &tags,
                "pubkey"
            ).unwrap();
            black_box(id);
        });
    });
    
    // Setup data for read benchmarks
    for i in 0..1000 {
        db.store_file(
            &format!("read_test_{}", i),
            &format!("read_key_{}", i),
            "read_test.txt",
            1024,
            upload_time,
            &tags,
            "pubkey"
        ).unwrap();
    }
    
    // Benchmark file retrieval by name
    group.bench_function("file_retrieval_by_name", |b| {
        b.iter_with_setup(
            || format!("read_test_{}", fastrand::usize(..1000)),
            |name| {
                let result = db.get_file_by_name(black_box(&name)).unwrap();
                black_box(result);
            },
        );
    });
    
    // Benchmark file retrieval by key
    group.bench_function("file_retrieval_by_key", |b| {
        b.iter_with_setup(
            || format!("read_key_{}", fastrand::usize(..1000)),
            |key| {
                let result = db.get_file_by_key(black_box(&key)).unwrap();
                black_box(result);
            },
        );
    });
    
    // Benchmark file listing
    group.bench_function("file_listing", |b| {
        b.iter(|| {
            let files = db.list_files(None).unwrap();
            black_box(files);
        });
    });
    
    // Benchmark file search
    group.bench_function("file_search", |b| {
        b.iter_with_setup(
            || format!("read_test_{}", fastrand::usize(..100)),
            |query| {
                let results = db.search_files(black_box(&query)).unwrap();
                black_box(results);
            },
        );
    });
    
    // Benchmark statistics calculation
    group.bench_function("statistics_calculation", |b| {
        b.iter(|| {
            let stats = db.get_stats().unwrap();
            black_box(stats);
        });
    });
    
    group.finish();
}

fn benchmark_thread_safe_database(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("thread_safe_database");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("thread_safe_benchmark.db");
    let db = ThreadSafeDatabaseManager::new(&db_path).unwrap();
    
    // Benchmark thread-safe operations
    let upload_time = chrono::Local::now();
    let tags = vec!["thread_safe".to_string(), "benchmark".to_string()];
    
    group.bench_function("thread_safe_insertion", |b| {
        let mut counter = 0;
        b.iter(|| {
            let name = format!("ts_file_{}", counter);
            let key = format!("ts_key_{}", counter);
            counter += 1;
            
            let id = db.store_file(
                black_box(&name),
                black_box(&key),
                "ts_benchmark.txt",
                1024,
                upload_time,
                &tags,
                "pubkey"
            ).unwrap();
            black_box(id);
        });
    });
    
    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        b.to_async(&rt).iter(|| async {
            let futures: Vec<_> = (0..10)
                .map(|i| {
                    let db = &db;
                    async move {
                        let name = format!("concurrent_{}", i);
                        db.get_file_by_name(&name)
                    }
                })
                .collect();
            
            let results = futures::future::join_all(futures).await;
            black_box(results);
        });
    });
    
    group.finish();
}

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    
    let temp_dir = TempDir::new().unwrap();
    let file_sizes = vec![
        1024,           // 1 KB
        10 * 1024,      // 10 KB
        100 * 1024,     // 100 KB
        1024 * 1024,    // 1 MB
        10 * 1024 * 1024, // 10 MB
    ];
    
    for size in file_sizes {
        let data = vec![0u8; size];
        let file_path = temp_dir.path().join(format!("test_{}.bin", size));
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark file writing
        group.bench_with_input(BenchmarkId::new("file_write", size), &data, |b, data| {
            b.iter_with_setup(
                || temp_dir.path().join(format!("write_{}.bin", fastrand::u64(..))),
                |path| {
                    std::fs::write(black_box(&path), black_box(data)).unwrap();
                },
            );
        });
        
        // Setup file for reading
        std::fs::write(&file_path, &data).unwrap();
        
        // Benchmark file reading
        group.bench_with_input(BenchmarkId::new("file_read", size), &file_path, |b, path| {
            b.iter(|| {
                let content = std::fs::read(black_box(path)).unwrap();
                black_box(content);
            });
        });
        
        // Benchmark memory mapping
        group.bench_with_input(BenchmarkId::new("memory_map", size), &file_path, |b, path| {
            b.iter(|| {
                let file = std::fs::File::open(black_box(path)).unwrap();
                let metadata = file.metadata().unwrap();
                let len = metadata.len() as usize;
                black_box(len);
            });
        });
    }
    
    group.finish();
}

fn benchmark_smart_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("smart_cache");
    
    // Benchmark cache creation
    group.bench_function("cache_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = CacheConfig::default();
            let cache = SmartCacheManager::new(config).await.unwrap();
            black_box(cache);
        });
    });
    
    // Setup cache for operation benchmarks
    let cache = rt.block_on(async {
        let config = CacheConfig {
            max_file_cache_size: 100,
            max_chunk_cache_size: 1000,
            file_cache_ttl_seconds: 3600,
            chunk_cache_ttl_seconds: 1800,
            enable_lru_eviction: true,
            enable_access_tracking: true,
        };
        SmartCacheManager::new(config).await.unwrap()
    });
    
    let test_data_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB
    
    for size in test_data_sizes {
        let data = vec![0u8; size];
        let file_key = format!("cache_test_{}", size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark cache insertion
        group.bench_with_input(
            BenchmarkId::new("cache_insert", size),
            &(file_key.clone(), data.clone()),
            |b, (key, data)| {
                b.to_async(&rt).iter(|| async {
                    cache.cache_file_data(black_box(key), black_box(data.clone())).await;
                });
            },
        );
        
        // Insert data for retrieval benchmark
        rt.block_on(cache.cache_file_data(&file_key, data.clone()));
        
        // Benchmark cache retrieval
        group.bench_with_input(
            BenchmarkId::new("cache_retrieve", size),
            &file_key,
            |b, key| {
                b.to_async(&rt).iter(|| async {
                    let result = cache.get_cached_file(black_box(key)).await;
                    black_box(result);
                });
            },
        );
        
        // Benchmark cache eviction
        group.bench_with_input(
            BenchmarkId::new("cache_evict", size),
            &file_key,
            |b, key| {
                b.to_async(&rt).iter(|| async {
                    cache.evict_file(black_box(key)).await;
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");
    
    let data_sizes = vec![1024, 10240, 102400, 1048576]; // 1KB to 1MB
    
    for size in data_sizes {
        let data = (0..size).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark data chunking
        group.bench_with_input(BenchmarkId::new("data_chunking", size), &data, |b, data| {
            b.iter(|| {
                let chunk_size = 65536; // 64KB chunks
                let chunks: Vec<_> = black_box(data)
                    .chunks(chunk_size)
                    .map(|chunk| chunk.to_vec())
                    .collect();
                black_box(chunks);
            });
        });
        
        // Benchmark data compression (if available)
        #[cfg(feature = "compression")]
        group.bench_with_input(BenchmarkId::new("compression", size), &data, |b, data| {
            b.iter(|| {
                let compressed = compress_data(black_box(data));
                black_box(compressed);
            });
        });
        
        // Benchmark checksum calculation
        group.bench_with_input(BenchmarkId::new("checksum_blake3", size), &data, |b, data| {
            b.iter(|| {
                let hash = blake3::hash(black_box(data));
                black_box(hash);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("checksum_sha256", size), &data, |b, data| {
            use sha2::{Sha256, Digest};
            b.iter(|| {
                let mut hasher = Sha256::new();
                hasher.update(black_box(data));
                let hash = hasher.finalize();
                black_box(hash);
            });
        });
        
        // Benchmark data validation
        group.bench_with_input(BenchmarkId::new("data_validation", size), &data, |b, data| {
            b.iter(|| {
                // Simulate data validation checks
                let is_valid = !black_box(data).is_empty() 
                    && data.len() <= 100 * 1024 * 1024  // Max 100MB
                    && data.iter().any(|&b| b != 0);     // Not all zeros
                black_box(is_valid);
            });
        });
    }
    
    group.finish();
}

fn benchmark_metadata_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_operations");
    
    // Create sample metadata
    let file_entry = FileEntry {
        id: 1,
        name: "benchmark_metadata_test".to_string(),
        file_key: "abc123def456ghi789jkl".to_string(),
        original_filename: "test_document_for_metadata_benchmark.pdf".to_string(),
        file_size: 1024 * 1024, // 1MB
        upload_time: chrono::Local::now(),
        tags: vec![
            "benchmark".to_string(),
            "metadata".to_string(),
            "performance".to_string(),
            "test".to_string(),
        ],
        public_key_hex: "0123456789abcdef0123456789abcdef".to_string(),
        chunks_total: 16,
        chunks_healthy: 16,
    };
    
    // Benchmark metadata serialization
    group.bench_function("metadata_json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&file_entry)).unwrap();
            black_box(json);
        });
    });
    
    let json_metadata = serde_json::to_string(&file_entry).unwrap();
    group.bench_function("metadata_json_deserialize", |b| {
        b.iter(|| {
            let entry: FileEntry = serde_json::from_str(black_box(&json_metadata)).unwrap();
            black_box(entry);
        });
    });
    
    // Benchmark metadata validation
    group.bench_function("metadata_validation", |b| {
        b.iter(|| {
            let entry = black_box(&file_entry);
            let is_valid = !entry.name.is_empty()
                && !entry.file_key.is_empty()
                && !entry.original_filename.is_empty()
                && entry.file_size > 0
                && entry.chunks_total > 0
                && entry.chunks_healthy <= entry.chunks_total;
            black_box(is_valid);
        });
    });
    
    // Benchmark tag operations
    let tags = &file_entry.tags;
    group.bench_function("tag_operations", |b| {
        b.iter(|| {
            let tag_string = black_box(tags).join(",");
            let parsed_tags: Vec<&str> = tag_string.split(',').collect();
            let contains_benchmark = parsed_tags.contains(&"benchmark");
            black_box((tag_string, parsed_tags, contains_benchmark));
        });
    });
    
    group.finish();
}

fn benchmark_storage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_patterns");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("patterns.db");
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Benchmark batch operations
    group.bench_function("batch_insert", |b| {
        b.iter_with_setup(
            || {
                let upload_time = chrono::Local::now();
                let files: Vec<_> = (0..100)
                    .map(|i| {
                        (
                            format!("batch_file_{}", i),
                            format!("batch_key_{}", i),
                            format!("batch_{}.txt", i),
                            1024_u64,
                            upload_time,
                            vec!["batch".to_string()],
                            "batch_pubkey".to_string(),
                        )
                    })
                    .collect();
                files
            },
            |files| {
                for (name, key, filename, size, time, tags, pubkey) in black_box(files) {
                    db.store_file(&name, &key, &filename, size, time, &tags, &pubkey).unwrap();
                }
            },
        );
    });
    
    // Benchmark sequential access pattern
    group.bench_function("sequential_access", |b| {
        // Setup sequential data
        let upload_time = chrono::Local::now();
        for i in 0..1000 {
            db.store_file(
                &format!("seq_{}", i),
                &format!("seq_key_{}", i),
                &format!("seq_{}.txt", i),
                1024,
                upload_time,
                &vec!["sequential".to_string()],
                "seq_pubkey"
            ).unwrap();
        }
        
        b.iter(|| {
            for i in 0..100 {
                let name = format!("seq_{}", i);
                let result = db.get_file_by_name(black_box(&name)).unwrap();
                black_box(result);
            }
        });
    });
    
    // Benchmark random access pattern
    group.bench_function("random_access", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let i = fastrand::usize(..1000);
                let name = format!("seq_{}", i);
                let result = db.get_file_by_name(black_box(&name)).unwrap();
                black_box(result);
            }
        });
    });
    
    group.finish();
}

fn benchmark_concurrent_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_storage");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("concurrent.db");
    let db = std::sync::Arc::new(ThreadSafeDatabaseManager::new(&db_path).unwrap());
    
    // Benchmark concurrent writes
    group.bench_function("concurrent_writes", |b| {
        b.to_async(&rt).iter(|| async {
            let upload_time = chrono::Local::now();
            let tasks: Vec<_> = (0..10)
                .map(|i| {
                    let db = db.clone();
                    tokio::spawn(async move {
                        let name = format!("concurrent_write_{}", i);
                        let key = format!("concurrent_key_{}", i);
                        db.store_file(
                            &name,
                            &key,
                            &format!("concurrent_{}.txt", i),
                            1024,
                            upload_time,
                            &vec!["concurrent".to_string()],
                            "concurrent_pubkey"
                        )
                    })
                })
                .collect();
            
            let results = futures::future::join_all(tasks).await;
            black_box(results);
        });
    });
    
    // Setup data for concurrent reads
    let upload_time = chrono::Local::now();
    for i in 0..100 {
        db.store_file(
            &format!("read_concurrent_{}", i),
            &format!("read_key_{}", i),
            &format!("read_{}.txt", i),
            1024,
            upload_time,
            &vec!["read".to_string()],
            "read_pubkey"
        ).unwrap();
    }
    
    // Benchmark concurrent reads
    group.bench_function("concurrent_reads", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks: Vec<_> = (0..20)
                .map(|_| {
                    let db = db.clone();
                    tokio::spawn(async move {
                        let i = fastrand::usize(..100);
                        let name = format!("read_concurrent_{}", i);
                        db.get_file_by_name(&name)
                    })
                })
                .collect();
            
            let results = futures::future::join_all(tasks).await;
            black_box(results);
        });
    });
    
    group.finish();
}

#[cfg(feature = "compression")]
fn compress_data(data: &[u8]) -> Vec<u8> {
    // Placeholder for compression implementation
    // In a real scenario, this would use a compression library like flate2
    data.to_vec()
}

// Configure benchmark groups
criterion_group!(
    name = storage_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets = 
        benchmark_database_operations,
        benchmark_thread_safe_database,
        benchmark_file_operations,
        benchmark_smart_cache,
        benchmark_data_processing,
        benchmark_metadata_operations,
        benchmark_storage_patterns,
        benchmark_concurrent_storage
);

criterion_main!(storage_benches);