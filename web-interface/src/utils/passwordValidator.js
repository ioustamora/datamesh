/**
 * Enhanced password validation utility
 * Implements strong password requirements and security checks
 */

// Common weak passwords and patterns
const COMMON_PASSWORDS = [
  'password', '123456', '123456789', '12345678', '12345', '1234567',
  'password123', 'admin', 'qwerty', 'abc123', 'letmein', 'monkey',
  'master', 'dragon', 'welcome', 'login', 'football', 'baseball',
  'superman', 'access', 'shadow', 'trustno1', 'passw0rd'
]

const SEQUENTIAL_PATTERNS = [
  'abcdefg', 'qwertyui', 'asdfghj', 'zxcvbnm', '1234567', '7654321'
]

const REPEATED_PATTERNS = [
  'aaaaaa', '111111', '000000', '......', '------', '______'
]

/**
 * Password validation configuration
 */
const PASSWORD_CONFIG = {
  minLength: 12,
  maxLength: 128,
  requireUppercase: true,
  requireLowercase: true,
  requireNumbers: true,
  requireSymbols: true,
  maxConsecutiveChars: 3,
  minUniqueChars: 6,
  forbidPersonalInfo: true,
  forbidCommonPasswords: true,
  forbidKeyboardPatterns: true,
  forbidDictionaryWords: true,
  requirePasswordChange: 90, // days
  preventReuse: 5 // previous passwords
}

/**
 * Password strength levels
 */
const STRENGTH_LEVELS = {
  VERY_WEAK: 0,
  WEAK: 1,
  FAIR: 2,
  GOOD: 3,
  STRONG: 4,
  VERY_STRONG: 5
}

/**
 * Password validation result
 */
class PasswordValidationResult {
  constructor() {
    this.isValid = false
    this.strength = STRENGTH_LEVELS.VERY_WEAK
    this.score = 0
    this.feedback = []
    this.errors = []
    this.warnings = []
    this.suggestions = []
    this.entropy = 0
    this.estimatedCrackTime = 0
  }
}

/**
 * Password Validator class
 */
class PasswordValidator {
  constructor(config = {}) {
    this.config = { ...PASSWORD_CONFIG, ...config }
    this.dictionaryWords = new Set()
    this.loadDictionary()
  }

  /**
   * Load dictionary words (in production, this would be loaded from a service)
   */
  loadDictionary() {
    // Basic dictionary words for validation
    const basicWords = [
      'password', 'admin', 'user', 'login', 'welcome', 'hello', 'world',
      'computer', 'internet', 'security', 'system', 'network', 'server',
      'database', 'application', 'software', 'hardware', 'technology'
    ]
    
    basicWords.forEach(word => {
      this.dictionaryWords.add(word.toLowerCase())
      // Add common variations
      this.dictionaryWords.add(word.toLowerCase() + '1')
      this.dictionaryWords.add(word.toLowerCase() + '123')
      this.dictionaryWords.add('1' + word.toLowerCase())
    })
  }

  /**
   * Validate password against all rules
   * @param {string} password - Password to validate
   * @param {Object} userInfo - User information for personal info check
   * @returns {PasswordValidationResult} Validation result
   */
  validate(password, userInfo = {}) {
    const result = new PasswordValidationResult()
    
    if (!password || typeof password !== 'string') {
      result.errors.push('Password is required')
      return result
    }

    // Basic length validation
    this.validateLength(password, result)
    
    // Character composition validation
    this.validateCharacterComposition(password, result)
    
    // Pattern validation
    this.validatePatterns(password, result)
    
    // Security checks
    this.validateSecurity(password, result, userInfo)
    
    // Calculate strength and entropy
    this.calculateStrength(password, result)
    
    // Generate suggestions
    this.generateSuggestions(password, result)
    
    // Final validation
    result.isValid = result.errors.length === 0
    
    return result
  }

  /**
   * Validate password length
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   */
  validateLength(password, result) {
    if (password.length < this.config.minLength) {
      result.errors.push(`Password must be at least ${this.config.minLength} characters long`)
    }
    
    if (password.length > this.config.maxLength) {
      result.errors.push(`Password must be no more than ${this.config.maxLength} characters long`)
    }
    
    if (password.length < 8) {
      result.feedback.push('Very short passwords are easily cracked')
    }
  }

  /**
   * Validate character composition
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   */
  validateCharacterComposition(password, result) {
    const hasUppercase = /[A-Z]/.test(password)
    const hasLowercase = /[a-z]/.test(password)
    const hasNumbers = /[0-9]/.test(password)
    const hasSymbols = /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?~`]/.test(password)
    
    if (this.config.requireUppercase && !hasUppercase) {
      result.errors.push('Password must contain at least one uppercase letter')
    }
    
    if (this.config.requireLowercase && !hasLowercase) {
      result.errors.push('Password must contain at least one lowercase letter')
    }
    
    if (this.config.requireNumbers && !hasNumbers) {
      result.errors.push('Password must contain at least one number')
    }
    
    if (this.config.requireSymbols && !hasSymbols) {
      result.errors.push('Password must contain at least one special character')
    }
    
    // Count character types
    let charTypes = 0
    if (hasUppercase) charTypes++
    if (hasLowercase) charTypes++
    if (hasNumbers) charTypes++
    if (hasSymbols) charTypes++
    
    if (charTypes < 3) {
      result.warnings.push('Use a mix of character types for better security')
    }
    
    // Check for unique characters
    const uniqueChars = new Set(password.toLowerCase()).size
    if (uniqueChars < this.config.minUniqueChars) {
      result.errors.push(`Password must contain at least ${this.config.minUniqueChars} unique characters`)
    }
  }

  /**
   * Validate patterns and sequences
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   */
  validatePatterns(password, result) {
    const lowerPassword = password.toLowerCase()
    
    // Check for consecutive characters
    let consecutiveCount = 1
    for (let i = 1; i < password.length; i++) {
      if (password[i] === password[i - 1]) {
        consecutiveCount++
        if (consecutiveCount > this.config.maxConsecutiveChars) {
          result.errors.push(`Password cannot contain more than ${this.config.maxConsecutiveChars} consecutive identical characters`)
          break
        }
      } else {
        consecutiveCount = 1
      }
    }
    
    // Check for sequential patterns
    if (this.config.forbidKeyboardPatterns) {
      SEQUENTIAL_PATTERNS.forEach(pattern => {
        if (lowerPassword.includes(pattern) || lowerPassword.includes(pattern.split('').reverse().join(''))) {
          result.errors.push('Password cannot contain keyboard patterns or sequences')
        }
      })
    }
    
    // Check for repeated patterns
    REPEATED_PATTERNS.forEach(pattern => {
      if (lowerPassword.includes(pattern)) {
        result.errors.push('Password cannot contain repeated character patterns')
      }
    })
    
    // Check for simple substitutions
    const commonSubstitutions = {
      'a': '@', 'e': '3', 'i': '1', 'o': '0', 's': '$', 't': '7'
    }
    
    let hasSubstitutions = false
    Object.keys(commonSubstitutions).forEach(char => {
      if (password.includes(commonSubstitutions[char])) {
        hasSubstitutions = true
      }
    })
    
    if (hasSubstitutions) {
      result.warnings.push('Avoid simple character substitutions (@ for a, 3 for e, etc.)')
    }
  }

  /**
   * Validate security requirements
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   * @param {Object} userInfo - User information
   */
  validateSecurity(password, result, userInfo) {
    const lowerPassword = password.toLowerCase()
    
    // Check against common passwords
    if (this.config.forbidCommonPasswords) {
      COMMON_PASSWORDS.forEach(common => {
        if (lowerPassword.includes(common)) {
          result.errors.push('Password cannot contain common passwords or phrases')
        }
      })
    }
    
    // Check against dictionary words
    if (this.config.forbidDictionaryWords) {
      this.dictionaryWords.forEach(word => {
        if (lowerPassword.includes(word)) {
          result.warnings.push('Avoid using dictionary words in passwords')
        }
      })
    }
    
    // Check against personal information
    if (this.config.forbidPersonalInfo && userInfo) {
      const personalInfo = [
        userInfo.username,
        userInfo.email?.split('@')[0],
        userInfo.firstName,
        userInfo.lastName,
        userInfo.displayName,
        userInfo.company
      ].filter(Boolean)
      
      personalInfo.forEach(info => {
        if (info && lowerPassword.includes(info.toLowerCase())) {
          result.errors.push('Password cannot contain personal information')
        }
      })
    }
    
    // Check for year patterns
    const currentYear = new Date().getFullYear()
    for (let year = currentYear - 10; year <= currentYear + 5; year++) {
      if (password.includes(year.toString())) {
        result.warnings.push('Avoid using years in passwords')
        break
      }
    }
    
    // Check for common number patterns
    const numberPatterns = ['123', '321', '111', '000', '999', '456', '789']
    numberPatterns.forEach(pattern => {
      if (password.includes(pattern)) {
        result.warnings.push('Avoid predictable number patterns')
      }
    })
  }

  /**
   * Calculate password strength and entropy
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   */
  calculateStrength(password, result) {
    let score = 0
    let entropy = 0
    
    // Character set size
    let charSetSize = 0
    if (/[a-z]/.test(password)) charSetSize += 26
    if (/[A-Z]/.test(password)) charSetSize += 26
    if (/[0-9]/.test(password)) charSetSize += 10
    if (/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?~`]/.test(password)) charSetSize += 32
    
    // Calculate entropy
    entropy = password.length * Math.log2(charSetSize)
    result.entropy = Math.round(entropy)
    
    // Length score
    if (password.length >= 12) score += 2
    else if (password.length >= 8) score += 1
    
    // Character diversity score
    const charTypes = [
      /[a-z]/.test(password),
      /[A-Z]/.test(password),
      /[0-9]/.test(password),
      /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?~`]/.test(password)
    ].filter(Boolean).length
    
    score += charTypes
    
    // Unique characters score
    const uniqueChars = new Set(password.toLowerCase()).size
    if (uniqueChars >= password.length * 0.8) score += 1
    
    // Penalty for common patterns
    if (result.warnings.length > 0) score -= 1
    if (result.errors.length > 0) score -= 2
    
    // Entropy bonus
    if (entropy >= 60) score += 2
    else if (entropy >= 40) score += 1
    
    // Normalize score
    score = Math.max(0, Math.min(5, score))
    result.score = score
    
    // Determine strength level
    if (score >= 5) result.strength = STRENGTH_LEVELS.VERY_STRONG
    else if (score >= 4) result.strength = STRENGTH_LEVELS.STRONG
    else if (score >= 3) result.strength = STRENGTH_LEVELS.GOOD
    else if (score >= 2) result.strength = STRENGTH_LEVELS.FAIR
    else if (score >= 1) result.strength = STRENGTH_LEVELS.WEAK
    else result.strength = STRENGTH_LEVELS.VERY_WEAK
    
    // Estimate crack time
    const guessesPerSecond = 1000000 // 1 million guesses per second
    const possibleCombinations = Math.pow(charSetSize, password.length)
    const averageGuesses = possibleCombinations / 2
    const crackTimeSeconds = averageGuesses / guessesPerSecond
    
    result.estimatedCrackTime = crackTimeSeconds
  }

  /**
   * Generate improvement suggestions
   * @param {string} password - Password to validate
   * @param {PasswordValidationResult} result - Validation result
   */
  generateSuggestions(password, result) {
    if (password.length < this.config.minLength) {
      result.suggestions.push(`Make your password at least ${this.config.minLength} characters long`)
    }
    
    if (!/[A-Z]/.test(password)) {
      result.suggestions.push('Add uppercase letters')
    }
    
    if (!/[a-z]/.test(password)) {
      result.suggestions.push('Add lowercase letters')
    }
    
    if (!/[0-9]/.test(password)) {
      result.suggestions.push('Add numbers')
    }
    
    if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?~`]/.test(password)) {
      result.suggestions.push('Add special characters (!@#$%^&*)')
    }
    
    if (result.entropy < 40) {
      result.suggestions.push('Use a longer password with more character variety')
    }
    
    if (result.warnings.length > 0) {
      result.suggestions.push('Avoid common patterns and dictionary words')
    }
    
    if (result.strength < STRENGTH_LEVELS.GOOD) {
      result.suggestions.push('Consider using a passphrase with random words')
    }
  }

  /**
   * Get human-readable strength description
   * @param {number} strength - Strength level
   * @returns {string} Strength description
   */
  getStrengthDescription(strength) {
    switch (strength) {
      case STRENGTH_LEVELS.VERY_WEAK:
        return 'Very Weak'
      case STRENGTH_LEVELS.WEAK:
        return 'Weak'
      case STRENGTH_LEVELS.FAIR:
        return 'Fair'
      case STRENGTH_LEVELS.GOOD:
        return 'Good'
      case STRENGTH_LEVELS.STRONG:
        return 'Strong'
      case STRENGTH_LEVELS.VERY_STRONG:
        return 'Very Strong'
      default:
        return 'Unknown'
    }
  }

  /**
   * Get human-readable crack time estimate
   * @param {number} seconds - Crack time in seconds
   * @returns {string} Human-readable time
   */
  getCrackTimeDescription(seconds) {
    if (seconds < 60) return 'Less than a minute'
    if (seconds < 3600) return `${Math.round(seconds / 60)} minutes`
    if (seconds < 86400) return `${Math.round(seconds / 3600)} hours`
    if (seconds < 2592000) return `${Math.round(seconds / 86400)} days`
    if (seconds < 31536000) return `${Math.round(seconds / 2592000)} months`
    return `${Math.round(seconds / 31536000)} years`
  }

  /**
   * Generate a secure password suggestion
   * @param {number} length - Desired password length
   * @returns {string} Generated password
   */
  generateSecurePassword(length = 16) {
    const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?'
    let password = ''
    
    // Ensure at least one character from each required type
    const requiredChars = [
      'abcdefghijklmnopqrstuvwxyz',
      'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
      '0123456789',
      '!@#$%^&*()_+-=[]{}|;:,.<>?'
    ]
    
    // Add one character from each required type
    requiredChars.forEach(chars => {
      const randomIndex = Math.floor(Math.random() * chars.length)
      password += chars[randomIndex]
    })
    
    // Fill remaining length with random characters
    for (let i = password.length; i < length; i++) {
      const randomIndex = Math.floor(Math.random() * charset.length)
      password += charset[randomIndex]
    }
    
    // Shuffle the password to avoid predictable patterns
    password = password.split('').sort(() => Math.random() - 0.5).join('')
    
    return password
  }
}

// Export singleton instance
export const passwordValidator = new PasswordValidator()

// Export classes and constants
export {
  PasswordValidator,
  PasswordValidationResult,
  STRENGTH_LEVELS,
  PASSWORD_CONFIG
}

export default passwordValidator