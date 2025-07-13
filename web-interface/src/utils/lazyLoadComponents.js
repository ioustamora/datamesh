/**
 * Lazy loading utilities for code splitting and bundle optimization
 */

import { defineAsyncComponent } from 'vue'
import { ElSkeleton } from 'element-plus'

/**
 * Create a lazy-loaded component with loading state
 * @param {Function} loader - Component loader function
 * @param {Object} options - Loading options
 * @returns {Object} Async component definition
 */
export function createLazyComponent(loader, options = {}) {
  const {
    loadingComponent = ElSkeleton,
    errorComponent = null,
    delay = 200,
    timeout = 10000,
    suspensible = true,
    retryOnError = true,
    maxRetries = 3
  } = options

  let retryCount = 0

  const wrappedLoader = async () => {
    try {
      const module = await loader()
      retryCount = 0 // Reset retry count on successful load
      return module
    } catch (error) {
      if (retryOnError && retryCount < maxRetries) {
        retryCount++
        console.warn(`Component load failed, retrying (${retryCount}/${maxRetries})...`)
        
        // Exponential backoff
        const delay = Math.min(1000 * Math.pow(2, retryCount - 1), 5000)
        await new Promise(resolve => setTimeout(resolve, delay))
        
        return wrappedLoader()
      }
      
      console.error('Component loading failed:', error)
      throw error
    }
  }

  return defineAsyncComponent({
    loader: wrappedLoader,
    loadingComponent,
    errorComponent,
    delay,
    timeout,
    suspensible
  })
}

/**
 * Lazy load route components with chunk naming
 */
export const lazyRouteComponents = {
  // Dashboard components
  Dashboard: () => import(/* webpackChunkName: "dashboard" */ '@/views/Dashboard.vue'),
  
  // File management components
  FileManager: () => import(/* webpackChunkName: "file-manager" */ '@/views/FileManager.vue'),
  
  // Analytics components
  Analytics: () => import(/* webpackChunkName: "analytics" */ '@/views/Analytics.vue'),
  
  // Governance components
  Governance: () => import(/* webpackChunkName: "governance" */ '@/views/Governance.vue'),
  
  // Profile components
  Profile: () => import(/* webpackChunkName: "profile" */ '@/views/Profile.vue'),
  
  // Administration components
  Administration: () => import(/* webpackChunkName: "admin" */ '@/views/Administration.vue'),
  
  // Settings components
  Settings: () => import(/* webpackChunkName: "settings" */ '@/views/Settings.vue'),
  
  // Advanced components
  NetworkTopology: () => import(/* webpackChunkName: "network-topology" */ '@/views/NetworkTopology.vue'),
  
  // Error pages
  NotFound: () => import(/* webpackChunkName: "error-pages" */ '@/views/errors/NotFound.vue'),
  ServerError: () => import(/* webpackChunkName: "error-pages" */ '@/views/errors/ServerError.vue')
}

/**
 * Lazy load utility components
 */
export const lazyUtilityComponents = {
  // Charts
  PerformanceChart: () => import(/* webpackChunkName: "charts" */ '@/components/charts/PerformanceChart.vue'),
  UsageChart: () => import(/* webpackChunkName: "charts" */ '@/components/charts/UsageChart.vue'),
  NetworkChart: () => import(/* webpackChunkName: "charts" */ '@/components/charts/NetworkChart.vue'),
  
  // Advanced file features
  FilePreview: () => import(/* webpackChunkName: "file-preview" */ '@/components/files/FilePreview.vue'),
  FileEditor: () => import(/* webpackChunkName: "file-editor" */ '@/components/files/FileEditor.vue'),
  
  // Dialogs
  ContributionSetupDialog: () => import(/* webpackChunkName: "dialogs" */ '@/components/dashboard/ContributionSetupDialog.vue'),
  
  // Advanced features
  NetworkTopologyVisualization: () => import(/* webpackChunkName: "network-viz" */ '@/components/network/NetworkTopologyVisualization.vue'),
  
  // Media components
  VideoPlayer: () => import(/* webpackChunkName: "media" */ '@/components/media/VideoPlayer.vue'),
  AudioPlayer: () => import(/* webpackChunkName: "media" */ '@/components/media/AudioPlayer.vue'),
  ImageGallery: () => import(/* webpackChunkName: "media" */ '@/components/media/ImageGallery.vue')
}

/**
 * Preload critical components
 */
export function preloadCriticalComponents() {
  // Preload dashboard and file manager components
  const criticalComponents = [
    lazyRouteComponents.Dashboard,
    lazyRouteComponents.FileManager
  ]
  
  criticalComponents.forEach(loader => {
    requestIdleCallback(() => {
      loader().catch(err => {
        console.warn('Failed to preload component:', err)
      })
    })
  })
}

/**
 * Component bundle analyzer helper
 */
export function analyzeComponentUsage() {
  const usage = {
    loaded: new Set(),
    failed: new Set(),
    pending: new Set()
  }
  
  const originalDefineAsyncComponent = defineAsyncComponent
  
  // Override defineAsyncComponent to track usage
  window.defineAsyncComponent = function(options) {
    const { loader } = options
    
    const wrappedLoader = async () => {
      usage.pending.add(loader.name || 'anonymous')
      
      try {
        const result = await loader()
        usage.loaded.add(loader.name || 'anonymous')
        usage.pending.delete(loader.name || 'anonymous')
        return result
      } catch (error) {
        usage.failed.add(loader.name || 'anonymous')
        usage.pending.delete(loader.name || 'anonymous')
        throw error
      }
    }
    
    return originalDefineAsyncComponent({
      ...options,
      loader: wrappedLoader
    })
  }
  
  // Expose usage stats
  window.componentUsageStats = () => ({
    loaded: Array.from(usage.loaded),
    failed: Array.from(usage.failed),
    pending: Array.from(usage.pending)
  })
}

/**
 * Progressive component loading
 */
export class ProgressiveLoader {
  constructor() {
    this.queue = []
    this.loading = false
    this.maxConcurrent = 3
    this.currentLoading = 0
  }
  
  add(loader, priority = 0) {
    this.queue.push({ loader, priority })
    this.queue.sort((a, b) => b.priority - a.priority)
    this.processQueue()
  }
  
  async processQueue() {
    if (this.loading || this.queue.length === 0) return
    
    this.loading = true
    
    while (this.queue.length > 0 && this.currentLoading < this.maxConcurrent) {
      const { loader } = this.queue.shift()
      this.currentLoading++
      
      loader()
        .then(() => {
          this.currentLoading--
          this.processQueue()
        })
        .catch(error => {
          console.warn('Progressive loading failed:', error)
          this.currentLoading--
          this.processQueue()
        })
    }
    
    if (this.currentLoading === 0) {
      this.loading = false
    }
  }
}

export const progressiveLoader = new ProgressiveLoader()

/**
 * Bundle size optimization utilities
 */
export const bundleOptimization = {
  // Tree shaking utilities
  removeUnusedImports: () => {
    // This would be implemented at build time
    console.log('Tree shaking unused imports...')
  },
  
  // Dynamic imports for libraries
  loadLibrary: {
    chart: () => import(/* webpackChunkName: "chart-lib" */ 'chart.js'),
    // PDF.js requires special handling - load from CDN in production
    pdf: () => {
      if (import.meta.env.PROD) {
        return new Promise((resolve, reject) => {
          const script = document.createElement('script')
          script.src = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js'
          script.onload = () => resolve(window.pdfjsLib)
          script.onerror = reject
          document.head.appendChild(script)
        })
      } else {
        // In development, use a mock or return null
        return Promise.resolve(null)
      }
    },
    markdown: () => {
      if (import.meta.env.PROD) {
        // In production, load from CDN
        return new Promise((resolve, reject) => {
          if (window.marked) {
            resolve({ default: window.marked })
            return
          }
          
          const script = document.createElement('script')
          script.src = 'https://cdn.jsdelivr.net/npm/marked@5.1.1/marked.min.js'
          script.onload = () => {
            resolve({ default: window.marked })
          }
          script.onerror = reject
          document.head.appendChild(script)
        })
      } else {
        // In development, use a mock
        return Promise.resolve({
          default: {
            parse: (text) => `<p>${text}</p>`
          }
        })
      }
    },
    excel: () => {
      if (import.meta.env.PROD) {
        // In production, load from CDN
        return new Promise((resolve, reject) => {
          if (window.XLSX) {
            resolve({ default: window.XLSX })
            return
          }
          
          const script = document.createElement('script')
          script.src = 'https://cdn.jsdelivr.net/npm/xlsx@0.18.5/dist/xlsx.full.min.js'
          script.onload = () => {
            resolve({ default: window.XLSX })
          }
          script.onerror = reject
          document.head.appendChild(script)
        })
      } else {
        // In development, use a mock
        return Promise.resolve({
          default: {
            read: () => ({ SheetNames: [], Sheets: {} }),
            write: () => new ArrayBuffer(0)
          }
        })
      }
    },
    zip: () => {
      if (import.meta.env.PROD) {
        // In production, load from CDN
        return new Promise((resolve, reject) => {
          if (window.JSZip) {
            resolve({ default: window.JSZip })
            return
          }
          
          const script = document.createElement('script')
          script.src = 'https://cdn.jsdelivr.net/npm/jszip@3.10.1/dist/jszip.min.js'
          script.onload = () => {
            resolve({ default: window.JSZip })
          }
          script.onerror = reject
          document.head.appendChild(script)
        })
      } else {
        // In development, use a mock
        return Promise.resolve({
          default: class MockJSZip {
            constructor() {
              this.files = {}
            }
            file(name, content) {
              if (content !== undefined) {
                this.files[name] = content
              }
              return this.files[name]
            }
            generateAsync() {
              return Promise.resolve(new ArrayBuffer(0))
            }
          }
        })
      }
    }
  },
  
  // Critical resource hints
  prefetchResources: [
    '/api/v1/files',
    '/api/v1/user/profile',
    '/api/v1/governance/proposals'
  ],
  
  preloadResources: [
    '/static/fonts/inter.woff2',
    '/static/icons/sprite.svg'
  ]
}

// Initialize component usage tracking in development
if (import.meta.env.DEV) {
  analyzeComponentUsage()
}

export default {
  createLazyComponent,
  lazyRouteComponents,
  lazyUtilityComponents,
  preloadCriticalComponents,
  progressiveLoader,
  bundleOptimization
}
