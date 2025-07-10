/**
 * Request deduplication and intelligent caching system
 * Prevents duplicate requests and provides smart caching with TTL and invalidation
 */

/**
 * Request cache with TTL and intelligent invalidation
 */
export class RequestCache {
  constructor(options = {}) {
    this.cache = new Map()
    this.pendingRequests = new Map()
    this.config = {
      defaultTTL: options.defaultTTL || 5 * 60 * 1000, // 5 minutes
      maxCacheSize: options.maxCacheSize || 500,
      staleWhileRevalidate: options.staleWhileRevalidate || true,
      ...options
    }
  }

  /**
   * Generate cache key from request parameters
   */
  generateKey(url, options = {}) {
    const method = options.method || 'GET'
    const body = options.body ? JSON.stringify(options.body) : ''
    const params = new URLSearchParams(options.params || {}).toString()
    
    return `${method}:${url}:${params}:${body}`
  }

  /**
   * Get cached response or make request
   */
  async get(url, options = {}) {
    const key = this.generateKey(url, options)
    const now = Date.now()
    
    // Check for pending request (deduplication)
    if (this.pendingRequests.has(key)) {
      return this.pendingRequests.get(key)
    }
    
    // Check cache
    const cached = this.cache.get(key)
    if (cached) {
      const { data, timestamp, ttl, stale } = cached
      
      // Return fresh data
      if (now - timestamp < ttl) {
        return { data, fromCache: true, fresh: true }
      }
      
      // Stale-while-revalidate
      if (this.config.staleWhileRevalidate && !stale) {
        // Mark as stale and trigger background refresh
        cached.stale = true
        this.backgroundRefresh(key, url, options)
        return { data, fromCache: true, fresh: false }
      }
      
      // Remove expired entry
      this.cache.delete(key)
    }
    
    // Make fresh request
    const promise = this.makeRequest(url, options)
      .then(response => {
        this.pendingRequests.delete(key)
        
        // Store in cache
        const ttl = this.getTTL(url, options)
        this.set(key, response, ttl)
        
        return { data: response, fromCache: false, fresh: true }
      })
      .catch(error => {
        this.pendingRequests.delete(key)
        
        // Return stale data if available on error
        if (cached && cached.data) {
          return { data: cached.data, fromCache: true, fresh: false, error }
        }
        
        throw error
      })
    
    this.pendingRequests.set(key, promise)
    return promise
  }

  /**
   * Set cache entry
   */
  set(key, data, ttl) {
    // Enforce cache size limit
    if (this.cache.size >= this.config.maxCacheSize) {
      // Remove oldest entries (simple LRU)
      const oldestKey = this.cache.keys().next().value
      this.cache.delete(oldestKey)
    }
    
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.config.defaultTTL,
      stale: false
    })
  }

  /**
   * Get TTL for specific request
   */
  getTTL(url, options) {
    // Dynamic TTL based on request type
    if (url.includes('/files/')) {
      return 10 * 60 * 1000 // 10 minutes for file data
    }
    
    if (url.includes('/analytics/')) {
      return 2 * 60 * 1000 // 2 minutes for analytics
    }
    
    if (url.includes('/governance/')) {
      return 5 * 60 * 1000 // 5 minutes for governance
    }
    
    if (url.includes('/user/') || url.includes('/profile/')) {
      return 15 * 60 * 1000 // 15 minutes for user data
    }
    
    return this.config.defaultTTL
  }

  /**
   * Background refresh for stale-while-revalidate
   */
  async backgroundRefresh(key, url, options) {
    try {
      const response = await this.makeRequest(url, options)
      const ttl = this.getTTL(url, options)
      this.set(key, response, ttl)
    } catch (error) {
      console.warn('Background refresh failed:', error)
    }
  }

  /**
   * Make actual HTTP request
   */
  async makeRequest(url, options = {}) {
    const config = {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers
      },
      ...options
    }
    
    // Add query parameters
    if (options.params) {
      const params = new URLSearchParams(options.params)
      url += (url.includes('?') ? '&' : '?') + params.toString()
    }
    
    const response = await fetch(url, config)
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    
    return response.json()
  }

  /**
   * Invalidate cache entries
   */
  invalidate(pattern) {
    if (typeof pattern === 'string') {
      // Simple string match
      for (const key of this.cache.keys()) {
        if (key.includes(pattern)) {
          this.cache.delete(key)
        }
      }
    } else if (pattern instanceof RegExp) {
      // Regex pattern
      for (const key of this.cache.keys()) {
        if (pattern.test(key)) {
          this.cache.delete(key)
        }
      }
    }
  }

  /**
   * Clear all cache
   */
  clear() {
    this.cache.clear()
    this.pendingRequests.clear()
  }

  /**
   * Get cache statistics
   */
  getStats() {
    return {
      cacheSize: this.cache.size,
      pendingRequests: this.pendingRequests.size,
      keys: Array.from(this.cache.keys())
    }
  }
}

/**
 * Smart query deduplication for similar requests
 */
export class QueryDeduplicator {
  constructor() {
    this.activeQueries = new Map()
    this.similarityThreshold = 0.8
  }

  /**
   * Check if query is similar to existing ones
   */
  findSimilarQuery(query, activeQueries) {
    const queryTokens = this.tokenize(query)
    
    for (const [activeQuery, promise] of activeQueries) {
      const activeTokens = this.tokenize(activeQuery)
      const similarity = this.calculateSimilarity(queryTokens, activeTokens)
      
      if (similarity > this.similarityThreshold) {
        return promise
      }
    }
    
    return null
  }

  /**
   * Tokenize query for similarity comparison
   */
  tokenize(query) {
    return query.toLowerCase()
      .replace(/[^\w\s]/g, '')
      .split(/\s+/)
      .filter(token => token.length > 2)
  }

  /**
   * Calculate similarity between two token arrays
   */
  calculateSimilarity(tokens1, tokens2) {
    const set1 = new Set(tokens1)
    const set2 = new Set(tokens2)
    
    const intersection = new Set([...set1].filter(x => set2.has(x)))
    const union = new Set([...set1, ...set2])
    
    return intersection.size / union.size
  }

  /**
   * Deduplicate search query
   */
  async deduplicateQuery(query, requestFn) {
    const key = `search:${query}`
    
    // Check for exact match
    if (this.activeQueries.has(key)) {
      return this.activeQueries.get(key)
    }
    
    // Check for similar queries
    const similarPromise = this.findSimilarQuery(query, this.activeQueries)
    if (similarPromise) {
      return similarPromise
    }
    
    // Make new request
    const promise = requestFn(query)
      .finally(() => {
        this.activeQueries.delete(key)
      })
    
    this.activeQueries.set(key, promise)
    return promise
  }
}

/**
 * Intelligent prefetching system
 */
export class PrefetchManager {
  constructor(cache, options = {}) {
    this.cache = cache
    this.config = {
      maxPrefetchDistance: options.maxPrefetchDistance || 5,
      prefetchDelay: options.prefetchDelay || 100,
      ...options
    }
    this.prefetchQueue = []
    this.processing = false
  }

  /**
   * Add URLs to prefetch queue
   */
  prefetch(urls, priority = 'low') {
    const prefetchItems = urls.map(url => ({
      url,
      priority,
      timestamp: Date.now()
    }))
    
    // Sort by priority and timestamp
    this.prefetchQueue.push(...prefetchItems)
    this.prefetchQueue.sort((a, b) => {
      const priorityOrder = { high: 3, medium: 2, low: 1 }
      const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority]
      return priorityDiff !== 0 ? priorityDiff : a.timestamp - b.timestamp
    })
    
    this.processPrefetchQueue()
  }

  /**
   * Process prefetch queue
   */
  async processPrefetchQueue() {
    if (this.processing || this.prefetchQueue.length === 0) {
      return
    }
    
    this.processing = true
    
    while (this.prefetchQueue.length > 0) {
      const item = this.prefetchQueue.shift()
      
      try {
        // Check if already cached
        const key = this.cache.generateKey(item.url)
        if (!this.cache.cache.has(key)) {
          await this.cache.get(item.url)
        }
        
        // Add delay to prevent overwhelming server
        await new Promise(resolve => setTimeout(resolve, this.config.prefetchDelay))
      } catch (error) {
        console.warn('Prefetch failed:', item.url, error)
      }
    }
    
    this.processing = false
  }

  /**
   * Prefetch pagination data
   */
  prefetchPagination(currentPage, totalPages, baseUrl) {
    const urls = []
    const distance = this.config.maxPrefetchDistance
    
    // Prefetch next pages
    for (let i = 1; i <= distance && currentPage + i <= totalPages; i++) {
      urls.push(`${baseUrl}?page=${currentPage + i}`)
    }
    
    // Prefetch previous pages
    for (let i = 1; i <= distance && currentPage - i >= 1; i++) {
      urls.push(`${baseUrl}?page=${currentPage - i}`)
    }
    
    this.prefetch(urls, 'medium')
  }
}

/**
 * Global cache instance
 */
export const globalCache = new RequestCache({
  defaultTTL: 5 * 60 * 1000, // 5 minutes
  maxCacheSize: 1000,
  staleWhileRevalidate: true
})

export const queryDeduplicator = new QueryDeduplicator()
export const prefetchManager = new PrefetchManager(globalCache)

/**
 * Cached fetch utility
 */
export const cachedFetch = (url, options = {}) => {
  return globalCache.get(url, options)
}

/**
 * Invalidate cache utility
 */
export const invalidateCache = (pattern) => {
  globalCache.invalidate(pattern)
}

/**
 * Clear all cache
 */
export const clearCache = () => {
  globalCache.clear()
}

export default {
  RequestCache,
  QueryDeduplicator,
  PrefetchManager,
  globalCache,
  queryDeduplicator,
  prefetchManager,
  cachedFetch,
  invalidateCache,
  clearCache
}