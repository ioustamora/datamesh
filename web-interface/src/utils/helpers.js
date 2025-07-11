/**
 * Utility functions and helpers for the DataMesh application
 */

/**
 * Debounce function to limit the rate of function execution
 * @param {Function} func - Function to debounce
 * @param {number} wait - Wait time in milliseconds
 * @param {boolean} immediate - Execute immediately on first call
 * @returns {Function} Debounced function
 */
export function debounce(func, wait, immediate = false) {
  let timeout
  return function executedFunction(...args) {
    const later = () => {
      timeout = null
      if (!immediate) func(...args)
    }
    const callNow = immediate && !timeout
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
    if (callNow) func(...args)
  }
}

/**
 * Throttle function to limit the frequency of function execution
 * @param {Function} func - Function to throttle
 * @param {number} limit - Time limit in milliseconds
 * @returns {Function} Throttled function
 */
export function throttle(func, limit) {
  let inThrottle
  return function(...args) {
    if (!inThrottle) {
      func.apply(this, args)
      inThrottle = true
      setTimeout(() => inThrottle = false, limit)
    }
  }
}

/**
 * Format file size in human readable format
 * @param {number} bytes - File size in bytes
 * @param {number} decimals - Number of decimal places
 * @returns {string} Formatted file size
 */
export function formatFileSize(bytes, decimals = 1) {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

/**
 * Format date in relative time (e.g., "2 hours ago")
 * @param {Date|string} date - Date to format
 * @returns {string} Relative time string
 */
export function formatRelativeTime(date) {
  const now = new Date()
  const targetDate = new Date(date)
  const diffInSeconds = Math.floor((now - targetDate) / 1000)
  
  if (diffInSeconds < 60) return 'Just now'
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`
  if (diffInSeconds < 2592000) return `${Math.floor(diffInSeconds / 86400)}d ago`
  if (diffInSeconds < 31536000) return `${Math.floor(diffInSeconds / 2592000)}mo ago`
  return `${Math.floor(diffInSeconds / 31536000)}y ago`
}

/**
 * Format date for display
 * @param {Date|string} date - Date to format
 * @param {string} locale - Locale for formatting
 * @returns {string} Formatted date string
 */
export function formatDate(date, locale = 'en-US') {
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(date))
}

/**
 * Generate a random UUID v4
 * @returns {string} UUID string
 */
export function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0
    const v = c === 'x' ? r : (r & 0x3 | 0x8)
    return v.toString(16)
  })
}

/**
 * Deep clone an object
 * @param {*} obj - Object to clone
 * @returns {*} Cloned object
 */
export function deepClone(obj) {
  if (obj === null || typeof obj !== 'object') return obj
  if (obj instanceof Date) return new Date(obj.getTime())
  if (obj instanceof Array) return obj.map(item => deepClone(item))
  if (typeof obj === 'object') {
    const clonedObj = {}
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        clonedObj[key] = deepClone(obj[key])
      }
    }
    return clonedObj
  }
}

/**
 * Check if a value is empty (null, undefined, empty string, empty array, empty object)
 * @param {*} value - Value to check
 * @returns {boolean} True if empty
 */
export function isEmpty(value) {
  if (value === null || value === undefined) return true
  if (typeof value === 'string') return value.trim() === ''
  if (Array.isArray(value)) return value.length === 0
  if (typeof value === 'object') return Object.keys(value).length === 0
  return false
}

/**
 * Capitalize first letter of a string
 * @param {string} str - String to capitalize
 * @returns {string} Capitalized string
 */
export function capitalize(str) {
  if (!str) return ''
  return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase()
}

/**
 * Convert camelCase to kebab-case
 * @param {string} str - String to convert
 * @returns {string} Kebab-case string
 */
export function camelToKebab(str) {
  return str.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase()
}

/**
 * Convert kebab-case to camelCase
 * @param {string} str - String to convert
 * @returns {string} CamelCase string
 */
export function kebabToCamel(str) {
  return str.replace(/-([a-z])/g, (match, letter) => letter.toUpperCase())
}

/**
 * Truncate text to specified length
 * @param {string} text - Text to truncate
 * @param {number} length - Maximum length
 * @param {string} suffix - Suffix to add when truncated
 * @returns {string} Truncated text
 */
export function truncateText(text, length = 100, suffix = '...') {
  if (!text || text.length <= length) return text
  return text.substring(0, length).trim() + suffix
}

/**
 * Extract file extension from filename
 * @param {string} filename - Filename
 * @returns {string} File extension (without dot)
 */
export function getFileExtension(filename) {
  if (!filename) return ''
  const lastDot = filename.lastIndexOf('.')
  return lastDot >= 0 ? filename.substring(lastDot + 1).toLowerCase() : ''
}

/**
 * Get file type category based on extension
 * @param {string} filename - Filename
 * @returns {string} File type category
 */
export function getFileType(filename) {
  const ext = getFileExtension(filename)
  
  const imageTypes = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'ico']
  const videoTypes = ['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm', 'm4v']
  const audioTypes = ['mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a']
  const documentTypes = ['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt']
  const spreadsheetTypes = ['xls', 'xlsx', 'csv', 'ods']
  const presentationTypes = ['ppt', 'pptx', 'odp']
  const archiveTypes = ['zip', 'rar', '7z', 'tar', 'gz', 'bz2']
  const codeTypes = ['js', 'ts', 'html', 'css', 'py', 'java', 'cpp', 'c', 'php', 'rb', 'go', 'rs', 'vue', 'jsx', 'tsx']
  
  if (imageTypes.includes(ext)) return 'image'
  if (videoTypes.includes(ext)) return 'video'
  if (audioTypes.includes(ext)) return 'audio'
  if (documentTypes.includes(ext)) return 'document'
  if (spreadsheetTypes.includes(ext)) return 'spreadsheet'
  if (presentationTypes.includes(ext)) return 'presentation'
  if (archiveTypes.includes(ext)) return 'archive'
  if (codeTypes.includes(ext)) return 'code'
  
  return 'file'
}

/**
 * Download a file from URL
 * @param {string} url - File URL
 * @param {string} filename - Desired filename
 */
export function downloadFile(url, filename) {
  const link = document.createElement('a')
  link.href = url
  link.download = filename || 'download'
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
}

/**
 * Copy text to clipboard
 * @param {string} text - Text to copy
 * @returns {Promise<boolean>} Success status
 */
export async function copyToClipboard(text) {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text)
      return true
    } else {
      // Fallback for older browsers
      const textArea = document.createElement('textarea')
      textArea.value = text
      textArea.style.position = 'fixed'
      textArea.style.left = '-999999px'
      textArea.style.top = '-999999px'
      document.body.appendChild(textArea)
      textArea.focus()
      textArea.select()
      const success = document.execCommand('copy')
      document.body.removeChild(textArea)
      return success
    }
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
    return false
  }
}

/**
 * Validate email address
 * @param {string} email - Email to validate
 * @returns {boolean} True if valid
 */
export function isValidEmail(email) {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email)
}

/**
 * Validate URL
 * @param {string} url - URL to validate
 * @returns {boolean} True if valid
 */
export function isValidUrl(url) {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * Generate a random color
 * @param {string} format - Color format ('hex', 'rgb', 'hsl')
 * @returns {string} Random color
 */
export function generateRandomColor(format = 'hex') {
  switch (format) {
    case 'hex':
      return '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0')
    case 'rgb':
      const r = Math.floor(Math.random() * 256)
      const g = Math.floor(Math.random() * 256)
      const b = Math.floor(Math.random() * 256)
      return `rgb(${r}, ${g}, ${b})`
    case 'hsl':
      const h = Math.floor(Math.random() * 360)
      const s = Math.floor(Math.random() * 100)
      const l = Math.floor(Math.random() * 100)
      return `hsl(${h}, ${s}%, ${l}%)`
    default:
      return generateRandomColor('hex')
  }
}

/**
 * Calculate reading time for text
 * @param {string} text - Text to analyze
 * @param {number} wpm - Words per minute (default 200)
 * @returns {number} Reading time in minutes
 */
export function calculateReadingTime(text, wpm = 200) {
  const words = text.trim().split(/\s+/).length
  return Math.ceil(words / wpm)
}

/**
 * Smooth scroll to element
 * @param {string|Element} target - Target element or selector
 * @param {number} offset - Offset from top
 * @param {string} behavior - Scroll behavior
 */
export function scrollToElement(target, offset = 0, behavior = 'smooth') {
  const element = typeof target === 'string' ? document.querySelector(target) : target
  if (!element) return
  
  const elementPosition = element.getBoundingClientRect().top + window.pageYOffset
  const offsetPosition = elementPosition - offset
  
  window.scrollTo({
    top: offsetPosition,
    behavior
  })
}

/**
 * Check if element is in viewport
 * @param {Element} element - Element to check
 * @param {number} threshold - Threshold ratio (0-1)
 * @returns {boolean} True if in viewport
 */
export function isElementInViewport(element, threshold = 0) {
  const rect = element.getBoundingClientRect()
  const windowHeight = window.innerHeight || document.documentElement.clientHeight
  const windowWidth = window.innerWidth || document.documentElement.clientWidth
  
  const vertInView = (rect.top + (rect.height * threshold)) <= windowHeight && 
                     (rect.bottom - (rect.height * threshold)) >= 0
  const horInView = (rect.left + (rect.width * threshold)) <= windowWidth && 
                    (rect.right - (rect.width * threshold)) >= 0
  
  return vertInView && horInView
}

/**
 * Load image with promise
 * @param {string} src - Image source
 * @returns {Promise<HTMLImageElement>} Loaded image element
 */
export function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = reject
    img.src = src
  })
}

/**
 * Create a delay/sleep function
 * @param {number} ms - Milliseconds to wait
 * @returns {Promise} Promise that resolves after delay
 */
export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Retry function with exponential backoff
 * @param {Function} fn - Function to retry
 * @param {number} maxAttempts - Maximum retry attempts
 * @param {number} baseDelay - Base delay in milliseconds
 * @returns {Promise} Promise resolving to function result
 */
export async function retryWithBackoff(fn, maxAttempts = 3, baseDelay = 1000) {
  let attempt = 1
  
  while (attempt <= maxAttempts) {
    try {
      return await fn()
    } catch (error) {
      if (attempt === maxAttempts) {
        throw error
      }
      
      const delay = baseDelay * Math.pow(2, attempt - 1)
      await sleep(delay)
      attempt++
    }
  }
}

/**
 * Format number with thousand separators
 * @param {number} num - Number to format
 * @param {string} locale - Locale for formatting
 * @returns {string} Formatted number
 */
export function formatNumber(num, locale = 'en-US') {
  return new Intl.NumberFormat(locale).format(num)
}

/**
 * Calculate percentage
 * @param {number} value - Current value
 * @param {number} total - Total value
 * @param {number} decimals - Number of decimal places
 * @returns {number} Percentage
 */
export function calculatePercentage(value, total, decimals = 1) {
  if (total === 0) return 0
  return Math.round((value / total) * 100 * Math.pow(10, decimals)) / Math.pow(10, decimals)
}

/**
 * Clamp a value between min and max
 * @param {number} value - Value to clamp
 * @param {number} min - Minimum value
 * @param {number} max - Maximum value
 * @returns {number} Clamped value
 */
export function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max)
}

/**
 * Convert RGB to Hex
 * @param {number} r - Red component (0-255)
 * @param {number} g - Green component (0-255)
 * @param {number} b - Blue component (0-255)
 * @returns {string} Hex color
 */
export function rgbToHex(r, g, b) {
  return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)
}

/**
 * Convert Hex to RGB
 * @param {string} hex - Hex color
 * @returns {object} RGB object {r, g, b}
 */
export function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null
}