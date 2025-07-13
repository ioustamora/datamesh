/**
 * Progressive Web App Implementation
 */

import { ElMessage, ElNotification } from 'element-plus'

export class PWAManager {
  constructor() {
    this.isInstallable = false
    this.isInstalled = false
    this.updateAvailable = false
    this.registration = null
    this.deferredPrompt = null
    
    this.init()
  }

  async init() {
    // Check if PWA is already installed
    this.checkInstallStatus()
    
    // Register service worker
    await this.registerServiceWorker()
    
    // Set up install prompt
    this.setupInstallPrompt()
    
    // Set up update detection
    this.setupUpdateDetection()
    
    // Set up offline handling
    this.setupOfflineHandling()
  }

  /**
   * Check PWA install status
   */
  checkInstallStatus() {
    // Check if running as installed PWA
    this.isInstalled = window.matchMedia('(display-mode: standalone)').matches ||
                      window.navigator.standalone ||
                      document.referrer.includes('android-app://')
  }

  /**
   * Register service worker
   */
  async registerServiceWorker() {
    if ('serviceWorker' in navigator) {
      try {
        this.registration = await navigator.serviceWorker.register('/sw.js', {
          scope: '/'
        })

        console.log('Service Worker registered successfully')
        
        // Listen for updates
        this.registration.addEventListener('updatefound', () => {
          this.handleServiceWorkerUpdate()
        })

      } catch (error) {
        console.error('Service Worker registration failed:', error)
      }
    }
  }

  /**
   * Handle service worker updates
   */
  handleServiceWorkerUpdate() {
    const newWorker = this.registration.installing
    
    newWorker.addEventListener('statechange', () => {
      if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
        this.updateAvailable = true
        this.showUpdateNotification()
      }
    })
  }

  /**
   * Show update notification
   */
  showUpdateNotification() {
    ElNotification({
      title: 'Update Available',
      message: 'A new version of DataMesh is available. Click to update.',
      type: 'info',
      duration: 0,
      onClick: () => {
        this.applyUpdate()
      }
    })
  }

  /**
   * Apply update
   */
  async applyUpdate() {
    if (this.registration && this.registration.waiting) {
      this.registration.waiting.postMessage({ type: 'SKIP_WAITING' })
      
      // Reload page after service worker takes control
      navigator.serviceWorker.addEventListener('controllerchange', () => {
        window.location.reload()
      })
    }
  }

  /**
   * Set up install prompt
   */
  setupInstallPrompt() {
    window.addEventListener('beforeinstallprompt', (e) => {
      e.preventDefault()
      this.deferredPrompt = e
      this.isInstallable = true
      
      // Show install banner after delay
      setTimeout(() => {
        this.showInstallBanner()
      }, 30000) // Show after 30 seconds
    })

    // Track install result
    window.addEventListener('appinstalled', () => {
      this.isInstalled = true
      this.isInstallable = false
      this.deferredPrompt = null
      
      ElMessage.success('DataMesh installed successfully!')
    })
  }

  /**
   * Show install banner
   */
  showInstallBanner() {
    if (!this.isInstallable || this.isInstalled) return

    ElNotification({
      title: 'Install DataMesh',
      message: 'Install DataMesh for a better experience with offline access.',
      type: 'info',
      duration: 10000,
      customClass: 'pwa-install-notification',
      showClose: true,
      onClick: () => {
        this.promptInstall()
      }
    })
  }

  /**
   * Prompt user to install PWA
   */
  async promptInstall() {
    if (!this.deferredPrompt) return

    try {
      this.deferredPrompt.prompt()
      const result = await this.deferredPrompt.userChoice
      
      if (result.outcome === 'accepted') {
        console.log('User accepted the install prompt')
      } else {
        console.log('User dismissed the install prompt')
      }
      
      this.deferredPrompt = null
      this.isInstallable = false
    } catch (error) {
      console.error('Install prompt failed:', error)
    }
  }

  /**
   * Set up offline handling
   */
  setupOfflineHandling() {
    // Listen for online/offline events
    window.addEventListener('online', () => {
      this.handleOnline()
    })

    window.addEventListener('offline', () => {
      this.handleOffline()
    })

    // Check initial online status
    if (!navigator.onLine) {
      this.handleOffline()
    }
  }

  /**
   * Handle online state
   */
  handleOnline() {
    ElMessage.success('Connection restored')
    
    // Sync pending data
    this.syncPendingData()
    
    // Update UI state
    document.body.classList.remove('offline')
  }

  /**
   * Handle offline state
   */
  handleOffline() {
    ElMessage.warning('You are offline. Some features may be limited.')
    
    // Update UI state
    document.body.classList.add('offline')
  }

  /**
   * Sync pending data when back online
   */
  async syncPendingData() {
    try {
      // Get pending operations from IndexedDB
      const pendingOps = await this.getPendingOperations()
      
      for (const op of pendingOps) {
        try {
          await this.executePendingOperation(op)
          await this.removePendingOperation(op.id)
        } catch (error) {
          console.error('Failed to sync operation:', op, error)
        }
      }
    } catch (error) {
      console.error('Failed to sync pending data:', error)
    }
  }

  /**
   * Get pending operations from storage
   */
  async getPendingOperations() {
    // Implementation would use IndexedDB
    return []
  }

  /**
   * Execute a pending operation
   */
  async executePendingOperation(operation) {
    // Implementation would replay the operation
    console.log('Executing pending operation:', operation)
  }

  /**
   * Remove completed operation from storage
   */
  async removePendingOperation(operationId) {
    // Implementation would remove from IndexedDB
    console.log('Removing completed operation:', operationId)
  }

  /**
   * Get PWA status
   */
  getStatus() {
    return {
      isInstalled: this.isInstalled,
      isInstallable: this.isInstallable,
      updateAvailable: this.updateAvailable,
      isOnline: navigator.onLine,
      registration: !!this.registration
    }
  }
}

/**
 * PWA Configuration
 */
export const pwaConfig = {
  name: 'DataMesh',
  shortName: 'DataMesh',
  description: 'Distributed Storage Management Platform',
  themeColor: '#409EFF',
  backgroundColor: '#ffffff',
  display: 'standalone',
  orientation: 'portrait-primary',
  scope: '/',
  startUrl: '/',
  icons: [
    {
      src: '/icons/icon-72x72.png',
      sizes: '72x72',
      type: 'image/png'
    },
    {
      src: '/icons/icon-96x96.png',
      sizes: '96x96',
      type: 'image/png'
    },
    {
      src: '/icons/icon-128x128.png',
      sizes: '128x128',
      type: 'image/png'
    },
    {
      src: '/icons/icon-144x144.png',
      sizes: '144x144',
      type: 'image/png'
    },
    {
      src: '/icons/icon-152x152.png',
      sizes: '152x152',
      type: 'image/png'
    },
    {
      src: '/icons/icon-192x192.png',
      sizes: '192x192',
      type: 'image/png'
    },
    {
      src: '/icons/icon-384x384.png',
      sizes: '384x384',
      type: 'image/png'
    },
    {
      src: '/icons/icon-512x512.png',
      sizes: '512x512',
      type: 'image/png'
    }
  ]
}

export const pwaManager = new PWAManager()

export default {
  PWAManager,
  pwaConfig,
  pwaManager
}
