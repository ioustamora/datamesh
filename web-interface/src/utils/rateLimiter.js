/**
 * Client-side rate limiter to prevent abuse and improve UX
 * Works in conjunction with server-side rate limiting
 */

class RateLimiter {
  constructor() {
    this.requests = new Map()
    this.retryQueue = new Map()
    this.config = {
      // Default rate limits (requests per minute)
      login: 5,
      register: 3,
      forgotPassword: 3,
      fileUpload: 20,
      search: 30,
      general: 60,
      
      // Retry configuration
      maxRetries: 3,
      retryDelay: 1000, // ms
      backoffMultiplier: 2,
      
      // Burst allowance
      burstLimit: 5,
      burstWindow: 1000, // ms
    }
  }

  /**
   * Get the rate limit for a specific endpoint
   * @param {string} endpoint - API endpoint
   * @returns {number} Rate limit per minute
   */
  getRateLimit(endpoint) {
    if (endpoint.includes('/auth/login')) return this.config.login
    if (endpoint.includes('/auth/register')) return this.config.register
    if (endpoint.includes('/auth/forgot-password')) return this.config.forgotPassword
    if (endpoint.includes('/files') && endpoint.includes('POST')) return this.config.fileUpload
    if (endpoint.includes('/search')) return this.config.search
    return this.config.general
  }

  /**
   * Generate a key for tracking requests
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   * @returns {string} Tracking key
   */
  getTrackingKey(endpoint, method = 'GET') {
    // Normalize endpoint to remove query parameters and IDs
    const normalizedEndpoint = endpoint
      .replace(/\/\d+/g, '/:id') // Replace numeric IDs
      .replace(/\?.*$/, '') // Remove query parameters
    
    return `${method}:${normalizedEndpoint}`
  }

  /**
   * Check if a request should be rate limited
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   * @returns {Object} Rate limit result
   */
  checkRateLimit(endpoint, method = 'GET') {
    const key = this.getTrackingKey(endpoint, method)
    const now = Date.now()
    const limit = this.getRateLimit(endpoint)
    const windowMs = 60000 // 1 minute
    
    if (!this.requests.has(key)) {
      this.requests.set(key, [])
    }
    
    const requests = this.requests.get(key)
    
    // Clean up old requests outside the window
    const windowStart = now - windowMs
    const recentRequests = requests.filter(timestamp => timestamp > windowStart)
    this.requests.set(key, recentRequests)
    
    // Check burst limit
    const burstStart = now - this.config.burstWindow
    const burstRequests = recentRequests.filter(timestamp => timestamp > burstStart)
    
    if (burstRequests.length >= this.config.burstLimit) {
      return {
        allowed: false,
        reason: 'burst_limit_exceeded',
        retryAfter: this.config.burstWindow - (now - burstRequests[0]),
        remaining: 0,
        limit: this.config.burstLimit
      }
    }
    
    // Check rate limit
    if (recentRequests.length >= limit) {
      return {
        allowed: false,
        reason: 'rate_limit_exceeded',
        retryAfter: windowMs - (now - recentRequests[0]),
        remaining: 0,
        limit: limit
      }
    }
    
    // Request is allowed
    recentRequests.push(now)
    this.requests.set(key, recentRequests)
    
    return {
      allowed: true,
      remaining: limit - recentRequests.length,
      limit: limit,
      resetTime: windowStart + windowMs
    }
  }

  /**
   * Add a request to the retry queue
   * @param {Function} requestFn - Function to make the request
   * @param {string} key - Tracking key
   * @param {number} attempt - Current attempt number
   * @returns {Promise} Promise that resolves when request succeeds
   */
  async addToRetryQueue(requestFn, key, attempt = 0) {
    if (attempt >= this.config.maxRetries) {
      throw new Error('Maximum retry attempts exceeded')
    }
    
    const delay = this.config.retryDelay * Math.pow(this.config.backoffMultiplier, attempt)
    
    return new Promise((resolve, reject) => {
      setTimeout(async () => {
        try {
          const result = await requestFn()
          resolve(result)
        } catch (error) {
          if (error.response?.status === 429) {
            // Rate limited, try again
            try {
              const retryResult = await this.addToRetryQueue(requestFn, key, attempt + 1)
              resolve(retryResult)
            } catch (retryError) {
              reject(retryError)
            }
          } else {
            reject(error)
          }
        }
      }, delay)
    })
  }

  /**
   * Execute a request with rate limiting
   * @param {Function} requestFn - Function to make the request
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   * @returns {Promise} Promise that resolves with the request result
   */
  async executeRequest(requestFn, endpoint, method = 'GET') {
    const rateLimit = this.checkRateLimit(endpoint, method)
    
    if (!rateLimit.allowed) {
      if (rateLimit.reason === 'rate_limit_exceeded') {
        // Add to retry queue
        const key = this.getTrackingKey(endpoint, method)
        return this.addToRetryQueue(requestFn, key)
      } else {
        // Burst limit exceeded, reject immediately
        const error = new Error('Rate limit exceeded')
        error.code = 'RATE_LIMIT_EXCEEDED'
        error.retryAfter = rateLimit.retryAfter
        error.reason = rateLimit.reason
        throw error
      }
    }
    
    try {
      return await requestFn()
    } catch (error) {
      if (error.response?.status === 429) {
        // Server-side rate limit, add to retry queue
        const key = this.getTrackingKey(endpoint, method)
        return this.addToRetryQueue(requestFn, key)
      }
      throw error
    }
  }

  /**
   * Get rate limit status for an endpoint
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   * @returns {Object} Rate limit status
   */
  getStatus(endpoint, method = 'GET') {
    const key = this.getTrackingKey(endpoint, method)
    const now = Date.now()
    const limit = this.getRateLimit(endpoint)
    const windowMs = 60000
    
    if (!this.requests.has(key)) {
      return {
        remaining: limit,
        limit: limit,
        resetTime: now + windowMs
      }
    }
    
    const requests = this.requests.get(key)
    const windowStart = now - windowMs
    const recentRequests = requests.filter(timestamp => timestamp > windowStart)
    
    return {
      remaining: Math.max(0, limit - recentRequests.length),
      limit: limit,
      resetTime: windowStart + windowMs
    }
  }

  /**
   * Clear rate limiting data for an endpoint
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   */
  clear(endpoint, method = 'GET') {
    const key = this.getTrackingKey(endpoint, method)
    this.requests.delete(key)
    this.retryQueue.delete(key)
  }

  /**
   * Clear all rate limiting data
   */
  clearAll() {
    this.requests.clear()
    this.retryQueue.clear()
  }
}

// Export singleton instance
export const rateLimiter = new RateLimiter()

/**
 * CSRF Token Manager
 * Handles CSRF token generation and validation
 */
class CSRFTokenManager {
  constructor() {
    this.token = null
    this.tokenExpiry = null
    this.refreshInterval = null
    this.config = {
      tokenLength: 32,
      refreshInterval: 15 * 60 * 1000, // 15 minutes
      metaTagName: 'csrf-token',
      headerName: 'X-CSRF-Token'
    }
  }

  /**
   * Generate a random CSRF token
   * @returns {string} CSRF token
   */
  generateToken() {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
    let token = ''
    
    if (window.crypto && window.crypto.getRandomValues) {
      // Use cryptographically secure random values
      const array = new Uint8Array(this.config.tokenLength)
      window.crypto.getRandomValues(array)
      
      for (let i = 0; i < array.length; i++) {
        token += chars[array[i] % chars.length]
      }
    } else {
      // Fallback to Math.random (less secure)
      for (let i = 0; i < this.config.tokenLength; i++) {
        token += chars[Math.floor(Math.random() * chars.length)]
      }
    }
    
    return token
  }

  /**
   * Initialize CSRF token
   */
  init() {
    // Try to get token from meta tag first
    const metaTag = document.querySelector(`meta[name="${this.config.metaTagName}"]`)
    if (metaTag) {
      this.token = metaTag.getAttribute('content')
    }
    
    // If no token found, generate one
    if (!this.token) {
      this.refreshToken()
    }
    
    // Set up automatic token refresh
    this.startAutoRefresh()
  }

  /**
   * Refresh CSRF token
   */
  refreshToken() {
    this.token = this.generateToken()
    this.tokenExpiry = Date.now() + this.config.refreshInterval
    
    // Update meta tag
    let metaTag = document.querySelector(`meta[name="${this.config.metaTagName}"]`)
    if (!metaTag) {
      metaTag = document.createElement('meta')
      metaTag.name = this.config.metaTagName
      document.head.appendChild(metaTag)
    }
    metaTag.content = this.token
    
    // Notify about token refresh
    window.dispatchEvent(new CustomEvent('csrf-token-refreshed', {
      detail: { token: this.token }
    }))
  }

  /**
   * Get current CSRF token
   * @returns {string} CSRF token
   */
  getToken() {
    if (!this.token || (this.tokenExpiry && Date.now() > this.tokenExpiry)) {
      this.refreshToken()
    }
    return this.token
  }

  /**
   * Start automatic token refresh
   */
  startAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval)
    }
    
    this.refreshInterval = setInterval(() => {
      this.refreshToken()
    }, this.config.refreshInterval)
  }

  /**
   * Stop automatic token refresh
   */
  stopAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval)
      this.refreshInterval = null
    }
  }

  /**
   * Validate CSRF token
   * @param {string} token - Token to validate
   * @returns {boolean} Whether token is valid
   */
  validateToken(token) {
    return token === this.token && token && token.length === this.config.tokenLength
  }

  /**
   * Get CSRF headers for requests
   * @returns {Object} Headers object
   */
  getHeaders() {
    return {
      [this.config.headerName]: this.getToken()
    }
  }
}

// Export singleton instance
export const csrfTokenManager = new CSRFTokenManager()

// Initialize CSRF token when module is loaded
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      csrfTokenManager.init()
    })
  } else {
    csrfTokenManager.init()
  }
}

export default { rateLimiter, csrfTokenManager }