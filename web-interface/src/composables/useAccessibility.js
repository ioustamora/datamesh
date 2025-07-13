/**
 * Advanced Accessibility Features for DataMesh
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'

// Accessibility levels
export const A11Y_LEVELS = {
  A: 'A',
  AA: 'AA',
  AAA: 'AAA'
}

// Accessibility preferences
export const A11Y_PREFERENCES = {
  REDUCED_MOTION: 'reduced-motion',
  HIGH_CONTRAST: 'high-contrast',
  LARGE_TEXT: 'large-text',
  SCREEN_READER: 'screen-reader',
  FOCUS_VISIBLE: 'focus-visible',
  KEYBOARD_NAVIGATION: 'keyboard-navigation'
}

// Color contrast ratios
export const CONTRAST_RATIOS = {
  NORMAL_AA: 4.5,
  NORMAL_AAA: 7,
  LARGE_AA: 3,
  LARGE_AAA: 4.5
}

// Screen reader utilities
export class ScreenReaderUtils {
  static announce(message, priority = 'polite') {
    const announcer = document.getElementById('sr-announcer')
    if (announcer) {
      announcer.setAttribute('aria-live', priority)
      announcer.textContent = message
      
      // Clear after announcement
      setTimeout(() => {
        announcer.textContent = ''
      }, 1000)
    }
  }

  static createAnnouncer() {
    if (!document.getElementById('sr-announcer')) {
      const announcer = document.createElement('div')
      announcer.id = 'sr-announcer'
      announcer.setAttribute('aria-live', 'polite')
      announcer.setAttribute('aria-atomic', 'true')
      announcer.style.cssText = `
        position: absolute;
        left: -10000px;
        width: 1px;
        height: 1px;
        overflow: hidden;
      `
      document.body.appendChild(announcer)
    }
  }

  static describeElement(element, description) {
    const descriptionId = `desc-${Math.random().toString(36).substr(2, 9)}`
    
    // Create description element
    const descriptionEl = document.createElement('div')
    descriptionEl.id = descriptionId
    descriptionEl.textContent = description
    descriptionEl.style.cssText = `
      position: absolute;
      left: -10000px;
      width: 1px;
      height: 1px;
      overflow: hidden;
    `
    
    document.body.appendChild(descriptionEl)
    element.setAttribute('aria-describedby', descriptionId)
    
    return descriptionId
  }
}

// Focus management
export class FocusManager {
  constructor() {
    this.focusHistory = []
    this.trapStack = []
    this.currentTrap = null
  }

  saveFocus() {
    const activeElement = document.activeElement
    if (activeElement && activeElement !== document.body) {
      this.focusHistory.push(activeElement)
    }
  }

  restoreFocus() {
    const lastFocus = this.focusHistory.pop()
    if (lastFocus && typeof lastFocus.focus === 'function') {
      lastFocus.focus()
    }
  }

  trapFocus(container) {
    this.saveFocus()
    
    const focusableElements = this.getFocusableElements(container)
    if (focusableElements.length === 0) return

    const firstElement = focusableElements[0]
    const lastElement = focusableElements[focusableElements.length - 1]

    const trapHandler = (event) => {
      if (event.key === 'Tab') {
        if (event.shiftKey) {
          if (document.activeElement === firstElement) {
            event.preventDefault()
            lastElement.focus()
          }
        } else {
          if (document.activeElement === lastElement) {
            event.preventDefault()
            firstElement.focus()
          }
        }
      }
      
      if (event.key === 'Escape') {
        this.releaseFocus()
      }
    }

    const trap = {
      container,
      handler: trapHandler,
      firstElement,
      lastElement
    }

    this.trapStack.push(trap)
    this.currentTrap = trap

    container.addEventListener('keydown', trapHandler)
    firstElement.focus()

    return trap
  }

  releaseFocus() {
    if (this.currentTrap) {
      this.currentTrap.container.removeEventListener('keydown', this.currentTrap.handler)
      this.trapStack.pop()
      this.currentTrap = this.trapStack[this.trapStack.length - 1]
      this.restoreFocus()
    }
  }

  getFocusableElements(container) {
    const focusableSelectors = [
      'a[href]',
      'area[href]',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'button:not([disabled])',
      'iframe',
      'object',
      'embed',
      '[contenteditable]',
      '[tabindex]:not([tabindex^="-"])'
    ]

    return Array.from(container.querySelectorAll(focusableSelectors.join(', ')))
      .filter(el => {
        return !el.hasAttribute('disabled') && 
               !el.getAttribute('aria-hidden') &&
               this.isVisible(el)
      })
  }

  isVisible(element) {
    const style = window.getComputedStyle(element)
    return style.display !== 'none' && 
           style.visibility !== 'hidden' && 
           style.opacity !== '0'
  }
}

// Color contrast checker
export class ContrastChecker {
  static getRGB(color) {
    const canvas = document.createElement('canvas')
    canvas.width = 1
    canvas.height = 1
    const ctx = canvas.getContext('2d')
    ctx.fillStyle = color
    ctx.fillRect(0, 0, 1, 1)
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data
    return { r, g, b }
  }

  static getLuminance(rgb) {
    const { r, g, b } = rgb
    const [rs, gs, bs] = [r, g, b].map(c => {
      c = c / 255
      return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
    })
    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs
  }

  static getContrastRatio(color1, color2) {
    const rgb1 = this.getRGB(color1)
    const rgb2 = this.getRGB(color2)
    const l1 = this.getLuminance(rgb1)
    const l2 = this.getLuminance(rgb2)
    const lighter = Math.max(l1, l2)
    const darker = Math.min(l1, l2)
    return (lighter + 0.05) / (darker + 0.05)
  }

  static checkContrast(foreground, background, level = 'AA', isLarge = false) {
    const ratio = this.getContrastRatio(foreground, background)
    const requiredRatio = this.getRequiredRatio(level, isLarge)
    
    return {
      ratio,
      requiredRatio,
      passes: ratio >= requiredRatio,
      level
    }
  }

  static getRequiredRatio(level, isLarge) {
    if (level === 'AAA') {
      return isLarge ? CONTRAST_RATIOS.LARGE_AAA : CONTRAST_RATIOS.NORMAL_AAA
    }
    return isLarge ? CONTRAST_RATIOS.LARGE_AA : CONTRAST_RATIOS.NORMAL_AA
  }
}

// Accessibility composable
export function useAccessibility() {
  const preferences = ref({
    reducedMotion: false,
    highContrast: false,
    largeText: false,
    screenReader: false,
    focusVisible: true,
    keyboardNavigation: true
  })

  const focusManager = new FocusManager()

  // Detect user preferences
  const detectPreferences = () => {
    // Reduced motion
    if (window.matchMedia) {
      const reducedMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
      preferences.value.reducedMotion = reducedMotionQuery.matches
      
      reducedMotionQuery.addEventListener('change', (e) => {
        preferences.value.reducedMotion = e.matches
        updateMotionPreference(e.matches)
      })

      // High contrast
      const highContrastQuery = window.matchMedia('(prefers-contrast: high)')
      preferences.value.highContrast = highContrastQuery.matches
      
      highContrastQuery.addEventListener('change', (e) => {
        preferences.value.highContrast = e.matches
        updateContrastPreference(e.matches)
      })

      // Color scheme
      const colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
      colorSchemeQuery.addEventListener('change', (e) => {
        updateColorScheme(e.matches ? 'dark' : 'light')
      })
    }

    // Screen reader detection
    preferences.value.screenReader = detectScreenReader()
  }

  const detectScreenReader = () => {
    // Check for common screen reader indicators
    return !!(
      window.navigator.userAgent.match(/NVDA|JAWS|VoiceOver|TalkBack/i) ||
      window.speechSynthesis ||
      document.body.classList.contains('screen-reader')
    )
  }

  const updateMotionPreference = (reducedMotion) => {
    document.documentElement.style.setProperty(
      '--motion-duration',
      reducedMotion ? '0.01ms' : '0.3s'
    )
    
    document.documentElement.setAttribute('data-reduced-motion', reducedMotion)
  }

  const updateContrastPreference = (highContrast) => {
    document.documentElement.setAttribute('data-high-contrast', highContrast)
  }

  const updateColorScheme = (scheme) => {
    document.documentElement.setAttribute('data-color-scheme', scheme)
  }

  // Keyboard navigation
  const setupKeyboardNavigation = () => {
    // Skip links
    createSkipLinks()
    
    // Keyboard shortcuts
    document.addEventListener('keydown', handleKeyboardShortcuts)
    
    // Focus indicators
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Tab') {
        document.body.classList.add('using-keyboard')
      }
    })
    
    document.addEventListener('mousedown', () => {
      document.body.classList.remove('using-keyboard')
    })
  }

  const createSkipLinks = () => {
    const skipLinks = document.createElement('div')
    skipLinks.className = 'skip-links'
    skipLinks.innerHTML = `
      <a href="#main-content" class="skip-link">Skip to main content</a>
      <a href="#main-navigation" class="skip-link">Skip to navigation</a>
      <a href="#search" class="skip-link">Skip to search</a>
    `
    
    document.body.insertBefore(skipLinks, document.body.firstChild)
  }

  const handleKeyboardShortcuts = (event) => {
    // Alt + M = Main navigation
    if (event.altKey && event.key === 'm') {
      event.preventDefault()
      const nav = document.getElementById('main-navigation')
      if (nav) nav.focus()
    }
    
    // Alt + S = Search
    if (event.altKey && event.key === 's') {
      event.preventDefault()
      const search = document.getElementById('search')
      if (search) search.focus()
    }
    
    // Alt + H = Help
    if (event.altKey && event.key === 'h') {
      event.preventDefault()
      showHelp()
    }
    
    // Escape = Close modals/overlays
    if (event.key === 'Escape') {
      closeTopLevelOverlay()
    }
  }

  const showHelp = () => {
    ScreenReaderUtils.announce('Help dialog opened')
    // Implementation depends on your help system
  }

  const closeTopLevelOverlay = () => {
    // Close any open modals, dropdowns, etc.
    const overlays = document.querySelectorAll('[role="dialog"], [role="menu"], .dropdown-open')
    overlays.forEach(overlay => {
      if (overlay.style.display !== 'none') {
        overlay.style.display = 'none'
        focusManager.releaseFocus()
      }
    })
  }

  // ARIA utilities
  const updateAriaLabel = (element, label) => {
    element.setAttribute('aria-label', label)
  }

  const updateAriaDescription = (element, description) => {
    const descId = ScreenReaderUtils.describeElement(element, description)
    return descId
  }

  const announceChange = (message, priority = 'polite') => {
    ScreenReaderUtils.announce(message, priority)
  }

  // Form accessibility
  const setupFormAccessibility = (form) => {
    const inputs = form.querySelectorAll('input, select, textarea')
    
    inputs.forEach(input => {
      // Associate labels
      const label = form.querySelector(`label[for="${input.id}"]`)
      if (!label && input.id) {
        const labelText = input.getAttribute('placeholder') || input.name
        const labelEl = document.createElement('label')
        labelEl.setAttribute('for', input.id)
        labelEl.className = 'sr-only'
        labelEl.textContent = labelText
        input.parentNode.insertBefore(labelEl, input)
      }
      
      // Required field indicators
      if (input.required) {
        input.setAttribute('aria-required', 'true')
        const requiredIndicator = document.createElement('span')
        requiredIndicator.textContent = ' *'
        requiredIndicator.setAttribute('aria-label', 'required')
        requiredIndicator.className = 'required-indicator'
        input.parentNode.appendChild(requiredIndicator)
      }
      
      // Error states
      input.addEventListener('invalid', () => {
        input.setAttribute('aria-invalid', 'true')
        const errorMessage = input.validationMessage
        if (errorMessage) {
          const errorId = `error-${input.id}`
          let errorEl = document.getElementById(errorId)
          
          if (!errorEl) {
            errorEl = document.createElement('div')
            errorEl.id = errorId
            errorEl.className = 'error-message'
            errorEl.setAttribute('role', 'alert')
            input.parentNode.appendChild(errorEl)
          }
          
          errorEl.textContent = errorMessage
          input.setAttribute('aria-describedby', errorId)
          announceChange(`Error: ${errorMessage}`, 'assertive')
        }
      })
      
      input.addEventListener('input', () => {
        if (input.validity.valid) {
          input.removeAttribute('aria-invalid')
          const errorId = `error-${input.id}`
          const errorEl = document.getElementById(errorId)
          if (errorEl) {
            errorEl.remove()
            input.removeAttribute('aria-describedby')
          }
        }
      })
    })
  }

  // Table accessibility
  const setupTableAccessibility = (table) => {
    // Add role if not present
    if (!table.getAttribute('role')) {
      table.setAttribute('role', 'table')
    }
    
    // Caption
    if (!table.querySelector('caption')) {
      const caption = document.createElement('caption')
      caption.textContent = table.getAttribute('data-caption') || 'Data table'
      caption.className = 'sr-only'
      table.insertBefore(caption, table.firstChild)
    }
    
    // Headers
    const headers = table.querySelectorAll('th')
    headers.forEach((header, index) => {
      if (!header.id) {
        header.id = `header-${index}`
      }
      header.setAttribute('scope', header.parentNode.tagName === 'THEAD' ? 'col' : 'row')
    })
    
    // Cells
    const cells = table.querySelectorAll('td')
    cells.forEach(cell => {
      const row = cell.parentNode
      const cellIndex = Array.from(row.children).indexOf(cell)
      const header = table.querySelector(`th:nth-child(${cellIndex + 1})`)
      if (header) {
        cell.setAttribute('headers', header.id)
      }
    })
  }

  // Initialize accessibility features
  const initializeAccessibility = () => {
    // Create screen reader announcer
    ScreenReaderUtils.createAnnouncer()
    
    // Detect user preferences
    detectPreferences()
    
    // Setup keyboard navigation
    setupKeyboardNavigation()
    
    // Add CSS for accessibility
    addAccessibilityStyles()
    
    // Setup automatic form and table accessibility
    setupAutomaticAccessibility()
  }

  const addAccessibilityStyles = () => {
    const style = document.createElement('style')
    style.textContent = `
      .sr-only {
        position: absolute;
        left: -10000px;
        width: 1px;
        height: 1px;
        overflow: hidden;
      }
      
      .skip-links {
        position: absolute;
        top: -100px;
        left: 0;
        z-index: 1000;
      }
      
      .skip-link {
        position: absolute;
        top: 0;
        left: 0;
        background: #000;
        color: #fff;
        padding: 0.5rem 1rem;
        text-decoration: none;
        transform: translateY(-100%);
        transition: transform 0.3s;
      }
      
      .skip-link:focus {
        transform: translateY(0);
      }
      
      .using-keyboard *:focus {
        outline: 2px solid #007acc;
        outline-offset: 2px;
      }
      
      [data-reduced-motion="true"] *,
      [data-reduced-motion="true"] *::before,
      [data-reduced-motion="true"] *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
      }
      
      [data-high-contrast="true"] {
        filter: contrast(150%);
      }
      
      .required-indicator {
        color: #d32f2f;
        font-weight: bold;
      }
      
      .error-message {
        color: #d32f2f;
        font-size: 0.875rem;
        margin-top: 0.25rem;
      }
      
      [aria-invalid="true"] {
        border-color: #d32f2f;
      }
    `
    
    document.head.appendChild(style)
  }

  const setupAutomaticAccessibility = () => {
    // Setup accessibility for existing forms
    const forms = document.querySelectorAll('form')
    forms.forEach(setupFormAccessibility)
    
    // Setup accessibility for existing tables
    const tables = document.querySelectorAll('table')
    tables.forEach(setupTableAccessibility)
    
    // Observer for dynamic content
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === 1) { // Element node
            const forms = node.querySelectorAll ? node.querySelectorAll('form') : []
            const tables = node.querySelectorAll ? node.querySelectorAll('table') : []
            
            forms.forEach(setupFormAccessibility)
            tables.forEach(setupTableAccessibility)
          }
        })
      })
    })
    
    observer.observe(document.body, {
      childList: true,
      subtree: true
    })
  }

  // Public API
  return {
    preferences: computed(() => preferences.value),
    focusManager,
    ContrastChecker,
    ScreenReaderUtils,
    
    // Methods
    announceChange,
    updateAriaLabel,
    updateAriaDescription,
    setupFormAccessibility,
    setupTableAccessibility,
    initializeAccessibility,
    
    // Utilities
    trapFocus: focusManager.trapFocus.bind(focusManager),
    releaseFocus: focusManager.releaseFocus.bind(focusManager),
    
    // Computed properties
    isScreenReaderUser: computed(() => preferences.value.screenReader),
    hasReducedMotion: computed(() => preferences.value.reducedMotion),
    hasHighContrast: computed(() => preferences.value.highContrast),
    usesLargeText: computed(() => preferences.value.largeText)
  }
}

// Accessibility directive
export const a11yDirective = {
  mounted(el, binding) {
    const { value } = binding
    
    if (value.label) {
      el.setAttribute('aria-label', value.label)
    }
    
    if (value.description) {
      ScreenReaderUtils.describeElement(el, value.description)
    }
    
    if (value.role) {
      el.setAttribute('role', value.role)
    }
    
    if (value.hidden) {
      el.setAttribute('aria-hidden', 'true')
    }
  },
  
  updated(el, binding) {
    const { value } = binding
    
    if (value.label) {
      el.setAttribute('aria-label', value.label)
    }
  }
}

export default {
  useAccessibility,
  a11yDirective,
  ScreenReaderUtils,
  FocusManager,
  ContrastChecker,
  A11Y_LEVELS,
  A11Y_PREFERENCES,
  CONTRAST_RATIOS
}
