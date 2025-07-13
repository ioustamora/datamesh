/**
 * Advanced Performance Monitoring and Optimization System
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'

// Performance metrics collection
export class PerformanceMonitor {
  constructor() {
    this.metrics = {
      vitals: ref({}),
      navigation: ref({}),
      resources: ref([]),
      userTiming: ref([]),
      customMetrics: ref({}),
      errors: ref([])
    }
    
    this.observers = []
    this.isMonitoring = false
    this.reportingEndpoint = '/api/v1/performance'
    this.reportingInterval = 30000 // 30 seconds
    this.reportingTimer = null
  }

  start() {
    if (this.isMonitoring) return
    
    this.isMonitoring = true
    this.setupObservers()
    this.measureNavigationTiming()
    this.measureResourceTiming()
    this.startReporting()
    
    console.log('Performance monitoring started')
  }

  stop() {
    if (!this.isMonitoring) return
    
    this.isMonitoring = false
    this.observers.forEach(observer => observer.disconnect())
    this.observers = []
    
    if (this.reportingTimer) {
      clearInterval(this.reportingTimer)
      this.reportingTimer = null
    }
    
    console.log('Performance monitoring stopped')
  }

  setupObservers() {
    // Core Web Vitals
    this.observeWebVitals()
    
    // Long Tasks
    this.observeLongTasks()
    
    // Layout Shifts
    this.observeLayoutShifts()
    
    // User Timing
    this.observeUserTiming()
    
    // Memory Usage
    this.observeMemoryUsage()
  }

  observeWebVitals() {
    if (!('PerformanceObserver' in window)) return

    // Largest Contentful Paint (LCP)
    const lcpObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      const lcp = entries[entries.length - 1]
      
      this.metrics.vitals.value.lcp = {
        value: lcp.startTime,
        element: lcp.element?.tagName || 'unknown',
        url: lcp.url || window.location.href,
        timestamp: Date.now()
      }
    })
    
    try {
      lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] })
      this.observers.push(lcpObserver)
    } catch (e) {
      console.warn('LCP observation not supported')
    }

    // First Input Delay (FID)
    const fidObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      
      entries.forEach(entry => {
        this.metrics.vitals.value.fid = {
          value: entry.processingStart - entry.startTime,
          startTime: entry.startTime,
          processingStart: entry.processingStart,
          timestamp: Date.now()
        }
      })
    })
    
    try {
      fidObserver.observe({ entryTypes: ['first-input'] })
      this.observers.push(fidObserver)
    } catch (e) {
      console.warn('FID observation not supported')
    }

    // Cumulative Layout Shift (CLS)
    this.clsValue = 0
    const clsObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      
      entries.forEach(entry => {
        if (!entry.hadRecentInput) {
          this.clsValue += entry.value
        }
      })
      
      this.metrics.vitals.value.cls = {
        value: this.clsValue,
        timestamp: Date.now()
      }
    })
    
    try {
      clsObserver.observe({ entryTypes: ['layout-shift'] })
      this.observers.push(clsObserver)
    } catch (e) {
      console.warn('CLS observation not supported')
    }
  }

  observeLongTasks() {
    if (!('PerformanceObserver' in window)) return

    const longTaskObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      
      entries.forEach(entry => {
        const longTask = {
          duration: entry.duration,
          startTime: entry.startTime,
          attribution: entry.attribution || [],
          timestamp: Date.now()
        }
        
        // Store long tasks
        if (!this.metrics.customMetrics.value.longTasks) {
          this.metrics.customMetrics.value.longTasks = []
        }
        this.metrics.customMetrics.value.longTasks.push(longTask)
        
        // Keep only last 50 long tasks
        if (this.metrics.customMetrics.value.longTasks.length > 50) {
          this.metrics.customMetrics.value.longTasks.shift()
        }
        
        // Log warning for very long tasks
        if (entry.duration > 100) {
          console.warn(`Long task detected: ${entry.duration}ms`)
        }
      })
    })
    
    try {
      longTaskObserver.observe({ entryTypes: ['longtask'] })
      this.observers.push(longTaskObserver)
    } catch (e) {
      console.warn('Long task observation not supported')
    }
  }

  observeLayoutShifts() {
    if (!('PerformanceObserver' in window)) return

    const layoutShiftObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      
      entries.forEach(entry => {
        const shift = {
          value: entry.value,
          startTime: entry.startTime,
          hadRecentInput: entry.hadRecentInput,
          sources: entry.sources || [],
          timestamp: Date.now()
        }
        
        // Store layout shifts
        if (!this.metrics.customMetrics.value.layoutShifts) {
          this.metrics.customMetrics.value.layoutShifts = []
        }
        this.metrics.customMetrics.value.layoutShifts.push(shift)
        
        // Keep only last 100 shifts
        if (this.metrics.customMetrics.value.layoutShifts.length > 100) {
          this.metrics.customMetrics.value.layoutShifts.shift()
        }
      })
    })
    
    try {
      layoutShiftObserver.observe({ entryTypes: ['layout-shift'] })
      this.observers.push(layoutShiftObserver)
    } catch (e) {
      console.warn('Layout shift observation not supported')
    }
  }

  observeUserTiming() {
    if (!('PerformanceObserver' in window)) return

    const userTimingObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      
      entries.forEach(entry => {
        const timing = {
          name: entry.name,
          duration: entry.duration,
          startTime: entry.startTime,
          entryType: entry.entryType,
          timestamp: Date.now()
        }
        
        this.metrics.userTiming.value.push(timing)
        
        // Keep only last 200 entries
        if (this.metrics.userTiming.value.length > 200) {
          this.metrics.userTiming.value.shift()
        }
      })
    })
    
    try {
      userTimingObserver.observe({ entryTypes: ['measure', 'mark'] })
      this.observers.push(userTimingObserver)
    } catch (e) {
      console.warn('User timing observation not supported')
    }
  }

  observeMemoryUsage() {
    if (!performance.memory) return

    const measureMemory = () => {
      if (!this.isMonitoring) return
      
      const memory = {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize,
        limit: performance.memory.jsHeapSizeLimit,
        timestamp: Date.now()
      }
      
      this.metrics.customMetrics.value.memory = memory
      
      // Check for memory leaks
      const usagePercent = (memory.used / memory.total) * 100
      if (usagePercent > 90) {
        console.warn(`High memory usage: ${usagePercent.toFixed(1)}%`)
      }
    }

    // Measure every 10 seconds
    const memoryInterval = setInterval(measureMemory, 10000)
    measureMemory() // Initial measurement
    
    // Clean up on stop
    const originalStop = this.stop.bind(this)
    this.stop = () => {
      clearInterval(memoryInterval)
      originalStop()
    }
  }

  measureNavigationTiming() {
    if (!performance.navigation || !performance.timing) return

    const navigation = performance.timing
    const navigationTiming = {
      // DNS lookup
      dnsLookup: navigation.domainLookupEnd - navigation.domainLookupStart,
      
      // TCP connection
      tcpConnection: navigation.connectEnd - navigation.connectStart,
      
      // SSL handshake
      sslHandshake: navigation.connectEnd - navigation.secureConnectionStart,
      
      // Time to first byte
      ttfb: navigation.responseStart - navigation.requestStart,
      
      // Response time
      responseTime: navigation.responseEnd - navigation.responseStart,
      
      // DOM processing
      domProcessing: navigation.domComplete - navigation.domLoading,
      
      // Load complete
      loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
      
      // Total page load time
      totalLoadTime: navigation.loadEventEnd - navigation.navigationStart,
      
      // Time to interactive
      timeToInteractive: navigation.domInteractive - navigation.navigationStart,
      
      // Time to DOM content loaded
      domContentLoaded: navigation.domContentLoadedEventEnd - navigation.navigationStart,
      
      timestamp: Date.now()
    }
    
    this.metrics.navigation.value = navigationTiming
  }

  measureResourceTiming() {
    if (!performance.getEntriesByType) return

    const resources = performance.getEntriesByType('resource')
    const resourceMetrics = resources.map(resource => ({
      name: resource.name,
      type: this.getResourceType(resource.name),
      duration: resource.duration,
      size: resource.transferSize || resource.encodedBodySize,
      startTime: resource.startTime,
      dnsLookup: resource.domainLookupEnd - resource.domainLookupStart,
      tcpConnection: resource.connectEnd - resource.connectStart,
      ttfb: resource.responseStart - resource.requestStart,
      timestamp: Date.now()
    }))
    
    this.metrics.resources.value = resourceMetrics
  }

  getResourceType(url) {
    if (/\.(js|mjs)$/i.test(url)) return 'script'
    if (/\.(css)$/i.test(url)) return 'stylesheet'
    if (/\.(png|jpg|jpeg|gif|webp|svg)$/i.test(url)) return 'image'
    if (/\.(woff|woff2|ttf|otf)$/i.test(url)) return 'font'
    if (/\.(mp4|webm|ogg|mp3|wav)$/i.test(url)) return 'media'
    return 'other'
  }

  // Custom metrics
  markStart(name) {
    if (performance.mark) {
      performance.mark(`${name}-start`)
    }
  }

  markEnd(name) {
    if (performance.mark && performance.measure) {
      performance.mark(`${name}-end`)
      performance.measure(name, `${name}-start`, `${name}-end`)
    }
  }

  recordCustomMetric(name, value, unit = 'ms') {
    if (!this.metrics.customMetrics.value[name]) {
      this.metrics.customMetrics.value[name] = []
    }
    
    this.metrics.customMetrics.value[name].push({
      value,
      unit,
      timestamp: Date.now()
    })
    
    // Keep only last 100 values
    if (this.metrics.customMetrics.value[name].length > 100) {
      this.metrics.customMetrics.value[name].shift()
    }
  }

  // Reporting
  startReporting() {
    this.reportingTimer = setInterval(() => {
      this.sendReport()
    }, this.reportingInterval)
  }

  async sendReport() {
    try {
      const report = {
        timestamp: Date.now(),
        url: window.location.href,
        userAgent: navigator.userAgent,
        connection: this.getConnectionInfo(),
        vitals: this.metrics.vitals.value,
        navigation: this.metrics.navigation.value,
        resources: this.summarizeResources(),
        customMetrics: this.metrics.customMetrics.value
      }
      
      // Send to server
      await fetch(this.reportingEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(report)
      })
      
      console.log('Performance report sent')
    } catch (error) {
      console.error('Failed to send performance report:', error)
    }
  }

  getConnectionInfo() {
    if (navigator.connection) {
      return {
        effectiveType: navigator.connection.effectiveType,
        downlink: navigator.connection.downlink,
        rtt: navigator.connection.rtt,
        saveData: navigator.connection.saveData
      }
    }
    return null
  }

  summarizeResources() {
    const resources = this.metrics.resources.value
    const summary = {
      total: resources.length,
      totalSize: resources.reduce((sum, r) => sum + (r.size || 0), 0),
      totalDuration: resources.reduce((sum, r) => sum + r.duration, 0),
      byType: {}
    }
    
    resources.forEach(resource => {
      if (!summary.byType[resource.type]) {
        summary.byType[resource.type] = {
          count: 0,
          size: 0,
          duration: 0
        }
      }
      
      summary.byType[resource.type].count++
      summary.byType[resource.type].size += resource.size || 0
      summary.byType[resource.type].duration += resource.duration
    })
    
    return summary
  }

  // Analysis
  analyzePerformance() {
    const analysis = {
      score: 0,
      issues: [],
      recommendations: []
    }
    
    // Analyze Core Web Vitals
    this.analyzeCoreWebVitals(analysis)
    
    // Analyze resource loading
    this.analyzeResourceLoading(analysis)
    
    // Analyze memory usage
    this.analyzeMemoryUsage(analysis)
    
    // Analyze long tasks
    this.analyzeLongTasks(analysis)
    
    return analysis
  }

  analyzeCoreWebVitals(analysis) {
    const vitals = this.metrics.vitals.value
    
    // LCP analysis
    if (vitals.lcp) {
      if (vitals.lcp.value > 4000) {
        analysis.issues.push('Poor Largest Contentful Paint (>4s)')
        analysis.recommendations.push('Optimize image loading and critical resources')
      } else if (vitals.lcp.value > 2500) {
        analysis.issues.push('Needs improvement in Largest Contentful Paint')
      }
    }
    
    // FID analysis
    if (vitals.fid) {
      if (vitals.fid.value > 300) {
        analysis.issues.push('Poor First Input Delay (>300ms)')
        analysis.recommendations.push('Reduce JavaScript execution time')
      } else if (vitals.fid.value > 100) {
        analysis.issues.push('Needs improvement in First Input Delay')
      }
    }
    
    // CLS analysis
    if (vitals.cls) {
      if (vitals.cls.value > 0.25) {
        analysis.issues.push('Poor Cumulative Layout Shift (>0.25)')
        analysis.recommendations.push('Reserve space for dynamic content')
      } else if (vitals.cls.value > 0.1) {
        analysis.issues.push('Needs improvement in Cumulative Layout Shift')
      }
    }
  }

  analyzeResourceLoading(analysis) {
    const resources = this.metrics.resources.value
    
    // Large resources
    const largeResources = resources.filter(r => r.size > 1000000) // >1MB
    if (largeResources.length > 0) {
      analysis.issues.push(`${largeResources.length} large resources (>1MB)`)
      analysis.recommendations.push('Optimize large resources or implement lazy loading')
    }
    
    // Slow resources
    const slowResources = resources.filter(r => r.duration > 2000) // >2s
    if (slowResources.length > 0) {
      analysis.issues.push(`${slowResources.length} slow resources (>2s)`)
      analysis.recommendations.push('Optimize slow-loading resources')
    }
  }

  analyzeMemoryUsage(analysis) {
    const memory = this.metrics.customMetrics.value.memory
    if (memory) {
      const usagePercent = (memory.used / memory.total) * 100
      if (usagePercent > 80) {
        analysis.issues.push('High memory usage (>80%)')
        analysis.recommendations.push('Investigate memory leaks and optimize memory usage')
      }
    }
  }

  analyzeLongTasks(analysis) {
    const longTasks = this.metrics.customMetrics.value.longTasks || []
    const recentLongTasks = longTasks.filter(task => 
      Date.now() - task.timestamp < 60000 // Last minute
    )
    
    if (recentLongTasks.length > 5) {
      analysis.issues.push('Frequent long tasks detected')
      analysis.recommendations.push('Break up long-running JavaScript tasks')
    }
  }

  // Utilities
  getMetricValue(metric) {
    return this.metrics.customMetrics.value[metric]?.slice(-1)[0]?.value || 0
  }

  getAverageMetric(metric, timeWindow = 60000) {
    const values = this.metrics.customMetrics.value[metric] || []
    const recent = values.filter(v => Date.now() - v.timestamp < timeWindow)
    
    if (recent.length === 0) return 0
    
    const sum = recent.reduce((acc, v) => acc + v.value, 0)
    return sum / recent.length
  }
}

export const performanceMonitor = new PerformanceMonitor()

// Performance composable
export function usePerformanceMonitoring() {
  const isMonitoring = ref(false)
  const metrics = performanceMonitor.metrics
  
  const start = () => {
    performanceMonitor.start()
    isMonitoring.value = true
  }
  
  const stop = () => {
    performanceMonitor.stop()
    isMonitoring.value = false
  }
  
  const markStart = (name) => {
    performanceMonitor.markStart(name)
  }
  
  const markEnd = (name) => {
    performanceMonitor.markEnd(name)
  }
  
  const recordMetric = (name, value, unit) => {
    performanceMonitor.recordCustomMetric(name, value, unit)
  }
  
  const analyze = () => {
    return performanceMonitor.analyzePerformance()
  }
  
  const coreWebVitals = computed(() => metrics.vitals.value)
  const navigationTiming = computed(() => metrics.navigation.value)
  const resourceTiming = computed(() => metrics.resources.value)
  const customMetrics = computed(() => metrics.customMetrics.value)
  
  onMounted(() => {
    if (import.meta.env.PROD) {
      start()
    }
  })
  
  onUnmounted(() => {
    stop()
  })
  
  return {
    isMonitoring: computed(() => isMonitoring.value),
    metrics,
    start,
    stop,
    markStart,
    markEnd,
    recordMetric,
    analyze,
    coreWebVitals,
    navigationTiming,
    resourceTiming,
    customMetrics
  }
}

// Performance directive for measuring component render times
export const performanceDirective = {
  beforeMount(el, binding) {
    const name = binding.value || el.tagName.toLowerCase()
    performanceMonitor.markStart(`component-${name}`)
  },
  
  mounted(el, binding) {
    const name = binding.value || el.tagName.toLowerCase()
    performanceMonitor.markEnd(`component-${name}`)
  }
}

export default {
  PerformanceMonitor,
  performanceMonitor,
  usePerformanceMonitoring,
  performanceDirective
}
