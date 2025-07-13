import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Upload } from '../../src/pages/Upload'
import { useEconomyStore } from '../../src/stores/economyStore'

// Mock the economy store
vi.mock('../../src/stores/economyStore', () => ({
  useEconomyStore: vi.fn(),
}))

// Mock API utilities
vi.mock('../../src/utils/api', () => ({
  uploadFile: vi.fn(),
  formatFileSize: vi.fn((size: number) => {
    if (size >= 1024 * 1024 * 1024) return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`
    if (size >= 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`
    if (size >= 1024) return `${(size / 1024).toFixed(1)} KB`
    return `${size} B`
  }),
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
  CloudArrowUpIcon: () => <div data-testid="cloud-arrow-up-icon" />,
  DocumentIcon: () => <div data-testid="document-icon" />,
  ExclamationTriangleIcon: () => <div data-testid="exclamation-triangle-icon" />,
  CheckCircleIcon: () => <div data-testid="check-circle-icon" />,
  XMarkIcon: () => <div data-testid="x-mark-icon" />,
}))

describe('Upload Page', () => {
  const mockStore = {
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
    fetchUserProfile: vi.fn(),
    fetchQuotaStatus: vi.fn(),
    getUploadQuotaPercentage: vi.fn(() => 12.2),
    getIsQuotaLow: vi.fn(() => false),
  }

  beforeEach(() => {
    vi.mocked(useEconomyStore).mockReturnValue(mockStore)
  })

  it('renders upload page correctly', () => {
    render(<Upload />)
    
    expect(screen.getByText('Upload Files')).toBeInTheDocument()
    expect(screen.getByText('Drop files here or click to upload')).toBeInTheDocument()
  })

  it('displays quota information', () => {
    render(<Upload />)
    
    expect(screen.getByText('Upload Quota')).toBeInTheDocument()
    expect(screen.getByText(/500\.0 MB \/ 4\.0 GB/)).toBeInTheDocument()
  })

  it('shows quota warning when quota is low', () => {
    mockStore.getIsQuotaLow.mockReturnValue(true)
    
    render(<Upload />)
    
    expect(screen.getByText('Upload Quota Running Low')).toBeInTheDocument()
    expect(screen.getByTestId('exclamation-triangle-icon')).toBeInTheDocument()
  })

  it('handles file selection through input', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      expect(screen.getByText('test.txt')).toBeInTheDocument()
    })
  })

  it('handles drag and drop file upload', async () => {
    render(<Upload />)
    
    const dropZone = screen.getByText('Drop files here or click to upload').closest('div')
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    
    // Simulate drag enter
    fireEvent.dragEnter(dropZone!, {
      dataTransfer: {
        files: [file],
      },
    })
    
    // Simulate drop
    fireEvent.drop(dropZone!, {
      dataTransfer: {
        files: [file],
      },
    })
    
    await waitFor(() => {
      expect(screen.getByText('test.txt')).toBeInTheDocument()
    })
  })

  it('validates quota before adding files to queue', async () => {
    const user = userEvent.setup()
    
    // Set quota very low
    mockStore.userProfile.upload_quota_used = 4200000000 // ~4GB used
    mockStore.userProfile.upload_quota_limit = 4294967296 // 4GB limit
    
    render(<Upload />)
    
    // Try to upload a large file that exceeds quota
    const largeFile = new File(['x'.repeat(100 * 1024 * 1024)], 'large.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, largeFile)
    
    // File should not be added to queue due to quota limitation
    expect(screen.queryByText('large.txt')).not.toBeInTheDocument()
  })

  it('displays file list with correct information', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      expect(screen.getByText('test.txt')).toBeInTheDocument()
      expect(screen.getByTestId('document-icon')).toBeInTheDocument()
    })
  })

  it('handles individual file upload', async () => {
    const user = userEvent.setup()
    const { uploadFile } = await import('../../src/utils/api')
    
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const uploadButton = screen.getByText('Upload')
      fireEvent.click(uploadButton)
    })
    
    expect(uploadFile).toHaveBeenCalledWith(file, expect.any(Object))
  })

  it('handles upload all files functionality', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const file1 = new File(['test content 1'], 'test1.txt', { type: 'text/plain' })
    const file2 = new File(['test content 2'], 'test2.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, [file1, file2])
    
    await waitFor(() => {
      const uploadAllButton = screen.getByText('Upload All')
      expect(uploadAllButton).toBeInTheDocument()
      fireEvent.click(uploadAllButton)
    })
  })

  it('shows upload progress during file upload', async () => {
    const user = userEvent.setup()
    const { uploadFile } = await import('../../src/utils/api')
    
    // Mock uploadFile to simulate progress
    vi.mocked(uploadFile).mockImplementation((file, options) => {
      return new Promise((resolve) => {
        // Simulate progress updates
        setTimeout(() => options.onProgress?.(25), 100)
        setTimeout(() => options.onProgress?.(50), 200)
        setTimeout(() => options.onProgress?.(75), 300)
        setTimeout(() => {
          options.onProgress?.(100)
          resolve(undefined)
        }, 400)
      })
    })
    
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const uploadButton = screen.getByText('Upload')
      fireEvent.click(uploadButton)
    })
    
    // Should show progress
    await waitFor(() => {
      expect(screen.getByText(/\d+% uploaded/)).toBeInTheDocument()
    })
  })

  it('handles file removal from queue', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const removeButton = screen.getByTestId('x-mark-icon').closest('button')
      fireEvent.click(removeButton!)
    })
    
    expect(screen.queryByText('test.txt')).not.toBeInTheDocument()
  })

  it('handles tags input', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const tagsInput = screen.getByPlaceholderText('Enter tags separated by commas')
    await user.type(tagsInput, 'tag1, tag2, tag3')
    
    expect(tagsInput).toHaveValue('tag1, tag2, tag3')
  })

  it('clears completed uploads', async () => {
    const user = userEvent.setup()
    const { uploadFile } = await import('../../src/utils/api')
    
    // Mock successful upload
    vi.mocked(uploadFile).mockResolvedValue(undefined)
    
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const uploadButton = screen.getByText('Upload')
      fireEvent.click(uploadButton)
    })
    
    // Wait for upload completion
    await waitFor(() => {
      expect(screen.getByTestId('check-circle-icon')).toBeInTheDocument()
    })
    
    // Clear completed files
    const clearButton = screen.getByText('Clear Completed')
    fireEvent.click(clearButton)
    
    expect(screen.queryByText('test.txt')).not.toBeInTheDocument()
  })

  it('displays remaining quota correctly', () => {
    render(<Upload />)
    
    const remainingQuota = mockStore.userProfile.upload_quota_limit - mockStore.userProfile.upload_quota_used
    const expectedRemaining = (remainingQuota / (1024 * 1024 * 1024)).toFixed(1) // Convert to GB
    
    expect(screen.getByText(new RegExp(`Remaining: ${expectedRemaining} GB`))).toBeInTheDocument()
  })

  it('shows pending upload size', async () => {
    const user = userEvent.setup()
    render(<Upload />)
    
    const file = new File(['x'.repeat(10 * 1024 * 1024)], 'test.txt', { type: 'text/plain' }) // 10MB
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      expect(screen.getByText(/Pending uploads: 10\.0 MB/)).toBeInTheDocument()
    })
  })

  it('handles upload errors gracefully', async () => {
    const user = userEvent.setup()
    const { uploadFile } = await import('../../src/utils/api')
    
    // Mock upload failure
    vi.mocked(uploadFile).mockRejectedValue(new Error('Upload failed'))
    
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const uploadButton = screen.getByText('Upload')
      fireEvent.click(uploadButton)
    })
    
    // Should show error state
    await waitFor(() => {
      expect(screen.getByTestId('exclamation-triangle-icon')).toBeInTheDocument()
    })
  })

  it('refreshes quota after successful upload', async () => {
    const user = userEvent.setup()
    const { uploadFile } = await import('../../src/utils/api')
    
    // Mock successful upload
    vi.mocked(uploadFile).mockResolvedValue(undefined)
    
    render(<Upload />)
    
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' })
    const input = screen.getByLabelText(/Drop files here or click to upload/)
    
    await user.upload(input, file)
    
    await waitFor(() => {
      const uploadButton = screen.getByText('Upload')
      fireEvent.click(uploadButton)
    })
    
    // Should refresh quota data after successful upload
    await waitFor(() => {
      expect(mockStore.fetchUserProfile).toHaveBeenCalled()
      expect(mockStore.fetchQuotaStatus).toHaveBeenCalled()
    })
  })
})