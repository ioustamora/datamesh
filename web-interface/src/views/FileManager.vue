<template>
  <div class="file-manager">
    <div class="file-manager-header">
      <div class="header-actions">
        <el-button type="primary" @click="showUploadDialog = true">
          <el-icon><Upload /></el-icon>
          Upload Files
        </el-button>
        <el-button @click="refreshFiles">
          <el-icon><Refresh /></el-icon>
          Refresh
        </el-button>
        <el-button @click="toggleView">
          <el-icon><Grid v-if="viewMode === 'list'" /><List v-else /></el-icon>
          {{ viewMode === 'list' ? 'Grid View' : 'List View' }}
        </el-button>
      </div>
      
      <div class="header-search">
        <el-input
          v-model="searchQuery"
          placeholder="Search files..."
          @input="searchFiles"
          clearable
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>
    </div>

    <!-- File Upload Dialog -->
    <el-dialog
      v-model="showUploadDialog"
      title="Upload Files"
      width="600px"
      @close="resetUpload"
    >
      <div class="upload-container">
        <div
          class="upload-drop-zone"
          :class="{ 'drag-over': isDragOver }"
          @drop="handleDrop"
          @dragover.prevent="isDragOver = true"
          @dragleave="isDragOver = false"
          @dragenter.prevent
        >
          <div class="drop-zone-content">
            <el-icon class="upload-icon"><UploadFilled /></el-icon>
            <p>Drag & drop files here or click to select</p>
            <el-button type="primary" @click="triggerFileSelect">
              Select Files
            </el-button>
          </div>
          <input
            ref="fileInput"
            type="file"
            multiple
            @change="handleFileSelect"
            style="display: none"
          />
        </div>
        
        <!-- Upload Queue -->
        <div v-if="uploadQueue.length > 0" class="upload-queue">
          <h4>Upload Queue</h4>
          <div v-for="(item, index) in uploadQueue" :key="index" class="upload-item">
            <div class="upload-item-info">
              <span class="file-name">{{ item.file.name }}</span>
              <span class="file-size">{{ formatFileSize(item.file.size) }}</span>
            </div>
            <div class="upload-item-progress">
              <el-progress
                :percentage="item.progress"
                :status="item.status"
                :stroke-width="6"
              />
              <el-button
                v-if="item.status === 'pending'"
                size="small"
                @click="removeFromQueue(index)"
              >
                Remove
              </el-button>
            </div>
          </div>
          
          <div class="upload-actions">
            <el-button type="primary" @click="startUpload" :disabled="isUploading">
              Start Upload
            </el-button>
            <el-button @click="clearQueue">Clear Queue</el-button>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- File List/Grid -->
    <div class="file-content">
      <el-loading v-loading="loading" element-loading-text="Loading files...">
        <div v-if="viewMode === 'list'" class="file-list">
          <el-table
            :data="filteredFiles"
            style="width: 100%"
            @selection-change="handleSelectionChange"
          >
            <el-table-column type="selection" width="55" />
            <el-table-column prop="name" label="Name" min-width="200">
              <template #default="{ row }">
                <div class="file-item">
                  <el-icon class="file-icon" :class="getFileIconClass(row.type)">
                    <component :is="getFileIcon(row.type)" />
                  </el-icon>
                  <span class="file-name">{{ row.name }}</span>
                </div>
              </template>
            </el-table-column>
            <el-table-column prop="size" label="Size" width="100">
              <template #default="{ row }">
                {{ formatFileSize(row.size) }}
              </template>
            </el-table-column>
            <el-table-column prop="type" label="Type" width="100" />
            <el-table-column prop="uploadDate" label="Upload Date" width="180">
              <template #default="{ row }">
                {{ formatDate(row.uploadDate) }}
              </template>
            </el-table-column>
            <el-table-column prop="status" label="Status" width="100">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="Actions" width="150">
              <template #default="{ row }">
                <el-button size="small" @click="downloadFile(row)">
                  <el-icon><Download /></el-icon>
                </el-button>
                <el-button size="small" @click="shareFile(row)">
                  <el-icon><Share /></el-icon>
                </el-button>
                <el-button size="small" type="danger" @click="deleteFile(row)">
                  <el-icon><Delete /></el-icon>
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
        
        <div v-else class="file-grid">
          <div
            v-for="file in filteredFiles"
            :key="file.id"
            class="file-card"
            @click="selectFile(file)"
            @dblclick="downloadFile(file)"
          >
            <div class="file-card-thumbnail">
              <el-icon class="file-icon-large" :class="getFileIconClass(file.type)">
                <component :is="getFileIcon(file.type)" />
              </el-icon>
              <img v-if="file.thumbnail" :src="file.thumbnail" class="thumbnail-image" />
            </div>
            <div class="file-card-info">
              <div class="file-name" :title="file.name">{{ file.name }}</div>
              <div class="file-meta">
                <span class="file-size">{{ formatFileSize(file.size) }}</span>
                <span class="file-date">{{ formatDate(file.uploadDate) }}</span>
              </div>
            </div>
            <div class="file-card-actions">
              <el-button size="small" @click.stop="downloadFile(file)">
                <el-icon><Download /></el-icon>
              </el-button>
              <el-button size="small" @click.stop="shareFile(file)">
                <el-icon><Share /></el-icon>
              </el-button>
              <el-dropdown @command="handleFileAction">
                <el-button size="small">
                  <el-icon><More /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item :command="{ action: 'info', file }">
                      File Info
                    </el-dropdown-item>
                    <el-dropdown-item :command="{ action: 'rename', file }">
                      Rename
                    </el-dropdown-item>
                    <el-dropdown-item :command="{ action: 'delete', file }" divided>
                      Delete
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </div>
        </div>
      </el-loading>
    </div>

    <!-- File Info Dialog -->
    <el-dialog
      v-model="showFileInfo"
      title="File Information"
      width="500px"
    >
      <div v-if="selectedFile" class="file-info">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="Name">{{ selectedFile.name }}</el-descriptions-item>
          <el-descriptions-item label="Size">{{ formatFileSize(selectedFile.size) }}</el-descriptions-item>
          <el-descriptions-item label="Type">{{ selectedFile.type }}</el-descriptions-item>
          <el-descriptions-item label="Upload Date">{{ formatDate(selectedFile.uploadDate) }}</el-descriptions-item>
          <el-descriptions-item label="Status">
            <el-tag :type="getStatusType(selectedFile.status)">
              {{ selectedFile.status }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="File ID">{{ selectedFile.id }}</el-descriptions-item>
          <el-descriptions-item label="Chunks">{{ selectedFile.chunks || 'N/A' }}</el-descriptions-item>
          <el-descriptions-item label="Redundancy">{{ selectedFile.redundancy || 'N/A' }}</el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>

    <!-- Batch Actions -->
    <div v-if="selectedFiles.length > 0" class="batch-actions">
      <el-button type="primary" @click="downloadSelected">
        <el-icon><Download /></el-icon>
        Download Selected ({{ selectedFiles.length }})
      </el-button>
      <el-button type="danger" @click="deleteSelected">
        <el-icon><Delete /></el-icon>
        Delete Selected
      </el-button>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useFilesStore } from '@/store/files'
import { useLoadingStore } from '@/store/loading'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Upload,
  UploadFilled,
  Download,
  Delete,
  Share,
  Refresh,
  Search,
  Grid,
  List,
  More,
  Document,
  Picture,
  VideoPlay,
  Headphones,
  Folder,
  Files
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'

export default {
  name: 'FileManager',
  components: {
    Upload,
    UploadFilled,
    Download,
    Delete,
    Share,
    Refresh,
    Search,
    Grid,
    List,
    More,
    Document,
    Picture,
    VideoPlay,
    Headphones,
    Folder,
    Files
  },
  setup() {
    const filesStore = useFilesStore()
    const loadingStore = useLoadingStore()
    
    // Reactive state
    const viewMode = ref('list')
    const searchQuery = ref('')
    const showUploadDialog = ref(false)
    const showFileInfo = ref(false)
    const selectedFile = ref(null)
    const selectedFiles = ref([])
    const uploadQueue = ref([])
    const isUploading = ref(false)
    const isDragOver = ref(false)
    const fileInput = ref(null)
    
    // Computed properties
    const loading = computed(() => loadingStore.isLoading)
    const files = computed(() => filesStore.files)
    const filteredFiles = computed(() => {
      if (!searchQuery.value) return files.value
      return files.value.filter(file =>
        file.name.toLowerCase().includes(searchQuery.value.toLowerCase())
      )
    })
    
    // Methods
    const refreshFiles = async () => {
      loadingStore.setLoading(true)
      try {
        await filesStore.fetchFiles()
      } finally {
        loadingStore.setLoading(false)
      }
    }
    
    const searchFiles = () => {
      // Search is handled by computed property
    }
    
    const toggleView = () => {
      viewMode.value = viewMode.value === 'list' ? 'grid' : 'list'
    }
    
    const triggerFileSelect = () => {
      fileInput.value.click()
    }
    
    const handleFileSelect = (event) => {
      const files = Array.from(event.target.files)
      addFilesToQueue(files)
    }
    
    const handleDrop = (event) => {
      event.preventDefault()
      isDragOver.value = false
      const files = Array.from(event.dataTransfer.files)
      addFilesToQueue(files)
    }
    
    const addFilesToQueue = (files) => {
      files.forEach(file => {
        uploadQueue.value.push({
          file,
          progress: 0,
          status: 'pending'
        })
      })
    }
    
    const removeFromQueue = (index) => {
      uploadQueue.value.splice(index, 1)
    }
    
    const clearQueue = () => {
      uploadQueue.value = []
    }
    
    const startUpload = async () => {
      isUploading.value = true
      
      for (const item of uploadQueue.value) {
        if (item.status === 'pending') {
          item.status = 'uploading'
          
          try {
            await filesStore.uploadFile(item.file, {
              onUploadProgress: (progressEvent) => {
                item.progress = Math.round(
                  (progressEvent.loaded * 100) / progressEvent.total
                )
              }
            })
            
            item.status = 'success'
            item.progress = 100
          } catch (error) {
            item.status = 'exception'
            ElMessage.error(`Failed to upload ${item.file.name}: ${error.message}`)
          }
        }
      }
      
      isUploading.value = false
      await refreshFiles()
    }
    
    const resetUpload = () => {
      uploadQueue.value = []
      isUploading.value = false
      isDragOver.value = false
    }
    
    const downloadFile = async (file) => {
      try {
        loadingStore.setLoading(true)
        await filesStore.downloadFile(file.id)
        ElMessage.success('File downloaded successfully')
      } catch (error) {
        ElMessage.error(`Download failed: ${error.message}`)
      } finally {
        loadingStore.setLoading(false)
      }
    }
    
    const shareFile = (file) => {
      // Generate shareable link
      const shareLink = `${window.location.origin}/share/${file.id}`
      navigator.clipboard.writeText(shareLink)
      ElMessage.success('Share link copied to clipboard')
    }
    
    const deleteFile = async (file) => {
      try {
        await ElMessageBox.confirm(
          `Are you sure you want to delete "${file.name}"?`,
          'Delete File',
          {
            confirmButtonText: 'Delete',
            cancelButtonText: 'Cancel',
            type: 'warning'
          }
        )
        
        await filesStore.deleteFile(file.id)
        ElMessage.success('File deleted successfully')
        await refreshFiles()
      } catch (error) {
        if (error !== 'cancel') {
          ElMessage.error(`Delete failed: ${error.message}`)
        }
      }
    }
    
    const selectFile = (file) => {
      selectedFile.value = file
    }
    
    const handleSelectionChange = (selection) => {
      selectedFiles.value = selection
    }
    
    const handleFileAction = async ({ action, file }) => {
      switch (action) {
        case 'info':
          selectedFile.value = file
          showFileInfo.value = true
          break
        case 'rename':
          // Implement rename functionality
          break
        case 'delete':
          await deleteFile(file)
          break
      }
    }
    
    const downloadSelected = async () => {
      for (const file of selectedFiles.value) {
        await downloadFile(file)
      }
    }
    
    const deleteSelected = async () => {
      try {
        await ElMessageBox.confirm(
          `Are you sure you want to delete ${selectedFiles.value.length} files?`,
          'Delete Files',
          {
            confirmButtonText: 'Delete',
            cancelButtonText: 'Cancel',
            type: 'warning'
          }
        )
        
        for (const file of selectedFiles.value) {
          await filesStore.deleteFile(file.id)
        }
        
        ElMessage.success('Files deleted successfully')
        selectedFiles.value = []
        await refreshFiles()
      } catch (error) {
        if (error !== 'cancel') {
          ElMessage.error(`Delete failed: ${error.message}`)
        }
      }
    }
    
    // Utility functions
    const formatFileSize = (bytes) => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }
    
    const formatDate = (date) => {
      return dayjs(date).format('YYYY-MM-DD HH:mm:ss')
    }
    
    const getFileIcon = (type) => {
      const typeMap = {
        'image': Picture,
        'video': VideoPlay,
        'audio': Headphones,
        'document': Document,
        'archive': Folder,
        'default': Files
      }
      return typeMap[type] || typeMap.default
    }
    
    const getFileIconClass = (type) => {
      const classMap = {
        'image': 'file-icon-image',
        'video': 'file-icon-video',
        'audio': 'file-icon-audio',
        'document': 'file-icon-document',
        'archive': 'file-icon-archive',
        'default': 'file-icon-default'
      }
      return classMap[type] || classMap.default
    }
    
    const getStatusType = (status) => {
      const typeMap = {
        'available': 'success',
        'uploading': 'warning',
        'error': 'danger',
        'pending': 'info'
      }
      return typeMap[status] || 'info'
    }
    
    // Lifecycle hooks
    onMounted(() => {
      refreshFiles()
    })
    
    onUnmounted(() => {
      // Cleanup
    })
    
    return {
      // State
      viewMode,
      searchQuery,
      showUploadDialog,
      showFileInfo,
      selectedFile,
      selectedFiles,
      uploadQueue,
      isUploading,
      isDragOver,
      fileInput,
      
      // Computed
      loading,
      files,
      filteredFiles,
      
      // Methods
      refreshFiles,
      searchFiles,
      toggleView,
      triggerFileSelect,
      handleFileSelect,
      handleDrop,
      removeFromQueue,
      clearQueue,
      startUpload,
      resetUpload,
      downloadFile,
      shareFile,
      deleteFile,
      selectFile,
      handleSelectionChange,
      handleFileAction,
      downloadSelected,
      deleteSelected,
      
      // Utilities
      formatFileSize,
      formatDate,
      getFileIcon,
      getFileIconClass,
      getStatusType
    }
  }
}
</script>

<style scoped>
.file-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.file-manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.header-search {
  width: 300px;
}

.file-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.upload-container {
  max-height: 600px;
  overflow-y: auto;
}

.upload-drop-zone {
  border: 2px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  margin-bottom: 20px;
  transition: all 0.3s ease;
}

.upload-drop-zone:hover,
.upload-drop-zone.drag-over {
  border-color: var(--el-color-primary);
  background-color: var(--el-fill-color-light);
}

.drop-zone-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.upload-icon {
  font-size: 48px;
  color: var(--el-color-info);
}

.upload-queue {
  margin-top: 20px;
}

.upload-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
  margin-bottom: 8px;
}

.upload-item-info {
  flex: 1;
}

.file-name {
  font-weight: 500;
  display: block;
}

.file-size {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.upload-item-progress {
  flex: 1;
  margin-left: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.upload-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-icon {
  font-size: 18px;
}

.file-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.file-card {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.file-card:hover {
  border-color: var(--el-color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.file-card-thumbnail {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 80px;
  margin-bottom: 12px;
  position: relative;
}

.file-icon-large {
  font-size: 48px;
}

.thumbnail-image {
  max-width: 100%;
  max-height: 100%;
  border-radius: 4px;
}

.file-card-info {
  margin-bottom: 12px;
}

.file-card-info .file-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 4px;
}

.file-meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.file-card-actions {
  display: flex;
  gap: 4px;
}

.batch-actions {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--el-bg-color);
  padding: 12px 20px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border: 1px solid var(--el-border-color-lighter);
  display: flex;
  gap: 8px;
}

.file-info {
  max-height: 400px;
  overflow-y: auto;
}

@media (max-width: 768px) {
  .file-manager-header {
    flex-direction: column;
    gap: 12px;
  }
  
  .header-search {
    width: 100%;
  }
  
  .file-grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  }
}
</style>