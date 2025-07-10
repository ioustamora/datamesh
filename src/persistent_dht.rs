use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::num::NonZeroUsize;
use tokio::sync::RwLock;
use tokio::time::{interval, Interval};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use lru::LruCache;
use libp2p::kad::{store::RecordStore, Record, RecordKey};
use libp2p::PeerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableRecord {
    key: Vec<u8>,
    value: Vec<u8>,
    publisher: Option<Vec<u8>>,
    expires: Option<u64>,
}

impl From<Record> for SerializableRecord {
    fn from(record: Record) -> Self {
        Self {
            key: record.key.to_vec(),
            value: record.value,
            publisher: record.publisher.map(|p| p.to_bytes()),
            expires: record.expires.map(|e| {
                let now = std::time::Instant::now();
                if e > now {
                    e.duration_since(now).as_secs()
                } else {
                    0
                }
            }),
        }
    }
}

impl From<SerializableRecord> for Record {
    fn from(sr: SerializableRecord) -> Self {
        Record {
            key: RecordKey::from(sr.key),
            value: sr.value,
            publisher: sr.publisher.map(|bytes| {
                PeerId::from_bytes(&bytes).unwrap_or_else(|_| PeerId::random())
            }),
            expires: sr.expires.map(|secs| {
                std::time::Instant::now() + Duration::from_secs(secs)
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub ttl: u64,
    pub created_at: u64,
    pub replication_count: u8,
    pub size: usize,
}

impl ChunkMetadata {
    pub fn new(ttl: Duration, replication_count: u8, size: usize) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            ttl: ttl.as_secs(),
            created_at,
            replication_count,
            size,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.created_at + self.ttl
    }
}

#[derive(Clone)]
pub struct PersistentDHTStorage {
    db: Arc<Db>,
    cache: Arc<RwLock<LruCache<RecordKey, Record>>>,
    metadata_cache: Arc<RwLock<LruCache<RecordKey, ChunkMetadata>>>,
    replication_factor: u8,
    cleanup_interval: Duration,
    peer_id: PeerId,
}

impl PersistentDHTStorage {
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        cache_size: usize,
        replication_factor: u8,
        cleanup_interval: Duration,
        peer_id: PeerId,
    ) -> Result<Self> {
        let db = sled::open(db_path)?;
        
        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))),
            metadata_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))),
            replication_factor,
            cleanup_interval,
            peer_id,
        })
    }

    pub async fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let db = self.db.clone();
        let metadata_cache = self.metadata_cache.clone();
        let cache = self.cache.clone();
        let cleanup_interval = self.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                if let Err(e) = Self::cleanup_expired_chunks(&db, &metadata_cache, &cache).await {
                    eprintln!("Error during cleanup: {}", e);
                }
            }
        })
    }

    async fn cleanup_expired_chunks(
        db: &Arc<Db>,
        metadata_cache: &Arc<RwLock<LruCache<RecordKey, ChunkMetadata>>>,
        cache: &Arc<RwLock<LruCache<RecordKey, Record>>>,
    ) -> Result<()> {
        let mut expired_keys = Vec::new();
        
        // Iterate through all keys with metadata prefix
        for item in db.scan_prefix("metadata:") {
            let (key, value) = item?;
            let key_str = String::from_utf8(key.to_vec())?;
            
            if let Ok(metadata) = bincode::deserialize::<ChunkMetadata>(&value) {
                if metadata.is_expired() {
                    let chunk_key = key_str.strip_prefix("metadata:").unwrap();
                    expired_keys.push(chunk_key.to_string());
                }
            }
        }
        
        for key in expired_keys {
            let record_key = RecordKey::new(&key);
            
            db.remove(format!("chunk:{}", key))?;
            db.remove(format!("metadata:{}", key))?;
            
            let mut cache_guard = cache.write().await;
            cache_guard.pop(&record_key);
            
            let mut metadata_guard = metadata_cache.write().await;
            metadata_guard.pop(&record_key);
        }
        
        Ok(())
    }

    async fn get_from_cache(&self, key: &RecordKey) -> Option<Record> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }

    async fn put_in_cache(&self, key: RecordKey, record: Record) {
        let mut cache = self.cache.write().await;
        cache.put(key, record);
    }

    async fn get_metadata_from_cache(&self, key: &RecordKey) -> Option<ChunkMetadata> {
        let mut cache = self.metadata_cache.write().await;
        cache.get(key).cloned()
    }

    async fn put_metadata_in_cache(&self, key: RecordKey, metadata: ChunkMetadata) {
        let mut cache = self.metadata_cache.write().await;
        cache.put(key, metadata);
    }

    fn key_to_string(key: &RecordKey) -> String {
        hex::encode(key.as_ref())
    }
}

impl RecordStore for PersistentDHTStorage {
    type RecordsIter<'a> = std::vec::IntoIter<std::borrow::Cow<'a, Record>>;
    type ProvidedIter<'a> = std::vec::IntoIter<std::borrow::Cow<'a, libp2p::kad::ProviderRecord>>;

    fn get(&self, key: &RecordKey) -> Option<std::borrow::Cow<Record>> {
        let key_str = Self::key_to_string(key);
        
        // Try cache first - use try_read to avoid blocking
        if let Ok(cache) = self.cache.try_read() {
            if let Some(record) = cache.peek(key) {
                return Some(std::borrow::Cow::Owned(record.clone()));
            }
        }
        
        // Try database
        if let Ok(Some(data)) = self.db.get(format!("chunk:{}", key_str)) {
            if let Ok(serializable_record) = bincode::deserialize::<SerializableRecord>(&data) {
                let record = Record::from(serializable_record);
                // Update cache asynchronously without blocking
                if let Ok(mut cache) = self.cache.try_write() {
                    cache.put(key.clone(), record.clone());
                }
                return Some(std::borrow::Cow::Owned(record));
            }
        }
        
        None
    }

    fn put(&mut self, record: Record) -> libp2p::kad::store::Result<()> {
        let key = record.key.clone();
        let key_str = Self::key_to_string(&key);
        
        // Create metadata
        let metadata = ChunkMetadata::new(
            Duration::from_secs(24 * 60 * 60), // 24 hours default TTL
            self.replication_factor,
            record.value.len(),
        );
        
        // Store in database
        let serializable_record = SerializableRecord::from(record.clone());
        let record_data = bincode::serialize(&serializable_record)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        let metadata_data = bincode::serialize(&metadata)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        
        self.db.insert(format!("chunk:{}", key_str), record_data)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        self.db.insert(format!("metadata:{}", key_str), metadata_data)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        
        // Update caches asynchronously without blocking
        if let Ok(mut cache) = self.cache.try_write() {
            cache.put(key.clone(), record);
        }
        if let Ok(mut metadata_cache) = self.metadata_cache.try_write() {
            metadata_cache.put(key, metadata);
        }
        
        Ok(())
    }

    fn remove(&mut self, key: &RecordKey) {
        let key_str = Self::key_to_string(key);
        
        // Remove from database
        let _ = self.db.remove(format!("chunk:{}", key_str));
        let _ = self.db.remove(format!("metadata:{}", key_str));
        
        // Remove from caches asynchronously without blocking
        if let Ok(mut cache) = self.cache.try_write() {
            cache.pop(key);
        }
        if let Ok(mut metadata_cache) = self.metadata_cache.try_write() {
            metadata_cache.pop(key);
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        let mut records = Vec::new();
        
        for item in self.db.scan_prefix("chunk:") {
            if let Ok((key, value)) = item {
                let key_str = String::from_utf8_lossy(&key);
                if key_str.starts_with("chunk:") {
                    if let Ok(serializable_record) = bincode::deserialize::<SerializableRecord>(&value) {
                        let record = Record::from(serializable_record);
                        records.push(std::borrow::Cow::Owned(record));
                    }
                }
            }
        }
        
        records.into_iter()
    }

    fn add_provider(&mut self, _record: libp2p::kad::ProviderRecord) -> libp2p::kad::store::Result<()> {
        // Provider records are handled separately in libp2p
        Ok(())
    }

    fn providers(&self, _key: &RecordKey) -> Vec<libp2p::kad::ProviderRecord> {
        Vec::new()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        Vec::new().into_iter()
    }

    fn remove_provider(&mut self, _key: &RecordKey, _provider: &PeerId) {
        // Provider records are handled separately in libp2p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use libp2p::identity::Keypair;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_persistent_storage() {
        let temp_dir = tempdir().unwrap();
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        let mut storage = PersistentDHTStorage::new(
            temp_dir.path().join("test_db"),
            100,
            3,
            Duration::from_secs(3600),
            peer_id,
        ).unwrap();
        
        let key = RecordKey::new(&"test_key");
        let record = Record::new(key.clone(), vec![1, 2, 3, 4]);
        
        // Test put
        storage.put(record.clone()).unwrap();
        
        // Test get
        let retrieved = storage.get(&key).unwrap();
        assert_eq!(retrieved.value, record.value);
        
        // Test remove
        storage.remove(&key);
        assert!(storage.get(&key).is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_metadata_expiration() {
        let metadata = ChunkMetadata::new(
            Duration::from_secs(1),
            3,
            100,
        );
        
        assert!(!metadata.is_expired());
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(metadata.is_expired());
    }
}