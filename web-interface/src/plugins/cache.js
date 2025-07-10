/**
 * Cache plugin for global cache management
 * Provides cache invalidation hooks and automatic cleanup
 */

import { globalCache, invalidateCache } from '../utils/requestCache'

export default {
  install(app) {
    // Global cache instance
    app.config.globalProperties.$cache = globalCache
    
    // Global cache utilities
    app.provide('cache', {
      get: globalCache.get.bind(globalCache),
      set: globalCache.set.bind(globalCache),
      invalidate: invalidateCache,
      clear: globalCache.clear.bind(globalCache),
      stats: globalCache.getStats.bind(globalCache)
    })
    
    // Cache invalidation hooks for common operations
    const setupCacheInvalidation = () => {
      // File operations
      app.config.globalProperties.$onFileUpload = (file) => {
        invalidateCache('/files/')
        invalidateCache('/analytics/')
      }
      
      app.config.globalProperties.$onFileDelete = (fileId) => {
        invalidateCache('/files/')
        invalidateCache(`/files/${fileId}`)
        invalidateCache('/analytics/')
      }
      
      app.config.globalProperties.$onFileUpdate = (fileId) => {
        invalidateCache(`/files/${fileId}`)
        invalidateCache('/files/')
      }
      
      // User operations
      app.config.globalProperties.$onUserUpdate = (userId) => {
        invalidateCache(`/users/${userId}`)
        invalidateCache('/users/')
        invalidateCache('/profile/')
      }
      
      // Governance operations
      app.config.globalProperties.$onGovernanceUpdate = () => {
        invalidateCache('/governance/')
        invalidateCache('/analytics/')
      }
      
      // System operations
      app.config.globalProperties.$onSystemUpdate = () => {
        invalidateCache('/system/')
        invalidateCache('/analytics/')
      }
    }
    
    setupCacheInvalidation()
    
    // Development tools
    if (process.env.NODE_ENV === 'development') {
      // Expose cache to window for debugging
      window.$cache = globalCache
      
      // Add cache inspection tools
      window.$cacheStats = () => globalCache.getStats()
      window.$clearCache = () => globalCache.clear()
      window.$invalidateCache = (pattern) => invalidateCache(pattern)
      
      console.log('Cache debugging tools available:')
      console.log('- window.$cache: Cache instance')
      console.log('- window.$cacheStats(): Get cache statistics')
      console.log('- window.$clearCache(): Clear all cache')
      console.log('- window.$invalidateCache(pattern): Invalidate cache entries')
    }
  }
}