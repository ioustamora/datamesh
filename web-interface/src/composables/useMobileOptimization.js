import { ref, computed, onMounted, onUnmounted } from 'vue'

export function useGestureRecognition() {
  const currentGesture = ref(null)
  const gestureHistory = ref([])
  const swipeThreshold = 50
  const velocityThreshold = 0.5
  
  let touchStartTime = 0
  let gestureCallbacks = {}
  
  const registerGesture = (gestureName, callback) => {
    gestureCallbacks[gestureName] = callback
  }
  
  const recognizeGesture = ({ deltaX, deltaY, touchStartX, touchStartY }) => {
    const now = Date.now()
    const timeDiff = now - touchStartTime
    const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY)
    const velocity = distance / timeDiff
    
    if (velocity < velocityThreshold) return
    
    let gesture = null
    
    // Determine gesture type
    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      // Horizontal swipe
      if (Math.abs(deltaX) > swipeThreshold) {
        if (deltaX > 0) {
          gesture = 'swipe-right'
        } else {
          gesture = 'swipe-left'
        }
      }
    } else {
      // Vertical swipe
      if (Math.abs(deltaY) > swipeThreshold) {
        if (deltaY > 0) {
          gesture = 'swipe-down'
        } else {
          gesture = 'swipe-up'
        }
      }
    }
    
    // Advanced gesture recognition
    if (gesture) {
      const gestureData = {
        type: gesture,
        deltaX,
        deltaY,
        velocity,
        timestamp: now,
        startPosition: { x: touchStartX, y: touchStartY }
      }
      
      currentGesture.value = gestureData
      gestureHistory.value.push(gestureData)
      
      // Keep only last 10 gestures
      if (gestureHistory.value.length > 10) {
        gestureHistory.value = gestureHistory.value.slice(-10)
      }
      
      // Execute callback if registered
      if (gestureCallbacks[gesture]) {
        gestureCallbacks[gesture](gestureData)
      }
      
      // Check for complex gestures
      checkComplexGestures()
    }
  }
  
  const checkComplexGestures = () => {
    const recent = gestureHistory.value.slice(-3)
    
    // Double swipe detection
    if (recent.length >= 2) {
      const [first, second] = recent.slice(-2)
      const timeDiff = second.timestamp - first.timestamp
      
      if (timeDiff < 500 && first.type === second.type) {
        const doubleGesture = `double-${first.type}`
        if (gestureCallbacks[doubleGesture]) {
          gestureCallbacks[doubleGesture]({ gestures: [first, second] })
        }
      }
    }
    
    // Circular gesture detection
    if (recent.length >= 4) {
      const types = recent.map(g => g.type)
      if (types.includes('swipe-right') && types.includes('swipe-down') && 
          types.includes('swipe-left') && types.includes('swipe-up')) {
        if (gestureCallbacks['circular-swipe']) {
          gestureCallbacks['circular-swipe']({ gestures: recent })
        }
      }
    }
  }
  
  const setupGestureListeners = () => {
    const handleTouchStart = (e) => {
      touchStartTime = Date.now()
    }
    
    document.addEventListener('touchstart', handleTouchStart, { passive: true })
    
    return () => {
      document.removeEventListener('touchstart', handleTouchStart)
    }
  }
  
  const cleanupGestureListeners = setupGestureListeners()
  
  // Predefined gesture patterns
  const gesturePatterns = {
    'quick-actions': {
      pattern: ['swipe-up', 'swipe-down'],
      maxTime: 800,
      description: 'Quick actions menu'
    },
    'navigation-back': {
      pattern: ['swipe-right'],
      minDistance: 100,
      description: 'Navigate back'
    },
    'navigation-forward': {
      pattern: ['swipe-left'],
      minDistance: 100,
      description: 'Navigate forward'
    },
    'refresh': {
      pattern: ['swipe-down'],
      minDistance: 80,
      fromTop: true,
      description: 'Pull to refresh'
    },
    'search': {
      pattern: ['double-swipe-down'],
      description: 'Open search'
    },
    'settings': {
      pattern: ['circular-swipe'],
      description: 'Quick settings'
    }
  }
  
  const getGestureHints = () => {
    return Object.entries(gesturePatterns).map(([name, pattern]) => ({
      name,
      description: pattern.description,
      pattern: pattern.pattern
    }))
  }
  
  const isGestureEnabled = (gestureName) => {
    // Check user preferences and device capabilities
    const userPrefs = JSON.parse(localStorage.getItem('gesturePreferences') || '{}')
    return userPrefs[gestureName] !== false
  }
  
  const enableGesture = (gestureName, enabled = true) => {
    const userPrefs = JSON.parse(localStorage.getItem('gesturePreferences') || '{}')
    userPrefs[gestureName] = enabled
    localStorage.setItem('gesturePreferences', JSON.stringify(userPrefs))
  }
  
  return {
    currentGesture,
    gestureHistory,
    registerGesture,
    recognizeGesture,
    setupGestureListeners,
    cleanupGestureListeners,
    getGestureHints,
    isGestureEnabled,
    enableGesture,
    gesturePatterns
  }
}

export function useHapticFeedback() {
  const isHapticSupported = computed(() => {
    return 'vibrate' in navigator || 'hapticEngine' in navigator
  })
  
  const hapticPatterns = {
    light: [10],
    medium: [50],
    heavy: [100],
    success: [10, 50, 10],
    error: [100, 50, 100],
    warning: [50, 50, 50],
    notification: [10, 100, 10, 100],
    heartbeat: [50, 50, 50, 50, 150],
    click: [5],
    double-click: [5, 50, 5],
    long-press: [200],
    selection: [10, 20, 10]
  }
  
  const triggerHaptic = (pattern = 'light', customPattern = null) => {
    if (!isHapticSupported.value) return
    
    const vibrationPattern = customPattern || hapticPatterns[pattern] || hapticPatterns.light
    
    try {
      // Modern haptic feedback API
      if ('hapticEngine' in navigator) {
        navigator.hapticEngine.vibrate(vibrationPattern)
      } else if ('vibrate' in navigator) {
        navigator.vibrate(vibrationPattern)
      }
    } catch (error) {
      console.warn('Haptic feedback not available:', error)
    }
  }
  
  const isHapticEnabled = () => {
    return localStorage.getItem('hapticEnabled') !== 'false'
  }
  
  const setHapticEnabled = (enabled) => {
    localStorage.setItem('hapticEnabled', enabled.toString())
  }
  
  const triggerConditionalHaptic = (pattern) => {
    if (isHapticEnabled()) {
      triggerHaptic(pattern)
    }
  }
  
  return {
    isHapticSupported,
    triggerHaptic,
    triggerConditionalHaptic,
    isHapticEnabled,
    setHapticEnabled,
    hapticPatterns
  }
}

export function useAdaptiveNavigation() {
  const navigationHistory = ref([])
  const userBehavior = ref({})
  const contextualActions = ref([])
  
  const getAdaptiveNavigation = (baseItems, context) => {
    const { userLevel, deviceType, oneHandedMode, currentRoute } = context
    
    // Filter items based on user level
    let filteredItems = baseItems.filter(item => {
      if (item.minLevel && userLevel < item.minLevel) return false
      return true
    })
    
    // Adapt for one-handed mode
    if (oneHandedMode) {
      // Prioritize most used items
      filteredItems = filteredItems
        .sort((a, b) => (userBehavior.value[b.id] || 0) - (userBehavior.value[a.id] || 0))
        .slice(0, 4) // Show only top 4 items
    }
    
    // Adapt for device type
    if (deviceType === 'phone') {
      // Show icons only for small screens
      filteredItems = filteredItems.map(item => ({
        ...item,
        showLabel: !oneHandedMode && filteredItems.length <= 5
      }))
    }
    
    // Add contextual items
    const contextualItems = getContextualNavigationItems(currentRoute)
    if (contextualItems.length > 0) {
      filteredItems = [...filteredItems, ...contextualItems]
    }
    
    return filteredItems
  }
  
  const getContextualNavigationItems = (currentRoute) => {
    const contextualItems = []
    
    // Add route-specific items
    if (currentRoute === 'files') {
      contextualItems.push({
        id: 'upload',
        label: 'Upload',
        icon: 'Upload',
        route: 'upload',
        contextual: true
      })
    } else if (currentRoute === 'dashboard') {
      contextualItems.push({
        id: 'shortcuts',
        label: 'Shortcuts',
        icon: 'Lightning',
        route: 'shortcuts',
        contextual: true
      })
    }
    
    return contextualItems
  }
  
  const getContextualActions = (context) => {
    const { currentRoute, userBehavior, deviceContext } = context
    const actions = []
    
    // Common actions
    const commonActions = [
      {
        id: 'add',
        type: 'primary',
        icon: 'Plus',
        primary: true,
        execute: () => {
          // Route-specific add action
          if (currentRoute === 'files') {
            // Upload file
          } else if (currentRoute === 'dashboard') {
            // Quick add
          }
        }
      }
    ]
    
    // Route-specific actions
    if (currentRoute === 'files') {
      actions.push({
        id: 'search',
        type: 'default',
        icon: 'Search',
        execute: () => {
          // Open file search
        }
      })
    } else if (currentRoute === 'dashboard') {
      actions.push({
        id: 'refresh',
        type: 'default',
        icon: 'Refresh',
        execute: () => {
          // Refresh dashboard
        }
      })
    }
    
    // Behavioral actions
    if (userBehavior.recentSearches?.length > 0) {
      actions.push({
        id: 'recent-search',
        type: 'info',
        icon: 'History',
        execute: () => {
          // Show recent searches
        }
      })
    }
    
    // Device context actions
    if (deviceContext.oneHandedMode) {
      // Reduce number of actions
      return [...commonActions, ...actions.slice(0, 2)]
    }
    
    return [...commonActions, ...actions]
  }
  
  const updateNavigationContext = (route) => {
    navigationHistory.value.push({
      route,
      timestamp: Date.now()
    })
    
    // Keep only last 50 navigation events
    if (navigationHistory.value.length > 50) {
      navigationHistory.value = navigationHistory.value.slice(-50)
    }
    
    // Update user behavior
    userBehavior.value[route] = (userBehavior.value[route] || 0) + 1
    
    // Save to localStorage
    localStorage.setItem('navigationBehavior', JSON.stringify(userBehavior.value))
  }
  
  const getNavigationSuggestions = () => {
    const recent = navigationHistory.value.slice(-5)
    const frequent = Object.entries(userBehavior.value)
      .sort(([,a], [,b]) => b - a)
      .slice(0, 3)
      .map(([route]) => route)
    
    return {
      recent: recent.map(item => item.route),
      frequent
    }
  }
  
  const predictNextNavigation = () => {
    const recentRoutes = navigationHistory.value.slice(-3).map(item => item.route)
    
    // Simple pattern matching
    const patterns = {
      'dashboard->files': 'upload',
      'files->dashboard': 'analytics',
      'settings->profile': 'dashboard'
    }
    
    const pattern = recentRoutes.slice(-2).join('->')
    return patterns[pattern] || null
  }
  
  // Load saved behavior on initialization
  onMounted(() => {
    const savedBehavior = localStorage.getItem('navigationBehavior')
    if (savedBehavior) {
      userBehavior.value = JSON.parse(savedBehavior)
    }
  })
  
  return {
    navigationHistory,
    userBehavior,
    contextualActions,
    getAdaptiveNavigation,
    getContextualActions,
    updateNavigationContext,
    getNavigationSuggestions,
    predictNextNavigation
  }
}

export function useDeviceOptimization() {
  const deviceInfo = ref({})
  const performanceMetrics = ref({})
  const optimizations = ref({})
  
  const detectDevice = () => {
    const ua = navigator.userAgent
    const screen = window.screen
    
    deviceInfo.value = {
      userAgent: ua,
      screenWidth: screen.width,
      screenHeight: screen.height,
      pixelRatio: window.devicePixelRatio || 1,
      orientation: screen.orientation?.type || 'portrait',
      connection: navigator.connection?.effectiveType || 'unknown',
      memory: navigator.deviceMemory || 'unknown',
      cores: navigator.hardwareConcurrency || 'unknown',
      battery: navigator.getBattery ? 'supported' : 'not-supported',
      touchSupport: 'ontouchstart' in window || navigator.maxTouchPoints > 0,
      
      // Device classification
      isMobile: /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(ua),
      isTablet: /iPad|Android(?!.*Mobile)/i.test(ua),
      isDesktop: !/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(ua),
      
      // Performance indicators
      isLowEnd: (navigator.deviceMemory && navigator.deviceMemory < 4) || 
                (navigator.hardwareConcurrency && navigator.hardwareConcurrency < 4),
      isHighEnd: (navigator.deviceMemory && navigator.deviceMemory >= 8) && 
                 (navigator.hardwareConcurrency && navigator.hardwareConcurrency >= 8),
      
      // Network conditions
      isSlowNetwork: navigator.connection?.effectiveType === 'slow-2g' || 
                     navigator.connection?.effectiveType === '2g',
      isFastNetwork: navigator.connection?.effectiveType === '4g' || 
                     navigator.connection?.effectiveType === '5g'
    }
    
    return deviceInfo.value
  }
  
  const measurePerformance = () => {
    const start = performance.now()
    
    // Measure rendering performance
    requestAnimationFrame(() => {
      const renderTime = performance.now() - start
      
      performanceMetrics.value = {
        renderTime,
        memoryUsage: performance.memory ? {
          used: performance.memory.usedJSHeapSize,
          total: performance.memory.totalJSHeapSize,
          limit: performance.memory.jsHeapSizeLimit
        } : null,
        timestamp: Date.now()
      }
    })
  }
  
  const getOptimizations = () => {
    const device = deviceInfo.value
    const recommendations = []
    
    // Performance optimizations
    if (device.isLowEnd) {
      recommendations.push({
        type: 'performance',
        action: 'reduce-animations',
        description: 'Disable animations for better performance'
      })
      
      recommendations.push({
        type: 'performance',
        action: 'lazy-loading',
        description: 'Enable aggressive lazy loading'
      })
    }
    
    // Network optimizations
    if (device.isSlowNetwork) {
      recommendations.push({
        type: 'network',
        action: 'compress-images',
        description: 'Use compressed images and lazy loading'
      })
      
      recommendations.push({
        type: 'network',
        action: 'reduce-requests',
        description: 'Minimize API requests and use caching'
      })
    }
    
    // Battery optimizations
    if (device.battery === 'supported') {
      navigator.getBattery().then(battery => {
        if (battery.level < 0.2) {
          recommendations.push({
            type: 'battery',
            action: 'power-saving',
            description: 'Enable power saving mode'
          })
        }
      })
    }
    
    // Accessibility optimizations
    if (device.screenWidth < 400) {
      recommendations.push({
        type: 'accessibility',
        action: 'large-touch-targets',
        description: 'Increase touch target sizes'
      })
    }
    
    optimizations.value = recommendations
    return recommendations
  }
  
  const applyOptimizations = (optimizationIds) => {
    const applied = []
    
    optimizationIds.forEach(id => {
      const optimization = optimizations.value.find(opt => opt.action === id)
      if (optimization) {
        switch (id) {
          case 'reduce-animations':
            document.documentElement.style.setProperty('--animation-duration', '0s')
            break
          case 'lazy-loading':
            // Configure lazy loading
            break
          case 'compress-images':
            // Configure image compression
            break
          case 'power-saving':
            // Enable power saving features
            break
        }
        applied.push(optimization)
      }
    })
    
    return applied
  }
  
  const monitorPerformance = () => {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      entries.forEach(entry => {
        if (entry.entryType === 'measure') {
          performanceMetrics.value[entry.name] = entry.duration
        }
      })
    })
    
    observer.observe({ entryTypes: ['measure', 'navigation'] })
    
    // Measure every 30 seconds
    const interval = setInterval(measurePerformance, 30000)
    
    return () => {
      observer.disconnect()
      clearInterval(interval)
    }
  }
  
  onMounted(() => {
    detectDevice()
    measurePerformance()
    getOptimizations()
    
    const cleanup = monitorPerformance()
    
    onUnmounted(() => {
      cleanup()
    })
  })
  
  return {
    deviceInfo,
    performanceMetrics,
    optimizations,
    detectDevice,
    measurePerformance,
    getOptimizations,
    applyOptimizations,
    monitorPerformance
  }
}
