import React, { useState, useCallback, useEffect } from 'react'
import { useEconomyStore } from '../stores/economyStore'
import { uploadFile, formatFileSize } from '../utils/api'
import { 
  CloudArrowUpIcon, 
  DocumentIcon, 
  ExclamationTriangleIcon,
  CheckCircleIcon,
  XMarkIcon 
} from '@heroicons/react/24/outline'
import toast from 'react-hot-toast'

interface UploadFile {
  file: File
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  id: string
}

export function Upload() {
  const {
    userProfile,
    quotaStatus,
    fetchUserProfile,
    fetchQuotaStatus,
    getUploadQuotaPercentage,
    getIsQuotaLow,
  } = useEconomyStore()

  const [files, setFiles] = useState<UploadFile[]>([])
  const [dragActive, setDragActive] = useState(false)
  const [tags, setTags] = useState('')

  useEffect(() => {
    // Fetch initial quota data
    fetchUserProfile()
    fetchQuotaStatus()
  }, [fetchUserProfile, fetchQuotaStatus])

  const uploadQuotaPercentage = getUploadQuotaPercentage()
  const isQuotaLow = getIsQuotaLow()

  const checkQuotaAvailable = (fileSize: number): boolean => {
    if (!userProfile) return false
    
    const remainingQuota = userProfile.upload_quota_limit - userProfile.upload_quota_used
    return fileSize <= remainingQuota
  }

  const handleDrag = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true)
    } else if (e.type === "dragleave") {
      setDragActive(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
    
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFiles(e.dataTransfer.files)
    }
  }, [])

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    e.preventDefault()
    if (e.target.files && e.target.files[0]) {
      handleFiles(e.target.files)
    }
  }

  const handleFiles = (fileList: FileList) => {
    const newFiles: UploadFile[] = []
    
    for (let i = 0; i < fileList.length; i++) {
      const file = fileList[i]
      
      // Check quota before adding to queue
      if (!checkQuotaAvailable(file.size)) {
        toast.error(`File "${file.name}" exceeds available upload quota`)
        continue
      }
      
      newFiles.push({
        file,
        progress: 0,
        status: 'pending',
        id: `${file.name}-${Date.now()}-${i}`,
      })
    }
    
    setFiles(prev => [...prev, ...newFiles])
  }

  const uploadSingleFile = async (uploadFile: UploadFile) => {
    setFiles(prev => prev.map(f => 
      f.id === uploadFile.id 
        ? { ...f, status: 'uploading' as const }
        : f
    ))

    try {
      await uploadFile(uploadFile.file, {
        tags,
        onProgress: (progress) => {
          setFiles(prev => prev.map(f => 
            f.id === uploadFile.id 
              ? { ...f, progress }
              : f
          ))
        }
      })

      setFiles(prev => prev.map(f => 
        f.id === uploadFile.id 
          ? { ...f, status: 'completed' as const, progress: 100 }
          : f
      ))

      // Refresh quota after successful upload
      await Promise.all([
        fetchUserProfile(),
        fetchQuotaStatus(),
      ])

      toast.success(`File "${uploadFile.file.name}" uploaded successfully`)
    } catch (error: any) {
      setFiles(prev => prev.map(f => 
        f.id === uploadFile.id 
          ? { ...f, status: 'error' as const, error: error.message }
          : f
      ))
      toast.error(`Failed to upload "${uploadFile.file.name}": ${error.message}`)
    }
  }

  const uploadAllFiles = async () => {
    const pendingFiles = files.filter(f => f.status === 'pending')
    
    for (const file of pendingFiles) {
      await uploadSingleFile(file)
    }
  }

  const removeFile = (id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id))
  }

  const clearCompleted = () => {
    setFiles(prev => prev.filter(f => f.status !== 'completed'))
  }

  const remainingQuota = userProfile ? 
    userProfile.upload_quota_limit - userProfile.upload_quota_used : 0
  
  const totalPendingSize = files
    .filter(f => f.status === 'pending')
    .reduce((total, f) => total + f.file.size, 0)

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Upload Files</h1>
        {files.some(f => f.status === 'completed') && (
          <button
            onClick={clearCompleted}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            Clear Completed
          </button>
        )}
      </div>

      {/* Quota Warning */}
      {isQuotaLow && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="flex">
            <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800">
                Upload Quota Running Low
              </h3>
              <div className="mt-2 text-sm text-yellow-700">
                <p>
                  You have used {uploadQuotaPercentage}% of your monthly upload quota. 
                  Remaining: {formatFileSize(remainingQuota)}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Upload Area */}
      <div
        className={`relative border-2 border-dashed rounded-lg p-6 transition-colors ${
          dragActive
            ? 'border-blue-400 bg-blue-50'
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragEnter={handleDrag}
        onDragLeave={handleDrag}
        onDragOver={handleDrag}
        onDrop={handleDrop}
      >
        <input
          type="file"
          id="file-upload"
          multiple
          onChange={handleChange}
          className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
        />
        
        <div className="text-center">
          <CloudArrowUpIcon className="mx-auto h-12 w-12 text-gray-400" />
          <div className="mt-4">
            <label htmlFor="file-upload" className="cursor-pointer">
              <span className="mt-2 block text-sm font-medium text-gray-900">
                Drop files here or click to upload
              </span>
            </label>
            <p className="mt-2 block text-xs text-gray-500">
              Maximum file size: 100MB per file
            </p>
          </div>
        </div>
      </div>

      {/* Tags Input */}
      <div>
        <label htmlFor="tags" className="block text-sm font-medium text-gray-700 mb-2">
          Tags (optional)
        </label>
        <input
          type="text"
          id="tags"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
          placeholder="Enter tags separated by commas"
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Quota Info */}
      {userProfile && (
        <div className="bg-gray-50 rounded-lg p-4">
          <h3 className="text-sm font-medium text-gray-900 mb-2">Upload Quota</h3>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Used:</span>
              <span>{formatFileSize(userProfile.upload_quota_used)} / {formatFileSize(userProfile.upload_quota_limit)}</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div 
                className={`h-2 rounded-full ${uploadQuotaPercentage > 85 ? 'bg-red-500' : 'bg-blue-500'}`}
                style={{ width: `${uploadQuotaPercentage}%` }}
              ></div>
            </div>
            <div className="flex justify-between text-xs text-gray-500">
              <span>{uploadQuotaPercentage}% used</span>
              <span>Remaining: {formatFileSize(remainingQuota)}</span>
            </div>
            {totalPendingSize > 0 && (
              <div className="text-xs text-blue-600">
                Pending uploads: {formatFileSize(totalPendingSize)}
              </div>
            )}
          </div>
        </div>
      )}

      {/* File List */}
      {files.length > 0 && (
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h2 className="text-lg font-medium text-gray-900">Files to Upload ({files.length})</h2>
            {files.some(f => f.status === 'pending') && (
              <button
                onClick={uploadAllFiles}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Upload All
              </button>
            )}
          </div>

          <div className="space-y-2">
            {files.map((uploadFile) => (
              <div key={uploadFile.id} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <DocumentIcon className="h-8 w-8 text-gray-400" />
                    <div>
                      <p className="text-sm font-medium text-gray-900">{uploadFile.file.name}</p>
                      <p className="text-xs text-gray-500">{formatFileSize(uploadFile.file.size)}</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    {uploadFile.status === 'pending' && (
                      <button
                        onClick={() => uploadSingleFile(uploadFile)}
                        className="text-blue-600 hover:text-blue-700 text-sm"
                      >
                        Upload
                      </button>
                    )}
                    
                    {uploadFile.status === 'completed' && (
                      <CheckCircleIcon className="h-5 w-5 text-green-500" />
                    )}
                    
                    {uploadFile.status === 'error' && (
                      <ExclamationTriangleIcon className="h-5 w-5 text-red-500" />
                    )}
                    
                    <button
                      onClick={() => removeFile(uploadFile.id)}
                      className="text-gray-400 hover:text-gray-600"
                    >
                      <XMarkIcon className="h-5 w-5" />
                    </button>
                  </div>
                </div>

                {/* Progress Bar */}
                {uploadFile.status === 'uploading' && (
                  <div className="mt-2">
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="h-2 bg-blue-500 rounded-full transition-all duration-300"
                        style={{ width: `${uploadFile.progress}%` }}
                      ></div>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">{uploadFile.progress.toFixed(0)}% uploaded</p>
                  </div>
                )}

                {/* Error Message */}
                {uploadFile.status === 'error' && uploadFile.error && (
                  <div className="mt-2">
                    <p className="text-xs text-red-600">{uploadFile.error}</p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}