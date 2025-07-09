import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { filesAPI } from '../services/api'

export const useFilesStore = defineStore('files', () => {
  // State
  const files = ref([])
  const currentFile = ref(null)
  const uploadQueue = ref([])
  const searchResults = ref([])
  const stats = ref({
    total_files: 0,
    total_storage_bytes: 0,
    cache_hit_ratio: 0,
    api_requests_last_hour: 0,
    system_status: 'unknown'
  })
  
  const loading = ref(false)
  const uploading = ref(false)
  const error = ref(null)
  const searchQuery = ref('')
  const currentPage = ref(1)
  const pageSize = ref(20)
  const totalFiles = ref(0)
  const selectedFiles = ref([])
  
  // Getters
  const isLoading = computed(() => loading.value)
  const isUploading = computed(() => uploading.value)
  const hasFiles = computed(() => files.value.length > 0)
  const hasSearchResults = computed(() => searchResults.value.length > 0)
  const activeUploads = computed(() => uploadQueue.value.filter(u => u.status === 'uploading'))
  const completedUploads = computed(() => uploadQueue.value.filter(u => u.status === 'completed'))
  const failedUploads = computed(() => uploadQueue.value.filter(u => u.status === 'failed'))
  const totalPages = computed(() => Math.ceil(totalFiles.value / pageSize.value))
  const hasSelectedFiles = computed(() => selectedFiles.value.length > 0)
  const selectedFileCount = computed(() => selectedFiles.value.length)
  
  // File type helpers
  const getFilesByType = computed(() => {
    const types = {}
    files.value.forEach(file => {
      const ext = file.file_name.split('.').pop()?.toLowerCase() || 'unknown'
      if (!types[ext]) types[ext] = []
      types[ext].push(file)
    })
    return types
  })
  
  const getFilesBySize = computed(() => {
    return files.value.sort((a, b) => b.file_size - a.file_size)
  })
  
  const getRecentFiles = computed(() => {
    return files.value.sort((a, b) => new Date(b.uploaded_at) - new Date(a.uploaded_at))
  })
  
  // Actions
  const fetchFiles = async (params = {}) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await filesAPI.listFiles({
        page: currentPage.value,
        page_size: pageSize.value,
        ...params
      })
      
      files.value = response.data.files
      totalFiles.value = response.data.total
      currentPage.value = response.data.page
      pageSize.value = response.data.page_size
      
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchFileMetadata = async (fileKey) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await filesAPI.getFileMetadata(fileKey)
      currentFile.value = response.data
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const uploadFile = async (fileData, options = {}) => {
    const uploadId = Date.now().toString()
    const uploadItem = {
      id: uploadId,
      file: fileData,
      name: fileData.name,
      size: fileData.size,
      progress: 0,
      status: 'uploading',
      error: null,
      uploaded_at: new Date(),
      ...options
    }
    
    uploadQueue.value.push(uploadItem)
    uploading.value = true
    
    try {
      const formData = new FormData()
      formData.append('file', fileData)
      
      if (options.name) formData.append('name', options.name)
      if (options.tags) formData.append('tags', options.tags)
      if (options.public_key) formData.append('public_key', options.public_key)
      
      const response = await filesAPI.uploadFile(formData, {
        onUploadProgress: (progressEvent) => {
          const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
          updateUploadProgress(uploadId, progress)
        }
      })
      
      // Update upload item
      const uploadIndex = uploadQueue.value.findIndex(u => u.id === uploadId)
      if (uploadIndex !== -1) {
        uploadQueue.value[uploadIndex] = {
          ...uploadQueue.value[uploadIndex],
          status: 'completed',
          progress: 100,
          result: response.data
        }
      }
      
      // Add to files list
      files.value.unshift(response.data)
      totalFiles.value += 1
      
      return response.data
    } catch (err) {
      // Update upload item with error
      const uploadIndex = uploadQueue.value.findIndex(u => u.id === uploadId)
      if (uploadIndex !== -1) {
        uploadQueue.value[uploadIndex] = {
          ...uploadQueue.value[uploadIndex],
          status: 'failed',
          error: err.message
        }
      }
      
      throw err
    } finally {
      // Check if any uploads are still in progress
      const hasActiveUploads = uploadQueue.value.some(u => u.status === 'uploading')
      if (!hasActiveUploads) {
        uploading.value = false
      }
    }
  }
  
  const uploadMultipleFiles = async (filesData, options = {}) => {
    const uploadPromises = filesData.map(file => uploadFile(file, options))
    
    try {
      const results = await Promise.allSettled(uploadPromises)
      const successful = results.filter(r => r.status === 'fulfilled').map(r => r.value)
      const failed = results.filter(r => r.status === 'rejected').map(r => r.reason)
      
      return { successful, failed }
    } catch (err) {
      throw err
    }
  }
  
  const downloadFile = async (fileKey, filename) => {
    loading.value = true
    error.value = null
    
    try {
      const response = await filesAPI.downloadFile(fileKey)
      
      // Create download link
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', filename)
      document.body.appendChild(link)
      link.click()
      link.remove()
      window.URL.revokeObjectURL(url)
      
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const deleteFile = async (fileKey) => {
    loading.value = true
    error.value = null
    
    try {
      await filesAPI.deleteFile(fileKey)
      
      // Remove from files list
      files.value = files.value.filter(f => f.file_key !== fileKey)
      totalFiles.value -= 1
      
      // Remove from selected files
      selectedFiles.value = selectedFiles.value.filter(f => f.file_key !== fileKey)
      
      return true
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const deleteMultipleFiles = async (fileKeys) => {
    const deletePromises = fileKeys.map(key => deleteFile(key))
    
    try {
      const results = await Promise.allSettled(deletePromises)
      const successful = results.filter(r => r.status === 'fulfilled').length
      const failed = results.filter(r => r.status === 'rejected').length
      
      return { successful, failed }
    } catch (err) {
      throw err
    }
  }
  
  const searchFiles = async (query, filters = {}) => {
    loading.value = true
    error.value = null
    searchQuery.value = query
    
    try {
      const response = await filesAPI.searchFiles({
        query,
        page: currentPage.value,
        page_size: pageSize.value,
        ...filters
      })
      
      searchResults.value = response.data.files
      totalFiles.value = response.data.total
      
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const clearSearch = () => {
    searchResults.value = []
    searchQuery.value = ''
    currentPage.value = 1
  }
  
  const fetchStats = async () => {
    try {
      const response = await filesAPI.getStats()
      stats.value = response.data
      return response.data
    } catch (err) {
      console.error('Failed to fetch stats:', err)
      throw err
    }
  }
  
  // Upload queue management
  const updateUploadProgress = (uploadId, progress) => {
    const uploadIndex = uploadQueue.value.findIndex(u => u.id === uploadId)
    if (uploadIndex !== -1) {
      uploadQueue.value[uploadIndex].progress = progress
    }
  }
  
  const removeUploadItem = (uploadId) => {
    uploadQueue.value = uploadQueue.value.filter(u => u.id !== uploadId)
  }
  
  const clearUploadQueue = () => {
    uploadQueue.value = []
  }
  
  const retryUpload = async (uploadId) => {
    const uploadItem = uploadQueue.value.find(u => u.id === uploadId)
    if (!uploadItem || uploadItem.status !== 'failed') return
    
    // Reset upload item
    uploadItem.status = 'uploading'
    uploadItem.progress = 0
    uploadItem.error = null
    
    try {
      await uploadFile(uploadItem.file, {
        name: uploadItem.name,
        tags: uploadItem.tags,
        public_key: uploadItem.public_key
      })
    } catch (err) {
      console.error('Retry upload failed:', err)
    }
  }
  
  // Selection management
  const selectFile = (file) => {
    if (!selectedFiles.value.find(f => f.file_key === file.file_key)) {
      selectedFiles.value.push(file)
    }
  }
  
  const deselectFile = (fileKey) => {
    selectedFiles.value = selectedFiles.value.filter(f => f.file_key !== fileKey)
  }
  
  const toggleFileSelection = (file) => {
    const isSelected = selectedFiles.value.find(f => f.file_key === file.file_key)
    if (isSelected) {
      deselectFile(file.file_key)
    } else {
      selectFile(file)
    }
  }
  
  const selectAllFiles = () => {
    const currentFiles = searchResults.value.length > 0 ? searchResults.value : files.value
    selectedFiles.value = [...currentFiles]
  }
  
  const deselectAllFiles = () => {
    selectedFiles.value = []
  }
  
  // Pagination
  const setPage = (page) => {
    currentPage.value = page
  }
  
  const setPageSize = (size) => {
    pageSize.value = size
    currentPage.value = 1
  }
  
  const nextPage = () => {
    if (currentPage.value < totalPages.value) {
      currentPage.value += 1
    }
  }
  
  const prevPage = () => {
    if (currentPage.value > 1) {
      currentPage.value -= 1
    }
  }
  
  // Initialize store
  const init = async () => {
    try {
      await Promise.all([
        fetchFiles(),
        fetchStats()
      ])
    } catch (error) {
      console.error('Failed to initialize files store:', error)
    }
  }
  
  return {
    // State
    files,
    currentFile,
    uploadQueue,
    searchResults,
    stats,
    loading,
    uploading,
    error,
    searchQuery,
    currentPage,
    pageSize,
    totalFiles,
    selectedFiles,
    
    // Getters
    isLoading,
    isUploading,
    hasFiles,
    hasSearchResults,
    activeUploads,
    completedUploads,
    failedUploads,
    totalPages,
    hasSelectedFiles,
    selectedFileCount,
    getFilesByType,
    getFilesBySize,
    getRecentFiles,
    
    // Actions
    fetchFiles,
    fetchFileMetadata,
    uploadFile,
    uploadMultipleFiles,
    downloadFile,
    deleteFile,
    deleteMultipleFiles,
    searchFiles,
    clearSearch,
    fetchStats,
    updateUploadProgress,
    removeUploadItem,
    clearUploadQueue,
    retryUpload,
    selectFile,
    deselectFile,
    toggleFileSelection,
    selectAllFiles,
    deselectAllFiles,
    setPage,
    setPageSize,
    nextPage,
    prevPage,
    init
  }
})