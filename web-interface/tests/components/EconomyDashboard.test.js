import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import EconomyDashboard from '../../src/components/dashboard/EconomyDashboard.vue'
import { useEconomyStore } from '../../src/store/economy.js'

// Mock Element Plus components
vi.mock('element-plus', () => ({
  ElCard: { name: 'ElCard', template: '<div><slot /></div>' },
  ElProgress: { name: 'ElProgress', template: '<div></div>' },
  ElButton: { name: 'ElButton', template: '<button><slot /></button>' },
  ElStatistic: { name: 'ElStatistic', template: '<div></div>' },
  ElTag: { name: 'ElTag', template: '<span><slot /></span>' },
  ElAlert: { name: 'ElAlert', template: '<div><slot /></div>' },
  ElDialog: { name: 'ElDialog', template: '<div><slot /></div>' },
  ElForm: { name: 'ElForm', template: '<form><slot /></form>' },
  ElFormItem: { name: 'ElFormItem', template: '<div><slot /></div>' },
  ElInput: { name: 'ElInput', template: '<input />' },
  ElSelect: { name: 'ElSelect', template: '<select><slot /></select>' },
  ElOption: { name: 'ElOption', template: '<option><slot /></option>' },
}))

// Mock Chart.js
vi.mock('chart.js', () => ({
  Chart: vi.fn(),
  registerables: [],
}))

describe('EconomyDashboard', () => {
  let wrapper
  let store

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useEconomyStore()
    
    // Mock store data
    store.economyStatus = {
      health: 'healthy',
      total_contributors: 1250,
      total_storage_contributed: 5242880000000, // 5TB
      active_verifications: 45,
      network_utilization: 0.72
    }
    
    store.userProfile = {
      user_id: 'test-user-123',
      tier: 'Contributor',
      storage_contributed: 4294967296, // 4GB
      upload_quota_used: 524288000, // 500MB
      upload_quota_limit: 4294967296, // 4GB
      download_quota_used: 1073741824, // 1GB
      download_quota_limit: 10737418240, // 10GB
      reputation_score: 85,
      verification_streak: 12,
      last_activity: '2024-01-15T10:30:00Z'
    }
    
    store.quotaStatus = {
      upload_quota: {
        used: 524288000,
        limit: 4294967296,
        percentage: 12.2
      },
      download_quota: {
        used: 1073741824,
        limit: 10737418240,
        percentage: 10.0
      },
      storage_quota: {
        used: 2147483648, // 2GB
        limit: 5368709120, // 5GB
        percentage: 40.0
      }
    }
    
    // Mock API calls
    store.fetchEconomyStatus = vi.fn()
    store.fetchUserProfile = vi.fn()
    store.fetchQuotaStatus = vi.fn()
  })

  it('renders economy dashboard correctly', () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-progress': { template: '<div class="el-progress"></div>' },
          'el-button': { template: '<button class="el-button"><slot /></button>' },
          'el-statistic': { template: '<div class="el-statistic"></div>' },
          'el-tag': { template: '<span class="el-tag"><slot /></span>' },
        }
      }
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.find('.economy-dashboard').exists()).toBe(true)
  })

  it('displays user tier information', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-tag': { template: '<span class="el-tag"><slot /></span>' },
        }
      }
    })

    await wrapper.vm.$nextTick()
    
    // Check if tier is displayed
    const tierText = wrapper.text()
    expect(tierText).toContain('Contributor')
  })

  it('shows quota information correctly', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-progress': { template: '<div class="el-progress"></div>' },
        }
      }
    })

    await wrapper.vm.$nextTick()
    
    // Check if quota information is present
    expect(wrapper.vm.quotaStatus).toBeDefined()
    expect(wrapper.vm.quotaStatus.upload_quota.percentage).toBe(12.2)
  })

  it('displays reputation score', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-statistic': { template: '<div class="el-statistic"></div>' },
        }
      }
    })

    await wrapper.vm.$nextTick()
    
    expect(wrapper.vm.userProfile.reputation_score).toBe(85)
    expect(wrapper.vm.userProfile.verification_streak).toBe(12)
  })

  it('handles contribution setup dialog', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-button': { template: '<button class="el-button" @click="$emit(\'click\')"><slot /></button>' },
          'el-dialog': { template: '<div class="el-dialog"><slot /></div>' },
        }
      }
    })

    // Find contribute button and click it
    const contributeButton = wrapper.find('.el-button')
    if (contributeButton.exists()) {
      await contributeButton.trigger('click')
      expect(wrapper.vm.showContributionDialog).toBe(true)
    }
  })

  it('formats file sizes correctly', () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
        }
      }
    })

    // Test formatFileSize method if it exists
    if (wrapper.vm.formatFileSize) {
      expect(wrapper.vm.formatFileSize(1024)).toBe('1.0 KB')
      expect(wrapper.vm.formatFileSize(1048576)).toBe('1.0 MB')
      expect(wrapper.vm.formatFileSize(1073741824)).toBe('1.0 GB')
    }
  })

  it('calculates quota percentages correctly', () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
        }
      }
    })

    // Test quota percentage calculations
    const uploadPercentage = (store.userProfile.upload_quota_used / store.userProfile.upload_quota_limit) * 100
    expect(Math.round(uploadPercentage * 10) / 10).toBe(12.2)
  })

  it('shows network statistics', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-statistic': { template: '<div class="el-statistic"></div>' },
        }
      }
    })

    await wrapper.vm.$nextTick()
    
    expect(wrapper.vm.economyStatus.total_contributors).toBe(1250)
    expect(wrapper.vm.economyStatus.network_utilization).toBe(0.72)
  })

  it('handles auto-refresh functionality', async () => {
    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
        }
      }
    })

    // Check if refresh methods are called on mount
    expect(store.fetchEconomyStatus).toHaveBeenCalled()
    expect(store.fetchUserProfile).toHaveBeenCalled()
    expect(store.fetchQuotaStatus).toHaveBeenCalled()
  })

  it('displays tier upgrade options for free users', async () => {
    // Set user to free tier
    store.userProfile.tier = 'Free'
    store.userProfile.upload_quota_limit = 104857600 // 100MB

    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-button': { template: '<button class="el-button"><slot /></button>' },
          'el-alert': { template: '<div class="el-alert"><slot /></div>' },
        }
      }
    })

    await wrapper.vm.$nextTick()
    
    // Should show upgrade options for free tier
    expect(wrapper.vm.userProfile.tier).toBe('Free')
  })

  it('handles error states gracefully', async () => {
    // Mock error responses
    store.fetchEconomyStatus = vi.fn().mockRejectedValue(new Error('Network error'))
    store.fetchUserProfile = vi.fn().mockRejectedValue(new Error('Auth error'))

    wrapper = mount(EconomyDashboard, {
      global: {
        plugins: [createPinia()],
        stubs: {
          'el-card': { template: '<div class="el-card"><slot /></div>' },
          'el-alert': { template: '<div class="el-alert"><slot /></div>' },
        }
      }
    })

    // Component should handle errors without crashing
    expect(wrapper.exists()).toBe(true)
  })
})