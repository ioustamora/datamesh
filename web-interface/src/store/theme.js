import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  // State
  const theme = ref(localStorage.getItem('datamesh_theme') || 'auto')
  const primaryColor = ref(localStorage.getItem('datamesh_primary_color') || '#409EFF')
  const systemTheme = ref('light')
  const breakpoint = ref('desktop')
  
  // Getters
  const isDark = computed(() => {
    if (theme.value === 'dark') return true
    if (theme.value === 'light') return false
    return systemTheme.value === 'dark'
  })
  
  const isLight = computed(() => !isDark.value)
  const isMobile = computed(() => breakpoint.value === 'mobile')
  const isTablet = computed(() => breakpoint.value === 'tablet')
  const isDesktop = computed(() => breakpoint.value === 'desktop')
  
  // Actions
  const setTheme = (newTheme) => {
    theme.value = newTheme
    localStorage.setItem('datamesh_theme', newTheme)
    applyTheme()
  }
  
  const setPrimaryColor = (color) => {
    primaryColor.value = color
    localStorage.setItem('datamesh_primary_color', color)
    applyPrimaryColor()
  }
  
  const toggleTheme = () => {
    if (theme.value === 'light') {
      setTheme('dark')
    } else if (theme.value === 'dark') {
      setTheme('light')
    } else {
      // Auto mode, toggle based on current system theme
      setTheme(systemTheme.value === 'dark' ? 'light' : 'dark')
    }
  }
  
  const applyTheme = () => {
    const html = document.documentElement
    
    if (isDark.value) {
      html.classList.add('dark')
      html.classList.remove('light')
    } else {
      html.classList.add('light')
      html.classList.remove('dark')
    }
    
    // Update meta theme-color
    updateMetaThemeColor()
  }
  
  const applyPrimaryColor = () => {
    const root = document.documentElement
    root.style.setProperty('--el-color-primary', primaryColor.value)
    
    // Generate color palette
    const colors = generateColorPalette(primaryColor.value)
    colors.forEach((color, index) => {
      root.style.setProperty(`--el-color-primary-light-${index + 1}`, color)
    })
  }
  
  const updateMetaThemeColor = () => {
    const metaThemeColor = document.querySelector('meta[name="theme-color"]')
    if (metaThemeColor) {
      metaThemeColor.content = isDark.value ? '#1f2937' : '#ffffff'
    }
  }
  
  const generateColorPalette = (baseColor) => {
    const colors = []
    const base = hexToRgb(baseColor)
    
    for (let i = 1; i <= 9; i++) {
      const ratio = i / 10
      const color = {
        r: Math.round(base.r + (255 - base.r) * ratio),
        g: Math.round(base.g + (255 - base.g) * ratio),
        b: Math.round(base.b + (255 - base.b) * ratio)
      }
      colors.push(`rgb(${color.r}, ${color.g}, ${color.b})`)
    }
    
    return colors
  }
  
  const hexToRgb = (hex) => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : null
  }
  
  const watchSystemTheme = () => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    
    const handleChange = (e) => {
      systemTheme.value = e.matches ? 'dark' : 'light'
      if (theme.value === 'auto') {
        applyTheme()
      }
    }
    
    // Set initial value
    systemTheme.value = mediaQuery.matches ? 'dark' : 'light'
    
    // Listen for changes
    mediaQuery.addEventListener('change', handleChange)
    
    // Return cleanup function
    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  }
  
  const updateBreakpoint = () => {
    const width = window.innerWidth
    
    if (width < 768) {
      breakpoint.value = 'mobile'
    } else if (width < 1024) {
      breakpoint.value = 'tablet'
    } else {
      breakpoint.value = 'desktop'
    }
  }
  
  const initializeTheme = () => {
    // Apply saved theme
    applyTheme()
    applyPrimaryColor()
    
    // Watch for system theme changes
    watchSystemTheme()
    
    // Update breakpoint
    updateBreakpoint()
    
    // Listen for window resize
    window.addEventListener('resize', updateBreakpoint)
  }
  
  // Predefined theme presets
  const themePresets = ref([
    {
      name: 'Default Blue',
      primary: '#409EFF',
      description: 'Classic Element Plus blue'
    },
    {
      name: 'DataMesh Purple',
      primary: '#667eea',
      description: 'DataMesh brand purple'
    },
    {
      name: 'Success Green',
      primary: '#67C23A',
      description: 'Fresh green theme'
    },
    {
      name: 'Warning Orange',
      primary: '#E6A23C',
      description: 'Warm orange theme'
    },
    {
      name: 'Danger Red',
      primary: '#F56C6C',
      description: 'Bold red theme'
    },
    {
      name: 'Info Gray',
      primary: '#909399',
      description: 'Neutral gray theme'
    }
  ])
  
  const applyPreset = (preset) => {
    setPrimaryColor(preset.primary)
  }
  
  // Custom CSS variables for advanced theming
  const customProperties = ref({
    '--datamesh-sidebar-width': '240px',
    '--datamesh-header-height': '60px',
    '--datamesh-border-radius': '8px',
    '--datamesh-box-shadow': '0 2px 12px 0 rgba(0, 0, 0, 0.1)',
    '--datamesh-transition': 'all 0.3s ease'
  })
  
  const updateCustomProperty = (property, value) => {
    customProperties.value[property] = value
    document.documentElement.style.setProperty(property, value)
  }
  
  const applyCustomProperties = () => {
    Object.entries(customProperties.value).forEach(([property, value]) => {
      document.documentElement.style.setProperty(property, value)
    })
  }
  
  return {
    // State
    theme,
    primaryColor,
    systemTheme,
    breakpoint,
    themePresets,
    customProperties,
    
    // Getters
    isDark,
    isLight,
    isMobile,
    isTablet,
    isDesktop,
    
    // Actions
    setTheme,
    setPrimaryColor,
    toggleTheme,
    applyTheme,
    applyPrimaryColor,
    watchSystemTheme,
    updateBreakpoint,
    initializeTheme,
    applyPreset,
    updateCustomProperty,
    applyCustomProperties
  }
})