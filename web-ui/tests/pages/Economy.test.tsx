import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { Economy } from '../../src/pages/Economy'
import { useEconomyStore } from '../../src/stores/economyStore'

// Mock the economy store
vi.mock('../../src/stores/economyStore', () => ({
  useEconomyStore: vi.fn(),
}))

// Mock react-hot-toast
vi.mock('react-hot-toast', () => ({
  default: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

// Mock Heroicons
vi.mock('@heroicons/react/24/outline', () => ({
  ChartBarIcon: () => <div data-testid="chart-bar-icon" />,
  CloudArrowUpIcon: () => <div data-testid="cloud-arrow-up-icon" />,
  CogIcon: () => <div data-testid="cog-icon" />,
  DocumentIcon: () => <div data-testid="document-icon" />,
  ExclamationTriangleIcon: () => <div data-testid="exclamation-triangle-icon" />,
  CheckCircleIcon: () => <div data-testid="check-circle-icon" />,
  XMarkIcon: () => <div data-testid="x-mark-icon" />,
}))

// Mock recharts
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: any) => <div data-testid="responsive-container">{children}</div>,
  LineChart: ({ children }: any) => <div data-testid="line-chart">{children}</div>,
  Line: () => <div data-testid="line" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: () => <div data-testid="tooltip" />,
}))

describe('Economy Page', () => {
  const mockStore = {
    economyStatus: {
      health: 'healthy',
      total_contributors: 1250,
      total_storage_contributed: 5242880000000, // 5TB
      active_verifications: 45,
      network_utilization: 0.72,
    },
    userProfile: {
      user_id: 'test-user-123',
      tier: 'Contributor',
      storage_contributed: 4294967296, // 4GB
      upload_quota_used: 524288000, // 500MB
      upload_quota_limit: 4294967296, // 4GB
      download_quota_used: 1073741824, // 1GB
      download_quota_limit: 10737418240, // 10GB
      reputation_score: 85,
      verification_streak: 12,
      last_activity: '2024-01-15T10:30:00Z',
    },
    quotaStatus: {
      upload_quota: {
        used: 524288000,
        limit: 4294967296,
        percentage: 12.2,
      },
      download_quota: {
        used: 1073741824,
        limit: 10737418240,
        percentage: 10.0,
      },
      storage_quota: {
        used: 2147483648, // 2GB
        limit: 5368709120, // 5GB
        percentage: 40.0,
      },
    },
    fetchEconomyStatus: vi.fn(),
    fetchUserProfile: vi.fn(),
    fetchQuotaStatus: vi.fn(),
    getUploadQuotaPercentage: vi.fn(() => 12.2),
    getDownloadQuotaPercentage: vi.fn(() => 10.0),
    getIsQuotaLow: vi.fn(() => false),
    getIsQuotaCritical: vi.fn(() => false),
    getRemainingUploadQuota: vi.fn(() => 3770593280), // ~3.5GB
    getRemainingDownloadQuota: vi.fn(() => 9663676416), // ~9GB
    getCanContribute: vi.fn(() => true),
    getCanUpgrade: vi.fn(() => true),
    getTierDisplayName: vi.fn((tier: string) => tier),
    getTierColor: vi.fn(() => 'blue'),
  }

  beforeEach(() => {
    vi.mocked(useEconomyStore).mockReturnValue(mockStore)
  })

  it('renders economy page correctly', () => {
    render(<Economy />)
    
    expect(screen.getByText('Storage Economy')).toBeInTheDocument()
    expect(screen.getByText('Dashboard')).toBeInTheDocument()
  })

  it('displays user tier information', () => {
    render(<Economy />)
    
    expect(screen.getByText('Current Tier')).toBeInTheDocument()
    expect(screen.getByText('Contributor')).toBeInTheDocument()
  })

  it('shows quota information with correct percentages', () => {
    render(<Economy />)
    
    expect(screen.getByText('Upload Quota')).toBeInTheDocument()
    expect(screen.getByText('Download Quota')).toBeInTheDocument()
    
    // Check if quota percentages are displayed
    const quotaElements = screen.getAllByText(/12\.2%|10\.0%/)
    expect(quotaElements.length).toBeGreaterThan(0)
  })

  it('displays reputation score and verification streak', () => {
    render(<Economy />)
    
    expect(screen.getByText('Reputation')).toBeInTheDocument()
    expect(screen.getByText('85')).toBeInTheDocument()
    expect(screen.getByText('12')).toBeInTheDocument()
  })

  it('shows network statistics', () => {
    render(<Economy />)
    
    expect(screen.getByText('Network Health')).toBeInTheDocument()
    expect(screen.getByText('1,250')).toBeInTheDocument() // Total contributors
  })

  it('handles contribution setup for contributor tier', async () => {
    render(<Economy />)
    
    const contributeButton = screen.getByText('Setup Storage Contribution')
    expect(contributeButton).toBeInTheDocument()
    
    fireEvent.click(contributeButton)
    
    await waitFor(() => {
      expect(screen.getByText('Storage Contribution Setup')).toBeInTheDocument()
    })
  })

  it('displays quota warnings when quota is low', () => {
    mockStore.getIsQuotaLow.mockReturnValue(true)
    
    render(<Economy />)
    
    expect(screen.getByTestId('exclamation-triangle-icon')).toBeInTheDocument()
  })

  it('displays critical quota warning when quota is critical', () => {
    mockStore.getIsQuotaCritical.mockReturnValue(true)
    
    render(<Economy />)
    
    // Should show critical warning
    const warningElements = screen.getAllByTestId('exclamation-triangle-icon')
    expect(warningElements.length).toBeGreaterThan(0)
  })

  it('shows upgrade options for users who can upgrade', () => {
    render(<Economy />)
    
    const upgradeButton = screen.getByText('Upgrade Tier')
    expect(upgradeButton).toBeInTheDocument()
  })

  it('hides upgrade options for enterprise users', () => {
    mockStore.userProfile.tier = 'Enterprise'
    mockStore.getCanUpgrade.mockReturnValue(false)
    
    render(<Economy />)
    
    expect(screen.queryByText('Upgrade Tier')).not.toBeInTheDocument()
  })

  it('handles verification process for contributor tier', async () => {
    render(<Economy />)
    
    const verifyButton = screen.getByText('Run Verification')
    expect(verifyButton).toBeInTheDocument()
    
    fireEvent.click(verifyButton)
    
    // Should trigger verification process
    await waitFor(() => {
      expect(screen.getByText('Verification in progress...')).toBeInTheDocument()
    })
  })

  it('displays different content for free tier users', () => {
    mockStore.userProfile.tier = 'Free'
    mockStore.getCanContribute.mockReturnValue(false)
    
    render(<Economy />)
    
    expect(screen.getByText('Free')).toBeInTheDocument()
    expect(screen.queryByText('Setup Storage Contribution')).not.toBeInTheDocument()
  })

  it('shows premium tier benefits', () => {
    mockStore.userProfile.tier = 'Premium'
    
    render(<Economy />)
    
    expect(screen.getByText('Premium')).toBeInTheDocument()
    // Premium users should see no verification required
    expect(screen.queryByText('Run Verification')).not.toBeInTheDocument()
  })

  it('formats file sizes correctly', () => {
    render(<Economy />)
    
    // Check if file sizes are formatted (should show GB, MB, etc.)
    const sizeElements = screen.getAllByText(/GB|MB|KB/)
    expect(sizeElements.length).toBeGreaterThan(0)
  })

  it('displays storage contribution amount for contributors', () => {
    render(<Economy />)
    
    expect(screen.getByText('Storage Contributed')).toBeInTheDocument()
    expect(screen.getByText('4.0 GB')).toBeInTheDocument()
  })

  it('shows real-time data updates', async () => {
    render(<Economy />)
    
    // Should call fetch functions on mount
    expect(mockStore.fetchEconomyStatus).toHaveBeenCalled()
    expect(mockStore.fetchUserProfile).toHaveBeenCalled()
    expect(mockStore.fetchQuotaStatus).toHaveBeenCalled()
  })

  it('handles error states gracefully', () => {
    mockStore.fetchEconomyStatus.mockRejectedValue(new Error('Network error'))
    
    render(<Economy />)
    
    // Component should render without crashing even with errors
    expect(screen.getByText('Storage Economy')).toBeInTheDocument()
  })

  it('calculates quota remaining correctly', () => {
    render(<Economy />)
    
    expect(mockStore.getRemainingUploadQuota).toHaveBeenCalled()
    expect(mockStore.getRemainingDownloadQuota).toHaveBeenCalled()
  })

  it('shows contribution ratio information', () => {
    render(<Economy />)
    
    expect(screen.getByText('4:1 Contribution Ratio')).toBeInTheDocument()
  })

  it('displays last activity timestamp', () => {
    render(<Economy />)
    
    expect(screen.getByText('Last Activity')).toBeInTheDocument()
  })

  it('handles tier upgrade dialog', async () => {
    render(<Economy />)
    
    const upgradeButton = screen.getByText('Upgrade Tier')
    fireEvent.click(upgradeButton)
    
    await waitFor(() => {
      expect(screen.getByText('Choose Your Plan')).toBeInTheDocument()
    })
  })

  it('shows network utilization chart', () => {
    render(<Economy />)
    
    expect(screen.getByTestId('responsive-container')).toBeInTheDocument()
    expect(screen.getByTestId('line-chart')).toBeInTheDocument()
  })

  it('displays verification history when available', () => {
    render(<Economy />)
    
    expect(screen.getByText('Verification Streak')).toBeInTheDocument()
    expect(screen.getByText('12')).toBeInTheDocument()
  })
})