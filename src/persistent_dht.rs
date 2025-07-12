use anyhow::Result;
use libp2p::kad::{store::RecordStore, Record, RecordKey};
use libp2p::{Multiaddr, PeerId};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;

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
            publisher: sr
                .publisher
                .map(|bytes| PeerId::from_bytes(&bytes).unwrap_or_else(|_| PeerId::random())),
            expires: sr
                .expires
                .map(|secs| std::time::Instant::now() + Duration::from_secs(secs)),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: Vec<u8>,
    pub addresses: Vec<String>,
    pub last_seen: u64,
    pub connection_count: u32,
    pub distance: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTableEntry {
    pub peer_info: PeerInfo,
    pub bucket_index: usize,
    pub reliability_score: f64,
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
    peer_cache: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
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
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
            metadata_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
            peer_cache: Arc::new(RwLock::new(HashMap::new())),
            replication_factor,
            cleanup_interval,
            peer_id,
        })
    }

    pub async fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let storage = self.clone();
        let cleanup_interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

            loop {
                interval.tick().await;
                
                // Clean up expired chunks
                if let Err(e) = Self::cleanup_expired_chunks(&storage.db, &storage.metadata_cache, &storage.cache).await {
                    eprintln!("Error during chunk cleanup: {}", e);
                }
                
                // Clean up stale peers (older than 7 days)
                if let Err(e) = storage.cleanup_stale_peers(24 * 7).await {
                    eprintln!("Error during peer cleanup: {}", e);
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

    /// Store peer information in the routing table
    pub async fn store_peer(&self, peer_id: PeerId, addresses: Vec<Multiaddr>) -> Result<()> {
        let peer_info = PeerInfo {
            peer_id: peer_id.to_bytes(),
            addresses: addresses.iter().map(|addr| addr.to_string()).collect(),
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            connection_count: 1,
            distance: None,
        };

        // Store in memory cache
        {
            let mut peer_cache = self.peer_cache.write().await;
            peer_cache.insert(peer_id, peer_info.clone());
        }

        // Store in persistent database
        let key = format!("peer:{}", hex::encode(peer_id.to_bytes()));
        let value = bincode::serialize(&peer_info)?;
        self.db.insert(key, value)?;

        Ok(())
    }

    /// Retrieve peer information from the routing table
    pub async fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        // Try memory cache first
        {
            let peer_cache = self.peer_cache.read().await;
            if let Some(peer_info) = peer_cache.get(peer_id) {
                return Some(peer_info.clone());
            }
        }

        // Try persistent storage
        let key = format!("peer:{}", hex::encode(peer_id.to_bytes()));
        if let Ok(Some(data)) = self.db.get(key) {
            if let Ok(peer_info) = bincode::deserialize::<PeerInfo>(&data) {
                // Update memory cache
                {
                    let mut peer_cache = self.peer_cache.write().await;
                    peer_cache.insert(*peer_id, peer_info.clone());
                }
                return Some(peer_info);
            }
        }

        None
    }

    /// Get all known peers from the routing table
    pub async fn get_all_peers(&self) -> Vec<(PeerId, PeerInfo)> {
        let mut peers = Vec::new();

        // First, add all peers from memory cache
        {
            let peer_cache = self.peer_cache.read().await;
            for (peer_id, peer_info) in peer_cache.iter() {
                peers.push((*peer_id, peer_info.clone()));
            }
        }

        // Then, add any peers from persistent storage that aren't in memory
        let peer_ids_in_memory: std::collections::HashSet<PeerId> = peers.iter().map(|(id, _)| *id).collect();
        
        for item in self.db.scan_prefix("peer:") {
            if let Ok((key, value)) = item {
                let key_str = String::from_utf8_lossy(&key);
                if let Some(peer_id_hex) = key_str.strip_prefix("peer:") {
                    if let Ok(peer_id_bytes) = hex::decode(peer_id_hex) {
                        if let Ok(peer_id) = PeerId::from_bytes(&peer_id_bytes) {
                            if !peer_ids_in_memory.contains(&peer_id) {
                                if let Ok(peer_info) = bincode::deserialize::<PeerInfo>(&value) {
                                    peers.push((peer_id, peer_info));
                                }
                            }
                        }
                    }
                }
            }
        }

        peers
    }

    /// Update peer's last seen timestamp and connection count
    pub async fn update_peer_activity(&self, peer_id: PeerId) -> Result<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update memory cache
        {
            let mut peer_cache = self.peer_cache.write().await;
            if let Some(peer_info) = peer_cache.get_mut(&peer_id) {
                peer_info.last_seen = current_time;
                peer_info.connection_count += 1;
            }
        }

        // Update persistent storage
        let key = format!("peer:{}", hex::encode(peer_id.to_bytes()));
        if let Ok(Some(data)) = self.db.get(&key) {
            if let Ok(mut peer_info) = bincode::deserialize::<PeerInfo>(&data) {
                peer_info.last_seen = current_time;
                peer_info.connection_count += 1;
                
                let updated_value = bincode::serialize(&peer_info)?;
                self.db.insert(key, updated_value)?;
            }
        }

        Ok(())
    }

    /// Remove old/stale peers from the routing table
    pub async fn cleanup_stale_peers(&self, max_age_hours: u64) -> Result<()> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (max_age_hours * 3600);

        let mut stale_peers = Vec::new();

        // Find stale peers in persistent storage
        for item in self.db.scan_prefix("peer:") {
            if let Ok((key, value)) = item {
                let key_str = String::from_utf8_lossy(&key);
                if let Some(peer_id_hex) = key_str.strip_prefix("peer:") {
                    if let Ok(peer_info) = bincode::deserialize::<PeerInfo>(&value) {
                        if peer_info.last_seen < cutoff_time {
                            if let Ok(peer_id_bytes) = hex::decode(peer_id_hex) {
                                if let Ok(peer_id) = PeerId::from_bytes(&peer_id_bytes) {
                                    stale_peers.push((peer_id, key_str.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove stale peers
        for (peer_id, key) in stale_peers {
            // Remove from persistent storage
            self.db.remove(key)?;
            
            // Remove from memory cache
            {
                let mut peer_cache = self.peer_cache.write().await;
                peer_cache.remove(&peer_id);
            }
        }

        Ok(())
    }

    /// Load routing table from persistent storage into memory
    pub async fn load_routing_table(&self) -> Result<()> {
        let mut peer_cache = self.peer_cache.write().await;
        
        for item in self.db.scan_prefix("peer:") {
            if let Ok((key, value)) = item {
                let key_str = String::from_utf8_lossy(&key);
                if let Some(peer_id_hex) = key_str.strip_prefix("peer:") {
                    if let Ok(peer_id_bytes) = hex::decode(peer_id_hex) {
                        if let Ok(peer_id) = PeerId::from_bytes(&peer_id_bytes) {
                            if let Ok(peer_info) = bincode::deserialize::<PeerInfo>(&value) {
                                peer_cache.insert(peer_id, peer_info);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get count of known peers
    pub async fn get_peer_count(&self) -> usize {
        let peer_cache = self.peer_cache.read().await;
        peer_cache.len()
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
        let metadata_data =
            bincode::serialize(&metadata).map_err(|_| libp2p::kad::store::Error::MaxRecords)?;

        self.db
            .insert(format!("chunk:{}", key_str), record_data)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        self.db
            .insert(format!("metadata:{}", key_str), metadata_data)
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
                    if let Ok(serializable_record) =
                        bincode::deserialize::<SerializableRecord>(&value)
                    {
                        let record = Record::from(serializable_record);
                        records.push(std::borrow::Cow::Owned(record));
                    }
                }
            }
        }

        records.into_iter()
    }

    fn add_provider(
        &mut self,
        _record: libp2p::kad::ProviderRecord,
    ) -> libp2p::kad::store::Result<()> {
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
    use libp2p::identity::Keypair;
    use tempfile::tempdir;

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
        )
        .unwrap();

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
        let metadata = ChunkMetadata::new(Duration::from_secs(1), 3, 100);

        assert!(!metadata.is_expired());

        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(metadata.is_expired());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_peer_persistence() {
        let temp_dir = tempdir().unwrap();
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let test_peer_id = PeerId::random();

        let storage = PersistentDHTStorage::new(
            temp_dir.path().join("test_peer_db"),
            100,
            3,
            Duration::from_secs(3600),
            peer_id,
        )
        .unwrap();

        // Test storing a peer
        let addresses = vec!["/ip4/127.0.0.1/tcp/40871".parse().unwrap()];
        storage.store_peer(test_peer_id, addresses.clone()).await.unwrap();

        // Test retrieving the peer
        let retrieved_peer = storage.get_peer(&test_peer_id).await.unwrap();
        assert_eq!(retrieved_peer.peer_id, test_peer_id.to_bytes());
        assert_eq!(retrieved_peer.addresses.len(), 1);
        assert!(retrieved_peer.addresses[0].contains("127.0.0.1"));

        // Test updating peer activity
        storage.update_peer_activity(test_peer_id).await.unwrap();
        let updated_peer = storage.get_peer(&test_peer_id).await.unwrap();
        assert_eq!(updated_peer.connection_count, 2); // Should be incremented

        // Test getting all peers
        let all_peers = storage.get_all_peers().await;
        assert_eq!(all_peers.len(), 1);
        assert_eq!(all_peers[0].0, test_peer_id);

        // Test peer count
        let count = storage.get_peer_count().await;
        assert_eq!(count, 1);
    }
}
