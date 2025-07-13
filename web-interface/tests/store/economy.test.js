import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEconomyStore } from '../../src/store/economy.js'

// Mock axios
vi.mock('axios', () => ({
  default: {
    create: vi.fn(() => ({
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
    })),
  },
}))

// Mock API
vi.mock('../../src/services/api.js', () => ({
  economyAPI: {
    getStatus: vi.fn(),
    getProfile: vi.fn(),
    getQuotaStatus: vi.fn(),
    contribute: vi.fn(),
    verify: vi.fn(),
    getTiers: vi.fn(),
    upgrade: vi.fn(),
    getRewards: vi.fn(),
    getVerificationHistory: vi.fn(),
    getNetworkStats: vi.fn(),
  },
}))

describe('Economy Store', () => {
  let store

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useEconomyStore()
  })

  it('initializes with default state', () => {
    expect(store.economyStatus.health).toBe('unknown')
    expect(store.economyStatus.total_contributors).toBe(0)
    expect(store.userProfile.tier).toBe('Free')
    expect(store.userProfile.reputation_score).toBe(0)
    expect(store.loading).toBe(false)
    expect(store.error).toBe(null)
  })

  it('calculates upload quota percentage correctly', () => {
    store.userProfile.upload_quota_used = 500 * 1024 * 1024 // 500MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB

    const percentage = store.getUploadQuotaPercentage()
    expect(percentage).toBeCloseTo(48.83, 1)
  })

  it('calculates download quota percentage correctly', () => {
    store.userProfile.download_quota_used = 2 * 1024 * 1024 * 1024 // 2GB
    store.userProfile.download_quota_limit = 10 * 1024 * 1024 * 1024 // 10GB

    const percentage = store.getDownloadQuotaPercentage()
    expect(percentage).toBeCloseTo(20, 1)
  })

  it('identifies low quota correctly', () => {
    // Test high quota usage (should be considered low)
    store.userProfile.upload_quota_used = 900 * 1024 * 1024 // 900MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB
    expect(store.getIsQuotaLow()).toBe(true)

    // Test normal quota usage
    store.userProfile.upload_quota_used = 200 * 1024 * 1024 // 200MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB
    expect(store.getIsQuotaLow()).toBe(false)
  })

  it('identifies critical quota correctly', () => {
    // Test critical quota usage
    store.userProfile.upload_quota_used = 950 * 1024 * 1024 // 950MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB
    expect(store.getIsQuotaCritical()).toBe(true)

    // Test normal quota usage
    store.userProfile.upload_quota_used = 500 * 1024 * 1024 // 500MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB
    expect(store.getIsQuotaCritical()).toBe(false)
  })

  it('calculates remaining upload quota', () => {
    store.userProfile.upload_quota_used = 300 * 1024 * 1024 // 300MB
    store.userProfile.upload_quota_limit = 1024 * 1024 * 1024 // 1GB

    const remaining = store.getRemainingUploadQuota()
    expect(remaining).toBe(724 * 1024 * 1024) // 724MB
  })

  it('calculates remaining download quota', () => {
    store.userProfile.download_quota_used = 3 * 1024 * 1024 * 1024 // 3GB
    store.userProfile.download_quota_limit = 10 * 1024 * 1024 * 1024 // 10GB

    const remaining = store.getRemainingDownloadQuota()
    expect(remaining).toBe(7 * 1024 * 1024 * 1024) // 7GB
  })

  it('determines if user can contribute', () => {
    // Free tier - cannot contribute
    store.userProfile.tier = 'Free'
    expect(store.getCanContribute()).toBe(false)

    // Contributor tier - can contribute
    store.userProfile.tier = 'Contributor'
    expect(store.getCanContribute()).toBe(true)

    // Premium tier - can contribute
    store.userProfile.tier = 'Premium'
    expect(store.getCanContribute()).toBe(true)
  })

  it('determines if user can upgrade', () => {
    // Free tier - can upgrade
    store.userProfile.tier = 'Free'
    expect(store.getCanUpgrade()).toBe(true)

    // Contributor tier - can upgrade
    store.userProfile.tier = 'Contributor'
    expect(store.getCanUpgrade()).toBe(true)

    // Enterprise tier - cannot upgrade further
    store.userProfile.tier = 'Enterprise'
    expect(store.getCanUpgrade()).toBe(false)
  })

  it('calculates contribution ratio correctly', () => {
    store.userProfile.storage_contributed = 4 * 1024 * 1024 * 1024 // 4GB
    
    const ratio = store.getContributionRatio()
    expect(ratio).toBe(4) // 4:1 ratio means 4GB contribution gives 1GB storage
  })

  it('gets tier display name', () => {
    expect(store.getTierDisplayName('Free')).toBe('Free Tier')
    expect(store.getTierDisplayName('Contributor')).toBe('Contributor')
    expect(store.getTierDisplayName('Premium')).toBe('Premium')
    expect(store.getTierDisplayName('Enterprise')).toBe('Enterprise')
  })

  it('gets tier color correctly', () => {
    expect(store.getTierColor('Free')).toBe('info')
    expect(store.getTierColor('Contributor')).toBe('success')
    expect(store.getTierColor('Premium')).toBe('warning')
    expect(store.getTierColor('Enterprise')).toBe('danger')
  })

  it('calculates next quota reset date', () => {
    const resetDate = store.getNextQuotaReset()
    expect(resetDate).toBeInstanceOf(Date)
    expect(resetDate.getTime()).toBeGreaterThan(Date.now())
  })

  it('checks if verification is required', () => {
    // Contributor tier - verification required
    store.userProfile.tier = 'Contributor'
    expect(store.getIsVerificationRequired()).toBe(true)

    // Premium tier - no verification required
    store.userProfile.tier = 'Premium'
    expect(store.getIsVerificationRequired()).toBe(false)

    // Free tier - no verification required
    store.userProfile.tier = 'Free'
    expect(store.getIsVerificationRequired()).toBe(false)
  })

  it('handles loading states correctly', () => {
    expect(store.loading).toBe(false)
    
    // Simulate loading
    store.setLoading(true)
    expect(store.loading).toBe(true)
    
    store.setLoading(false)
    expect(store.loading).toBe(false)
  })

  it('handles error states correctly', () => {
    expect(store.error).toBe(null)
    
    const testError = 'Test error message'
    store.setError(testError)
    expect(store.error).toBe(testError)
    
    store.clearError()
    expect(store.error).toBe(null)
  })

  it('updates economy status', () => {
    const newStatus = {
      health: 'healthy',
      total_contributors: 500,
      total_storage_contributed: 1024 * 1024 * 1024 * 1024, // 1TB
      active_verifications: 25,
      network_utilization: 0.65
    }

    store.updateEconomyStatus(newStatus)
    expect(store.economyStatus).toEqual(newStatus)
  })

  it('updates user profile', () => {
    const newProfile = {
      user_id: 'new-user-456',
      tier: 'Premium',
      storage_contributed: 0,
      upload_quota_used: 100 * 1024 * 1024, // 100MB
      upload_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
      download_quota_used: 500 * 1024 * 1024, // 500MB
      download_quota_limit: 100 * 1024 * 1024 * 1024, // 100GB
      reputation_score: 95,
      verification_streak: 0,
      last_activity: new Date().toISOString()
    }

    store.updateUserProfile(newProfile)
    expect(store.userProfile).toEqual(newProfile)
  })

  it('updates quota status', () => {
    const newQuotaStatus = {
      upload_quota: {
        used: 200 * 1024 * 1024, // 200MB
        limit: 2 * 1024 * 1024 * 1024, // 2GB
        percentage: 10
      },
      download_quota: {
        used: 1 * 1024 * 1024 * 1024, // 1GB
        limit: 20 * 1024 * 1024 * 1024, // 20GB
        percentage: 5
      },
      storage_quota: {
        used: 500 * 1024 * 1024, // 500MB
        limit: 5 * 1024 * 1024 * 1024, // 5GB
        percentage: 10
      }
    }

    store.updateQuotaStatus(newQuotaStatus)
    expect(store.quotaStatus).toEqual(newQuotaStatus)
  })

  it('resets store to initial state', () => {
    // Modify store state
    store.economyStatus.health = 'healthy'
    store.userProfile.tier = 'Premium'
    store.loading = true
    store.error = 'Some error'

    // Reset store
    store.resetStore()

    // Check that state is back to initial values
    expect(store.economyStatus.health).toBe('unknown')
    expect(store.userProfile.tier).toBe('Free')
    expect(store.loading).toBe(false)
    expect(store.error).toBe(null)
  })
})