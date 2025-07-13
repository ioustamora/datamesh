/**
 * Advanced Internationalization System for DataMesh
 */

import { createI18n } from 'vue-i18n'
import { ref, computed } from 'vue'

// Supported languages
export const SUPPORTED_LANGUAGES = {
  'en': {
    name: 'English',
    nativeName: 'English',
    flag: '🇺🇸',
    rtl: false,
    dateFormat: 'MM/DD/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'USD',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'es': {
    name: 'Spanish',
    nativeName: 'Español',
    flag: '🇪🇸',
    rtl: false,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'EUR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'fr': {
    name: 'French',
    nativeName: 'Français',
    flag: '🇫🇷',
    rtl: false,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'EUR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'de': {
    name: 'German',
    nativeName: 'Deutsch',
    flag: '🇩🇪',
    rtl: false,
    dateFormat: 'DD.MM.YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'EUR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'it': {
    name: 'Italian',
    nativeName: 'Italiano',
    flag: '🇮🇹',
    rtl: false,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'EUR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'pt': {
    name: 'Portuguese',
    nativeName: 'Português',
    flag: '🇵🇹',
    rtl: false,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'EUR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'ru': {
    name: 'Russian',
    nativeName: 'Русский',
    flag: '🇷🇺',
    rtl: false,
    dateFormat: 'DD.MM.YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'RUB',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'zh': {
    name: 'Chinese',
    nativeName: '中文',
    flag: '🇨🇳',
    rtl: false,
    dateFormat: 'YYYY-MM-DD',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'CNY',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'ja': {
    name: 'Japanese',
    nativeName: '日本語',
    flag: '🇯🇵',
    rtl: false,
    dateFormat: 'YYYY/MM/DD',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'JPY',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 0
    }
  },
  'ko': {
    name: 'Korean',
    nativeName: '한국어',
    flag: '🇰🇷',
    rtl: false,
    dateFormat: 'YYYY-MM-DD',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'KRW',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 0
    }
  },
  'ar': {
    name: 'Arabic',
    nativeName: 'العربية',
    flag: '🇸🇦',
    rtl: true,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'SAR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  },
  'hi': {
    name: 'Hindi',
    nativeName: 'हिन्दी',
    flag: '🇮🇳',
    rtl: false,
    dateFormat: 'DD/MM/YYYY',
    timeFormat: 'HH:mm',
    numberFormat: {
      currency: 'INR',
      currencyDisplay: 'symbol',
      minimumFractionDigits: 2
    }
  }
}

// Translation loader
class TranslationLoader {
  constructor() {
    this.loadedLanguages = new Set(['en'])
    this.loadingLanguages = new Set()
    this.cache = new Map()
  }

  async loadLanguage(locale) {
    if (this.loadedLanguages.has(locale)) {
      return this.cache.get(locale)
    }

    if (this.loadingLanguages.has(locale)) {
      return new Promise((resolve) => {
        const checkLoaded = () => {
          if (this.loadedLanguages.has(locale)) {
            resolve(this.cache.get(locale))
          } else {
            setTimeout(checkLoaded, 100)
          }
        }
        checkLoaded()
      })
    }

    this.loadingLanguages.add(locale)

    try {
      // Load main translation file
      const mainTranslations = await import(`../locales/${locale}/index.js`)
      
      // Load additional modules
      const modules = await Promise.all([
        import(`../locales/${locale}/common.js`).catch(() => ({})),
        import(`../locales/${locale}/dashboard.js`).catch(() => ({})),
        import(`../locales/${locale}/files.js`).catch(() => ({})),
        import(`../locales/${locale}/governance.js`).catch(() => ({})),
        import(`../locales/${locale}/settings.js`).catch(() => ({})),
        import(`../locales/${locale}/errors.js`).catch(() => ({}))
      ])

      // Merge all translations
      const translations = {
        ...mainTranslations.default,
        ...modules.reduce((acc, module) => ({ ...acc, ...module.default }), {})
      }

      this.cache.set(locale, translations)
      this.loadedLanguages.add(locale)
      this.loadingLanguages.delete(locale)

      return translations
    } catch (error) {
      console.error(`Failed to load language ${locale}:`, error)
      this.loadingLanguages.delete(locale)
      
      // Fallback to English
      if (locale !== 'en') {
        return this.loadLanguage('en')
      }
      
      throw error
    }
  }

  async preloadLanguages(locales) {
    const promises = locales.map(locale => this.loadLanguage(locale))
    await Promise.all(promises)
  }
}

const translationLoader = new TranslationLoader()

// Language detection
export function detectLanguage() {
  // Check URL parameter
  const urlParams = new URLSearchParams(window.location.search)
  const urlLang = urlParams.get('lang')
  if (urlLang && SUPPORTED_LANGUAGES[urlLang]) {
    return urlLang
  }

  // Check localStorage
  const savedLang = localStorage.getItem('datamesh-language')
  if (savedLang && SUPPORTED_LANGUAGES[savedLang]) {
    return savedLang
  }

  // Check browser language
  const browserLang = navigator.language.split('-')[0]
  if (SUPPORTED_LANGUAGES[browserLang]) {
    return browserLang
  }

  // Check browser languages
  for (const lang of navigator.languages) {
    const langCode = lang.split('-')[0]
    if (SUPPORTED_LANGUAGES[langCode]) {
      return langCode
    }
  }

  // Default to English
  return 'en'
}

// Create i18n instance
export const i18n = createI18n({
  locale: detectLanguage(),
  fallbackLocale: 'en',
  legacy: false,
  messages: {
    en: {} // Will be loaded dynamically
  },
  datetimeFormats: Object.fromEntries(
    Object.entries(SUPPORTED_LANGUAGES).map(([locale, config]) => [
      locale,
      {
        short: {
          year: 'numeric',
          month: 'short',
          day: 'numeric'
        },
        long: {
          year: 'numeric',
          month: 'long',
          day: 'numeric',
          weekday: 'long',
          hour: 'numeric',
          minute: 'numeric'
        }
      }
    ])
  ),
  numberFormats: Object.fromEntries(
    Object.entries(SUPPORTED_LANGUAGES).map(([locale, config]) => [
      locale,
      {
        currency: {
          style: 'currency',
          currency: config.numberFormat.currency,
          currencyDisplay: config.numberFormat.currencyDisplay,
          minimumFractionDigits: config.numberFormat.minimumFractionDigits
        },
        decimal: {
          style: 'decimal',
          minimumFractionDigits: 0,
          maximumFractionDigits: 2
        },
        percent: {
          style: 'percent',
          minimumFractionDigits: 0,
          maximumFractionDigits: 1
        }
      }
    ])
  )
})

// Language management composable
export function useLanguage() {
  const currentLocale = ref(i18n.global.locale.value)
  const isLoading = ref(false)
  const error = ref(null)

  const currentLanguage = computed(() => SUPPORTED_LANGUAGES[currentLocale.value])
  const availableLanguages = computed(() => 
    Object.entries(SUPPORTED_LANGUAGES).map(([code, config]) => ({
      code,
      ...config
    }))
  )

  const changeLanguage = async (locale) => {
    if (!SUPPORTED_LANGUAGES[locale]) {
      throw new Error(`Unsupported language: ${locale}`)
    }

    if (currentLocale.value === locale) {
      return
    }

    isLoading.value = true
    error.value = null

    try {
      // Load translations
      const translations = await translationLoader.loadLanguage(locale)
      
      // Update i18n
      i18n.global.setLocaleMessage(locale, translations)
      i18n.global.locale.value = locale
      currentLocale.value = locale

      // Save to localStorage
      localStorage.setItem('datamesh-language', locale)

      // Update document language
      document.documentElement.lang = locale
      
      // Update document direction
      document.documentElement.dir = SUPPORTED_LANGUAGES[locale].rtl ? 'rtl' : 'ltr'
      
      // Update page title if needed
      updatePageTitle()
      
      // Notify other components
      window.dispatchEvent(new CustomEvent('language-changed', {
        detail: { locale, language: SUPPORTED_LANGUAGES[locale] }
      }))

    } catch (err) {
      error.value = err.message
      console.error('Failed to change language:', err)
    } finally {
      isLoading.value = false
    }
  }

  const updatePageTitle = () => {
    const { t } = i18n.global
    document.title = t('common.appName', 'DataMesh')
  }

  const formatDate = (date, format = 'short') => {
    return new Intl.DateTimeFormat(currentLocale.value, 
      i18n.global.datetimeFormats[currentLocale.value][format]
    ).format(date)
  }

  const formatNumber = (number, format = 'decimal') => {
    return new Intl.NumberFormat(currentLocale.value,
      i18n.global.numberFormats[currentLocale.value][format]
    ).format(number)
  }

  const formatCurrency = (amount, currency) => {
    return new Intl.NumberFormat(currentLocale.value, {
      style: 'currency',
      currency: currency || currentLanguage.value.numberFormat.currency
    }).format(amount)
  }

  const formatFileSize = (bytes) => {
    const { t } = i18n.global
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let size = bytes
    let unitIndex = 0

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024
      unitIndex++
    }

    return `${formatNumber(size, 'decimal')} ${t(`common.units.${units[unitIndex].toLowerCase()}`)}`
  }

  const formatRelativeTime = (date) => {
    const rtf = new Intl.RelativeTimeFormat(currentLocale.value, { numeric: 'auto' })
    const now = new Date()
    const diff = date - now
    const seconds = Math.floor(diff / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)

    if (Math.abs(days) >= 1) {
      return rtf.format(days, 'day')
    } else if (Math.abs(hours) >= 1) {
      return rtf.format(hours, 'hour')
    } else if (Math.abs(minutes) >= 1) {
      return rtf.format(minutes, 'minute')
    } else {
      return rtf.format(seconds, 'second')
    }
  }

  const pluralize = (count, key) => {
    const { t } = i18n.global
    return t(key, count)
  }

  const getDirection = () => {
    return currentLanguage.value.rtl ? 'rtl' : 'ltr'
  }

  const isRTL = computed(() => currentLanguage.value.rtl)

  return {
    currentLocale: computed(() => currentLocale.value),
    currentLanguage,
    availableLanguages,
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    isRTL,
    changeLanguage,
    formatDate,
    formatNumber,
    formatCurrency,
    formatFileSize,
    formatRelativeTime,
    pluralize,
    getDirection
  }
}

// Translation helper functions
export function t(key, params = {}) {
  return i18n.global.t(key, params)
}

export function tc(key, count, params = {}) {
  return i18n.global.tc(key, count, params)
}

export function te(key) {
  return i18n.global.te(key)
}

export function tm(key) {
  return i18n.global.tm(key)
}

// Lazy translation loading
export async function loadLanguageAsync(locale) {
  if (i18n.global.availableLocales.includes(locale)) {
    return
  }

  const translations = await translationLoader.loadLanguage(locale)
  i18n.global.setLocaleMessage(locale, translations)
}

// Preload languages
export async function preloadLanguages(locales = ['en', 'es', 'fr']) {
  await translationLoader.preloadLanguages(locales)
}

// Language persistence
export function saveLanguagePreference(locale) {
  localStorage.setItem('datamesh-language', locale)
}

export function getLanguagePreference() {
  return localStorage.getItem('datamesh-language')
}

// Initialize i18n
export async function initializeI18n() {
  const locale = detectLanguage()
  
  try {
    const translations = await translationLoader.loadLanguage(locale)
    i18n.global.setLocaleMessage(locale, translations)
    i18n.global.locale.value = locale
    
    // Update document
    document.documentElement.lang = locale
    document.documentElement.dir = SUPPORTED_LANGUAGES[locale].rtl ? 'rtl' : 'ltr'
    
    // Preload common languages
    preloadLanguages(['en', 'es', 'fr', 'de'])
    
    console.log(`Initialized i18n with locale: ${locale}`)
  } catch (error) {
    console.error('Failed to initialize i18n:', error)
  }
}

export default {
  i18n,
  useLanguage,
  SUPPORTED_LANGUAGES,
  detectLanguage,
  t,
  tc,
  te,
  tm,
  loadLanguageAsync,
  preloadLanguages,
  initializeI18n
}
