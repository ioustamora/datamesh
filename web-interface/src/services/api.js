import axios from 'axios'
import { secureStorage } from '../utils/secureStorage'
import { rateLimiter, csrfTokenManager } from '../utils/rateLimiter'

// Create axios instance with default configuration
const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// Rate-limited request wrapper
const rateLimitedRequest = async (config) => {
  const endpoint = config.url
  const method = config.method?.toUpperCase() || 'GET'
  
  return rateLimiter.executeRequest(
    () => api.request(config),
    endpoint,
    method
  )
}

// Request interceptor
api.interceptors.request.use(
  async (config) => {
    // Add auth token if available
    const token = await secureStorage.getToken()
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    
    // Add CSRF token
    const csrfHeaders = csrfTokenManager.getHeaders()
    Object.assign(config.headers, csrfHeaders)
    
    // Add rate limiting info to headers
    const rateStatus = rateLimiter.getStatus(config.url, config.method)
    config.headers['X-RateLimit-Remaining'] = rateStatus.remaining.toString()
    config.headers['X-RateLimit-Limit'] = rateStatus.limit.toString()
    
    // Log request in development
    if (import.meta.env.DEV) {
      console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`, config.data)
    }
    
    return config
  },
  (error) => {
    console.error('Request interceptor error:', error)
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  (response) => {
    // Log response in development
    if (import.meta.env.DEV) {
      console.log(`API Response: ${response.status} ${response.config.url}`, response.data)
    }
    
    return response
  },
  (error) => {
    console.error('API Error:', error.response?.data || error.message)
    
    // Handle common errors
    if (error.response?.status === 401) {
      // Unauthorized - clear tokens and redirect to login
      secureStorage.clear()
      window.location.href = '/auth/login'
      return Promise.reject(new Error('Authentication required'))
    }
    
    if (error.response?.status === 403) {
      return Promise.reject(new Error('Access forbidden'))
    }
    
    if (error.response?.status === 404) {
      return Promise.reject(new Error('Resource not found'))
    }
    
    if (error.response?.status === 429) {
      // Rate limited by server
      const retryAfter = error.response.headers['retry-after']
      const rateLimitError = new Error('Rate limit exceeded')
      rateLimitError.code = 'RATE_LIMIT_EXCEEDED'
      rateLimitError.retryAfter = retryAfter ? parseInt(retryAfter) * 1000 : 60000
      return Promise.reject(rateLimitError)
    }
    
    if (error.response?.status >= 500) {
      return Promise.reject(new Error('Server error occurred'))
    }
    
    if (error.code === 'ECONNABORTED') {
      return Promise.reject(new Error('Request timeout'))
    }
    
    if (error.code === 'RATE_LIMIT_EXCEEDED') {
      // Client-side rate limit exceeded
      return Promise.reject(error)
    }
    
    return Promise.reject(error.response?.data || error)
  }
)

// Authentication API
export const authAPI = {
  login: (credentials) => api.post('/auth/login', credentials),
  register: (userData) => api.post('/auth/register', userData),
  logout: () => api.post('/auth/logout'),
  me: () => api.get('/auth/me'),
  updateProfile: (profileData) => api.put('/auth/profile', profileData),
  changePassword: (passwordData) => api.put('/auth/password', passwordData),
  forgotPassword: (email) => api.post('/auth/forgot-password', { email }),
  resetPassword: (resetData) => api.post('/auth/reset-password', resetData),
  refreshToken: () => api.post('/auth/refresh'),
  
  setAuthToken: (token) => {
    if (token) {
      api.defaults.headers.common['Authorization'] = `Bearer ${token}`
    } else {
      delete api.defaults.headers.common['Authorization']
    }
  }
}

// Files API
export const filesAPI = {
  listFiles: (params = {}) => api.get('/files', { params }),
  uploadFile: (formData, config = {}) => api.post('/files', formData, {
    headers: {
      'Content-Type': 'multipart/form-data'
    },
    ...config
  }),
  downloadFile: (fileKey) => api.get(`/files/${fileKey}`, {
    responseType: 'blob'
  }),
  deleteFile: (fileKey) => api.delete(`/files/${fileKey}`),
  getFileMetadata: (fileKey) => api.get(`/files/${fileKey}/metadata`),
  searchFiles: (searchData) => api.post('/search', searchData),
  getStats: () => api.get('/stats')
}

// Governance API
export const governanceAPI = {
  getStatus: () => api.get('/governance/status'),
  getOperators: () => api.get('/governance/operators'),
  getOperator: (operatorId) => api.get(`/governance/operators/${operatorId}`),
  getOperatorDashboard: (operatorId) => api.get(`/governance/operators/${operatorId}/dashboard`),
  getNetworkHealth: () => api.get('/governance/network/health'),
  
  // Admin endpoints
  registerOperator: (operatorData) => api.post('/admin/operators', operatorData),
  registerService: (operatorId, serviceData) => api.post(`/admin/operators/${operatorId}/services`, serviceData),
  updateServiceHeartbeat: (operatorId, serviceId) => api.post(`/admin/operators/${operatorId}/services/${serviceId}/heartbeat`),
  executeAdminAction: (actionData) => api.post('/admin/actions', actionData),
  getAdminActions: () => api.get('/admin/actions'),
  cleanupInactiveOperators: () => api.post('/admin/cleanup/operators'),
  
  // Future governance features
  getProposals: () => api.get('/governance/proposals'),
  createProposal: (proposalData) => api.post('/governance/proposals', proposalData),
  getProposal: (proposalId) => api.get(`/governance/proposals/${proposalId}`),
  voteOnProposal: (proposalId, voteData) => api.post(`/governance/proposals/${proposalId}/vote`, voteData),
  getVotes: (proposalId) => api.get(`/governance/proposals/${proposalId}/votes`)
}

// Analytics API
export const analyticsAPI = {
  getSystemMetrics: (timeRange = '24h') => api.get('/analytics/system', { params: { range: timeRange } }),
  getStorageMetrics: (timeRange = '24h') => api.get('/analytics/storage', { params: { range: timeRange } }),
  getNetworkMetrics: (timeRange = '24h') => api.get('/analytics/network', { params: { range: timeRange } }),
  getPerformanceMetrics: (timeRange = '24h') => api.get('/analytics/performance', { params: { range: timeRange } }),
  getUserMetrics: (userId, timeRange = '24h') => api.get(`/analytics/users/${userId}`, { params: { range: timeRange } }),
  getOperatorMetrics: (operatorId, timeRange = '24h') => api.get(`/analytics/operators/${operatorId}`, { params: { range: timeRange } }),
  exportMetrics: (format = 'json', timeRange = '24h') => api.get('/analytics/export', { 
    params: { format, range: timeRange },
    responseType: format === 'csv' ? 'blob' : 'json'
  })
}

// Administration API
export const adminAPI = {
  getUsers: (params = {}) => api.get('/admin/users', { params }),
  getUser: (userId) => api.get(`/admin/users/${userId}`),
  createUser: (userData) => api.post('/admin/users', userData),
  updateUser: (userId, userData) => api.put(`/admin/users/${userId}`, userData),
  deleteUser: (userId) => api.delete(`/admin/users/${userId}`),
  getUserQuota: (userId) => api.get(`/admin/users/${userId}/quota`),
  updateUserQuota: (userId, quotaData) => api.put(`/admin/users/${userId}/quota`, quotaData),
  
  getSystemConfig: () => api.get('/admin/config'),
  updateSystemConfig: (configData) => api.put('/admin/config', configData),
  getSystemLogs: (params = {}) => api.get('/admin/logs', { params }),
  getAuditLogs: (params = {}) => api.get('/admin/audit', { params }),
  
  getSystemHealth: () => api.get('/admin/health'),
  performMaintenance: (maintenanceData) => api.post('/admin/maintenance', maintenanceData),
  createBackup: (backupData) => api.post('/admin/backup', backupData),
  restoreBackup: (backupId) => api.post(`/admin/backup/${backupId}/restore`)
}

// Settings API
export const settingsAPI = {
  getUserSettings: () => api.get('/settings'),
  updateUserSettings: (settings) => api.put('/settings', settings),
  getNotificationSettings: () => api.get('/settings/notifications'),
  updateNotificationSettings: (settings) => api.put('/settings/notifications', settings),
  getPrivacySettings: () => api.get('/settings/privacy'),
  updatePrivacySettings: (settings) => api.put('/settings/privacy', settings),
  exportUserData: () => api.get('/settings/export', { responseType: 'blob' }),
  deleteUserData: () => api.delete('/settings/data')
}

// Health check API
export const healthAPI = {
  check: () => api.get('/health'),
  detailed: () => api.get('/health/detailed'),
  ping: () => api.get('/health/ping')
}

// Utility functions
export const apiUtils = {
  // File upload with progress
  uploadWithProgress: (file, onProgress = () => {}) => {
    const formData = new FormData()
    formData.append('file', file)
    
    return api.post('/files', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      },
      onUploadProgress: (progressEvent) => {
        const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
        onProgress(progress)
      }
    })
  },
  
  // Batch operations
  batchRequest: (requests) => {
    return Promise.allSettled(requests.map(req => api(req)))
  },
  
  // Retry mechanism
  retryRequest: async (requestFn, maxRetries = 3, delay = 1000) => {
    for (let i = 0; i < maxRetries; i++) {
      try {
        return await requestFn()
      } catch (error) {
        if (i === maxRetries - 1) throw error
        await new Promise(resolve => setTimeout(resolve, delay * Math.pow(2, i)))
      }
    }
  },
  
  // Cancel token creation
  createCancelToken: () => axios.CancelToken.source(),
  
  // Check if error is due to cancellation
  isCancel: (error) => axios.isCancel(error),
  
  // Format error message
  formatError: (error) => {
    if (error.response) {
      return error.response.data?.message || error.response.statusText || 'Server error'
    }
    if (error.request) {
      return 'Network error - please check your connection'
    }
    return error.message || 'An unexpected error occurred'
  }
}

// Export default api instance
export default api