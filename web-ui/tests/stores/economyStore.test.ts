import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useEconomyStore } from '../../src/stores/economyStore'

// Mock API utilities
vi.mock('../../src/utils/api', () => ({
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
  beforeEach(() => {
    // Reset Zustand store before each test
    useEconomyStore.setState({
      economyStatus: {
        health: 'unknown',
        total_contributors: 0,
        total_storage_contributed: 0,
        active_verifications: 0,
        network_utilization: 0,
      },
      userProfile: null,
      quotaStatus: null,
      loading: false,
      error: null,
    })
  })

  it('initializes with default state', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    expect(result.current.economyStatus.health).toBe('unknown')
    expect(result.current.economyStatus.total_contributors).toBe(0)
    expect(result.current.userProfile).toBe(null)
    expect(result.current.loading).toBe(false)
    expect(result.current.error).toBe(null)
  })

  it('updates economy status', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const newStatus = {
      health: 'healthy',
      total_contributors: 500,
      total_storage_contributed: 1024 * 1024 * 1024 * 1024, // 1TB
      active_verifications: 25,
      network_utilization: 0.65,
    }

    act(() => {
      result.current.setEconomyStatus(newStatus)
    })

    expect(result.current.economyStatus).toEqual(newStatus)
  })

  it('updates user profile', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const userProfile = {
      user_id: 'test-user-123',
      tier: 'Premium' as const,
      storage_contributed: 0,
      upload_quota_used: 100 * 1024 * 1024, // 100MB
      upload_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
      download_quota_used: 500 * 1024 * 1024, // 500MB
      download_quota_limit: 100 * 1024 * 1024 * 1024, // 100GB
      reputation_score: 95,
      verification_streak: 0,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(userProfile)
    })

    expect(result.current.userProfile).toEqual(userProfile)
  })

  it('calculates upload quota percentage correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const userProfile = {
      user_id: 'test-user',
      tier: 'Free' as const,
      storage_contributed: 0,
      upload_quota_used: 50 * 1024 * 1024, // 50MB
      upload_quota_limit: 100 * 1024 * 1024, // 100MB
      download_quota_used: 0,
      download_quota_limit: 1024 * 1024 * 1024, // 1GB
      reputation_score: 0,
      verification_streak: 0,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(userProfile)
    })

    expect(result.current.getUploadQuotaPercentage()).toBe(50)
  })

  it('calculates download quota percentage correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const userProfile = {
      user_id: 'test-user',
      tier: 'Contributor' as const,
      storage_contributed: 4 * 1024 * 1024 * 1024, // 4GB
      upload_quota_used: 0,
      upload_quota_limit: 4 * 1024 * 1024 * 1024, // 4GB
      download_quota_used: 2 * 1024 * 1024 * 1024, // 2GB
      download_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
      reputation_score: 85,
      verification_streak: 12,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(userProfile)
    })

    expect(result.current.getDownloadQuotaPercentage()).toBe(20)
  })

  it('identifies low quota correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    // Test high quota usage (should be considered low)
    const highUsageProfile = {
      user_id: 'test-user',
      tier: 'Free' as const,
      storage_contributed: 0,
      upload_quota_used: 90 * 1024 * 1024, // 90MB
      upload_quota_limit: 100 * 1024 * 1024, // 100MB
      download_quota_used: 0,
      download_quota_limit: 1024 * 1024 * 1024, // 1GB
      reputation_score: 0,
      verification_streak: 0,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(highUsageProfile)
    })

    expect(result.current.getIsQuotaLow()).toBe(true)

    // Test normal quota usage
    const normalUsageProfile = {
      ...highUsageProfile,
      upload_quota_used: 20 * 1024 * 1024, // 20MB
    }

    act(() => {
      result.current.setUserProfile(normalUsageProfile)
    })

    expect(result.current.getIsQuotaLow()).toBe(false)
  })

  it('identifies critical quota correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    // Test critical quota usage
    const criticalUsageProfile = {
      user_id: 'test-user',
      tier: 'Free' as const,
      storage_contributed: 0,
      upload_quota_used: 95 * 1024 * 1024, // 95MB
      upload_quota_limit: 100 * 1024 * 1024, // 100MB
      download_quota_used: 0,
      download_quota_limit: 1024 * 1024 * 1024, // 1GB
      reputation_score: 0,
      verification_streak: 0,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(criticalUsageProfile)
    })

    expect(result.current.getIsQuotaCritical()).toBe(true)
  })

  it('calculates remaining quotas correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const userProfile = {
      user_id: 'test-user',
      tier: 'Contributor' as const,
      storage_contributed: 4 * 1024 * 1024 * 1024, // 4GB
      upload_quota_used: 1 * 1024 * 1024 * 1024, // 1GB
      upload_quota_limit: 4 * 1024 * 1024 * 1024, // 4GB
      download_quota_used: 3 * 1024 * 1024 * 1024, // 3GB
      download_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
      reputation_score: 85,
      verification_streak: 12,
      last_activity: '2024-01-15T10:30:00Z',
    }

    act(() => {
      result.current.setUserProfile(userProfile)
    })

    expect(result.current.getRemainingUploadQuota()).toBe(3 * 1024 * 1024 * 1024) // 3GB
    expect(result.current.getRemainingDownloadQuota()).toBe(7 * 1024 * 1024 * 1024) // 7GB
  })

  it('determines tier capabilities correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    // Free tier
    act(() => {
      result.current.setUserProfile({
        user_id: 'test-user',
        tier: 'Free',
        storage_contributed: 0,
        upload_quota_used: 0,
        upload_quota_limit: 100 * 1024 * 1024, // 100MB
        download_quota_used: 0,
        download_quota_limit: 1024 * 1024 * 1024, // 1GB
        reputation_score: 0,
        verification_streak: 0,
        last_activity: '2024-01-15T10:30:00Z',
      })
    })

    expect(result.current.getCanContribute()).toBe(false)
    expect(result.current.getCanUpgrade()).toBe(true)
    expect(result.current.getIsVerificationRequired()).toBe(false)

    // Contributor tier
    act(() => {
      result.current.setUserProfile({
        user_id: 'test-user',
        tier: 'Contributor',
        storage_contributed: 4 * 1024 * 1024 * 1024, // 4GB
        upload_quota_used: 0,
        upload_quota_limit: 4 * 1024 * 1024 * 1024, // 4GB
        download_quota_used: 0,
        download_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
        reputation_score: 85,
        verification_streak: 12,
        last_activity: '2024-01-15T10:30:00Z',
      })
    })

    expect(result.current.getCanContribute()).toBe(true)
    expect(result.current.getCanUpgrade()).toBe(true)
    expect(result.current.getIsVerificationRequired()).toBe(true)

    // Premium tier
    act(() => {
      result.current.setUserProfile({
        user_id: 'test-user',
        tier: 'Premium',
        storage_contributed: 0,
        upload_quota_used: 0,
        upload_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
        download_quota_used: 0,
        download_quota_limit: 100 * 1024 * 1024 * 1024, // 100GB
        reputation_score: 95,
        verification_streak: 0,
        last_activity: '2024-01-15T10:30:00Z',
      })
    })

    expect(result.current.getCanContribute()).toBe(true)
    expect(result.current.getCanUpgrade()).toBe(true)
    expect(result.current.getIsVerificationRequired()).toBe(false)

    // Enterprise tier
    act(() => {
      result.current.setUserProfile({
        user_id: 'test-user',
        tier: 'Enterprise',
        storage_contributed: 0,
        upload_quota_used: 0,
        upload_quota_limit: 100 * 1024 * 1024 * 1024, // 100GB
        download_quota_used: 0,
        download_quota_limit: 1024 * 1024 * 1024 * 1024, // 1TB
        reputation_score: 100,
        verification_streak: 0,
        last_activity: '2024-01-15T10:30:00Z',
      })
    })

    expect(result.current.getCanContribute()).toBe(true)
    expect(result.current.getCanUpgrade()).toBe(false)
    expect(result.current.getIsVerificationRequired()).toBe(false)
  })

  it('handles loading states', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    act(() => {
      result.current.setLoading(true)
    })
    
    expect(result.current.loading).toBe(true)
    
    act(() => {
      result.current.setLoading(false)
    })
    
    expect(result.current.loading).toBe(false)
  })

  it('handles error states', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const errorMessage = 'Test error message'
    
    act(() => {
      result.current.setError(errorMessage)
    })
    
    expect(result.current.error).toBe(errorMessage)
    
    act(() => {
      result.current.setError(null)
    })
    
    expect(result.current.error).toBe(null)
  })

  it('formats tier display names correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    expect(result.current.getTierDisplayName('Free')).toBe('Free Tier')
    expect(result.current.getTierDisplayName('Contributor')).toBe('Contributor')
    expect(result.current.getTierDisplayName('Premium')).toBe('Premium')
    expect(result.current.getTierDisplayName('Enterprise')).toBe('Enterprise')
  })

  it('gets tier colors correctly', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    expect(result.current.getTierColor('Free')).toBe('gray')
    expect(result.current.getTierColor('Contributor')).toBe('green')
    expect(result.current.getTierColor('Premium')).toBe('blue')
    expect(result.current.getTierColor('Enterprise')).toBe('purple')
  })

  it('calculates contribution ratio', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    act(() => {
      result.current.setUserProfile({
        user_id: 'test-user',
        tier: 'Contributor',
        storage_contributed: 4 * 1024 * 1024 * 1024, // 4GB
        upload_quota_used: 0,
        upload_quota_limit: 4 * 1024 * 1024 * 1024, // 4GB
        download_quota_used: 0,
        download_quota_limit: 10 * 1024 * 1024 * 1024, // 10GB
        reputation_score: 85,
        verification_streak: 12,
        last_activity: '2024-01-15T10:30:00Z',
      })
    })

    expect(result.current.getContributionRatio()).toBe(4)
  })

  it('handles quota status updates', () => {
    const { result } = renderHook(() => useEconomyStore())
    
    const quotaStatus = {
      upload_quota: {
        used: 500 * 1024 * 1024, // 500MB
        limit: 4 * 1024 * 1024 * 1024, // 4GB
        percentage: 12.2,
      },
      download_quota: {
        used: 1 * 1024 * 1024 * 1024, // 1GB
        limit: 10 * 1024 * 1024 * 1024, // 10GB
        percentage: 10.0,
      },
      storage_quota: {
        used: 2 * 1024 * 1024 * 1024, // 2GB
        limit: 5 * 1024 * 1024 * 1024, // 5GB
        percentage: 40.0,
      },
    }

    act(() => {
      result.current.setQuotaStatus(quotaStatus)
    })

    expect(result.current.quotaStatus).toEqual(quotaStatus)
  })
})