import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../store/auth'
import { useLoadingStore } from '../store/loading'

// Layout components
import MainLayout from '../components/layout/MainLayout.vue'
import AuthLayout from '../components/layout/AuthLayout.vue'

// View components
import Dashboard from '../views/Dashboard.vue'
import FileManager from '../views/FileManager.vue'
import Governance from '../views/Governance.vue'
import Administration from '../views/Administration.vue'
import Analytics from '../views/Analytics.vue'
import Settings from '../views/Settings.vue'
import Profile from '../views/Profile.vue'
import Login from '../views/auth/Login.vue'
import Register from '../views/auth/Register.vue'
import ForgotPassword from '../views/auth/ForgotPassword.vue'
import NotFound from '../views/errors/NotFound.vue'
import ServerError from '../views/errors/ServerError.vue'

// Governance sub-routes
import GovernanceOverview from '../views/governance/Overview.vue'
import Operators from '../views/governance/Operators.vue'
import Proposals from '../views/governance/Proposals.vue'
import Voting from '../views/governance/Voting.vue'
import NetworkHealth from '../views/governance/NetworkHealth.vue'

// Administration sub-routes
import AdminOverview from '../views/administration/Overview.vue'
import UserManagement from '../views/administration/UserManagement.vue'
import OperatorManagement from '../views/administration/OperatorManagement.vue'
import SystemConfiguration from '../views/administration/SystemConfiguration.vue'
import AuditLogs from '../views/administration/AuditLogs.vue'

// Route definitions
const routes = [
  // Authentication routes
  {
    path: '/auth',
    component: AuthLayout,
    children: [
      {
        path: 'login',
        name: 'Login',
        component: Login,
        meta: { title: 'Login', requiresGuest: true }
      },
      {
        path: 'register',
        name: 'Register',
        component: Register,
        meta: { title: 'Register', requiresGuest: true }
      },
      {
        path: 'forgot-password',
        name: 'ForgotPassword',
        component: ForgotPassword,
        meta: { title: 'Forgot Password', requiresGuest: true }
      }
    ]
  },
  
  // Main application routes
  {
    path: '/',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        name: 'Dashboard',
        component: Dashboard,
        meta: { title: 'Dashboard', icon: 'House' }
      },
      {
        path: 'files',
        name: 'FileManager',
        component: FileManager,
        meta: { title: 'File Manager', icon: 'FolderOpened' }
      },
      {
        path: 'governance',
        name: 'Governance',
        component: Governance,
        meta: { title: 'Governance', icon: 'Flag' },
        children: [
          {
            path: '',
            name: 'GovernanceOverview',
            component: GovernanceOverview,
            meta: { title: 'Governance Overview' }
          },
          {
            path: 'operators',
            name: 'Operators',
            component: Operators,
            meta: { title: 'Operators' }
          },
          {
            path: 'proposals',
            name: 'Proposals',
            component: Proposals,
            meta: { title: 'Proposals' }
          },
          {
            path: 'voting',
            name: 'Voting',
            component: Voting,
            meta: { title: 'Voting' }
          },
          {
            path: 'network-health',
            name: 'NetworkHealth',
            component: NetworkHealth,
            meta: { title: 'Network Health' }
          }
        ]
      },
      {
        path: 'administration',
        name: 'Administration',
        component: Administration,
        meta: { title: 'Administration', icon: 'Setting', requiresAdmin: true },
        children: [
          {
            path: '',
            name: 'AdminOverview',
            component: AdminOverview,
            meta: { title: 'Admin Overview' }
          },
          {
            path: 'users',
            name: 'UserManagement',
            component: UserManagement,
            meta: { title: 'User Management' }
          },
          {
            path: 'operators',
            name: 'OperatorManagement',
            component: OperatorManagement,
            meta: { title: 'Operator Management' }
          },
          {
            path: 'system',
            name: 'SystemConfiguration',
            component: SystemConfiguration,
            meta: { title: 'System Configuration' }
          },
          {
            path: 'audit',
            name: 'AuditLogs',
            component: AuditLogs,
            meta: { title: 'Audit Logs' }
          }
        ]
      },
      {
        path: 'analytics',
        name: 'Analytics',
        component: Analytics,
        meta: { title: 'Analytics', icon: 'TrendCharts' }
      },
      {
        path: 'settings',
        name: 'Settings',
        component: Settings,
        meta: { title: 'Settings', icon: 'Tools' }
      },
      {
        path: 'profile',
        name: 'Profile',
        component: Profile,
        meta: { title: 'Profile', icon: 'User' }
      }
    ]
  },
  
  // Error routes
  {
    path: '/error/500',
    name: 'ServerError',
    component: ServerError,
    meta: { title: 'Server Error' }
  },
  
  // 404 Not Found - must be last
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: NotFound,
    meta: { title: 'Page Not Found' }
  }
]

// Create router instance
const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition
    }
    if (to.hash) {
      return { el: to.hash, behavior: 'smooth' }
    }
    return { top: 0, behavior: 'smooth' }
  }
})

// Global navigation guards
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()
  const loadingStore = useLoadingStore()
  
  // Start loading
  loadingStore.setLoading(true)
  
  // Update document title
  document.title = to.meta.title ? `${to.meta.title} - DataMesh` : 'DataMesh'
  
  // Check authentication
  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/auth/login')
    return
  }
  
  // Check guest routes (redirect authenticated users)
  if (to.meta.requiresGuest && authStore.isAuthenticated) {
    next('/')
    return
  }
  
  // Check admin routes
  if (to.meta.requiresAdmin && !authStore.isAdmin) {
    next('/error/403')
    return
  }
  
  // Initialize auth if needed
  if (!authStore.initialized) {
    try {
      await authStore.checkAuth()
    } catch (error) {
      console.error('Auth check failed:', error)
      if (to.meta.requiresAuth) {
        next('/auth/login')
        return
      }
    }
  }
  
  next()
})

router.afterEach(() => {
  const loadingStore = useLoadingStore()
  // Stop loading after navigation
  setTimeout(() => {
    loadingStore.setLoading(false)
  }, 100)
})

// Error handling
router.onError((error) => {
  console.error('Router error:', error)
  router.push('/error/500')
})

export default router