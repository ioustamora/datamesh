import { create } from 'zustand'
import { economyAPI, formatFileSize } from '../utils/api'
import toast from 'react-hot-toast'

// Types
export interface StorageTier {
  name: string
  max_storage: number
  upload_quota: number
  download_quota: number
  monthly_cost?: number
  description: string
}

export interface UserEconomyProfile {
  user_id: string
  tier: string
  current_usage: number
  max_storage: number
  upload_quota_used: number
  upload_quota_limit: number
  download_quota_used: number
  download_quota_limit: number
  reputation_score: number
  violations_count: number
  last_activity: string
  can_contribute: boolean
}

export interface EconomyStatus {
  health: string
  total_contributors: number
  total_storage_contributed: number
  active_verifications: number
  network_utilization: number
}

export interface ContributionStatus {
  active: boolean
  contributed_amount: number
  verified_amount: number
  last_verification: string | null
  status: string
}

export interface VerificationStatus {
  verification_active: boolean
  last_challenge: string | null
  success_rate: number
  pending_challenges: number
  reputation_score: number
}

export interface EconomyTransaction {
  transaction_id: string
  transaction_type: string
  amount: number
  description: string
  timestamp: string
  status: string
}

export interface QuotaStatus {
  storage_used: number
  storage_limit: number
  upload_quota_used: number
  upload_quota_limit: number
  download_quota_used: number
  download_quota_limit: number
  next_reset: string
}

// Store interface
interface EconomyStore {
  // State
  economyStatus: EconomyStatus | null
  userProfile: UserEconomyProfile | null
  storageTiers: StorageTier[]
  contributionStatus: ContributionStatus | null
  verificationStatus: VerificationStatus | null
  transactions: EconomyTransaction[]
  quotaStatus: QuotaStatus | null
  
  // Loading states
  loading: {
    economyStatus: boolean
    userProfile: boolean
    storageTiers: boolean
    contributionStatus: boolean
    verificationStatus: boolean
    transactions: boolean
    quotaStatus: boolean
    upgrading: boolean
    contributing: boolean
  }
  
  // Error states
  errors: {
    economyStatus: string | null
    userProfile: string | null
    storageTiers: string | null
    contributionStatus: string | null
    verificationStatus: string | null
    transactions: string | null
    quotaStatus: string | null
    upgrading: string | null
    contributing: string | null
  }
  
  // Actions
  fetchEconomyStatus: () => Promise<void>
  fetchUserProfile: () => Promise<void>
  fetchStorageTiers: () => Promise<void>
  fetchContributionStatus: () => Promise<void>
  fetchVerificationStatus: () => Promise<void>
  fetchTransactions: (params?: any) => Promise<void>
  fetchQuotaStatus: () => Promise<void>
  startContribution: (contributionData: any) => Promise<any>
  stopContribution: () => Promise<any>
  upgradeTier: (upgradeData: any) => Promise<any>
  respondToChallenge: (challengeData: any) => Promise<any>
  initializeEconomyData: () => Promise<void>
  refreshData: () => Promise<void>
  clearErrors: () => void
  resetState: () => void
  
  // Computed getters
  getStorageUsagePercentage: () => number
  getUploadQuotaPercentage: () => number
  getDownloadQuotaPercentage: () => number
  getCanUpgradeTier: () => boolean
  getNextTier: () => StorageTier | null
  getTierColor: (tierName: string) => string
  getIsStorageLow: () => boolean
  getIsQuotaLow: () => boolean
}

export const useEconomyStore = create<EconomyStore>((set, get) => ({
  // Initial state
  economyStatus: null,
  userProfile: null,
  storageTiers: [],
  contributionStatus: null,
  verificationStatus: null,
  transactions: [],
  quotaStatus: null,
  
  loading: {
    economyStatus: false,
    userProfile: false,
    storageTiers: false,
    contributionStatus: false,
    verificationStatus: false,
    transactions: false,
    quotaStatus: false,
    upgrading: false,
    contributing: false,
  },
  
  errors: {
    economyStatus: null,
    userProfile: null,
    storageTiers: null,
    contributionStatus: null,
    verificationStatus: null,
    transactions: null,
    quotaStatus: null,
    upgrading: null,
    contributing: null,
  },
  
  // Actions
  fetchEconomyStatus: async () => {
    set((state) => ({
      loading: { ...state.loading, economyStatus: true },
      errors: { ...state.errors, economyStatus: null },
    }))
    
    try {
      const response = await economyAPI.getStatus()
      set({ economyStatus: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, economyStatus: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, economyStatus: false },
      }))
    }
  },
  
  fetchUserProfile: async () => {
    set((state) => ({
      loading: { ...state.loading, userProfile: true },
      errors: { ...state.errors, userProfile: null },
    }))
    
    try {
      const response = await economyAPI.getProfile()
      set({ userProfile: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, userProfile: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, userProfile: false },
      }))
    }
  },
  
  fetchStorageTiers: async () => {
    set((state) => ({
      loading: { ...state.loading, storageTiers: true },
      errors: { ...state.errors, storageTiers: null },
    }))
    
    try {
      const response = await economyAPI.getTiers()
      set({ storageTiers: response.data.tiers })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, storageTiers: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, storageTiers: false },
      }))
    }
  },
  
  fetchContributionStatus: async () => {
    set((state) => ({
      loading: { ...state.loading, contributionStatus: true },
      errors: { ...state.errors, contributionStatus: null },
    }))
    
    try {
      const response = await economyAPI.getContributionStatus()
      set({ contributionStatus: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, contributionStatus: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, contributionStatus: false },
      }))
    }
  },
  
  fetchVerificationStatus: async () => {
    set((state) => ({
      loading: { ...state.loading, verificationStatus: true },
      errors: { ...state.errors, verificationStatus: null },
    }))
    
    try {
      const response = await economyAPI.getVerificationStatus()
      set({ verificationStatus: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, verificationStatus: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, verificationStatus: false },
      }))
    }
  },
  
  fetchTransactions: async (params = {}) => {
    set((state) => ({
      loading: { ...state.loading, transactions: true },
      errors: { ...state.errors, transactions: null },
    }))
    
    try {
      const response = await economyAPI.getTransactions(params)
      set({ transactions: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, transactions: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, transactions: false },
      }))
    }
  },
  
  fetchQuotaStatus: async () => {
    set((state) => ({
      loading: { ...state.loading, quotaStatus: true },
      errors: { ...state.errors, quotaStatus: null },
    }))
    
    try {
      const response = await economyAPI.getQuotaStatus()
      set({ quotaStatus: response.data })
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, quotaStatus: errorMessage },
      }))
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, quotaStatus: false },
      }))
    }
  },
  
  startContribution: async (contributionData) => {
    set((state) => ({
      loading: { ...state.loading, contributing: true },
      errors: { ...state.errors, contributing: null },
    }))
    
    try {
      const response = await economyAPI.startContribution(contributionData)
      // Refresh related data
      await Promise.all([
        get().fetchContributionStatus(),
        get().fetchUserProfile(),
      ])
      toast.success('Storage contribution started successfully!')
      return response.data
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, contributing: errorMessage },
      }))
      toast.error('Failed to start contribution: ' + errorMessage)
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, contributing: false },
      }))
    }
  },
  
  stopContribution: async () => {
    set((state) => ({
      loading: { ...state.loading, contributing: true },
      errors: { ...state.errors, contributing: null },
    }))
    
    try {
      const response = await economyAPI.stopContribution()
      // Refresh related data
      await Promise.all([
        get().fetchContributionStatus(),
        get().fetchUserProfile(),
      ])
      toast.success('Storage contribution stopped successfully!')
      return response.data
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, contributing: errorMessage },
      }))
      toast.error('Failed to stop contribution: ' + errorMessage)
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, contributing: false },
      }))
    }
  },
  
  upgradeTier: async (upgradeData) => {
    set((state) => ({
      loading: { ...state.loading, upgrading: true },
      errors: { ...state.errors, upgrading: null },
    }))
    
    try {
      const response = await economyAPI.upgradeTier(upgradeData)
      // Refresh related data
      await Promise.all([
        get().fetchUserProfile(),
        get().fetchQuotaStatus(),
      ])
      toast.success('Tier upgrade initiated successfully!')
      return response.data
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      set((state) => ({
        errors: { ...state.errors, upgrading: errorMessage },
      }))
      toast.error('Failed to upgrade tier: ' + errorMessage)
      throw error
    } finally {
      set((state) => ({
        loading: { ...state.loading, upgrading: false },
      }))
    }
  },
  
  respondToChallenge: async (challengeData) => {
    try {
      const response = await economyAPI.respondToChallenge(challengeData)
      // Refresh related data
      await Promise.all([
        get().fetchVerificationStatus(),
        get().fetchUserProfile(),
      ])
      toast.success('Challenge response submitted successfully!')
      return response.data
    } catch (error: any) {
      const errorMessage = error.response?.data?.message || error.message
      toast.error('Failed to respond to challenge: ' + errorMessage)
      throw error
    }
  },
  
  initializeEconomyData: async () => {
    try {
      await Promise.allSettled([
        get().fetchEconomyStatus(),
        get().fetchUserProfile(),
        get().fetchStorageTiers(),
        get().fetchContributionStatus(),
        get().fetchVerificationStatus(),
        get().fetchQuotaStatus(),
        get().fetchTransactions({ limit: 10 }),
      ])
    } catch (error) {
      console.error('Failed to initialize economy data:', error)
    }
  },
  
  refreshData: async () => {
    await get().initializeEconomyData()
  },
  
  clearErrors: () => {
    set((state) => ({
      errors: Object.keys(state.errors).reduce((acc, key) => {
        acc[key as keyof typeof state.errors] = null
        return acc
      }, {} as typeof state.errors),
    }))
  },
  
  resetState: () => {
    set({
      economyStatus: null,
      userProfile: null,
      storageTiers: [],
      contributionStatus: null,
      verificationStatus: null,
      transactions: [],
      quotaStatus: null,
      loading: {
        economyStatus: false,
        userProfile: false,
        storageTiers: false,
        contributionStatus: false,
        verificationStatus: false,
        transactions: false,
        quotaStatus: false,
        upgrading: false,
        contributing: false,
      },
      errors: {
        economyStatus: null,
        userProfile: null,
        storageTiers: null,
        contributionStatus: null,
        verificationStatus: null,
        transactions: null,
        quotaStatus: null,
        upgrading: null,
        contributing: null,
      },
    })
  },
  
  // Computed getters
  getStorageUsagePercentage: () => {
    const { userProfile } = get()
    if (!userProfile || userProfile.max_storage === 0) return 0
    return Math.round((userProfile.current_usage / userProfile.max_storage) * 100)
  },
  
  getUploadQuotaPercentage: () => {
    const { userProfile } = get()
    if (!userProfile || userProfile.upload_quota_limit === 0) return 0
    return Math.round((userProfile.upload_quota_used / userProfile.upload_quota_limit) * 100)
  },
  
  getDownloadQuotaPercentage: () => {
    const { userProfile } = get()
    if (!userProfile || userProfile.download_quota_limit === 0) return 0
    return Math.round((userProfile.download_quota_used / userProfile.download_quota_limit) * 100)
  },
  
  getCanUpgradeTier: () => {
    const { userProfile, storageTiers } = get()
    if (!userProfile) return false
    const currentTierIndex = storageTiers.findIndex(tier => tier.name === userProfile.tier)
    return currentTierIndex !== -1 && currentTierIndex < storageTiers.length - 1
  },
  
  getNextTier: () => {
    const { userProfile, storageTiers } = get()
    if (!userProfile) return null
    const currentTierIndex = storageTiers.findIndex(tier => tier.name === userProfile.tier)
    if (currentTierIndex === -1 || currentTierIndex >= storageTiers.length - 1) return null
    return storageTiers[currentTierIndex + 1]
  },
  
  getTierColor: (tierName: string) => {
    const colors: Record<string, string> = {
      'Free': '#9CA3AF',
      'Contributor': '#10B981',
      'Premium': '#3B82F6',
      'Enterprise': '#8B5CF6',
    }
    return colors[tierName] || '#9CA3AF'
  },
  
  getIsStorageLow: () => {
    return get().getStorageUsagePercentage() > 85
  },
  
  getIsQuotaLow: () => {
    const uploadPercentage = get().getUploadQuotaPercentage()
    const downloadPercentage = get().getDownloadQuotaPercentage()
    return uploadPercentage > 85 || downloadPercentage > 85
  },
}))