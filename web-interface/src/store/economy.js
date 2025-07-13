import { defineStore } from 'pinia'
import { economyAPI } from '../services/api'

export const useEconomyStore = defineStore('economy', {
  state: () => ({
    // Economy status
    economyStatus: {
      health: 'unknown',
      total_contributors: 0,
      total_storage_contributed: 0,
      active_verifications: 0,
      network_utilization: 0
    },
    
    // User economy profile
    userProfile: {
      user_id: '',
      tier: 'Free',
      current_usage: 0,
      max_storage: 0,
      upload_quota_used: 0,
      upload_quota_limit: 0,
      download_quota_used: 0,
      download_quota_limit: 0,
      reputation_score: 0,
      violations_count: 0,
      last_activity: null,
      can_contribute: false
    },
    
    // Storage tiers
    storageTiers: [],
    
    // Contribution status
    contributionStatus: {
      active: false,
      contributed_amount: 0,
      verified_amount: 0,
      last_verification: null,
      status: 'inactive'
    },
    
    // Verification status
    verificationStatus: {
      verification_active: false,
      last_challenge: null,
      success_rate: 0,
      pending_challenges: 0,
      reputation_score: 0
    },
    
    // Transactions
    transactions: [],
    
    // Quota status
    quotaStatus: {
      storage_used: 0,
      storage_limit: 0,
      upload_quota_used: 0,
      upload_quota_limit: 0,
      download_quota_used: 0,
      download_quota_limit: 0,
      next_reset: null
    },
    
    // Loading states
    loading: {
      economyStatus: false,
      userProfile: false,
      storageTiers: false,
      contributionStatus: false,
      verificationStatus: false,
      transactions: false,
      quotaStatus: false,
      upgrading: false,
      contributing: false
    },
    
    // Error states
    errors: {
      economyStatus: null,
      userProfile: null,
      storageTiers: null,
      contributionStatus: null,
      verificationStatus: null,
      transactions: null,
      quotaStatus: null,
      upgrading: null,
      contributing: null
    }
  }),

  getters: {
    // Calculate storage usage percentage
    storageUsagePercentage: (state) => {
      if (state.userProfile.max_storage === 0) return 0
      return Math.round((state.userProfile.current_usage / state.userProfile.max_storage) * 100)
    },
    
    // Calculate upload quota percentage
    uploadQuotaPercentage: (state) => {
      if (state.userProfile.upload_quota_limit === 0) return 0
      return Math.round((state.userProfile.upload_quota_used / state.userProfile.upload_quota_limit) * 100)
    },
    
    // Calculate download quota percentage
    downloadQuotaPercentage: (state) => {
      if (state.userProfile.download_quota_limit === 0) return 0
      return Math.round((state.userProfile.download_quota_used / state.userProfile.download_quota_limit) * 100)
    },
    
    // Check if user can upgrade tier
    canUpgradeTier: (state) => {
      const currentTierIndex = state.storageTiers.findIndex(tier => tier.name === state.userProfile.tier)
      return currentTierIndex !== -1 && currentTierIndex < state.storageTiers.length - 1
    },
    
    // Get next available tier
    nextTier: (state) => {
      const currentTierIndex = state.storageTiers.findIndex(tier => tier.name === state.userProfile.tier)
      if (currentTierIndex === -1 || currentTierIndex >= state.storageTiers.length - 1) return null
      return state.storageTiers[currentTierIndex + 1]
    },
    
    // Format storage size
    formatStorageSize: () => (bytes) => {
      if (bytes === 0) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    },
    
    // Get tier color based on tier name
    getTierColor: () => (tierName) => {
      const colors = {
        'Free': '#9CA3AF',
        'Contributor': '#10B981', 
        'Premium': '#3B82F6',
        'Enterprise': '#8B5CF6'
      }
      return colors[tierName] || '#9CA3AF'
    },
    
    // Check if storage is running low
    isStorageLow: (state) => {
      return state.storageUsagePercentage > 85
    },
    
    // Check if quota is running low
    isQuotaLow: (state) => {
      return state.uploadQuotaPercentage > 85 || state.downloadQuotaPercentage > 85
    }
  },

  actions: {
    // Fetch economy status
    async fetchEconomyStatus() {
      this.loading.economyStatus = true
      this.errors.economyStatus = null
      
      try {
        const response = await economyAPI.getStatus()
        this.economyStatus = response.data
      } catch (error) {
        this.errors.economyStatus = error.message
        throw error
      } finally {
        this.loading.economyStatus = false
      }
    },
    
    // Fetch user economy profile
    async fetchUserProfile() {
      this.loading.userProfile = true
      this.errors.userProfile = null
      
      try {
        const response = await economyAPI.getProfile()
        this.userProfile = response.data
      } catch (error) {
        this.errors.userProfile = error.message
        throw error
      } finally {
        this.loading.userProfile = false
      }
    },
    
    // Fetch storage tiers
    async fetchStorageTiers() {
      this.loading.storageTiers = true
      this.errors.storageTiers = null
      
      try {
        const response = await economyAPI.getTiers()
        this.storageTiers = response.data.tiers
      } catch (error) {
        this.errors.storageTiers = error.message
        throw error
      } finally {
        this.loading.storageTiers = false
      }
    },
    
    // Fetch contribution status
    async fetchContributionStatus() {
      this.loading.contributionStatus = true
      this.errors.contributionStatus = null
      
      try {
        const response = await economyAPI.getContributionStatus()
        this.contributionStatus = response.data
      } catch (error) {
        this.errors.contributionStatus = error.message
        throw error
      } finally {
        this.loading.contributionStatus = false
      }
    },
    
    // Fetch verification status
    async fetchVerificationStatus() {
      this.loading.verificationStatus = true
      this.errors.verificationStatus = null
      
      try {
        const response = await economyAPI.getVerificationStatus()
        this.verificationStatus = response.data
      } catch (error) {
        this.errors.verificationStatus = error.message
        throw error
      } finally {
        this.loading.verificationStatus = false
      }
    },
    
    // Fetch transactions
    async fetchTransactions(params = {}) {
      this.loading.transactions = true
      this.errors.transactions = null
      
      try {
        const response = await economyAPI.getTransactions(params)
        this.transactions = response.data
      } catch (error) {
        this.errors.transactions = error.message
        throw error
      } finally {
        this.loading.transactions = false
      }
    },
    
    // Fetch quota status
    async fetchQuotaStatus() {
      this.loading.quotaStatus = true
      this.errors.quotaStatus = null
      
      try {
        const response = await economyAPI.getQuotaStatus()
        this.quotaStatus = response.data
      } catch (error) {
        this.errors.quotaStatus = error.message
        throw error
      } finally {
        this.loading.quotaStatus = false
      }
    },
    
    // Start storage contribution
    async startContribution(contributionData) {
      this.loading.contributing = true
      this.errors.contributing = null
      
      try {
        const response = await economyAPI.startContribution(contributionData)
        await this.fetchContributionStatus()
        await this.fetchUserProfile()
        return response.data
      } catch (error) {
        this.errors.contributing = error.message
        throw error
      } finally {
        this.loading.contributing = false
      }
    },
    
    // Stop storage contribution
    async stopContribution() {
      this.loading.contributing = true
      this.errors.contributing = null
      
      try {
        const response = await economyAPI.stopContribution()
        await this.fetchContributionStatus()
        await this.fetchUserProfile()
        return response.data
      } catch (error) {
        this.errors.contributing = error.message
        throw error
      } finally {
        this.loading.contributing = false
      }
    },
    
    // Upgrade storage tier
    async upgradeTier(upgradeData) {
      this.loading.upgrading = true
      this.errors.upgrading = null
      
      try {
        const response = await economyAPI.upgradeTier(upgradeData)
        await this.fetchUserProfile()
        await this.fetchQuotaStatus()
        return response.data
      } catch (error) {
        this.errors.upgrading = error.message
        throw error
      } finally {
        this.loading.upgrading = false
      }
    },
    
    // Respond to verification challenge
    async respondToChallenge(challengeData) {
      try {
        const response = await economyAPI.respondToChallenge(challengeData)
        await this.fetchVerificationStatus()
        await this.fetchUserProfile()
        return response.data
      } catch (error) {
        throw error
      }
    },
    
    // Initialize all economy data
    async initializeEconomyData() {
      try {
        await Promise.allSettled([
          this.fetchEconomyStatus(),
          this.fetchUserProfile(),
          this.fetchStorageTiers(),
          this.fetchContributionStatus(),
          this.fetchVerificationStatus(),
          this.fetchQuotaStatus(),
          this.fetchTransactions({ limit: 10 })
        ])
      } catch (error) {
        console.error('Failed to initialize economy data:', error)
      }
    },
    
    // Refresh all data
    async refreshData() {
      await this.initializeEconomyData()
    },
    
    // Clear errors
    clearErrors() {
      Object.keys(this.errors).forEach(key => {
        this.errors[key] = null
      })
    },
    
    // Reset state
    resetState() {
      this.$reset()
    }
  }
})