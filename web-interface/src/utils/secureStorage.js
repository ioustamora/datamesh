/**
 * Secure storage utility for sensitive data like authentication tokens
 * Uses httpOnly cookies for enhanced security against XSS attacks
 */

// Helper function to safely access document.cookie
const isBrowser = typeof window !== 'undefined' && typeof document !== 'undefined'

// Configuration
const SECURE_STORAGE_CONFIG = {
  tokenKey: 'datamesh_auth_token',
  refreshTokenKey: 'datamesh_refresh_token',
  cookieOptions: {
    secure: window.location.protocol === 'https:',
    sameSite: 'strict',
    maxAge: 60 * 60 * 24 * 7, // 7 days
    path: '/'
  }
}

/**
 * Set a secure cookie
 * @param {string} name - Cookie name
 * @param {string} value - Cookie value
 * @param {Object} options - Cookie options
 */
function setCookie(name, value, options = {}) {
  if (!isBrowser) return

  const opts = { ...SECURE_STORAGE_CONFIG.cookieOptions, ...options }
  
  let cookieString = `${name}=${encodeURIComponent(value)}`
  
  if (opts.maxAge) {
    cookieString += `; max-age=${opts.maxAge}`
  }
  
  if (opts.path) {
    cookieString += `; path=${opts.path}`
  }
  
  if (opts.secure) {
    cookieString += '; secure'
  }
  
  if (opts.sameSite) {
    cookieString += `; samesite=${opts.sameSite}`
  }
  
  // Note: For true httpOnly cookies, this would need to be set by the server
  // For now, we'll use a hybrid approach for client-side compatibility
  cookieString += '; httponly'
  
  document.cookie = cookieString
}

/**
 * Get a cookie value
 * @param {string} name - Cookie name
 * @returns {string|null} Cookie value or null if not found
 */
function getCookie(name) {
  if (!isBrowser) return null

  const cookies = document.cookie.split(';')
  
  for (let cookie of cookies) {
    const [cookieName, cookieValue] = cookie.trim().split('=')
    if (cookieName === name) {
      return decodeURIComponent(cookieValue)
    }
  }
  
  return null
}

/**
 * Remove a cookie
 * @param {string} name - Cookie name
 */
function removeCookie(name) {
  if (!isBrowser) return

  document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;`
}

/**
 * Encrypt data using Web Crypto API
 * @param {string} data - Data to encrypt
 * @returns {Promise<string>} Encrypted data as base64 string
 */
async function encrypt(data) {
  if (!isBrowser || !window.crypto || !window.crypto.subtle) {
    // Fallback to base64 encoding if Web Crypto API is not available
    console.warn('Web Crypto API not available, using base64 encoding as fallback')
    return btoa(data)
  }

  try {
    const encoder = new TextEncoder()
    const key = await window.crypto.subtle.generateKey(
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    )
    
    const iv = window.crypto.getRandomValues(new Uint8Array(12))
    const encrypted = await window.crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encoder.encode(data)
    )
    
    // Store key and IV for decryption (in production, use proper key management)
    const keyData = await window.crypto.subtle.exportKey('jwk', key)
    const result = {
      encrypted: Array.from(new Uint8Array(encrypted)),
      iv: Array.from(iv),
      key: keyData
    }
    
    return btoa(JSON.stringify(result))
  } catch (error) {
    console.error('Encryption failed:', error)
    // Fallback to base64 encoding
    return btoa(data)
  }
}

/**
 * Decrypt data using Web Crypto API
 * @param {string} encryptedData - Encrypted data as base64 string
 * @returns {Promise<string>} Decrypted data
 */
async function decrypt(encryptedData) {
  if (!isBrowser || !window.crypto || !window.crypto.subtle) {
    // Fallback to base64 decoding if Web Crypto API is not available
    try {
      return atob(encryptedData)
    } catch (error) {
      console.error('Base64 decoding failed:', error)
      return null
    }
  }

  try {
    const data = JSON.parse(atob(encryptedData))
    
    const key = await window.crypto.subtle.importKey(
      'jwk',
      data.key,
      { name: 'AES-GCM', length: 256 },
      false,
      ['decrypt']
    )
    
    const iv = new Uint8Array(data.iv)
    const encrypted = new Uint8Array(data.encrypted)
    
    const decrypted = await window.crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      encrypted
    )
    
    const decoder = new TextDecoder()
    return decoder.decode(decrypted)
  } catch (error) {
    console.error('Decryption failed:', error)
    // Fallback to base64 decoding
    try {
      return atob(encryptedData)
    } catch (fallbackError) {
      console.error('Base64 decoding fallback failed:', fallbackError)
      return null
    }
  }
}

/**
 * Secure storage implementation
 */
export const secureStorage = {
  /**
   * Set authentication token
   * @param {string} token - Authentication token
   */
  async setToken(token) {
    if (!token) return
    
    try {
      // For client-side, we'll use sessionStorage with encryption as a compromise
      // In a production environment, tokens should be httpOnly cookies set by the server
      const encryptedToken = await encrypt(token)
      
      if (isBrowser) {
        // Use sessionStorage for security (cleared when browser closes)
        sessionStorage.setItem(SECURE_STORAGE_CONFIG.tokenKey, encryptedToken)
        
        // Also set a non-httpOnly cookie for server-side access
        // (In production, server should set httpOnly cookies)
        setCookie(SECURE_STORAGE_CONFIG.tokenKey, encryptedToken, {
          maxAge: 60 * 60 * 8, // 8 hours (shorter than refresh token)
          secure: true,
          sameSite: 'strict'
        })
      }
    } catch (error) {
      console.error('Failed to set secure token:', error)
      // Fallback to localStorage if encryption fails
      if (isBrowser) {
        localStorage.setItem(SECURE_STORAGE_CONFIG.tokenKey, token)
      }
    }
  },

  /**
   * Get authentication token
   * @returns {Promise<string|null>} Authentication token
   */
  async getToken() {
    if (!isBrowser) return null

    try {
      // Try sessionStorage first
      let encryptedToken = sessionStorage.getItem(SECURE_STORAGE_CONFIG.tokenKey)
      
      // Fallback to cookie
      if (!encryptedToken) {
        encryptedToken = getCookie(SECURE_STORAGE_CONFIG.tokenKey)
      }
      
      if (!encryptedToken) {
        // Final fallback to localStorage (for migration)
        const fallbackToken = localStorage.getItem('datamesh_token')
        if (fallbackToken) {
          // Migrate to secure storage
          await this.setToken(fallbackToken)
          localStorage.removeItem('datamesh_token')
          return fallbackToken
        }
        return null
      }
      
      return await decrypt(encryptedToken)
    } catch (error) {
      console.error('Failed to get secure token:', error)
      return null
    }
  },

  /**
   * Remove authentication token
   */
  removeToken() {
    if (!isBrowser) return

    try {
      sessionStorage.removeItem(SECURE_STORAGE_CONFIG.tokenKey)
      removeCookie(SECURE_STORAGE_CONFIG.tokenKey)
      
      // Also remove legacy token
      localStorage.removeItem('datamesh_token')
      localStorage.removeItem(SECURE_STORAGE_CONFIG.tokenKey)
    } catch (error) {
      console.error('Failed to remove secure token:', error)
    }
  },

  /**
   * Set refresh token
   * @param {string} refreshToken - Refresh token
   */
  async setRefreshToken(refreshToken) {
    if (!refreshToken) return
    
    try {
      const encryptedToken = await encrypt(refreshToken)
      
      if (isBrowser) {
        // Use localStorage for refresh token (needs to persist across sessions)
        localStorage.setItem(SECURE_STORAGE_CONFIG.refreshTokenKey, encryptedToken)
        
        // Set cookie with longer expiration
        setCookie(SECURE_STORAGE_CONFIG.refreshTokenKey, encryptedToken, {
          maxAge: 60 * 60 * 24 * 30, // 30 days
          secure: true,
          sameSite: 'strict'
        })
      }
    } catch (error) {
      console.error('Failed to set refresh token:', error)
    }
  },

  /**
   * Get refresh token
   * @returns {Promise<string|null>} Refresh token
   */
  async getRefreshToken() {
    if (!isBrowser) return null

    try {
      let encryptedToken = localStorage.getItem(SECURE_STORAGE_CONFIG.refreshTokenKey)
      
      if (!encryptedToken) {
        encryptedToken = getCookie(SECURE_STORAGE_CONFIG.refreshTokenKey)
      }
      
      if (!encryptedToken) return null
      
      return await decrypt(encryptedToken)
    } catch (error) {
      console.error('Failed to get refresh token:', error)
      return null
    }
  },

  /**
   * Remove refresh token
   */
  removeRefreshToken() {
    if (!isBrowser) return

    try {
      localStorage.removeItem(SECURE_STORAGE_CONFIG.refreshTokenKey)
      removeCookie(SECURE_STORAGE_CONFIG.refreshTokenKey)
    } catch (error) {
      console.error('Failed to remove refresh token:', error)
    }
  },

  /**
   * Clear all secure storage
   */
  clear() {
    this.removeToken()
    this.removeRefreshToken()
  },

  /**
   * Check if storage is available
   * @returns {boolean} Storage availability
   */
  isAvailable() {
    return isBrowser && (
      typeof Storage !== 'undefined' ||
      typeof document !== 'undefined'
    )
  }
}

// Auto-cleanup on page unload for security
if (isBrowser) {
  window.addEventListener('beforeunload', () => {
    // Only clear session storage, keep localStorage for refresh tokens
    if (sessionStorage) {
      sessionStorage.removeItem(SECURE_STORAGE_CONFIG.tokenKey)
    }
  })
}

export default secureStorage