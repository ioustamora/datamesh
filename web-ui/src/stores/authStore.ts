import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { User, AuthResponse } from '@/types'
import { api } from '@/utils/api'

interface AuthState {
  user: User | null
  token: string | null
  isLoading: boolean
  error: string | null
  
  // Actions
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string, publicKey?: string) => Promise<void>
  logout: () => void
  clearError: () => void
  setLoading: (loading: boolean) => void
  initializeAuth: () => void
  refreshToken: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isLoading: false,
      error: null,

      login: async (email: string, password: string) => {
        set({ isLoading: true, error: null })
        
        try {
          const response = await api.post<AuthResponse>('/auth/login', {
            email,
            password,
          })
          
          const { access_token, user } = response.data
          
          set({
            user,
            token: access_token,
            isLoading: false,
            error: null,
          })
          
          // Set authorization header for future requests
          api.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          
        } catch (error: any) {
          const errorMessage = error.response?.data?.message || 'Login failed'
          set({
            isLoading: false,
            error: errorMessage,
            user: null,
            token: null,
          })
          throw new Error(errorMessage)
        }
      },

      register: async (email: string, password: string, publicKey?: string) => {
        set({ isLoading: true, error: null })
        
        try {
          const response = await api.post<AuthResponse>('/auth/register', {
            email,
            password,
            public_key: publicKey || '',
          })
          
          const { access_token, user } = response.data
          
          set({
            user,
            token: access_token,
            isLoading: false,
            error: null,
          })
          
          // Set authorization header for future requests
          api.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          
        } catch (error: any) {
          const errorMessage = error.response?.data?.message || 'Registration failed'
          set({
            isLoading: false,
            error: errorMessage,
            user: null,
            token: null,
          })
          throw new Error(errorMessage)
        }
      },

      logout: () => {
        // Call logout API
        api.post('/auth/logout').catch(() => {
          // Ignore errors during logout
        })
        
        // Clear auth header
        delete api.defaults.headers.common['Authorization']
        
        set({
          user: null,
          token: null,
          error: null,
          isLoading: false,
        })
      },

      clearError: () => {
        set({ error: null })
      },

      setLoading: (loading: boolean) => {
        set({ isLoading: loading })
      },

      initializeAuth: () => {
        const { token } = get()
        if (token) {
          // Set authorization header
          api.defaults.headers.common['Authorization'] = `Bearer ${token}`
          
          // Verify token is still valid by fetching user profile
          api.get('/auth/me')
            .then((response) => {
              set({ user: response.data })
            })
            .catch(() => {
              // Token is invalid, clear auth state
              get().logout()
            })
        }
      },

      refreshToken: async () => {
        const { token } = get()
        if (!token) {
          throw new Error('No refresh token available')
        }
        
        try {
          const response = await api.post<AuthResponse>('/auth/refresh', {
            refresh_token: token,
          })
          
          const { access_token, user } = response.data
          
          set({
            user,
            token: access_token,
          })
          
          // Update authorization header
          api.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          
        } catch (error) {
          // Refresh failed, logout user
          get().logout()
          throw error
        }
      },
    }),
    {
      name: 'datamesh-auth',
      partialize: (state) => ({
        token: state.token,
        user: state.user,
      }),
    }
  )
)