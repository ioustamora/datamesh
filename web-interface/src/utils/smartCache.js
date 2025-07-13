/**
 * Advanced Multi-Layer Caching Strategy
 */

/**
 * Smart cache with multiple layers
 */
export class SmartCache {
  constructor(options = {}) {
    this.options = {
      memoryLimit: options.memoryLimit || 50 * 1024 * 1024, // 50MB
      diskLimit: options.diskLimit || 200 * 1024 * 1024, // 200MB
      defaultTTL: options.defaultTTL || 5 * 60 * 1000, // 5 minutes
      staleWhileRevalidate: options.staleWhileRevalidate || true,
      compression: options.compression || true,
      ...options
    }

    // Memory cache (fastest)
    this.memoryCache = new Map()
    this.memoryUsage = 0

    // Disk cache (persistent)
    this.diskCache = null
    this.initDiskCache()

    // Network cache (service worker)
    this.networkCache = null
    this.initNetworkCache()

    // Cache statistics
    this.stats = {
      hits: 0,
      misses: 0,
      memoryHits: 0,
      diskHits: 0,
      networkHits: 0,
      evictions: 0
    }
  }

  /**
   * Initialize IndexedDB for disk cache
   */
  async initDiskCache() {
    if (!('indexedDB' in window)) return

    try {
      this.diskCache = await this.openIndexedDB()
    } catch (error) {
      console.warn('Failed to initialize disk cache:', error)
    }
  }

  /**
   * Open IndexedDB connection
   */
  openIndexedDB() {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open('DataMeshCache', 1)

      request.onerror = () => reject(request.error)
      request.onsuccess = () => resolve(request.result)

      request.onupgradeneeded = (event) => {
        const db = event.target.result
        
        if (!db.objectStoreNames.contains('cache')) {
          const store = db.createObjectStore('cache', { keyPath: 'key' })
          store.createIndex('expires', 'expires', { unique: false })
          store.createIndex('lastAccessed', 'lastAccessed', { unique: false })
        }
      }
    })
  }

  /**
   * Initialize network cache (service worker)
   */
  async initNetworkCache() {
    if ('serviceWorker' in navigator && 'caches' in window) {
      try {
        this.networkCache = await caches.open('datamesh-v1')
      } catch (error) {
        console.warn('Failed to initialize network cache:', error)
      }
    }
  }

  /**
   * Get item from cache with priority fallback
   */
  async get(key, options = {}) {
    const startTime = performance.now()
    
    try {
      // Check memory cache first (fastest)
      const memoryResult = this.getFromMemory(key)
      if (memoryResult) {
        this.stats.hits++
        this.stats.memoryHits++
        return this.processResult(memoryResult, 'memory', startTime)
      }

      // Check disk cache second (persistent)
      const diskResult = await this.getFromDisk(key)
      if (diskResult) {
        this.stats.hits++
        this.stats.diskHits++
        
        // Promote to memory cache
        this.setInMemory(key, diskResult)
        
        return this.processResult(diskResult, 'disk', startTime)
      }

      // Check network cache third (offline support)
      const networkResult = await this.getFromNetwork(key)
      if (networkResult) {
        this.stats.hits++
        this.stats.networkHits++
        return this.processResult(networkResult, 'network', startTime)
      }

      // Cache miss
      this.stats.misses++
      
      // Fetch from source if provided
      if (options.fetchFn) {
        const freshData = await options.fetchFn()
        await this.set(key, freshData, options)
        return this.processResult({ data: freshData, fresh: true }, 'source', startTime)
      }

      return null
    } catch (error) {
      console.error('Cache get error:', error)
      this.stats.misses++
      return null
    }
  }

  /**
   * Process cache result
   */
  processResult(result, source, startTime) {
    const duration = performance.now() - startTime
    
    return {
      data: result.data,
      fromCache: source !== 'source',
      source,
      fresh: result.fresh || false,
      duration,
      expires: result.expires,
      lastAccessed: Date.now()
    }
  }

  /**
   * Set item in all cache layers
   */
  async set(key, data, options = {}) {
    const ttl = options.ttl || this.options.defaultTTL
    const expires = Date.now() + ttl
    const priority = options.priority || 'normal'
    
    const cacheItem = {
      key,
      data: await this.compressData(data),
      expires,
      lastAccessed: Date.now(),
      priority,
      size: this.calculateSize(data),
      fresh: true
    }

    // Set in memory cache
    this.setInMemory(key, cacheItem)

    // Set in disk cache
    await this.setInDisk(key, cacheItem)

    // Set in network cache for specific types
    if (this.shouldCacheInNetwork(key, options)) {
      await this.setInNetwork(key, cacheItem)
    }
  }

  /**
   * Get from memory cache
   */
  getFromMemory(key) {
    const item = this.memoryCache.get(key)
    
    if (!item) return null
    
    // Check expiration
    if (item.expires && Date.now() > item.expires) {
      this.memoryCache.delete(key)
      this.memoryUsage -= item.size
      return null
    }

    // Update last accessed
    item.lastAccessed = Date.now()
    
    return item
  }

  /**
   * Set in memory cache with LRU eviction
   */
  setInMemory(key, item) {
    // Check memory limit
    if (this.memoryUsage + item.size > this.options.memoryLimit) {
      this.evictMemoryItems(item.size)
    }

    this.memoryCache.set(key, item)
    this.memoryUsage += item.size
  }

  /**
   * Evict items from memory cache (LRU)
   */
  evictMemoryItems(neededSpace) {
    const items = Array.from(this.memoryCache.entries())
      .sort((a, b) => a[1].lastAccessed - b[1].lastAccessed)

    let freedSpace = 0
    for (const [key, item] of items) {
      this.memoryCache.delete(key)
      this.memoryUsage -= item.size
      freedSpace += item.size
      this.stats.evictions++

      if (freedSpace >= neededSpace) break
    }
  }

  /**
   * Get from disk cache (IndexedDB)
   */
  async getFromDisk(key) {
    if (!this.diskCache) return null

    try {
      const transaction = this.diskCache.transaction(['cache'], 'readonly')
      const store = transaction.objectStore('cache')
      const request = store.get(key)

      return new Promise((resolve, reject) => {
        request.onsuccess = () => {
          const result = request.result
          
          if (!result) {
            resolve(null)
            return
          }

          // Check expiration
          if (result.expires && Date.now() > result.expires) {
            this.deleteFromDisk(key)
            resolve(null)
            return
          }

          resolve({
            ...result,
            data: this.decompressData(result.data)
          })
        }
        
        request.onerror = () => reject(request.error)
      })
    } catch (error) {
      console.warn('Disk cache get error:', error)
      return null
    }
  }

  /**
   * Set in disk cache (IndexedDB)
   */
  async setInDisk(key, item) {
    if (!this.diskCache) return

    try {
      const transaction = this.diskCache.transaction(['cache'], 'readwrite')
      const store = transaction.objectStore('cache')
      
      await new Promise((resolve, reject) => {
        const request = store.put(item)
        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })

      // Clean up expired items periodically
      if (Math.random() < 0.1) { // 10% chance
        this.cleanupDiskCache()
      }
    } catch (error) {
      console.warn('Disk cache set error:', error)
    }
  }

  /**
   * Get from network cache (Cache API)
   */
  async getFromNetwork(key) {
    if (!this.networkCache) return null

    try {
      const response = await this.networkCache.match(key)
      if (response) {
        const data = await response.json()
        return {
          data,
          expires: null, // Network cache handles expiration
          fresh: false
        }
      }
    } catch (error) {
      console.warn('Network cache get error:', error)
    }

    return null
  }

  /**
   * Set in network cache (Cache API)
   */
  async setInNetwork(key, item) {
    if (!this.networkCache) return

    try {
      const response = new Response(JSON.stringify(item.data), {
        headers: {
          'Content-Type': 'application/json',
          'Cache-Control': `max-age=${Math.floor((item.expires - Date.now()) / 1000)}`
        }
      })

      await this.networkCache.put(key, response)
    } catch (error) {
      console.warn('Network cache set error:', error)
    }
  }

  /**
   * Delete from disk cache
   */
  async deleteFromDisk(key) {
    if (!this.diskCache) return

    try {
      const transaction = this.diskCache.transaction(['cache'], 'readwrite')
      const store = transaction.objectStore('cache')
      await new Promise((resolve, reject) => {
        const request = store.delete(key)
        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })
    } catch (error) {
      console.warn('Disk cache delete error:', error)
    }
  }

  /**
   * Cleanup expired items from disk cache
   */
  async cleanupDiskCache() {
    if (!this.diskCache) return

    try {
      const transaction = this.diskCache.transaction(['cache'], 'readwrite')
      const store = transaction.objectStore('cache')
      const index = store.index('expires')
      
      const request = index.openCursor(IDBKeyRange.upperBound(Date.now()))
      
      request.onsuccess = (event) => {
        const cursor = event.target.result
        if (cursor) {
          cursor.delete()
          cursor.continue()
        }
      }
    } catch (error) {
      console.warn('Disk cache cleanup error:', error)
    }
  }

  /**
   * Calculate data size for memory management
   */
  calculateSize(data) {
    return JSON.stringify(data).length * 2 // Rough estimate (UTF-16)
  }

  /**
   * Compress data if compression is enabled
   */
  async compressData(data) {
    if (!this.options.compression) return data

    try {
      // Simple compression using JSON.stringify optimization
      return JSON.stringify(data)
    } catch (error) {
      return data
    }
  }

  /**
   * Decompress data
   */
  decompressData(data) {
    if (!this.options.compression) return data

    try {
      return typeof data === 'string' ? JSON.parse(data) : data
    } catch (error) {
      return data
    }
  }

  /**
   * Check if item should be cached in network cache
   */
  shouldCacheInNetwork(key, options) {
    // Cache API calls and static resources in network cache
    return key.startsWith('/api/') || 
           key.includes('.js') || 
           key.includes('.css') ||
           options.networkCache === true
  }

  /**
   * Invalidate cache entry
   */
  async invalidate(key) {
    // Remove from memory
    const memItem = this.memoryCache.get(key)
    if (memItem) {
      this.memoryCache.delete(key)
      this.memoryUsage -= memItem.size
    }

    // Remove from disk
    await this.deleteFromDisk(key)

    // Remove from network cache
    if (this.networkCache) {
      try {
        await this.networkCache.delete(key)
      } catch (error) {
        console.warn('Network cache invalidation error:', error)
      }
    }
  }

  /**
   * Clear all caches
   */
  async clear() {
    // Clear memory
    this.memoryCache.clear()
    this.memoryUsage = 0

    // Clear disk
    if (this.diskCache) {
      try {
        const transaction = this.diskCache.transaction(['cache'], 'readwrite')
        const store = transaction.objectStore('cache')
        await new Promise((resolve, reject) => {
          const request = store.clear()
          request.onsuccess = () => resolve()
          request.onerror = () => reject(request.error)
        })
      } catch (error) {
        console.warn('Disk cache clear error:', error)
      }
    }

    // Clear network cache
    if (this.networkCache) {
      try {
        const keys = await this.networkCache.keys()
        await Promise.all(keys.map(key => this.networkCache.delete(key)))
      } catch (error) {
        console.warn('Network cache clear error:', error)
      }
    }

    // Reset stats
    this.stats = {
      hits: 0,
      misses: 0,
      memoryHits: 0,
      diskHits: 0,
      networkHits: 0,
      evictions: 0
    }
  }

  /**
   * Get cache statistics
   */
  getStats() {
    const totalRequests = this.stats.hits + this.stats.misses
    const hitRate = totalRequests > 0 ? (this.stats.hits / totalRequests) * 100 : 0

    return {
      ...this.stats,
      hitRate,
      memoryUsage: this.memoryUsage,
      memoryItems: this.memoryCache.size,
      totalRequests
    }
  }
}

export const smartCache = new SmartCache()

export default {
  SmartCache,
  smartCache
}
