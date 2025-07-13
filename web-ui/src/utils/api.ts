import axios, { AxiosError } from 'axios'
import toast from 'react-hot-toast'

// Create axios instance
export const api = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    // Add timestamp to prevent caching
    if (config.method === 'get') {
      config.params = {
        ...config.params,
        _t: Date.now(),
      }
    }
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  (response) => {
    return response
  },
  (error: AxiosError) => {
    // Handle common errors
    if (error.response) {
      const status = error.response.status
      const data = error.response.data as any
      
      switch (status) {
        case 401:
          // Unauthorized - redirect to login
          toast.error('Session expired. Please login again.')
          // Clear auth state
          localStorage.removeItem('datamesh-auth')
          window.location.href = '/login'
          break
          
        case 403:
          toast.error('Access denied')
          break
          
        case 404:
          toast.error('Resource not found')
          break
          
        case 429:
          toast.error('Too many requests. Please try again later.')
          break
          
        case 500:
          toast.error('Internal server error. Please try again.')
          break
          
        default:
          if (data?.message) {
            toast.error(data.message)
          } else {
            toast.error('An unexpected error occurred')
          }
      }
    } else if (error.request) {
      // Network error
      toast.error('Network error. Please check your connection.')
    } else {
      // Request setup error
      toast.error('Request failed')
    }
    
    return Promise.reject(error)
  }
)

// File upload helper
export const uploadFile = async (
  file: File,
  options: {
    tags?: string
    publicKey?: string
    onProgress?: (progress: number) => void
  } = {}
) => {
  const formData = new FormData()
  formData.append('file', file)
  
  if (options.tags) {
    formData.append('tags', options.tags)
  }
  
  if (options.publicKey) {
    formData.append('public_key', options.publicKey)
  }
  
  return api.post('/files', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
    onUploadProgress: (progressEvent) => {
      if (progressEvent.total && options.onProgress) {
        const progress = (progressEvent.loaded / progressEvent.total) * 100
        options.onProgress(progress)
      }
    },
  })
}

// File download helper
export const downloadFile = async (fileKey: string, fileName?: string) => {
  const response = await api.get(`/files/${fileKey}`, {
    responseType: 'blob',
  })
  
  // Create download link
  const url = window.URL.createObjectURL(new Blob([response.data]))
  const link = document.createElement('a')
  link.href = url
  
  // Get filename from response headers or use provided name
  const contentDisposition = response.headers['content-disposition']
  const fileNameMatch = contentDisposition?.match(/filename="(.+)"/)
  const downloadFileName = fileName || fileNameMatch?.[1] || `file_${fileKey}`
  
  link.setAttribute('download', downloadFileName)
  document.body.appendChild(link)
  link.click()
  link.remove()
  window.URL.revokeObjectURL(url)
}

// Format file size
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// Format date
export const formatDate = (dateString: string): string => {
  const date = new Date(dateString)
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

// Debounce utility
export const debounce = <T extends (...args: any[]) => any>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: NodeJS.Timeout
  
  return (...args: Parameters<T>) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

// Throttle utility
export const throttle = <T extends (...args: any[]) => any>(
  func: T,
  limit: number
): ((...args: Parameters<T>) => void) => {
  let inThrottle: boolean
  
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

// Validate file type
export const isValidFileType = (file: File, allowedTypes: string[]): boolean => {
  return allowedTypes.some(type => {
    if (type.endsWith('/*')) {
      const category = type.replace('/*', '')
      return file.type.startsWith(category)
    }
    return file.type === type
  })
}

// Get file type icon
export const getFileTypeIcon = (fileName: string): string => {
  const extension = fileName.split('.').pop()?.toLowerCase()
  
  switch (extension) {
    case 'pdf':
      return '📄'
    case 'doc':
    case 'docx':
      return '📝'
    case 'xls':
    case 'xlsx':
      return '📊'
    case 'ppt':
    case 'pptx':
      return '📊'
    case 'jpg':
    case 'jpeg':
    case 'png':
    case 'gif':
    case 'webp':
      return '🖼️'
    case 'mp4':
    case 'avi':
    case 'mov':
    case 'wmv':
      return '🎥'
    case 'mp3':
    case 'wav':
    case 'flac':
      return '🎵'
    case 'zip':
    case 'rar':
    case '7z':
      return '🗜️'
    case 'txt':
      return '📄'
    case 'js':
    case 'jsx':
    case 'ts':
    case 'tsx':
    case 'py':
    case 'java':
    case 'cpp':
    case 'c':
    case 'rs':
      return '💻'
    default:
      return '📁'
  }
}