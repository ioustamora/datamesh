import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'

import { Layout } from '@/components/Layout'
import { Dashboard } from '@/pages/Dashboard'
import { Files } from '@/pages/Files'
import { Upload } from '@/pages/Upload'
import { Network } from '@/pages/Network'
import { Governance } from '@/pages/Governance'
import { Settings } from '@/pages/Settings'
import { Login } from '@/pages/Login'
import { Register } from '@/pages/Register'
import { ProtectedRoute } from '@/components/ProtectedRoute'
import { useAuthStore } from '@/stores/authStore'
import { useWebSocket } from '@/hooks/useWebSocket'
import { useThemeStore } from '@/stores/themeStore'

function App() {
  const { user, initializeAuth } = useAuthStore()
  const { theme } = useThemeStore()
  
  // Initialize WebSocket connection if authenticated
  useWebSocket(user ? `ws://localhost:8080/ws` : null)

  useEffect(() => {
    // Apply theme to document
    document.documentElement.classList.toggle('dark', theme === 'dark')
  }, [theme])

  useEffect(() => {
    // Initialize authentication state from localStorage
    initializeAuth()
  }, [initializeAuth])

  return (
    <div className="min-h-screen bg-secondary-50 dark:bg-secondary-900 transition-colors duration-200">
      <Routes>
        {/* Public routes */}
        <Route 
          path="/login" 
          element={user ? <Navigate to="/dashboard" replace /> : <Login />} 
        />
        <Route 
          path="/register" 
          element={user ? <Navigate to="/dashboard" replace /> : <Register />} 
        />
        
        {/* Protected routes */}
        <Route path="/" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="files" element={<Files />} />
          <Route path="upload" element={<Upload />} />
          <Route path="network" element={<Network />} />
          <Route path="governance" element={<Governance />} />
          <Route path="settings" element={<Settings />} />
        </Route>
        
        {/* Catch all route */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </div>
  )
}

export default App