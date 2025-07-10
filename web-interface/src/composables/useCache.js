/**
 * Vue composable for request caching and deduplication
 * Provides reactive caching with automatic invalidation
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { globalCache, queryDeduplicator, prefetchManager } from '../utils/requestCache'

/**
 * Main caching composable
 */
export function useCache() {
  const cacheStats = ref({
    cacheSize: 0,
    pendingRequests: 0,
    keys: []
  })
  
  const updateStats = () => {
    cacheStats.value = globalCache.getStats()
  }
  
  const clearCache = () => {
    globalCache.clear()
    updateStats()
  }
  
  const invalidateCache = (pattern) => {
    globalCache.invalidate(pattern)
    updateStats()
  }
  
  // Update stats periodically
  let statsInterval
  onMounted(() => {
    updateStats()
    statsInterval = setInterval(updateStats, 5000)
  })
  
  onUnmounted(() => {
    if (statsInterval) {
      clearInterval(statsInterval)
    }
  })
  
  return {
    cacheStats,
    clearCache,
    invalidateCache,
    updateStats
  }
}

/**
 * Cached fetch composable
 */
export function useCachedFetch(url, options = {}) {
  const data = ref(null)
  const loading = ref(false)
  const error = ref(null)
  const fromCache = ref(false)
  const fresh = ref(true)
  
  const fetch = async (customUrl, customOptions) => {
    const fetchUrl = customUrl || url
    const fetchOptions = { ...options, ...customOptions }
    
    if (!fetchUrl) return
    
    loading.value = true
    error.value = null
    
    try {
      const result = await globalCache.get(fetchUrl, fetchOptions)
      
      data.value = result.data
      fromCache.value = result.fromCache
      fresh.value = result.fresh
      
      return result.data
    } catch (err) {
      error.value = err
      console.error('Cache fetch error:', err)
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const refresh = () => {
    // Clear cache for this request and refetch
    const key = globalCache.generateKey(url, options)
    globalCache.cache.delete(key)
    return fetch()
  }
  
  // Auto-fetch on mount if URL is provided
  onMounted(() => {
    if (url) {
      fetch()
    }
  })
  
  // Watch URL changes
  if (typeof url === 'object' && url.value !== undefined) {
    watch(url, (newUrl) => {
      if (newUrl) {
        fetch(newUrl)
      }
    })
  }
  
  return {
    data,
    loading,
    error,
    fromCache,
    fresh,
    fetch,
    refresh
  }
}

/**
 * Paginated data fetching with caching
 */
export function useCachedPagination(baseUrl, options = {}) {
  const currentPage = ref(options.initialPage || 1)
  const pageSize = ref(options.pageSize || 20)
  const totalPages = ref(0)
  const totalItems = ref(0)
  const data = ref([])
  const loading = ref(false)
  const error = ref(null)
  
  const url = computed(() => {
    if (!baseUrl) return null
    const params = new URLSearchParams({
      page: currentPage.value,
      limit: pageSize.value,
      ...options.params
    })
    return `${baseUrl}?${params.toString()}`
  })
  
  const { fetch: fetchPage } = useCachedFetch(url, options.fetchOptions)
  
  const fetchData = async () => {
    if (!url.value) return
    
    loading.value = true
    error.value = null
    
    try {
      const result = await fetchPage()
      
      data.value = result.data || []
      totalPages.value = result.totalPages || 0
      totalItems.value = result.totalItems || 0
      
      // Prefetch adjacent pages
      if (options.prefetch !== false) {
        prefetchManager.prefetchPagination(
          currentPage.value,
          totalPages.value,
          baseUrl
        )
      }
      
      return result
    } catch (err) {
      error.value = err
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const goToPage = (page) => {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page
    }
  }
  
  const nextPage = () => {
    if (currentPage.value < totalPages.value) {
      currentPage.value++
    }
  }
  
  const prevPage = () => {
    if (currentPage.value > 1) {
      currentPage.value--
    }
  }
  
  const refresh = () => {
    // Invalidate current page and refetch
    globalCache.invalidate(url.value)
    return fetchData()
  }
  
  // Watch for changes
  watch([currentPage, pageSize], fetchData, { immediate: true })
  
  // Watch for URL changes
  watch(url, (newUrl) => {
    if (newUrl) {
      fetchData()
    }
  })
  
  return {
    data,
    loading,
    error,
    currentPage,
    pageSize,
    totalPages,
    totalItems,
    goToPage,
    nextPage,
    prevPage,
    refresh,
    fetchData
  }
}

/**
 * Search with deduplication
 */
export function useCachedSearch(searchFn, options = {}) {
  const query = ref('')
  const results = ref([])
  const loading = ref(false)
  const error = ref(null)
  const debounceDelay = options.debounceDelay || 300
  
  let searchTimeout
  
  const search = async (searchQuery) => {
    const queryStr = searchQuery || query.value
    
    if (!queryStr.trim()) {
      results.value = []
      return
    }
    
    loading.value = true
    error.value = null
    
    try {
      const searchResults = await queryDeduplicator.deduplicateQuery(
        queryStr,
        searchFn
      )
      
      results.value = searchResults
      return searchResults
    } catch (err) {
      error.value = err
      console.error('Search error:', err)
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const debouncedSearch = (searchQuery) => {
    clearTimeout(searchTimeout)
    searchTimeout = setTimeout(() => {
      search(searchQuery)
    }, debounceDelay)
  }
  
  const clearResults = () => {
    results.value = []
    query.value = ''
  }
  
  // Watch query changes
  watch(query, (newQuery) => {
    if (newQuery) {
      debouncedSearch(newQuery)
    } else {
      clearResults()
    }
  })
  
  onUnmounted(() => {
    clearTimeout(searchTimeout)
  })
  
  return {
    query,
    results,
    loading,
    error,
    search,
    debouncedSearch,
    clearResults
  }
}

/**
 * Resource caching with automatic invalidation
 */
export function useCachedResource(key, fetchFn, options = {}) {
  const data = ref(null)
  const loading = ref(false)
  const error = ref(null)
  const lastUpdated = ref(null)
  
  const ttl = options.ttl || 5 * 60 * 1000 // 5 minutes default
  const autoRefresh = options.autoRefresh || false
  
  const fetch = async () => {
    loading.value = true
    error.value = null
    
    try {
      const result = await globalCache.get(key, {
        ttl,
        fetchFn
      })
      
      data.value = result.data
      lastUpdated.value = new Date()
      
      return result.data
    } catch (err) {
      error.value = err
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const refresh = () => {
    globalCache.invalidate(key)
    return fetch()
  }
  
  const invalidate = () => {
    globalCache.invalidate(key)
    data.value = null
    lastUpdated.value = null
  }
  
  // Auto-refresh setup
  let refreshInterval
  if (autoRefresh) {
    onMounted(() => {
      refreshInterval = setInterval(refresh, ttl)
    })
    
    onUnmounted(() => {
      if (refreshInterval) {
        clearInterval(refreshInterval)
      }
    })
  }
  
  // Initial fetch
  onMounted(fetch)
  
  return {
    data,
    loading,
    error,
    lastUpdated,
    fetch,
    refresh,
    invalidate
  }
}

/**
 * Batch requests with caching
 */
export function useBatchCache(batchSize = 10) {
  const batchQueue = ref([])
  const processing = ref(false)
  
  const addToBatch = (url, options = {}) => {
    return new Promise((resolve, reject) => {
      batchQueue.value.push({
        url,
        options,
        resolve,
        reject
      })
      
      if (batchQueue.value.length >= batchSize) {
        processBatch()
      }
    })
  }
  
  const processBatch = async () => {
    if (processing.value || batchQueue.value.length === 0) {
      return
    }
    
    processing.value = true
    const batch = batchQueue.value.splice(0, batchSize)
    
    try {
      const promises = batch.map(item => 
        globalCache.get(item.url, item.options)
          .then(result => ({ item, result, success: true }))
          .catch(error => ({ item, error, success: false }))
      )
      
      const results = await Promise.all(promises)
      
      results.forEach(({ item, result, error, success }) => {
        if (success) {
          item.resolve(result)
        } else {
          item.reject(error)
        }
      })
    } finally {
      processing.value = false
      
      // Process remaining items
      if (batchQueue.value.length > 0) {
        setTimeout(processBatch, 100)
      }
    }
  }
  
  // Auto-process batch periodically
  let batchInterval
  onMounted(() => {
    batchInterval = setInterval(() => {
      if (batchQueue.value.length > 0) {
        processBatch()
      }
    }, 1000)
  })
  
  onUnmounted(() => {
    if (batchInterval) {
      clearInterval(batchInterval)
    }
  })
  
  return {
    addToBatch,
    processBatch,
    batchQueue: computed(() => batchQueue.value.length),
    processing
  }
}

export default {
  useCache,
  useCachedFetch,
  useCachedPagination,
  useCachedSearch,
  useCachedResource,
  useBatchCache
}