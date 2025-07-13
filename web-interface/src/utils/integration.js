/**
 * Integration Configuration
 * Orchestrates all web UI improvements and integrates them with the existing Vue.js application
 */

import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

// Import all improvement modules
import { bundleOptimizer } from './bundleOptimizer.js'
import { pwaManager } from './pwa.js'
import { errorReporting } from './errorReporting.js'
import { smartCache } from './smartCache.js'
import { collaboration } from './collaboration.js'
import { intelligentDashboard } from './intelligentDashboard.js'
import { securityManager } from './security.js'
import { accessibilityManager } from './accessibility.js'
import { performanceMonitor } from './performanceMonitor.js'
import { testFramework } from './testFramework.js'
import { i18nManager } from './i18n.js'

/**
 * Application integration orchestrator
 */
export class AppIntegrator {
  constructor(options = {}) {
    this.options = {
      enablePWA: options.enablePWA !== false,
      enableCollaboration: options.enableCollaboration !== false,
      enableIntelligentDashboard: options.enableIntelligentDashboard !== false,
      enableSecurity: options.enableSecurity !== false,
      enableAccessibility: options.enableAccessibility !== false,
      enablePerformanceMonitoring: options.enablePerformanceMonitoring !== false,
      enableI18n: options.enableI18n !== false,
      enableTesting: options.enableTesting && import.meta.env.DEV,
      ...options
    }

    this.app = null
    this.router = null
    this.pinia = null
    this.services = new Map()
    this.plugins = []
    this.initialized = false
  }

  /**
   * Initialize the application with all improvements
   */
  async init(appComponent, routes = []) {
    if (this.initialized) {
      console.warn('🔧 Integration: Application already initialized')
      return this.app
    }

    console.log('🔧 Integration: Starting application initialization...')

    // Create Vue app
    this.app = createApp(appComponent)

    // Setup router
    this.router = createRouter({
      history: createWebHistory(),
      routes
    })

    // Setup Pinia store
    this.pinia = createPinia()

    // Install core plugins
    this.app.use(this.router)
    this.app.use(this.pinia)
    this.app.use(ElementPlus)

    // Initialize services
    await this.initializeServices()

    // Setup error handling
    this.setupErrorHandling()

    // Setup performance monitoring
    this.setupPerformanceMonitoring()

    // Setup security
    this.setupSecurity()

    // Setup accessibility
    this.setupAccessibility()

    // Setup internationalization
    await this.setupI18n()

    // Setup PWA
    await this.setupPWA()

    // Setup collaboration
    this.setupCollaboration()

    // Setup intelligent dashboard
    this.setupIntelligentDashboard()

    // Setup bundle optimization
    this.setupBundleOptimization()

    // Setup caching
    this.setupCaching()

    // Setup testing (development only)
    if (this.options.enableTesting) {
      this.setupTesting()
    }

    // Register global components and plugins
    this.registerGlobalComponents()
    this.registerGlobalPlugins()

    // Setup development tools
    if (import.meta.env.DEV) {
      this.setupDevelopmentTools()
    }

    this.initialized = true
    console.log('🔧 Integration: Application initialization completed')

    return this.app
  }

  /**
   * Initialize all services
   */
  async initializeServices() {
    console.log('🔧 Integration: Initializing services...')

    // Initialize smart cache
    this.services.set('cache', smartCache)

    // Initialize error reporting
    this.services.set('errorReporting', errorReporting)

    // Initialize performance monitor
    if (this.options.enablePerformanceMonitoring) {
      this.services.set('performance', performanceMonitor)
    }

    // Initialize security manager
    if (this.options.enableSecurity) {
      this.services.set('security', securityManager)
    }

    // Initialize accessibility manager
    if (this.options.enableAccessibility) {
      this.services.set('accessibility', accessibilityManager)
    }

    // Initialize collaboration
    if (this.options.enableCollaboration) {
      this.services.set('collaboration', collaboration)
    }

    // Initialize intelligent dashboard
    if (this.options.enableIntelligentDashboard) {
      this.services.set('dashboard', intelligentDashboard)
    }

    // Initialize PWA manager
    if (this.options.enablePWA) {
      this.services.set('pwa', pwaManager)
    }

    // Initialize i18n manager
    if (this.options.enableI18n) {
      this.services.set('i18n', i18nManager)
    }

    // Initialize test framework (development only)
    if (this.options.enableTesting) {
      this.services.set('testing', testFramework)
    }
  }

  /**
   * Setup error handling
   */
  setupErrorHandling() {
    console.log('🔧 Integration: Setting up error handling...')

    // Global error handler
    this.app.config.errorHandler = (error, instance, info) => {
      console.error('🚨 Vue Error:', error, info)
      
      if (this.services.has('errorReporting')) {
        this.services.get('errorReporting').reportError(error, {
          context: 'vue-error-handler',
          component: instance?.$options.name || 'Unknown',
          info
        })
      }
    }

    // Global warning handler
    this.app.config.warnHandler = (msg, instance, trace) => {
      console.warn('⚠️ Vue Warning:', msg, trace)
    }

    // Unhandled promise rejection
    window.addEventListener('unhandledrejection', (event) => {
      console.error('🚨 Unhandled Promise Rejection:', event.reason)
      
      if (this.services.has('errorReporting')) {
        this.services.get('errorReporting').reportError(event.reason, {
          context: 'unhandled-promise-rejection'
        })
      }
    })

    // Global error event
    window.addEventListener('error', (event) => {
      console.error('🚨 Global Error:', event.error)
      
      if (this.services.has('errorReporting')) {
        this.services.get('errorReporting').reportError(event.error, {
          context: 'global-error',
          filename: event.filename,
          lineno: event.lineno,
          colno: event.colno
        })
      }
    })
  }

  /**
   * Setup performance monitoring
   */
  setupPerformanceMonitoring() {
    if (!this.options.enablePerformanceMonitoring) return

    console.log('🔧 Integration: Setting up performance monitoring...')

    const performance = this.services.get('performance')
    if (performance) {
      // Start monitoring in production
      if (import.meta.env.PROD) {
        performance.start()
      }

      // Add performance directive
      this.app.directive('performance', {
        beforeMount(el, binding) {
          const name = binding.value || el.tagName.toLowerCase()
          performance.markStart(`component-${name}`)
        },
        mounted(el, binding) {
          const name = binding.value || el.tagName.toLowerCase()
          performance.markEnd(`component-${name}`)
        }
      })
    }
  }

  /**
   * Setup security
   */
  setupSecurity() {
    if (!this.options.enableSecurity) return

    console.log('🔧 Integration: Setting up security...')

    const security = this.services.get('security')
    if (security) {
      // Security is initialized automatically
      console.log('🔐 Security: Protection enabled')
    }
  }

  /**
   * Setup accessibility
   */
  setupAccessibility() {
    if (!this.options.enableAccessibility) return

    console.log('🔧 Integration: Setting up accessibility...')

    const accessibility = this.services.get('accessibility')
    if (accessibility) {
      // Accessibility is initialized automatically
      console.log('♿ Accessibility: Features enabled')
    }
  }

  /**
   * Setup internationalization
   */
  async setupI18n() {
    if (!this.options.enableI18n) return

    console.log('🔧 Integration: Setting up internationalization...')

    const i18n = this.services.get('i18n')
    if (i18n) {
      // Install i18n plugin
      this.app.use(i18n.getI18n())
      
      // Add global properties
      this.app.config.globalProperties.$formatNumber = i18n.formatNumber.bind(i18n)
      this.app.config.globalProperties.$formatDate = i18n.formatDate.bind(i18n)
      this.app.config.globalProperties.$formatRelativeTime = i18n.formatRelativeTime.bind(i18n)
      
      console.log('🌍 i18n: Internationalization enabled')
    }
  }

  /**
   * Setup PWA
   */
  async setupPWA() {
    if (!this.options.enablePWA) return

    console.log('🔧 Integration: Setting up PWA...')

    const pwa = this.services.get('pwa')
    if (pwa) {
      // PWA is initialized automatically
      console.log('📱 PWA: Progressive Web App features enabled')
    }
  }

  /**
   * Setup collaboration
   */
  setupCollaboration() {
    if (!this.options.enableCollaboration) return

    console.log('🔧 Integration: Setting up collaboration...')

    const collaboration = this.services.get('collaboration')
    if (collaboration) {
      // Add collaboration to global properties
      this.app.config.globalProperties.$collaboration = collaboration
      
      // Setup collaboration plugin
      this.app.use({
        install(app) {
          app.provide('collaboration', collaboration)
        }
      })
      
      console.log('🤝 Collaboration: Real-time features enabled')
    }
  }

  /**
   * Setup intelligent dashboard
   */
  setupIntelligentDashboard() {
    if (!this.options.enableIntelligentDashboard) return

    console.log('🔧 Integration: Setting up intelligent dashboard...')

    const dashboard = this.services.get('dashboard')
    if (dashboard) {
      // Add dashboard to global properties
      this.app.config.globalProperties.$dashboard = dashboard
      
      // Setup dashboard plugin
      this.app.use({
        install(app) {
          app.provide('dashboard', dashboard)
        }
      })
      
      console.log('🧠 Dashboard: Intelligent features enabled')
    }
  }

  /**
   * Setup bundle optimization
   */
  setupBundleOptimization() {
    console.log('🔧 Integration: Setting up bundle optimization...')

    // Bundle optimization is handled by the bundleOptimizer
    // during build time and component loading
    console.log('📦 Bundler: Optimization enabled')
  }

  /**
   * Setup caching
   */
  setupCaching() {
    console.log('🔧 Integration: Setting up caching...')

    const cache = this.services.get('cache')
    if (cache) {
      // Add cache to global properties
      this.app.config.globalProperties.$cache = cache
      
      // Setup cache plugin
      this.app.use({
        install(app) {
          app.provide('cache', cache)
        }
      })
      
      console.log('💾 Cache: Smart caching enabled')
    }
  }

  /**
   * Setup testing (development only)
   */
  setupTesting() {
    if (!import.meta.env.DEV) return

    console.log('🔧 Integration: Setting up testing framework...')

    const testing = this.services.get('testing')
    if (testing) {
      // Add testing utilities to global properties
      this.app.config.globalProperties.$testing = testing
      
      // Setup testing plugin
      this.app.use({
        install(app) {
          app.provide('testing', testing)
        }
      })
      
      console.log('🧪 Testing: Framework enabled')
    }
  }

  /**
   * Register global components
   */
  registerGlobalComponents() {
    console.log('🔧 Integration: Registering global components...')

    // Register commonly used components globally
    const globalComponents = {
      // Error boundary component
      ErrorBoundary: {
        template: `
          <div v-if="hasError" class="error-boundary">
            <h2>Something went wrong</h2>
            <p>{{ error.message }}</p>
            <button @click="retry">Try Again</button>
          </div>
          <slot v-else></slot>
        `,
        data() {
          return {
            hasError: false,
            error: null
          }
        },
        errorCaptured(error, instance, info) {
          this.hasError = true
          this.error = error
          
          if (this.$services?.errorReporting) {
            this.$services.errorReporting.reportError(error, {
              context: 'error-boundary',
              component: instance?.$options.name || 'Unknown',
              info
            })
          }
          
          return false
        },
        methods: {
          retry() {
            this.hasError = false
            this.error = null
          }
        }
      },

      // Loading component
      LoadingSpinner: {
        template: `
          <div class="loading-spinner" :class="{ 'loading-spinner--small': small }">
            <div class="spinner"></div>
            <span v-if="message">{{ message }}</span>
          </div>
        `,
        props: {
          small: Boolean,
          message: String
        }
      },

      // Performance marker component
      PerformanceMarker: {
        template: `<div><slot></slot></div>`,
        props: {
          name: {
            type: String,
            required: true
          }
        },
        mounted() {
          if (this.$services?.performance) {
            this.$services.performance.markStart(this.name)
          }
        },
        beforeUnmount() {
          if (this.$services?.performance) {
            this.$services.performance.markEnd(this.name)
          }
        }
      }
    }

    // Register components
    Object.entries(globalComponents).forEach(([name, component]) => {
      this.app.component(name, component)
    })
  }

  /**
   * Register global plugins
   */
  registerGlobalPlugins() {
    console.log('🔧 Integration: Registering global plugins...')

    // Services plugin
    this.app.use({
      install: (app) => {
        // Provide services to all components
        const services = Object.fromEntries(this.services)
        app.provide('services', services)
        app.config.globalProperties.$services = services
      }
    })

    // Utility plugin
    this.app.use({
      install: (app) => {
        // Add utility functions
        app.config.globalProperties.$utils = {
          formatBytes: (bytes) => {
            if (bytes === 0) return '0 Bytes'
            const k = 1024
            const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
            const i = Math.floor(Math.log(bytes) / Math.log(k))
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
          },
          
          formatDuration: (ms) => {
            const seconds = Math.floor(ms / 1000)
            const minutes = Math.floor(seconds / 60)
            const hours = Math.floor(minutes / 60)
            
            if (hours > 0) return `${hours}h ${minutes % 60}m`
            if (minutes > 0) return `${minutes}m ${seconds % 60}s`
            return `${seconds}s`
          },
          
          debounce: (func, wait) => {
            let timeout
            return function executedFunction(...args) {
              const later = () => {
                clearTimeout(timeout)
                func(...args)
              }
              clearTimeout(timeout)
              timeout = setTimeout(later, wait)
            }
          },
          
          throttle: (func, limit) => {
            let inThrottle
            return function executedFunction(...args) {
              if (!inThrottle) {
                func.apply(this, args)
                inThrottle = true
                setTimeout(() => inThrottle = false, limit)
              }
            }
          }
        }
      }
    })
  }

  /**
   * Setup development tools
   */
  setupDevelopmentTools() {
    console.log('🔧 Integration: Setting up development tools...')

    // Add development helpers to window
    window.__DATAMESH_DEV__ = {
      app: this.app,
      router: this.router,
      pinia: this.pinia,
      services: this.services,
      
      // Service accessors
      getCache: () => this.services.get('cache'),
      getPerformance: () => this.services.get('performance'),
      getSecurity: () => this.services.get('security'),
      getAccessibility: () => this.services.get('accessibility'),
      getCollaboration: () => this.services.get('collaboration'),
      getDashboard: () => this.services.get('dashboard'),
      getPWA: () => this.services.get('pwa'),
      getI18n: () => this.services.get('i18n'),
      getTesting: () => this.services.get('testing'),
      
      // Utility functions
      clearCache: () => this.services.get('cache')?.clear(),
      getPerformanceReport: () => this.services.get('performance')?.analyzePerformance(),
      getSecurityReport: () => this.services.get('security')?.getSecurityStatus(),
      getAccessibilityReport: () => this.services.get('accessibility')?.getAccessibilityReport(),
      
      // Testing helpers
      runTests: () => this.services.get('testing')?.runTestSuite(),
      generateTestData: (type, count) => this.services.get('testing')?.generateTestData(type, count)
    }

    console.log('🛠️ Development tools available at window.__DATAMESH_DEV__')
  }

  /**
   * Mount the application
   */
  mount(selector = '#app') {
    if (!this.initialized) {
      throw new Error('Application not initialized. Call init() first.')
    }

    console.log('🔧 Integration: Mounting application...')
    
    const mountedApp = this.app.mount(selector)
    
    console.log('🚀 Application mounted successfully!')
    
    return mountedApp
  }

  /**
   * Get service by name
   */
  getService(name) {
    return this.services.get(name)
  }

  /**
   * Get all services
   */
  getServices() {
    return Object.fromEntries(this.services)
  }

  /**
   * Check if service is enabled
   */
  isServiceEnabled(name) {
    return this.services.has(name)
  }

  /**
   * Get initialization status
   */
  isInitialized() {
    return this.initialized
  }

  /**
   * Get application statistics
   */
  getStats() {
    return {
      initialized: this.initialized,
      services: this.services.size,
      plugins: this.plugins.length,
      enabledFeatures: {
        pwa: this.options.enablePWA,
        collaboration: this.options.enableCollaboration,
        intelligentDashboard: this.options.enableIntelligentDashboard,
        security: this.options.enableSecurity,
        accessibility: this.options.enableAccessibility,
        performanceMonitoring: this.options.enablePerformanceMonitoring,
        i18n: this.options.enableI18n,
        testing: this.options.enableTesting
      }
    }
  }
}

// Default integrator instance
export const appIntegrator = new AppIntegrator()

// Export main integration function
export async function integrateApp(appComponent, routes = [], options = {}) {
  const integrator = new AppIntegrator(options)
  await integrator.init(appComponent, routes)
  return integrator
}

export default {
  AppIntegrator,
  appIntegrator,
  integrateApp
}
