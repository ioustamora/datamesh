import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authAPI } from '../services/api'

export const useAuthStore = defineStore('auth', () => {
  // State
  const user = ref(null)
  const token = ref(localStorage.getItem('datamesh_token') || null)
  const initialized = ref(false)
  const loading = ref(false)
  
  // Getters
  const isAuthenticated = computed(() => !!token.value && !!user.value)
  const isAdmin = computed(() => user.value?.role === 'admin' || user.value?.permissions?.includes('admin'))
  const isOperator = computed(() => user.value?.role === 'operator' || user.value?.permissions?.includes('operator'))
  const userQuota = computed(() => user.value?.quota || null)
  const currentUser = computed(() => user.value)
  
  // Actions
  const login = async (credentials) => {
    loading.value = true
    try {
      const response = await authAPI.login(credentials)
      
      token.value = response.data.token
      user.value = response.data.user
      
      // Store token in localStorage
      localStorage.setItem('datamesh_token', token.value)
      
      // Set default Authorization header
      authAPI.setAuthToken(token.value)
      
      return response.data
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const register = async (userData) => {
    loading.value = true
    try {
      const response = await authAPI.register(userData)
      
      // Auto-login after registration
      token.value = response.data.token
      user.value = response.data.user
      
      localStorage.setItem('datamesh_token', token.value)
      authAPI.setAuthToken(token.value)
      
      return response.data
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const logout = async () => {
    try {
      if (token.value) {
        await authAPI.logout()
      }
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      // Clear state regardless of API call success
      user.value = null
      token.value = null
      
      // Clear localStorage
      localStorage.removeItem('datamesh_token')
      
      // Clear auth header
      authAPI.setAuthToken(null)
    }
  }
  
  const checkAuth = async () => {
    if (!token.value) {
      initialized.value = true
      return false
    }
    
    loading.value = true
    try {
      // Set token in API service
      authAPI.setAuthToken(token.value)
      
      // Verify token with server
      const response = await authAPI.me()
      user.value = response.data
      
      initialized.value = true
      return true
    } catch (error) {
      console.error('Auth check failed:', error)
      
      // Clear invalid token
      token.value = null
      user.value = null
      localStorage.removeItem('datamesh_token')
      authAPI.setAuthToken(null)
      
      initialized.value = true
      return false
    } finally {
      loading.value = false
    }
  }
  
  const updateProfile = async (profileData) => {
    loading.value = true
    try {
      const response = await authAPI.updateProfile(profileData)
      user.value = { ...user.value, ...response.data }
      return response.data
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const changePassword = async (passwordData) => {
    loading.value = true
    try {
      await authAPI.changePassword(passwordData)
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const forgotPassword = async (email) => {
    loading.value = true
    try {
      await authAPI.forgotPassword(email)
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const resetPassword = async (resetData) => {
    loading.value = true
    try {
      await authAPI.resetPassword(resetData)
    } catch (error) {
      throw error
    } finally {
      loading.value = false
    }
  }
  
  const refreshToken = async () => {
    if (!token.value) return false
    
    try {
      const response = await authAPI.refreshToken()
      token.value = response.data.token
      localStorage.setItem('datamesh_token', token.value)
      authAPI.setAuthToken(token.value)
      return true
    } catch (error) {
      console.error('Token refresh failed:', error)
      await logout()
      return false
    }
  }
  
  // Initialize store
  const init = async () => {
    if (token.value) {
      await checkAuth()
    } else {
      initialized.value = true
    }
  }
  
  return {
    // State
    user,
    token,
    initialized,
    loading,
    
    // Getters
    isAuthenticated,
    isAdmin,
    isOperator,
    userQuota,
    currentUser,
    
    // Actions
    login,
    register,
    logout,
    checkAuth,
    updateProfile,
    changePassword,
    forgotPassword,
    resetPassword,
    refreshToken,
    init
  }
})