/**
 * Enhanced Security System
 */

/**
 * Comprehensive security manager
 */
export class SecurityManager {
  constructor(options = {}) {
    this.options = {
      csrfProtection: options.csrfProtection !== false,
      xssProtection: options.xssProtection !== false,
      contentSecurityPolicy: options.contentSecurityPolicy !== false,
      sessionTimeout: options.sessionTimeout || 30 * 60 * 1000, // 30 minutes
      maxLoginAttempts: options.maxLoginAttempts || 5,
      lockoutDuration: options.lockoutDuration || 15 * 60 * 1000, // 15 minutes
      ...options
    }

    // Security state
    this.csrfToken = null
    this.sessionData = new Map()
    this.loginAttempts = new Map()
    this.securityEvents = []
    this.permissions = new Map()
    this.encryptionKeys = new Map()

    // Security monitoring
    this.threats = new Map()
    this.anomalies = []
    this.securityScore = 100

    this.init()
  }

  /**
   * Initialize security system
   */
  async init() {
    await this.initCSRFProtection()
    await this.initXSSProtection()
    await this.initContentSecurityPolicy()
    await this.initSessionManagement()
    await this.initEncryption()
    
    this.startSecurityMonitoring()
    this.startThreatDetection()
  }

  /**
   * Initialize CSRF protection
   */
  async initCSRFProtection() {
    if (!this.options.csrfProtection) return

    try {
      // Generate CSRF token
      this.csrfToken = await this.generateCSRFToken()
      
      // Add token to all forms
      this.addCSRFTokenToForms()
      
      // Intercept AJAX requests
      this.interceptAjaxRequests()
      
      console.log('🛡️ Security: CSRF protection enabled')
    } catch (error) {
      console.error('🚫 Security: CSRF protection failed:', error)
    }
  }

  /**
   * Generate CSRF token
   */
  async generateCSRFToken() {
    const array = new Uint8Array(32)
    crypto.getRandomValues(array)
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('')
  }

  /**
   * Add CSRF token to forms
   */
  addCSRFTokenToForms() {
    const forms = document.querySelectorAll('form')
    
    forms.forEach(form => {
      // Check if token already exists
      const existingToken = form.querySelector('input[name="csrf_token"]')
      if (existingToken) return

      // Create hidden input with CSRF token
      const input = document.createElement('input')
      input.type = 'hidden'
      input.name = 'csrf_token'
      input.value = this.csrfToken
      
      form.appendChild(input)
    })
  }

  /**
   * Intercept AJAX requests to add CSRF token
   */
  interceptAjaxRequests() {
    // Override fetch
    const originalFetch = window.fetch
    window.fetch = async (url, options = {}) => {
      if (this.shouldAddCSRFToken(url, options)) {
        options.headers = {
          ...options.headers,
          'X-CSRF-Token': this.csrfToken
        }
      }
      
      return originalFetch(url, options)
    }

    // Override XMLHttpRequest
    const originalOpen = XMLHttpRequest.prototype.open
    XMLHttpRequest.prototype.open = function(method, url, ...args) {
      this._url = url
      this._method = method
      return originalOpen.call(this, method, url, ...args)
    }

    const originalSend = XMLHttpRequest.prototype.send
    XMLHttpRequest.prototype.send = function(data) {
      if (this._method !== 'GET' && this._url && !this._url.startsWith('http')) {
        this.setRequestHeader('X-CSRF-Token', this.csrfToken)
      }
      return originalSend.call(this, data)
    }.bind(this)
  }

  /**
   * Check if CSRF token should be added
   */
  shouldAddCSRFToken(url, options) {
    const method = options.method || 'GET'
    const isModifyingRequest = ['POST', 'PUT', 'DELETE', 'PATCH'].includes(method.toUpperCase())
    const isSameOrigin = !url.startsWith('http') || url.startsWith(window.location.origin)
    
    return isModifyingRequest && isSameOrigin
  }

  /**
   * Initialize XSS protection
   */
  async initXSSProtection() {
    if (!this.options.xssProtection) return

    try {
      // Sanitize DOM on load
      this.sanitizeDOM()
      
      // Monitor for XSS attempts
      this.monitorXSSAttempts()
      
      // Override dangerous methods
      this.overrideDangerousMethods()
      
      console.log('🛡️ Security: XSS protection enabled')
    } catch (error) {
      console.error('🚫 Security: XSS protection failed:', error)
    }
  }

  /**
   * Sanitize DOM content
   */
  sanitizeDOM() {
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_ALL,
      null,
      false
    )

    const nodesToSanitize = []
    let node

    while (node = walker.nextNode()) {
      if (this.isUnsafeNode(node)) {
        nodesToSanitize.push(node)
      }
    }

    nodesToSanitize.forEach(node => {
      this.sanitizeNode(node)
    })
  }

  /**
   * Check if node is unsafe
   */
  isUnsafeNode(node) {
    if (node.nodeType === Node.ELEMENT_NODE) {
      const tagName = node.tagName.toLowerCase()
      const dangerousTags = ['script', 'iframe', 'object', 'embed']
      
      if (dangerousTags.includes(tagName)) {
        return true
      }

      // Check for dangerous attributes
      const attributes = node.attributes
      for (let i = 0; i < attributes.length; i++) {
        const attr = attributes[i]
        if (attr.name.startsWith('on') || attr.value.includes('javascript:')) {
          return true
        }
      }
    }

    return false
  }

  /**
   * Sanitize unsafe node
   */
  sanitizeNode(node) {
    if (node.nodeType === Node.ELEMENT_NODE) {
      const tagName = node.tagName.toLowerCase()
      
      if (['script', 'iframe', 'object', 'embed'].includes(tagName)) {
        // Remove dangerous elements
        node.remove()
        this.logSecurityEvent('xss_blocked', `Removed dangerous ${tagName} element`)
        return
      }

      // Remove dangerous attributes
      const attributes = Array.from(node.attributes)
      attributes.forEach(attr => {
        if (attr.name.startsWith('on') || attr.value.includes('javascript:')) {
          node.removeAttribute(attr.name)
          this.logSecurityEvent('xss_blocked', `Removed dangerous attribute: ${attr.name}`)
        }
      })
    }
  }

  /**
   * Monitor XSS attempts
   */
  monitorXSSAttempts() {
    // Monitor URL parameters
    const urlParams = new URLSearchParams(window.location.search)
    urlParams.forEach((value, key) => {
      if (this.containsXSS(value)) {
        this.logSecurityEvent('xss_attempt', `XSS detected in URL parameter: ${key}`)
      }
    })

    // Monitor form inputs
    document.addEventListener('input', (event) => {
      if (this.containsXSS(event.target.value)) {
        this.logSecurityEvent('xss_attempt', 'XSS detected in form input')
        event.target.value = this.sanitizeInput(event.target.value)
      }
    })
  }

  /**
   * Check if content contains XSS
   */
  containsXSS(content) {
    const xssPatterns = [
      /<script[^>]*>.*?<\/script>/gi,
      /javascript:/gi,
      /on\w+\s*=/gi,
      /<iframe[^>]*>/gi,
      /eval\s*\(/gi,
      /document\.write/gi
    ]

    return xssPatterns.some(pattern => pattern.test(content))
  }

  /**
   * Sanitize input
   */
  sanitizeInput(input) {
    return input
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;')
      .replace(/\//g, '&#x2F;')
  }

  /**
   * Override dangerous methods
   */
  overrideDangerousMethods() {
    // Override eval
    const originalEval = window.eval
    window.eval = function(code) {
      console.warn('⚠️ Security: eval() usage detected')
      this.logSecurityEvent('eval_usage', code)
      return originalEval(code)
    }.bind(this)

    // Override innerHTML
    const originalInnerHTML = Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML')
    Object.defineProperty(Element.prototype, 'innerHTML', {
      set: function(value) {
        const sanitized = this.sanitizeHTML(value)
        return originalInnerHTML.set.call(this, sanitized)
      }.bind(this),
      get: originalInnerHTML.get
    })
  }

  /**
   * Sanitize HTML content
   */
  sanitizeHTML(html) {
    const div = document.createElement('div')
    div.innerHTML = html
    
    // Remove script tags
    const scripts = div.querySelectorAll('script')
    scripts.forEach(script => script.remove())
    
    return div.innerHTML
  }

  /**
   * Initialize Content Security Policy
   */
  async initContentSecurityPolicy() {
    if (!this.options.contentSecurityPolicy) return

    try {
      // Set CSP header via meta tag
      const meta = document.createElement('meta')
      meta.httpEquiv = 'Content-Security-Policy'
      meta.content = this.generateCSPHeader()
      
      document.head.appendChild(meta)
      
      // Monitor CSP violations
      this.monitorCSPViolations()
      
      console.log('🛡️ Security: Content Security Policy enabled')
    } catch (error) {
      console.error('🚫 Security: CSP initialization failed:', error)
    }
  }

  /**
   * Generate CSP header
   */
  generateCSPHeader() {
    return [
      "default-src 'self'",
      "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdnjs.cloudflare.com",
      "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
      "font-src 'self' https://fonts.gstatic.com",
      "img-src 'self' data: https:",
      "connect-src 'self' ws: wss:",
      "frame-src 'none'",
      "object-src 'none'",
      "base-uri 'self'",
      "form-action 'self'"
    ].join('; ')
  }

  /**
   * Monitor CSP violations
   */
  monitorCSPViolations() {
    document.addEventListener('securitypolicyviolation', (event) => {
      this.logSecurityEvent('csp_violation', {
        blockedURI: event.blockedURI,
        violatedDirective: event.violatedDirective,
        originalPolicy: event.originalPolicy
      })
    })
  }

  /**
   * Initialize session management
   */
  async initSessionManagement() {
    try {
      // Load existing session
      await this.loadSession()
      
      // Start session monitoring
      this.startSessionMonitoring()
      
      // Handle page visibility changes
      this.handleVisibilityChanges()
      
      console.log('🛡️ Security: Session management enabled')
    } catch (error) {
      console.error('🚫 Security: Session management failed:', error)
    }
  }

  /**
   * Load existing session
   */
  async loadSession() {
    try {
      const sessionData = localStorage.getItem('session_data')
      if (sessionData) {
        const session = JSON.parse(sessionData)
        
        // Check if session is expired
        if (session.expires && Date.now() > session.expires) {
          this.clearSession()
        } else {
          this.sessionData = new Map(session.data)
        }
      }
    } catch (error) {
      console.warn('Failed to load session:', error)
      this.clearSession()
    }
  }

  /**
   * Start session monitoring
   */
  startSessionMonitoring() {
    // Auto-save session periodically
    setInterval(() => {
      this.saveSession()
    }, 60000) // Every minute

    // Monitor user activity
    const events = ['click', 'keypress', 'scroll', 'mousemove']
    events.forEach(event => {
      document.addEventListener(event, () => {
        this.updateLastActivity()
      })
    })

    // Check for session timeout
    setInterval(() => {
      this.checkSessionTimeout()
    }, 30000) // Every 30 seconds
  }

  /**
   * Update last activity timestamp
   */
  updateLastActivity() {
    this.sessionData.set('lastActivity', Date.now())
  }

  /**
   * Check session timeout
   */
  checkSessionTimeout() {
    const lastActivity = this.sessionData.get('lastActivity')
    if (lastActivity && Date.now() - lastActivity > this.options.sessionTimeout) {
      this.handleSessionTimeout()
    }
  }

  /**
   * Handle session timeout
   */
  handleSessionTimeout() {
    this.clearSession()
    this.logSecurityEvent('session_timeout', 'Session expired due to inactivity')
    
    // Redirect to login or show timeout message
    this.handleLogout()
  }

  /**
   * Handle visibility changes
   */
  handleVisibilityChanges() {
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.sessionData.set('lastVisible', Date.now())
      } else {
        this.updateLastActivity()
      }
    })
  }

  /**
   * Save session
   */
  saveSession() {
    try {
      const sessionData = {
        data: Array.from(this.sessionData.entries()),
        expires: Date.now() + this.options.sessionTimeout
      }
      
      localStorage.setItem('session_data', JSON.stringify(sessionData))
    } catch (error) {
      console.warn('Failed to save session:', error)
    }
  }

  /**
   * Clear session
   */
  clearSession() {
    this.sessionData.clear()
    localStorage.removeItem('session_data')
  }

  /**
   * Initialize encryption
   */
  async initEncryption() {
    try {
      // Generate encryption key
      const key = await this.generateEncryptionKey()
      this.encryptionKeys.set('default', key)
      
      console.log('🛡️ Security: Encryption initialized')
    } catch (error) {
      console.error('🚫 Security: Encryption initialization failed:', error)
    }
  }

  /**
   * Generate encryption key
   */
  async generateEncryptionKey() {
    return await crypto.subtle.generateKey(
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    )
  }

  /**
   * Encrypt data
   */
  async encrypt(data, keyName = 'default') {
    const key = this.encryptionKeys.get(keyName)
    if (!key) throw new Error('Encryption key not found')

    const iv = crypto.getRandomValues(new Uint8Array(12))
    const encodedData = new TextEncoder().encode(JSON.stringify(data))

    const encryptedData = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encodedData
    )

    return {
      data: Array.from(new Uint8Array(encryptedData)),
      iv: Array.from(iv)
    }
  }

  /**
   * Decrypt data
   */
  async decrypt(encryptedData, keyName = 'default') {
    const key = this.encryptionKeys.get(keyName)
    if (!key) throw new Error('Encryption key not found')

    const data = new Uint8Array(encryptedData.data)
    const iv = new Uint8Array(encryptedData.iv)

    const decryptedData = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      data
    )

    const decodedData = new TextDecoder().decode(decryptedData)
    return JSON.parse(decodedData)
  }

  /**
   * Start security monitoring
   */
  startSecurityMonitoring() {
    // Monitor for suspicious activities
    setInterval(() => {
      this.checkForThreats()
    }, 30000) // Every 30 seconds

    // Monitor network requests
    this.monitorNetworkRequests()
    
    // Monitor DOM changes
    this.monitorDOMChanges()
  }

  /**
   * Check for threats
   */
  checkForThreats() {
    // Check for multiple login attempts
    this.checkLoginAttempts()
    
    // Check for suspicious patterns
    this.checkSuspiciousPatterns()
    
    // Update security score
    this.updateSecurityScore()
  }

  /**
   * Check login attempts
   */
  checkLoginAttempts() {
    const now = Date.now()
    
    for (const [ip, attempts] of this.loginAttempts) {
      // Clean up old attempts
      attempts.timestamps = attempts.timestamps.filter(
        timestamp => now - timestamp < this.options.lockoutDuration
      )

      // Check if IP should be locked
      if (attempts.timestamps.length >= this.options.maxLoginAttempts) {
        this.lockoutIP(ip)
      }
    }
  }

  /**
   * Record login attempt
   */
  recordLoginAttempt(ip, success) {
    if (!this.loginAttempts.has(ip)) {
      this.loginAttempts.set(ip, { timestamps: [], locked: false })
    }

    const attempts = this.loginAttempts.get(ip)
    
    if (success) {
      // Clear attempts on successful login
      attempts.timestamps = []
      attempts.locked = false
    } else {
      // Record failed attempt
      attempts.timestamps.push(Date.now())
    }

    this.logSecurityEvent('login_attempt', { ip, success })
  }

  /**
   * Lockout IP address
   */
  lockoutIP(ip) {
    const attempts = this.loginAttempts.get(ip)
    if (attempts) {
      attempts.locked = true
      this.logSecurityEvent('ip_lockout', { ip })
    }
  }

  /**
   * Check if IP is locked
   */
  isIPLocked(ip) {
    const attempts = this.loginAttempts.get(ip)
    return attempts && attempts.locked
  }

  /**
   * Monitor network requests
   */
  monitorNetworkRequests() {
    // Monitor fetch requests
    const originalFetch = window.fetch
    window.fetch = async (url, options = {}) => {
      this.logNetworkRequest(url, options)
      return originalFetch(url, options)
    }
  }

  /**
   * Log network request
   */
  logNetworkRequest(url, options) {
    // Check for suspicious requests
    if (this.isSuspiciousRequest(url, options)) {
      this.logSecurityEvent('suspicious_request', { url, options })
    }
  }

  /**
   * Check if request is suspicious
   */
  isSuspiciousRequest(url, options) {
    // Check for common attack patterns
    const suspiciousPatterns = [
      /\.\.\/.*\.\.\//, // Path traversal
      /admin|config|secret/, // Sensitive endpoints
      /eval|exec|system/ // Code execution
    ]

    return suspiciousPatterns.some(pattern => pattern.test(url))
  }

  /**
   * Monitor DOM changes
   */
  monitorDOMChanges() {
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === 'childList') {
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              this.checkNewElement(node)
            }
          })
        }
      })
    })

    observer.observe(document.body, {
      childList: true,
      subtree: true
    })
  }

  /**
   * Check new DOM element
   */
  checkNewElement(element) {
    if (this.isUnsafeNode(element)) {
      this.sanitizeNode(element)
    }
  }

  /**
   * Check suspicious patterns
   */
  checkSuspiciousPatterns() {
    // This would implement advanced pattern detection
    // For now, just check basic indicators
  }

  /**
   * Update security score
   */
  updateSecurityScore() {
    let score = 100
    
    // Deduct points for security events
    score -= this.securityEvents.length * 2
    
    // Deduct points for threats
    score -= this.threats.size * 5
    
    // Deduct points for anomalies
    score -= this.anomalies.length * 3
    
    this.securityScore = Math.max(0, score)
  }

  /**
   * Start threat detection
   */
  startThreatDetection() {
    // This would implement advanced threat detection
    // Using machine learning and behavioral analysis
    console.log('🔍 Security: Threat detection started')
  }

  /**
   * Log security event
   */
  logSecurityEvent(type, details) {
    const event = {
      type,
      details,
      timestamp: Date.now(),
      userAgent: navigator.userAgent,
      url: window.location.href
    }

    this.securityEvents.push(event)
    
    // Keep only recent events
    const cutoff = Date.now() - (24 * 60 * 60 * 1000) // 24 hours
    this.securityEvents = this.securityEvents.filter(e => e.timestamp > cutoff)
    
    console.warn(`⚠️ Security Event: ${type}`, details)
  }

  /**
   * Handle logout
   */
  handleLogout() {
    this.clearSession()
    this.logSecurityEvent('logout', 'User logged out')
    
    // Redirect to login page or emit event
    window.dispatchEvent(new CustomEvent('security:logout'))
  }

  /**
   * Get security status
   */
  getSecurityStatus() {
    return {
      score: this.securityScore,
      events: this.securityEvents.length,
      threats: this.threats.size,
      anomalies: this.anomalies.length,
      csrfEnabled: !!this.csrfToken,
      sessionActive: this.sessionData.size > 0
    }
  }

  /**
   * Get security events
   */
  getSecurityEvents() {
    return this.securityEvents
  }

  /**
   * Get threat information
   */
  getThreats() {
    return Array.from(this.threats.entries())
  }
}

export const securityManager = new SecurityManager()

export default {
  SecurityManager,
  securityManager
}
