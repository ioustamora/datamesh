import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'

import App from './App.vue'
import router from './router'
import { useThemeStore } from './store/theme'
import errorHandlerPlugin from './plugins/errorHandler'
import cachePlugin from './plugins/cache'

// Import new features
import { i18n, initializeI18n } from './i18n'
import { 
  setupGlobalErrorHandling, 
  setupResourceErrorHandling, 
  setupPerformanceErrorMonitoring 
} from './composables/useErrorBoundary'
import { performanceMonitor } from './utils/performanceMonitor'
import { preloadCriticalComponents } from './utils/lazyLoadComponents'

// Create Vue app
const app = createApp(App)

// Use plugins
app.use(createPinia())
app.use(router)
app.use(ElementPlus)
app.use(i18n)
app.use(errorHandlerPlugin)
app.use(cachePlugin)

// Register Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

// Initialize theme
const themeStore = useThemeStore()
themeStore.initializeTheme()

// Setup advanced error handling
setupGlobalErrorHandling()
setupResourceErrorHandling()
setupPerformanceErrorMonitoring()

// Global properties
app.config.globalProperties.$APP_VERSION = '1.0.0'
app.config.globalProperties.$API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

// Initialize app
async function initializeApp() {
  try {
    // Initialize i18n
    await initializeI18n()
    
    // Start performance monitoring in production
    if (import.meta.env.PROD) {
      performanceMonitor.start()
    }
    
    // Preload critical components
    preloadCriticalComponents()
    
    // Mount app
    app.mount('#app')
    
    console.log('DataMesh application initialized successfully')
  } catch (error) {
    console.error('Failed to initialize application:', error)
    // Fallback mount
    app.mount('#app')
  }
}

// Initialize the app
initializeApp()

// Enhanced Service Worker registration for PWA support
if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', async () => {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js')
      console.log('SW registered: ', registration)
      
      // Handle service worker updates
      registration.addEventListener('updatefound', () => {
        const newWorker = registration.installing
        newWorker.addEventListener('statechange', () => {
          if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
            // New content is available, show update notification
            const event = new CustomEvent('sw-update-available')
            window.dispatchEvent(event)
          }
        })
      })
      
      // Handle service worker messages
      navigator.serviceWorker.addEventListener('message', (event) => {
        if (event.data.type === 'CACHE_UPDATED') {
          console.log('Cache updated:', event.data.payload)
        }
      })
      
    } catch (registrationError) {
      console.log('SW registration failed: ', registrationError)
    }
  })
  
  // Request notification permission
  if ('Notification' in window && Notification.permission === 'default') {
    Notification.requestPermission().then(permission => {
      console.log('Notification permission:', permission)
    })
  }
}