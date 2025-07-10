/**
 * Accessibility utilities and helpers
 * Provides ARIA support, keyboard navigation, and screen reader optimizations
 */

/**
 * Focus management utilities
 */
class FocusManager {
  constructor() {
    this.focusHistory = []
    this.trapStack = []
  }

  /**
   * Save current focus and move to target element
   * @param {HTMLElement} target - Element to focus
   * @param {Object} options - Focus options
   */
  moveFocus(target, options = {}) {
    if (!target) return

    // Save current focus
    if (document.activeElement && document.activeElement !== document.body) {
      this.focusHistory.push(document.activeElement)
    }

    // Move focus to target
    target.focus(options)
  }

  /**
   * Restore previously saved focus
   */
  restoreFocus() {
    if (this.focusHistory.length > 0) {
      const previousFocus = this.focusHistory.pop()
      if (previousFocus && previousFocus.focus) {
        previousFocus.focus()
      }
    }
  }

  /**
   * Create focus trap within container
   * @param {HTMLElement} container - Container element
   * @returns {Function} Function to release trap
   */
  trapFocus(container) {
    if (!container) return () => {}

    const focusableElements = this.getFocusableElements(container)
    if (focusableElements.length === 0) return () => {}

    const firstElement = focusableElements[0]
    const lastElement = focusableElements[focusableElements.length - 1]

    const handleKeyDown = (event) => {
      if (event.key !== 'Tab') return

      if (event.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstElement) {
          event.preventDefault()
          lastElement.focus()
        }
      } else {
        // Tab
        if (document.activeElement === lastElement) {
          event.preventDefault()
          firstElement.focus()
        }
      }
    }

    container.addEventListener('keydown', handleKeyDown)
    this.trapStack.push({ container, handler: handleKeyDown })

    // Focus first element
    firstElement.focus()

    // Return release function
    return () => {
      container.removeEventListener('keydown', handleKeyDown)
      const trapIndex = this.trapStack.findIndex(trap => trap.container === container)
      if (trapIndex >= 0) {
        this.trapStack.splice(trapIndex, 1)
      }
    }
  }

  /**
   * Get all focusable elements within container
   * @param {HTMLElement} container - Container element
   * @returns {HTMLElement[]} Focusable elements
   */
  getFocusableElements(container) {
    const focusableSelectors = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]'
    ].join(', ')

    const elements = container.querySelectorAll(focusableSelectors)
    return Array.from(elements).filter(element => {
      return element.offsetWidth > 0 && element.offsetHeight > 0
    })
  }

  /**
   * Clear all focus traps
   */
  clearAllTraps() {
    this.trapStack.forEach(({ container, handler }) => {
      container.removeEventListener('keydown', handler)
    })
    this.trapStack = []
  }
}

/**
 * Screen reader utilities
 */
class ScreenReaderUtils {
  constructor() {
    this.announcements = []
    this.setupLiveRegion()
  }

  /**
   * Set up ARIA live region for announcements
   */
  setupLiveRegion() {
    // Create polite live region
    this.politeRegion = document.createElement('div')
    this.politeRegion.setAttribute('aria-live', 'polite')
    this.politeRegion.setAttribute('aria-atomic', 'true')
    this.politeRegion.className = 'sr-only'
    document.body.appendChild(this.politeRegion)

    // Create assertive live region
    this.assertiveRegion = document.createElement('div')
    this.assertiveRegion.setAttribute('aria-live', 'assertive')
    this.assertiveRegion.setAttribute('aria-atomic', 'true')
    this.assertiveRegion.className = 'sr-only'
    document.body.appendChild(this.assertiveRegion)

    // Create status region
    this.statusRegion = document.createElement('div')
    this.statusRegion.setAttribute('role', 'status')
    this.statusRegion.setAttribute('aria-live', 'polite')
    this.statusRegion.className = 'sr-only'
    document.body.appendChild(this.statusRegion)
  }

  /**
   * Announce message to screen readers
   * @param {string} message - Message to announce
   * @param {string} priority - Priority ('polite', 'assertive', 'status')
   * @param {number} delay - Delay before announcement (ms)
   */
  announce(message, priority = 'polite', delay = 100) {
    if (!message) return

    const announcement = { message, priority, timestamp: Date.now() }
    this.announcements.push(announcement)

    setTimeout(() => {
      let region
      switch (priority) {
        case 'assertive':
          region = this.assertiveRegion
          break
        case 'status':
          region = this.statusRegion
          break
        default:
          region = this.politeRegion
      }

      if (region) {
        region.textContent = ''
        setTimeout(() => {
          region.textContent = message
        }, 50)
      }
    }, delay)
  }

  /**
   * Announce page navigation
   * @param {string} pageName - Name of the page
   * @param {string} description - Optional description
   */
  announceNavigation(pageName, description = '') {
    const message = description 
      ? `Navigated to ${pageName}. ${description}`
      : `Navigated to ${pageName}`
    
    this.announce(message, 'polite', 200)
  }

  /**
   * Announce loading state
   * @param {boolean} isLoading - Loading state
   * @param {string} context - Context of loading
   */
  announceLoading(isLoading, context = 'content') {
    const message = isLoading 
      ? `Loading ${context}...`
      : `${context} loaded`
    
    this.announce(message, 'status')
  }

  /**
   * Announce error
   * @param {string} error - Error message
   */
  announceError(error) {
    this.announce(`Error: ${error}`, 'assertive')
  }

  /**
   * Announce success
   * @param {string} message - Success message
   */
  announceSuccess(message) {
    this.announce(`Success: ${message}`, 'polite')
  }
}

/**
 * Keyboard navigation utilities
 */
class KeyboardNavigation {
  constructor() {
    this.navigationHandlers = new Map()
  }

  /**
   * Add keyboard navigation to element
   * @param {HTMLElement} element - Element to add navigation to
   * @param {Object} options - Navigation options
   */
  addNavigation(element, options = {}) {
    const {
      keys = ['ArrowUp', 'ArrowDown', 'Home', 'End'],
      orientation = 'vertical',
      wrap = true,
      selector = '[role="option"], button, a, input',
      onNavigate = () => {}
    } = options

    const handler = (event) => {
      if (!keys.includes(event.key)) return

      const items = Array.from(element.querySelectorAll(selector))
        .filter(item => !item.disabled && item.offsetParent !== null)

      if (items.length === 0) return

      const currentIndex = items.findIndex(item => item === document.activeElement)
      let newIndex = currentIndex

      switch (event.key) {
        case 'ArrowUp':
          if (orientation === 'vertical') {
            event.preventDefault()
            newIndex = currentIndex > 0 ? currentIndex - 1 : (wrap ? items.length - 1 : 0)
          }
          break

        case 'ArrowDown':
          if (orientation === 'vertical') {
            event.preventDefault()
            newIndex = currentIndex < items.length - 1 ? currentIndex + 1 : (wrap ? 0 : items.length - 1)
          }
          break

        case 'ArrowLeft':
          if (orientation === 'horizontal') {
            event.preventDefault()
            newIndex = currentIndex > 0 ? currentIndex - 1 : (wrap ? items.length - 1 : 0)
          }
          break

        case 'ArrowRight':
          if (orientation === 'horizontal') {
            event.preventDefault()
            newIndex = currentIndex < items.length - 1 ? currentIndex + 1 : (wrap ? 0 : items.length - 1)
          }
          break

        case 'Home':
          event.preventDefault()
          newIndex = 0
          break

        case 'End':
          event.preventDefault()
          newIndex = items.length - 1
          break
      }

      if (newIndex !== currentIndex && items[newIndex]) {
        items[newIndex].focus()
        onNavigate(items[newIndex], newIndex)
      }
    }

    element.addEventListener('keydown', handler)
    this.navigationHandlers.set(element, handler)

    return () => this.removeNavigation(element)
  }

  /**
   * Remove keyboard navigation from element
   * @param {HTMLElement} element - Element to remove navigation from
   */
  removeNavigation(element) {
    const handler = this.navigationHandlers.get(element)
    if (handler) {
      element.removeEventListener('keydown', handler)
      this.navigationHandlers.delete(element)
    }
  }

  /**
   * Clear all navigation handlers
   */
  clearAll() {
    this.navigationHandlers.forEach((handler, element) => {
      element.removeEventListener('keydown', handler)
    })
    this.navigationHandlers.clear()
  }
}

/**
 * ARIA utilities
 */
class AriaUtils {
  /**
   * Set ARIA attributes on element
   * @param {HTMLElement} element - Target element
   * @param {Object} attributes - ARIA attributes to set
   */
  static setAttributes(element, attributes) {
    if (!element || !attributes) return

    Object.entries(attributes).forEach(([key, value]) => {
      if (value !== null && value !== undefined) {
        element.setAttribute(key, value)
      } else {
        element.removeAttribute(key)
      }
    })
  }

  /**
   * Create unique ID for accessibility
   * @param {string} prefix - ID prefix
   * @returns {string} Unique ID
   */
  static createId(prefix = 'a11y') {
    return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }

  /**
   * Set up combobox ARIA attributes
   * @param {HTMLElement} input - Input element
   * @param {HTMLElement} listbox - Listbox element
   * @param {Object} options - Configuration options
   */
  static setupCombobox(input, listbox, options = {}) {
    const {
      expanded = false,
      hasPopup = 'listbox',
      autocomplete = 'list'
    } = options

    const listboxId = listbox.id || AriaUtils.createId('listbox')
    listbox.id = listboxId

    AriaUtils.setAttributes(input, {
      'role': 'combobox',
      'aria-expanded': expanded.toString(),
      'aria-haspopup': hasPopup,
      'aria-owns': listboxId,
      'aria-autocomplete': autocomplete
    })

    AriaUtils.setAttributes(listbox, {
      'role': 'listbox',
      'aria-labelledby': input.id
    })
  }

  /**
   * Set up dialog ARIA attributes
   * @param {HTMLElement} dialog - Dialog element
   * @param {Object} options - Configuration options
   */
  static setupDialog(dialog, options = {}) {
    const {
      modal = true,
      labelledBy = '',
      describedBy = ''
    } = options

    AriaUtils.setAttributes(dialog, {
      'role': 'dialog',
      'aria-modal': modal.toString(),
      'aria-labelledby': labelledBy || null,
      'aria-describedby': describedBy || null
    })
  }

  /**
   * Set up table ARIA attributes
   * @param {HTMLElement} table - Table element
   * @param {Object} options - Configuration options
   */
  static setupTable(table, options = {}) {
    const {
      caption = '',
      rowCount = 0,
      colCount = 0,
      sortable = false
    } = options

    AriaUtils.setAttributes(table, {
      'role': 'table',
      'aria-label': caption || null,
      'aria-rowcount': rowCount > 0 ? rowCount.toString() : null,
      'aria-colcount': colCount > 0 ? colCount.toString() : null
    })

    if (sortable) {
      const headers = table.querySelectorAll('th')
      headers.forEach(header => {
        if (!header.getAttribute('aria-sort')) {
          header.setAttribute('aria-sort', 'none')
        }
      })
    }
  }
}

/**
 * Create global accessibility manager
 */
class AccessibilityManager {
  constructor() {
    this.focusManager = new FocusManager()
    this.screenReader = new ScreenReaderUtils()
    this.keyboardNav = new KeyboardNavigation()
    
    this.init()
  }

  init() {
    // Add skip link
    this.addSkipLink()
    
    // Set up global keyboard shortcuts
    this.setupGlobalShortcuts()
    
    // Add focus indicators for keyboard navigation
    this.addFocusIndicators()
    
    // Set up reduced motion preferences
    this.handleReducedMotion()
  }

  addSkipLink() {
    const skipLink = document.createElement('a')
    skipLink.href = '#main-content'
    skipLink.textContent = 'Skip to main content'
    skipLink.className = 'skip-link'
    skipLink.addEventListener('click', (e) => {
      e.preventDefault()
      const mainContent = document.getElementById('main-content') || document.querySelector('main')
      if (mainContent) {
        mainContent.focus()
        mainContent.scrollIntoView()
      }
    })
    
    document.body.insertBefore(skipLink, document.body.firstChild)
  }

  setupGlobalShortcuts() {
    document.addEventListener('keydown', (event) => {
      // Alt + M: Move to main content
      if (event.altKey && event.key === 'm') {
        event.preventDefault()
        const mainContent = document.getElementById('main-content') || document.querySelector('main')
        if (mainContent) {
          mainContent.focus()
          this.screenReader.announce('Moved to main content')
        }
      }
      
      // Alt + N: Move to navigation
      if (event.altKey && event.key === 'n') {
        event.preventDefault()
        const nav = document.querySelector('nav') || document.querySelector('[role="navigation"]')
        if (nav) {
          const firstLink = nav.querySelector('a, button')
          if (firstLink) {
            firstLink.focus()
            this.screenReader.announce('Moved to navigation')
          }
        }
      }
    })
  }

  addFocusIndicators() {
    // Add custom focus styles for better visibility
    const style = document.createElement('style')
    style.textContent = `
      .focus-visible {
        outline: 2px solid var(--el-color-primary, #409EFF);
        outline-offset: 2px;
      }
      
      .skip-link {
        position: absolute;
        top: -40px;
        left: 6px;
        background: var(--el-color-primary, #409EFF);
        color: white;
        padding: 8px;
        text-decoration: none;
        border-radius: 4px;
        z-index: 10000;
        transition: top 0.3s;
      }
      
      .skip-link:focus {
        top: 6px;
      }
      
      .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
      }
      
      @media (prefers-reduced-motion: reduce) {
        * {
          animation-duration: 0.01ms !important;
          animation-iteration-count: 1 !important;
          transition-duration: 0.01ms !important;
        }
      }
    `
    document.head.appendChild(style)
  }

  handleReducedMotion() {
    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
    
    const handleChange = (e) => {
      document.body.classList.toggle('reduced-motion', e.matches)
    }
    
    handleChange(prefersReducedMotion)
    prefersReducedMotion.addEventListener('change', handleChange)
  }

  destroy() {
    this.focusManager.clearAllTraps()
    this.keyboardNav.clearAll()
  }
}

// Create and export global instance
export const a11y = new AccessibilityManager()

// Export individual classes
export {
  FocusManager,
  ScreenReaderUtils,
  KeyboardNavigation,
  AriaUtils,
  AccessibilityManager
}

export default a11y