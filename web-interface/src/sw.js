/**
 * Advanced Service Worker with intelligent caching and offline support
 */

import { precacheAndRoute, cleanupOutdatedCaches } from 'workbox-precaching'
import { registerRoute } from 'workbox-routing'
import { StaleWhileRevalidate, CacheFirst, NetworkFirst, NetworkOnly } from 'workbox-strategies'
import { CacheableResponsePlugin } from 'workbox-cacheable-response'
import { ExpirationPlugin } from 'workbox-expiration'
import { BackgroundSyncPlugin } from 'workbox-background-sync'
import { BroadcastUpdatePlugin } from 'workbox-broadcast-update'
import { Queue } from 'workbox-background-sync'

// Constants
const CACHE_NAMES = {
  STATIC: 'datamesh-static-v1',
  DYNAMIC: 'datamesh-dynamic-v1',
  API: 'datamesh-api-v1',
  IMAGES: 'datamesh-images-v1',
  FONTS: 'datamesh-fonts-v1',
  OFFLINE: 'datamesh-offline-v1'
}

const API_BASE_URL = self.location.origin + '/api'

// Precache and route handling
precacheAndRoute(self.__WB_MANIFEST)
cleanupOutdatedCaches()

// Custom offline page
const OFFLINE_PAGE = '/offline.html'
const OFFLINE_FALLBACK_RESPONSE = new Response(
  `<!DOCTYPE html>
  <html>
    <head>
      <meta charset="utf-8">
      <title>DataMesh - Offline</title>
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <style>
        body { 
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          margin: 0; padding: 2rem; background: #f5f5f5; color: #333;
        }
        .container { max-width: 600px; margin: 0 auto; text-align: center; }
        .icon { font-size: 4rem; margin-bottom: 1rem; }
        h1 { color: #2563eb; margin-bottom: 1rem; }
        p { margin-bottom: 1.5rem; line-height: 1.6; }
        .actions { margin-top: 2rem; }
        button { 
          background: #2563eb; color: white; border: none; padding: 0.75rem 1.5rem;
          border-radius: 0.5rem; cursor: pointer; margin: 0 0.5rem;
        }
        button:hover { background: #1d4ed8; }
        .offline-files { 
          margin-top: 2rem; padding: 1rem; background: white; border-radius: 0.5rem;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="icon">📱</div>
        <h1>You're offline</h1>
        <p>Don't worry! DataMesh works offline. You can still access your cached files and data.</p>
        <div class="actions">
          <button onclick="window.location.reload()">Try Again</button>
          <button onclick="window.location.href='/dashboard'">Go to Dashboard</button>
        </div>
        <div class="offline-files">
          <h3>Available Offline</h3>
          <p>✓ Dashboard and file management<br>
             ✓ Cached files and folders<br>
             ✓ Recent governance proposals<br>
             ✓ Profile and settings</p>
        </div>
      </div>
    </body>
  </html>`,
  { headers: { 'Content-Type': 'text/html' } }
)

// Background sync for offline actions
const bgSyncPlugin = new BackgroundSyncPlugin('datamesh-background-sync', {
  maxRetentionTime: 24 * 60 // 24 hours
})

const syncQueue = new Queue('datamesh-sync-queue', {
  onSync: async ({ queue }) => {
    let entry
    while ((entry = await queue.shiftRequest())) {
      try {
        await fetch(entry.request)
        console.log('Background sync successful:', entry.request.url)
      } catch (error) {
        console.error('Background sync failed:', error)
        await queue.unshiftRequest(entry)
        throw error
      }
    }
  }
})

// Static assets caching (app shell)
registerRoute(
  ({ request }) => request.destination === 'document',
  new NetworkFirst({
    cacheName: CACHE_NAMES.STATIC,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new BroadcastUpdatePlugin()
    ]
  })
)

// JavaScript and CSS files
registerRoute(
  ({ request }) => 
    request.destination === 'script' || 
    request.destination === 'style',
  new StaleWhileRevalidate({
    cacheName: CACHE_NAMES.STATIC,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      })
    ]
  })
)

// Font files
registerRoute(
  ({ request }) => request.destination === 'font',
  new CacheFirst({
    cacheName: CACHE_NAMES.FONTS,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new ExpirationPlugin({
        maxAgeSeconds: 60 * 60 * 24 * 365, // 1 year
        maxEntries: 30
      })
    ]
  })
)

// Images
registerRoute(
  ({ request }) => request.destination === 'image',
  new CacheFirst({
    cacheName: CACHE_NAMES.IMAGES,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new ExpirationPlugin({
        maxAgeSeconds: 60 * 60 * 24 * 30, // 30 days
        maxEntries: 100
      })
    ]
  })
)

// API routes - different strategies for different endpoints
registerRoute(
  ({ url }) => url.pathname.startsWith('/api/v1/user'),
  new NetworkFirst({
    cacheName: CACHE_NAMES.API,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new ExpirationPlugin({
        maxAgeSeconds: 60 * 5, // 5 minutes
        maxEntries: 20
      }),
      bgSyncPlugin
    ]
  })
)

registerRoute(
  ({ url }) => url.pathname.startsWith('/api/v1/files'),
  new NetworkFirst({
    cacheName: CACHE_NAMES.API,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new ExpirationPlugin({
        maxAgeSeconds: 60 * 10, // 10 minutes
        maxEntries: 50
      }),
      bgSyncPlugin
    ]
  })
)

// Governance proposals - cache longer
registerRoute(
  ({ url }) => url.pathname.startsWith('/api/v1/governance'),
  new StaleWhileRevalidate({
    cacheName: CACHE_NAMES.API,
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200]
      }),
      new ExpirationPlugin({
        maxAgeSeconds: 60 * 30, // 30 minutes
        maxEntries: 30
      }),
      new BroadcastUpdatePlugin()
    ]
  })
)

// Real-time endpoints should not be cached
registerRoute(
  ({ url }) => 
    url.pathname.startsWith('/api/v1/websocket') ||
    url.pathname.startsWith('/api/v1/stream'),
  new NetworkOnly()
)

// File uploads and mutations
registerRoute(
  ({ request }) => 
    request.method === 'POST' || 
    request.method === 'PUT' || 
    request.method === 'DELETE',
  new NetworkOnly({
    plugins: [bgSyncPlugin]
  })
)

// Offline fallback
registerRoute(
  ({ request }) => request.mode === 'navigate',
  async ({ event }) => {
    try {
      const response = await fetch(event.request)
      return response
    } catch (error) {
      const cache = await caches.open(CACHE_NAMES.OFFLINE)
      const cachedResponse = await cache.match(OFFLINE_PAGE)
      return cachedResponse || OFFLINE_FALLBACK_RESPONSE
    }
  }
)

// Install event - cache offline page
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAMES.OFFLINE).then((cache) => {
      return cache.addAll([
        '/',
        '/dashboard',
        '/files',
        '/offline.html'
      ])
    })
  )
  self.skipWaiting()
})

// Activate event - claim clients
self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      // Clean up old caches
      const cacheNames = await caches.keys()
      await Promise.all(
        cacheNames
          .filter(name => !Object.values(CACHE_NAMES).includes(name))
          .map(name => caches.delete(name))
      )
      
      // Take control of all clients
      await self.clients.claim()
    })()
  )
})

// Message handling for cache management
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting()
  }
  
  if (event.data && event.data.type === 'CACHE_INVALIDATE') {
    const { pattern } = event.data
    invalidateCache(pattern)
  }
  
  if (event.data && event.data.type === 'PRELOAD_ROUTE') {
    const { url } = event.data
    preloadRoute(url)
  }
})

// Cache invalidation
async function invalidateCache(pattern) {
  const cacheNames = await caches.keys()
  
  for (const cacheName of cacheNames) {
    const cache = await caches.open(cacheName)
    const keys = await cache.keys()
    
    for (const key of keys) {
      if (key.url.includes(pattern)) {
        await cache.delete(key)
      }
    }
  }
}

// Route preloading
async function preloadRoute(url) {
  try {
    const response = await fetch(url)
    const cache = await caches.open(CACHE_NAMES.DYNAMIC)
    await cache.put(url, response.clone())
  } catch (error) {
    console.warn('Failed to preload route:', url, error)
  }
}

// Periodic background sync
self.addEventListener('periodicsync', (event) => {
  if (event.tag === 'datamesh-sync') {
    event.waitUntil(syncOfflineData())
  }
})

async function syncOfflineData() {
  try {
    // Sync any offline actions
    await syncQueue.replayRequests()
    
    // Update critical data
    const criticalUrls = [
      '/api/v1/user/profile',
      '/api/v1/files/recent',
      '/api/v1/governance/proposals'
    ]
    
    for (const url of criticalUrls) {
      try {
        const response = await fetch(url)
        if (response.ok) {
          const cache = await caches.open(CACHE_NAMES.API)
          await cache.put(url, response.clone())
        }
      } catch (error) {
        console.warn('Failed to sync:', url, error)
      }
    }
  } catch (error) {
    console.error('Background sync failed:', error)
  }
}

// Push notifications
self.addEventListener('push', (event) => {
  if (!event.data) return
  
  const data = event.data.json()
  const options = {
    body: data.body,
    icon: '/icons/icon-192x192.png',
    badge: '/icons/badge-72x72.png',
    data: data.data,
    actions: [
      {
        action: 'open',
        title: 'Open DataMesh',
        icon: '/icons/action-open.png'
      },
      {
        action: 'dismiss',
        title: 'Dismiss',
        icon: '/icons/action-dismiss.png'
      }
    ],
    requireInteraction: data.requireInteraction || false,
    silent: false,
    vibrate: [200, 100, 200]
  }
  
  event.waitUntil(
    self.registration.showNotification(data.title, options)
  )
})

// Notification clicks
self.addEventListener('notificationclick', (event) => {
  event.notification.close()
  
  if (event.action === 'open' || !event.action) {
    const url = event.notification.data?.url || '/dashboard'
    
    event.waitUntil(
      clients.matchAll({ type: 'window' }).then((clientList) => {
        // Focus existing window if available
        for (const client of clientList) {
          if (client.url === url && 'focus' in client) {
            return client.focus()
          }
        }
        
        // Open new window
        if (clients.openWindow) {
          return clients.openWindow(url)
        }
      })
    )
  }
})

// Share target handling
self.addEventListener('fetch', (event) => {
  if (event.request.url.includes('/files/upload') && event.request.method === 'POST') {
    event.respondWith(handleSharedFiles(event.request))
  }
})

async function handleSharedFiles(request) {
  const formData = await request.formData()
  const files = formData.getAll('file')
  
  if (files.length > 0) {
    // Store files for processing when online
    const cache = await caches.open(CACHE_NAMES.DYNAMIC)
    await cache.put('/shared-files', new Response(JSON.stringify({
      files: files.map(file => ({
        name: file.name,
        size: file.size,
        type: file.type
      })),
      timestamp: Date.now()
    })))
  }
  
  return Response.redirect('/files/upload?shared=true', 302)
}

console.log('DataMesh Service Worker loaded successfully')
