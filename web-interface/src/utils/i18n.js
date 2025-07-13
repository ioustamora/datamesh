/**
 * Internationalization (i18n) Support System
 * Multi-language and localization support for DataMesh
 */

import { createI18n } from 'vue-i18n'

/**
 * Internationalization manager
 */
export class I18nManager {
  constructor(options = {}) {
    this.options = {
      locale: options.locale || 'en',
      fallbackLocale: options.fallbackLocale || 'en',
      availableLocales: options.availableLocales || ['en', 'es', 'fr', 'de', 'zh', 'ja', 'ko'],
      autoDetect: options.autoDetect !== false,
      persistence: options.persistence !== false,
      ...options
    }

    this.i18n = null
    this.currentLocale = this.options.locale
    this.loadedLocales = new Set()
    this.translations = new Map()
    this.formatters = new Map()
    this.pluralRules = new Map()
    
    this.init()
  }

  /**
   * Initialize i18n system
   */
  async init() {
    // Detect user preferred locale
    if (this.options.autoDetect) {
      this.currentLocale = this.detectUserLocale()
    }

    // Load saved locale preference
    if (this.options.persistence) {
      const savedLocale = localStorage.getItem('datamesh-locale')
      if (savedLocale && this.options.availableLocales.includes(savedLocale)) {
        this.currentLocale = savedLocale
      }
    }

    // Create i18n instance
    this.i18n = createI18n({
      locale: this.currentLocale,
      fallbackLocale: this.options.fallbackLocale,
      legacy: false,
      globalInjection: true,
      messages: {}
    })

    // Load default messages
    await this.loadLocaleMessages(this.currentLocale)
    
    // Setup formatters
    this.setupFormatters()
    
    // Setup plural rules
    this.setupPluralRules()
    
    console.log(`🌍 i18n: Initialized with locale "${this.currentLocale}"`)
  }

  /**
   * Detect user's preferred locale
   */
  detectUserLocale() {
    // Check navigator languages
    const languages = navigator.languages || [navigator.language]
    
    for (const lang of languages) {
      // Check exact match
      if (this.options.availableLocales.includes(lang)) {
        return lang
      }
      
      // Check language code only (e.g., 'en' from 'en-US')
      const langCode = lang.split('-')[0]
      if (this.options.availableLocales.includes(langCode)) {
        return langCode
      }
    }
    
    return this.options.fallbackLocale
  }

  /**
   * Load locale messages
   */
  async loadLocaleMessages(locale) {
    if (this.loadedLocales.has(locale)) {
      return
    }

    try {
      // Load translation files
      const messages = await this.loadTranslationFiles(locale)
      
      // Set messages in i18n instance
      this.i18n.global.setLocaleMessage(locale, messages)
      
      // Cache translations
      this.translations.set(locale, messages)
      this.loadedLocales.add(locale)
      
      console.log(`🌍 i18n: Loaded messages for locale "${locale}"`)
    } catch (error) {
      console.error(`🌍 i18n: Failed to load messages for locale "${locale}":`, error)
    }
  }

  /**
   * Load translation files for a locale
   */
  async loadTranslationFiles(locale) {
    const messages = {}
    
    // Define translation modules
    const modules = [
      'common',
      'dashboard',
      'files',
      'network',
      'settings',
      'auth',
      'governance',
      'storage',
      'errors'
    ]

    // Load each module
    for (const module of modules) {
      try {
        const moduleMessages = await this.loadTranslationModule(locale, module)
        messages[module] = moduleMessages
      } catch (error) {
        console.warn(`🌍 i18n: Failed to load module "${module}" for locale "${locale}":`, error)
        // Use fallback locale for missing modules
        if (locale !== this.options.fallbackLocale) {
          try {
            const fallbackMessages = await this.loadTranslationModule(this.options.fallbackLocale, module)
            messages[module] = fallbackMessages
          } catch (fallbackError) {
            console.error(`🌍 i18n: Failed to load fallback module "${module}":`, fallbackError)
          }
        }
      }
    }

    return messages
  }

  /**
   * Load individual translation module
   */
  async loadTranslationModule(locale, module) {
    // In a real application, these would be loaded from files or API
    const translations = {
      en: {
        common: {
          yes: 'Yes',
          no: 'No',
          ok: 'OK',
          cancel: 'Cancel',
          save: 'Save',
          delete: 'Delete',
          edit: 'Edit',
          create: 'Create',
          update: 'Update',
          search: 'Search',
          filter: 'Filter',
          sort: 'Sort',
          loading: 'Loading...',
          error: 'Error',
          success: 'Success',
          warning: 'Warning',
          info: 'Information',
          close: 'Close',
          back: 'Back',
          next: 'Next',
          previous: 'Previous',
          home: 'Home',
          settings: 'Settings',
          help: 'Help',
          about: 'About',
          logout: 'Logout',
          login: 'Login',
          register: 'Register',
          password: 'Password',
          username: 'Username',
          email: 'Email',
          name: 'Name',
          status: 'Status',
          actions: 'Actions',
          date: 'Date',
          time: 'Time',
          size: 'Size',
          type: 'Type',
          total: 'Total',
          available: 'Available',
          used: 'Used',
          free: 'Free',
          online: 'Online',
          offline: 'Offline',
          connected: 'Connected',
          disconnected: 'Disconnected'
        },
        dashboard: {
          title: 'Dashboard',
          overview: 'Overview',
          storage: 'Storage',
          network: 'Network',
          files: 'Files',
          nodes: 'Nodes',
          performance: 'Performance',
          analytics: 'Analytics',
          alerts: 'Alerts',
          recent_activity: 'Recent Activity',
          system_health: 'System Health',
          capacity_usage: 'Capacity Usage',
          network_status: 'Network Status',
          file_operations: 'File Operations',
          node_management: 'Node Management'
        },
        files: {
          title: 'Files',
          upload: 'Upload',
          download: 'Download',
          share: 'Share',
          rename: 'Rename',
          move: 'Move',
          copy: 'Copy',
          properties: 'Properties',
          file_name: 'File Name',
          file_size: 'File Size',
          file_type: 'File Type',
          created_date: 'Created Date',
          modified_date: 'Modified Date',
          owner: 'Owner',
          permissions: 'Permissions',
          upload_file: 'Upload File',
          upload_folder: 'Upload Folder',
          create_folder: 'Create Folder',
          select_files: 'Select Files',
          drag_drop: 'Drag and drop files here',
          upload_progress: 'Upload Progress',
          download_progress: 'Download Progress'
        },
        network: {
          title: 'Network',
          peers: 'Peers',
          connections: 'Connections',
          bandwidth: 'Bandwidth',
          latency: 'Latency',
          throughput: 'Throughput',
          peer_id: 'Peer ID',
          peer_address: 'Peer Address',
          peer_status: 'Peer Status',
          connection_time: 'Connection Time',
          data_transferred: 'Data Transferred',
          network_map: 'Network Map',
          peer_discovery: 'Peer Discovery',
          connection_health: 'Connection Health'
        },
        settings: {
          title: 'Settings',
          general: 'General',
          security: 'Security',
          privacy: 'Privacy',
          notifications: 'Notifications',
          appearance: 'Appearance',
          language: 'Language',
          theme: 'Theme',
          timezone: 'Timezone',
          auto_save: 'Auto Save',
          backup: 'Backup',
          restore: 'Restore',
          export: 'Export',
          import: 'Import',
          reset: 'Reset',
          advanced: 'Advanced'
        },
        auth: {
          login: 'Login',
          logout: 'Logout',
          register: 'Register',
          forgot_password: 'Forgot Password',
          reset_password: 'Reset Password',
          change_password: 'Change Password',
          two_factor: 'Two Factor Authentication',
          sign_in: 'Sign In',
          sign_up: 'Sign Up',
          sign_out: 'Sign Out',
          welcome: 'Welcome',
          goodbye: 'Goodbye',
          session_expired: 'Session Expired',
          invalid_credentials: 'Invalid Credentials',
          account_locked: 'Account Locked',
          password_strength: 'Password Strength',
          confirm_password: 'Confirm Password'
        },
        governance: {
          title: 'Governance',
          policies: 'Policies',
          rules: 'Rules',
          compliance: 'Compliance',
          audit: 'Audit',
          permissions: 'Permissions',
          roles: 'Roles',
          users: 'Users',
          groups: 'Groups',
          access_control: 'Access Control',
          policy_management: 'Policy Management',
          audit_log: 'Audit Log',
          compliance_report: 'Compliance Report'
        },
        storage: {
          title: 'Storage',
          capacity: 'Capacity',
          usage: 'Usage',
          quota: 'Quota',
          redundancy: 'Redundancy',
          replication: 'Replication',
          backup: 'Backup',
          archive: 'Archive',
          cleanup: 'Cleanup',
          optimization: 'Optimization',
          storage_pool: 'Storage Pool',
          data_integrity: 'Data Integrity',
          storage_health: 'Storage Health'
        },
        errors: {
          generic: 'An error occurred',
          network: 'Network error',
          server: 'Server error',
          timeout: 'Request timeout',
          not_found: 'Not found',
          unauthorized: 'Unauthorized',
          forbidden: 'Forbidden',
          validation: 'Validation error',
          file_not_found: 'File not found',
          upload_failed: 'Upload failed',
          download_failed: 'Download failed',
          connection_failed: 'Connection failed',
          authentication_failed: 'Authentication failed',
          permission_denied: 'Permission denied',
          quota_exceeded: 'Quota exceeded',
          storage_full: 'Storage full',
          invalid_file_type: 'Invalid file type',
          file_too_large: 'File too large',
          operation_cancelled: 'Operation cancelled',
          try_again: 'Please try again',
          contact_support: 'Contact support if the problem persists'
        }
      },
      es: {
        common: {
          yes: 'Sí',
          no: 'No',
          ok: 'OK',
          cancel: 'Cancelar',
          save: 'Guardar',
          delete: 'Eliminar',
          edit: 'Editar',
          create: 'Crear',
          update: 'Actualizar',
          search: 'Buscar',
          filter: 'Filtrar',
          sort: 'Ordenar',
          loading: 'Cargando...',
          error: 'Error',
          success: 'Éxito',
          warning: 'Advertencia',
          info: 'Información',
          close: 'Cerrar',
          back: 'Atrás',
          next: 'Siguiente',
          previous: 'Anterior',
          home: 'Inicio',
          settings: 'Configuración',
          help: 'Ayuda',
          about: 'Acerca de',
          logout: 'Cerrar sesión',
          login: 'Iniciar sesión',
          register: 'Registrarse',
          password: 'Contraseña',
          username: 'Nombre de usuario',
          email: 'Correo electrónico',
          name: 'Nombre',
          status: 'Estado',
          actions: 'Acciones',
          date: 'Fecha',
          time: 'Hora',
          size: 'Tamaño',
          type: 'Tipo',
          total: 'Total',
          available: 'Disponible',
          used: 'Usado',
          free: 'Libre',
          online: 'En línea',
          offline: 'Fuera de línea',
          connected: 'Conectado',
          disconnected: 'Desconectado'
        },
        dashboard: {
          title: 'Panel de Control',
          overview: 'Resumen',
          storage: 'Almacenamiento',
          network: 'Red',
          files: 'Archivos',
          nodes: 'Nodos',
          performance: 'Rendimiento',
          analytics: 'Análisis',
          alerts: 'Alertas',
          recent_activity: 'Actividad Reciente',
          system_health: 'Salud del Sistema',
          capacity_usage: 'Uso de Capacidad',
          network_status: 'Estado de la Red',
          file_operations: 'Operaciones de Archivo',
          node_management: 'Gestión de Nodos'
        }
        // ... more Spanish translations
      },
      fr: {
        common: {
          yes: 'Oui',
          no: 'Non',
          ok: 'OK',
          cancel: 'Annuler',
          save: 'Sauvegarder',
          delete: 'Supprimer',
          edit: 'Modifier',
          create: 'Créer',
          update: 'Mettre à jour',
          search: 'Rechercher',
          filter: 'Filtrer',
          sort: 'Trier',
          loading: 'Chargement...',
          error: 'Erreur',
          success: 'Succès',
          warning: 'Avertissement',
          info: 'Information',
          close: 'Fermer',
          back: 'Retour',
          next: 'Suivant',
          previous: 'Précédent',
          home: 'Accueil',
          settings: 'Paramètres',
          help: 'Aide',
          about: 'À propos',
          logout: 'Déconnexion',
          login: 'Connexion',
          register: 'S\'inscrire',
          password: 'Mot de passe',
          username: 'Nom d\'utilisateur',
          email: 'Email',
          name: 'Nom',
          status: 'Statut',
          actions: 'Actions',
          date: 'Date',
          time: 'Heure',
          size: 'Taille',
          type: 'Type',
          total: 'Total',
          available: 'Disponible',
          used: 'Utilisé',
          free: 'Libre',
          online: 'En ligne',
          offline: 'Hors ligne',
          connected: 'Connecté',
          disconnected: 'Déconnecté'
        },
        dashboard: {
          title: 'Tableau de Bord',
          overview: 'Aperçu',
          storage: 'Stockage',
          network: 'Réseau',
          files: 'Fichiers',
          nodes: 'Nœuds',
          performance: 'Performance',
          analytics: 'Analytique',
          alerts: 'Alertes',
          recent_activity: 'Activité Récente',
          system_health: 'Santé du Système',
          capacity_usage: 'Utilisation de Capacité',
          network_status: 'État du Réseau',
          file_operations: 'Opérations de Fichier',
          node_management: 'Gestion des Nœuds'
        }
        // ... more French translations
      }
      // ... more languages
    }

    return translations[locale]?.[module] || translations[this.options.fallbackLocale]?.[module] || {}
  }

  /**
   * Setup number and date formatters
   */
  setupFormatters() {
    // Number formatters
    this.formatters.set('number', new Intl.NumberFormat(this.currentLocale))
    this.formatters.set('currency', new Intl.NumberFormat(this.currentLocale, {
      style: 'currency',
      currency: 'USD'
    }))
    this.formatters.set('percent', new Intl.NumberFormat(this.currentLocale, {
      style: 'percent',
      minimumFractionDigits: 1,
      maximumFractionDigits: 2
    }))
    this.formatters.set('filesize', (bytes) => {
      const units = ['B', 'KB', 'MB', 'GB', 'TB']
      let size = bytes
      let unitIndex = 0
      
      while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024
        unitIndex++
      }
      
      return `${size.toFixed(1)} ${units[unitIndex]}`
    })

    // Date formatters
    this.formatters.set('date', new Intl.DateTimeFormat(this.currentLocale))
    this.formatters.set('time', new Intl.DateTimeFormat(this.currentLocale, {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }))
    this.formatters.set('datetime', new Intl.DateTimeFormat(this.currentLocale, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }))
    this.formatters.set('relative', new Intl.RelativeTimeFormat(this.currentLocale, {
      numeric: 'auto'
    }))
  }

  /**
   * Setup plural rules
   */
  setupPluralRules() {
    this.pluralRules.set(this.currentLocale, new Intl.PluralRules(this.currentLocale))
  }

  /**
   * Change locale
   */
  async changeLocale(locale) {
    if (!this.options.availableLocales.includes(locale)) {
      throw new Error(`Locale "${locale}" is not available`)
    }

    // Load locale messages if not already loaded
    await this.loadLocaleMessages(locale)

    // Update i18n instance
    this.i18n.global.locale.value = locale
    this.currentLocale = locale

    // Update formatters
    this.setupFormatters()
    this.setupPluralRules()

    // Save preference
    if (this.options.persistence) {
      localStorage.setItem('datamesh-locale', locale)
    }

    // Update document direction and lang
    document.documentElement.lang = locale
    document.documentElement.dir = this.getRTLLocales().includes(locale) ? 'rtl' : 'ltr'

    console.log(`🌍 i18n: Changed locale to "${locale}"`)
  }

  /**
   * Get RTL locales
   */
  getRTLLocales() {
    return ['ar', 'he', 'fa', 'ur']
  }

  /**
   * Format number
   */
  formatNumber(value, type = 'number', options = {}) {
    const formatter = this.formatters.get(type)
    if (!formatter) {
      return value.toString()
    }

    if (typeof formatter === 'function') {
      return formatter(value)
    }

    return formatter.format(value)
  }

  /**
   * Format date
   */
  formatDate(value, type = 'date', options = {}) {
    const formatter = this.formatters.get(type)
    if (!formatter) {
      return value.toString()
    }

    const date = value instanceof Date ? value : new Date(value)
    return formatter.format(date)
  }

  /**
   * Format relative time
   */
  formatRelativeTime(value, unit = 'second') {
    const formatter = this.formatters.get('relative')
    if (!formatter) {
      return value.toString()
    }

    return formatter.format(value, unit)
  }

  /**
   * Get plural form
   */
  getPlural(count, forms) {
    const rule = this.pluralRules.get(this.currentLocale)
    if (!rule) {
      return forms.other || forms.one || ''
    }

    const category = rule.select(count)
    return forms[category] || forms.other || forms.one || ''
  }

  /**
   * Translate message
   */
  t(key, values = {}) {
    if (!this.i18n) {
      return key
    }

    return this.i18n.global.t(key, values)
  }

  /**
   * Check if translation exists
   */
  te(key) {
    if (!this.i18n) {
      return false
    }

    return this.i18n.global.te(key)
  }

  /**
   * Get current locale
   */
  getCurrentLocale() {
    return this.currentLocale
  }

  /**
   * Get available locales
   */
  getAvailableLocales() {
    return this.options.availableLocales
  }

  /**
   * Get locale info
   */
  getLocaleInfo(locale) {
    const localeInfo = {
      en: { name: 'English', nativeName: 'English' },
      es: { name: 'Spanish', nativeName: 'Español' },
      fr: { name: 'French', nativeName: 'Français' },
      de: { name: 'German', nativeName: 'Deutsch' },
      zh: { name: 'Chinese', nativeName: '中文' },
      ja: { name: 'Japanese', nativeName: '日本語' },
      ko: { name: 'Korean', nativeName: '한국어' },
      ar: { name: 'Arabic', nativeName: 'العربية' },
      he: { name: 'Hebrew', nativeName: 'עברית' },
      ru: { name: 'Russian', nativeName: 'Русский' },
      pt: { name: 'Portuguese', nativeName: 'Português' },
      it: { name: 'Italian', nativeName: 'Italiano' },
      nl: { name: 'Dutch', nativeName: 'Nederlands' },
      sv: { name: 'Swedish', nativeName: 'Svenska' },
      da: { name: 'Danish', nativeName: 'Dansk' },
      no: { name: 'Norwegian', nativeName: 'Norsk' },
      fi: { name: 'Finnish', nativeName: 'Suomi' },
      pl: { name: 'Polish', nativeName: 'Polski' },
      tr: { name: 'Turkish', nativeName: 'Türkçe' },
      hi: { name: 'Hindi', nativeName: 'हिन्दी' },
      th: { name: 'Thai', nativeName: 'ไทย' },
      vi: { name: 'Vietnamese', nativeName: 'Tiếng Việt' }
    }

    return localeInfo[locale] || { name: locale, nativeName: locale }
  }

  /**
   * Get i18n instance
   */
  getI18n() {
    return this.i18n
  }

  /**
   * Export translations
   */
  exportTranslations(locale) {
    const translations = this.translations.get(locale)
    if (!translations) {
      throw new Error(`Translations for locale "${locale}" not loaded`)
    }

    return JSON.stringify(translations, null, 2)
  }

  /**
   * Import translations
   */
  async importTranslations(locale, translationsJson) {
    try {
      const translations = JSON.parse(translationsJson)
      
      // Validate translations structure
      if (typeof translations !== 'object') {
        throw new Error('Invalid translations format')
      }

      // Set translations
      this.i18n.global.setLocaleMessage(locale, translations)
      this.translations.set(locale, translations)
      this.loadedLocales.add(locale)

      console.log(`🌍 i18n: Imported translations for locale "${locale}"`)
    } catch (error) {
      throw new Error(`Failed to import translations: ${error.message}`)
    }
  }

  /**
   * Get translation statistics
   */
  getTranslationStats() {
    const stats = {
      currentLocale: this.currentLocale,
      availableLocales: this.options.availableLocales.length,
      loadedLocales: this.loadedLocales.size,
      translations: {},
      coverage: {}
    }

    // Count translations for each locale
    for (const [locale, translations] of this.translations) {
      stats.translations[locale] = this.countTranslations(translations)
    }

    // Calculate coverage
    if (this.translations.has(this.options.fallbackLocale)) {
      const fallbackCount = stats.translations[this.options.fallbackLocale]
      
      for (const [locale, count] of Object.entries(stats.translations)) {
        stats.coverage[locale] = ((count / fallbackCount) * 100).toFixed(1)
      }
    }

    return stats
  }

  /**
   * Count translations recursively
   */
  countTranslations(obj) {
    let count = 0
    
    for (const value of Object.values(obj)) {
      if (typeof value === 'object' && value !== null) {
        count += this.countTranslations(value)
      } else if (typeof value === 'string') {
        count++
      }
    }
    
    return count
  }
}

// Vue composable for i18n
export function useI18n() {
  const i18nManager = new I18nManager()
  
  return {
    t: i18nManager.t.bind(i18nManager),
    te: i18nManager.te.bind(i18nManager),
    formatNumber: i18nManager.formatNumber.bind(i18nManager),
    formatDate: i18nManager.formatDate.bind(i18nManager),
    formatRelativeTime: i18nManager.formatRelativeTime.bind(i18nManager),
    getPlural: i18nManager.getPlural.bind(i18nManager),
    changeLocale: i18nManager.changeLocale.bind(i18nManager),
    getCurrentLocale: i18nManager.getCurrentLocale.bind(i18nManager),
    getAvailableLocales: i18nManager.getAvailableLocales.bind(i18nManager),
    getLocaleInfo: i18nManager.getLocaleInfo.bind(i18nManager)
  }
}

export const i18nManager = new I18nManager()

export default {
  I18nManager,
  i18nManager,
  useI18n
}
