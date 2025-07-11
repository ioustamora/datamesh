/// Cryptographic Performance Benchmarks for DataMesh
///
/// Comprehensive benchmarking of all cryptographic operations used in DataMesh
/// including encryption, hashing, key generation, and signature operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

// DataMesh cryptographic modules
use datamesh::key_manager::KeyManager;
use datamesh::secure_random::{generate_secure_bytes, generate_secure_nonce, generate_secure_salt};

fn benchmark_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_generation");
    
    group.bench_function("new_key_manager", |b| {
        b.iter(|| {
            let km = KeyManager::new().unwrap();
            black_box(km);
        });
    });
    
    group.finish();
}

fn benchmark_encryption_decryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_decryption");
    
    let km = KeyManager::new().unwrap();
    let data_sizes = vec![
        1024,           // 1 KB
        10 * 1024,      // 10 KB
        100 * 1024,     // 100 KB
        1024 * 1024,    // 1 MB
        10 * 1024 * 1024, // 10 MB
    ];
    
    for size in data_sizes {
        let data = generate_secure_bytes(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        group.bench_with_input(BenchmarkId::new("encrypt", size), &data, |b, data| {
            b.iter(|| {
                let encrypted = km.encrypt(black_box(data)).unwrap();
                black_box(encrypted);
            });
        });
        
        // Benchmark decryption
        let encrypted = km.encrypt(&data).unwrap();
        group.bench_with_input(BenchmarkId::new("decrypt", size), &encrypted, |b, encrypted| {
            b.iter(|| {
                let decrypted = km.decrypt(black_box(encrypted)).unwrap();
                black_box(decrypted);
            });
        });
        
        // Benchmark round-trip
        group.bench_with_input(BenchmarkId::new("round_trip", size), &data, |b, data| {
            b.iter(|| {
                let encrypted = km.encrypt(black_box(data)).unwrap();
                let decrypted = km.decrypt(&encrypted).unwrap();
                black_box(decrypted);
            });
        });
    }
    
    group.finish();
}

fn benchmark_hashing_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    
    let data_sizes = vec![
        1024,           // 1 KB
        10 * 1024,      // 10 KB
        100 * 1024,     // 100 KB
        1024 * 1024,    // 1 MB
        10 * 1024 * 1024, // 10 MB
    ];
    
    for size in data_sizes {
        let data = generate_secure_bytes(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // BLAKE3 hashing
        group.bench_with_input(BenchmarkId::new("blake3", size), &data, |b, data| {
            b.iter(|| {
                let hash = blake3::hash(black_box(data));
                black_box(hash);
            });
        });
        
        // SHA256 for comparison
        group.bench_with_input(BenchmarkId::new("sha256", size), &data, |b, data| {
            use sha2::{Sha256, Digest};
            b.iter(|| {
                let mut hasher = Sha256::new();
                hasher.update(black_box(data));
                let hash = hasher.finalize();
                black_box(hash);
            });
        });
    }
    
    group.finish();
}

fn benchmark_random_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_generation");
    
    // Benchmark different random generation functions
    group.bench_function("secure_nonce", |b| {
        b.iter(|| {
            let nonce = generate_secure_nonce();
            black_box(nonce);
        });
    });
    
    group.bench_function("secure_salt", |b| {
        b.iter(|| {
            let salt = generate_secure_salt();
            black_box(salt);
        });
    });
    
    // Benchmark secure bytes generation for different sizes
    let byte_sizes = vec![16, 32, 64, 128, 256, 512, 1024];
    
    for size in byte_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("secure_bytes", size), &size, |b, &size| {
            b.iter(|| {
                let bytes = generate_secure_bytes(black_box(size));
                black_box(bytes);
            });
        });
    }
    
    group.finish();
}

fn benchmark_key_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_serialization");
    
    let km = KeyManager::new().unwrap();
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let key_path = temp_file.path();
    
    group.bench_function("save_key", |b| {
        b.iter(|| {
            km.save_to_file(black_box(key_path)).unwrap();
        });
    });
    
    // Ensure key is saved for loading benchmark
    km.save_to_file(key_path).unwrap();
    
    group.bench_function("load_key", |b| {
        b.iter(|| {
            let loaded_km = KeyManager::load_from_file(black_box(key_path)).unwrap();
            black_box(loaded_km);
        });
    });
    
    group.finish();
}

fn benchmark_password_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_hashing");
    
    let passwords = vec![
        "short",
        "medium_length_password",
        "very_long_password_with_many_characters_and_symbols!@#$%^&*()",
    ];
    
    for password in passwords {
        group.bench_with_input(
            BenchmarkId::new("argon2_hash", password.len()),
            &password,
            |b, password| {
                use argon2::{Argon2, PasswordHasher};
                use argon2::password_hash::{rand_core::OsRng, SaltString};
                
                b.iter(|| {
                    let salt = SaltString::generate(&mut OsRng);
                    let argon2 = Argon2::default();
                    let hash = argon2.hash_password(black_box(password.as_bytes()), &salt).unwrap();
                    black_box(hash);
                });
            },
        );
        
        // Benchmark password verification
        use argon2::{Argon2, PasswordHasher, PasswordVerifier};
        use argon2::password_hash::{rand_core::OsRng, SaltString};
        
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("argon2_verify", password.len()),
            &(password, hash),
            |b, (password, hash)| {
                b.iter(|| {
                    let argon2 = Argon2::default();
                    let result = argon2.verify_password(black_box(password.as_bytes()), hash);
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_aes_gcm_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("aes_gcm");
    
    use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, AeadInPlace};
    use aes_gcm::aead::generic_array::GenericArray;
    
    let key = Key::<Aes256Gcm>::from_slice(&generate_secure_bytes(32));
    let cipher = Aes256Gcm::new(key);
    
    let data_sizes = vec![
        1024,           // 1 KB
        10 * 1024,      // 10 KB
        100 * 1024,     // 100 KB
        1024 * 1024,    // 1 MB
    ];
    
    for size in data_sizes {
        let mut data = generate_secure_bytes(size);
        let nonce_bytes = generate_secure_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        group.bench_with_input(BenchmarkId::new("encrypt", size), &mut data, |b, data| {
            b.iter(|| {
                let mut buffer = data.clone();
                let tag = cipher.encrypt_in_place_detached(black_box(nonce), b"", &mut buffer).unwrap();
                black_box((buffer, tag));
            });
        });
        
        // Prepare encrypted data for decryption benchmark
        let mut encrypted_data = data.clone();
        let tag = cipher.encrypt_in_place_detached(nonce, b"", &mut encrypted_data).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("decrypt", size),
            &(encrypted_data, tag),
            |b, (encrypted_data, tag)| {
                b.iter(|| {
                    let mut buffer = encrypted_data.clone();
                    cipher.decrypt_in_place_detached(black_box(nonce), b"", &mut buffer, tag).unwrap();
                    black_box(buffer);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_reed_solomon_coding(c: &mut Criterion) {
    let mut group = c.benchmark_group("reed_solomon");
    
    use reed_solomon_erasure::galois_8::ReedSolomon;
    
    // DataMesh uses 4 data shards + 2 parity shards
    let data_shards = 4;
    let parity_shards = 2;
    let rs = ReedSolomon::new(data_shards, parity_shards).unwrap();
    
    let chunk_sizes = vec![
        1024,           // 1 KB chunks
        4 * 1024,       // 4 KB chunks
        16 * 1024,      // 16 KB chunks
        64 * 1024,      // 64 KB chunks
    ];
    
    for chunk_size in chunk_sizes {
        let data = generate_secure_bytes(chunk_size * data_shards);
        let mut shards: Vec<Vec<u8>> = data
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        // Add empty parity shards
        for _ in 0..parity_shards {
            shards.push(vec![0; chunk_size]);
        }
        
        group.throughput(Throughput::Bytes((chunk_size * data_shards) as u64));
        
        group.bench_with_input(
            BenchmarkId::new("encode", chunk_size),
            &mut shards,
            |b, shards| {
                b.iter(|| {
                    let mut test_shards = shards.clone();
                    rs.encode(&mut test_shards).unwrap();
                    black_box(test_shards);
                });
            },
        );
        
        // Prepare encoded shards for reconstruction benchmark
        let mut encoded_shards = shards.clone();
        rs.encode(&mut encoded_shards).unwrap();
        
        // Simulate some data loss
        encoded_shards[0].clear(); // Lost shard
        encoded_shards[1].clear(); // Lost shard
        
        group.bench_with_input(
            BenchmarkId::new("reconstruct", chunk_size),
            &encoded_shards,
            |b, encoded_shards| {
                b.iter(|| {
                    let mut test_shards = encoded_shards.clone();
                    rs.reconstruct(&mut test_shards).unwrap();
                    black_box(test_shards);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_base58_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("base58");
    
    let data_sizes = vec![16, 32, 64, 128, 256];
    
    for size in data_sizes {
        let data = generate_secure_bytes(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        group.bench_with_input(BenchmarkId::new("encode", size), &data, |b, data| {
            b.iter(|| {
                let encoded = bs58::encode(black_box(data)).into_string();
                black_box(encoded);
            });
        });
        
        let encoded = bs58::encode(&data).into_string();
        group.bench_with_input(BenchmarkId::new("decode", size), &encoded, |b, encoded| {
            b.iter(|| {
                let decoded = bs58::decode(black_box(encoded)).into_vec().unwrap();
                black_box(decoded);
            });
        });
    }
    
    group.finish();
}

fn benchmark_hex_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex_encoding");
    
    let data_sizes = vec![16, 32, 64, 128, 256, 512];
    
    for size in data_sizes {
        let data = generate_secure_bytes(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        group.bench_with_input(BenchmarkId::new("encode", size), &data, |b, data| {
            b.iter(|| {
                let encoded = hex::encode(black_box(data));
                black_box(encoded);
            });
        });
        
        let encoded = hex::encode(&data);
        group.bench_with_input(BenchmarkId::new("decode", size), &encoded, |b, encoded| {
            b.iter(|| {
                let decoded = hex::decode(black_box(encoded)).unwrap();
                black_box(decoded);
            });
        });
    }
    
    group.finish();
}

fn benchmark_jwt_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt");
    
    use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        iat: usize,
        user_id: String,
        email: String,
    }
    
    let secret = "test_secret_key_for_jwt_benchmarking";
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    
    let claims = Claims {
        sub: "user123".to_string(),
        exp: 2000000000, // Far future
        iat: 1000000000, // Past
        user_id: "user123".to_string(),
        email: "user@example.com".to_string(),
    };
    
    group.bench_function("encode_jwt", |b| {
        b.iter(|| {
            let token = encode(&Header::default(), black_box(&claims), &encoding_key).unwrap();
            black_box(token);
        });
    });
    
    let token = encode(&Header::default(), &claims, &encoding_key).unwrap();
    let validation = Validation::default();
    
    group.bench_function("decode_jwt", |b| {
        b.iter(|| {
            let decoded = decode::<Claims>(black_box(&token), &decoding_key, &validation).unwrap();
            black_box(decoded);
        });
    });
    
    group.finish();
}

// Configure benchmark groups
criterion_group!(
    name = crypto_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets = 
        benchmark_key_generation,
        benchmark_encryption_decryption,
        benchmark_hashing_algorithms,
        benchmark_random_generation,
        benchmark_key_serialization,
        benchmark_password_hashing,
        benchmark_aes_gcm_encryption,
        benchmark_reed_solomon_coding,
        benchmark_base58_encoding,
        benchmark_hex_encoding,
        benchmark_jwt_operations
);

criterion_main!(crypto_benches);