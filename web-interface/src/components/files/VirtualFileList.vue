<template>
  <div class="virtual-file-list">
    <!-- File list header -->
    <div class="file-list-header">
      <div class="header-actions">
        <el-button 
          type="primary" 
          :aria-label="'Upload files'"
          @click="$emit('upload-click')"
        >
          <el-icon><Upload /></el-icon>
          Upload Files
        </el-button>
        
        <el-button 
          :aria-label="'Refresh file list'"
          @click="$emit('refresh')"
        >
          <el-icon><Refresh /></el-icon>
          Refresh
        </el-button>
        
        <el-button 
          :aria-label="`Switch to ${viewMode === 'list' ? 'grid' : 'list'} view`"
          @click="toggleView"
        >
          <el-icon><Grid v-if="viewMode === 'list'" /><List v-else /></el-icon>
          {{ viewMode === 'list' ? 'Grid View' : 'List View' }}
        </el-button>
        
        <!-- Selection actions -->
        <div
          v-if="selectedFiles.length > 0"
          class="selection-actions"
        >
          <el-button 
            type="warning" 
            :aria-label="`Download ${selectedFiles.length} selected files`"
            @click="downloadSelected"
          >
            <el-icon><Download /></el-icon>
            Download ({{ selectedFiles.length }})
          </el-button>
          
          <el-button 
            type="danger" 
            :aria-label="`Delete ${selectedFiles.length} selected files`"
            @click="deleteSelected"
          >
            <el-icon><Delete /></el-icon>
            Delete ({{ selectedFiles.length }})
          </el-button>
        </div>
      </div>
      
      <!-- Search and filters -->
      <div class="header-filters">
        <el-input
          v-model="searchQuery"
          placeholder="Search files..."
          clearable
          class="search-input"
          :aria-label="'Search files'"
          @input="handleSearch"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        
        <el-select
          v-model="filterType"
          placeholder="File Type"
          clearable
          class="type-filter"
          :aria-label="'Filter by file type'"
          @change="handleFilter"
        >
          <el-option
            label="All Types"
            value=""
          />
          <el-option
            label="Images"
            value="image"
          />
          <el-option
            label="Documents"
            value="document"
          />
          <el-option
            label="Videos"
            value="video"
          />
          <el-option
            label="Audio"
            value="audio"
          />
          <el-option
            label="Archives"
            value="archive"
          />
          <el-option
            label="Code"
            value="code"
          />
        </el-select>
        
        <el-select
          v-model="sortBy"
          class="sort-select"
          :aria-label="'Sort files'"
          @change="handleSort"
        >
          <el-option
            label="Name"
            value="name"
          />
          <el-option
            label="Size"
            value="size"
          />
          <el-option
            label="Date"
            value="date"
          />
          <el-option
            label="Type"
            value="type"
          />
        </el-select>
        
        <el-button
          :aria-label="`Sort ${sortOrder === 'asc' ? 'descending' : 'ascending'}`"
          @click="toggleSortOrder"
        >
          <el-icon><Sort /></el-icon>
          {{ sortOrder === 'asc' ? '↑' : '↓' }}
        </el-button>
      </div>
    </div>
    
    <!-- File count and stats -->
    <div
      class="file-stats"
      role="status"
      :aria-live="'polite'"
    >
      <span class="total-files">
        {{ filteredFiles.length }} files
        <span v-if="filteredFiles.length !== totalFiles">({{ totalFiles }} total)</span>
      </span>
      <span class="total-size">{{ formatBytes(totalSize) }}</span>
    </div>
    
    <!-- Virtual scroller -->
    <VirtualScroller
      :items="filteredFiles"
      :item-height="viewMode === 'list' ? 60 : 120"
      :container-height="'calc(100vh - 300px)'"
      :buffer="10"
      :infinite-scroll="true"
      :loading="loading"
      item-key="file_key"
      :aria-label="`File list with ${filteredFiles.length} files`"
      @load-more="loadMore"
      @item-click="handleItemClick"
      @selection-change="handleSelectionChange"
    >
      <template #default="{ item: file, index }">
        <div
          class="file-item"
          :class="{
            'file-item-list': viewMode === 'list',
            'file-item-grid': viewMode === 'grid',
            'file-item-selected': isSelected(file)
          }"
          :aria-label="`File: ${file.file_name}, ${formatBytes(file.file_size)}, uploaded ${formatDate(file.uploaded_at)}`"
          role="option"
          :aria-selected="isSelected(file)"
          tabindex="0"
          @click="toggleSelection(file)"
          @dblclick="openFile(file)"
          @contextmenu.prevent="showContextMenu(file, $event)"
          @keydown="handleKeydown($event, file)"
        >
          <!-- List view -->
          <template v-if="viewMode === 'list'">
            <div class="file-checkbox">
              <el-checkbox
                :model-value="isSelected(file)"
                :aria-label="`Select ${file.file_name}`"
                @change="toggleSelection(file)"
              />
            </div>
            
            <div class="file-icon-container">
              <FileIcon
                :file="file"
                size="medium"
              />
              <div
                v-if="file.is_shared"
                class="shared-indicator"
                title="Shared file"
              >
                <el-icon><Share /></el-icon>
              </div>
            </div>
            
            <div class="file-info">
              <div
                class="file-name"
                :title="file.file_name"
              >
                {{ file.file_name }}
              </div>
              <div class="file-meta">
                <span class="file-size">{{ formatBytes(file.file_size) }}</span>
                <span class="file-date">{{ formatDate(file.uploaded_at) }}</span>
                <span class="file-type">{{ getFileType(file.file_name) }}</span>
              </div>
            </div>
            
            <div class="file-status">
              <FileStatus :file="file" />
            </div>
            
            <div class="file-actions">
              <el-button-group>
                <el-button 
                  v-if="canPreview(file)" 
                  size="small"
                  :aria-label="`Preview ${file.file_name}`"
                  @click.stop="previewFile(file)"
                >
                  <el-icon><View /></el-icon>
                </el-button>
                
                <el-button 
                  size="small" 
                  :aria-label="`Download ${file.file_name}`"
                  @click.stop="downloadFile(file)"
                >
                  <el-icon><Download /></el-icon>
                </el-button>
                
                <el-button 
                  size="small" 
                  :aria-label="`Share ${file.file_name}`"
                  @click.stop="shareFile(file)"
                >
                  <el-icon><Share /></el-icon>
                </el-button>
                
                <el-dropdown
                  trigger="click"
                  @command="handleFileAction"
                >
                  <el-button
                    size="small"
                    :aria-label="`More actions for ${file.file_name}`"
                  >
                    <el-icon><More /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item :command="{ action: 'info', file }">
                        <el-icon><InfoFilled /></el-icon>
                        File Info
                      </el-dropdown-item>
                      <el-dropdown-item :command="{ action: 'rename', file }">
                        <el-icon><Edit /></el-icon>
                        Rename
                      </el-dropdown-item>
                      <el-dropdown-item :command="{ action: 'move', file }">
                        <el-icon><FolderOpened /></el-icon>
                        Move
                      </el-dropdown-item>
                      <el-dropdown-item :command="{ action: 'copy', file }">
                        <el-icon><CopyDocument /></el-icon>
                        Copy Link
                      </el-dropdown-item>
                      <el-dropdown-item 
                        :command="{ action: 'delete', file }" 
                        divided
                        class="danger-item"
                      >
                        <el-icon><Delete /></el-icon>
                        Delete
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </el-button-group>
            </div>
          </template>
          
          <!-- Grid view -->
          <template v-else>
            <div class="file-card">
              <div class="file-card-header">
                <el-checkbox
                  :model-value="isSelected(file)"
                  class="file-checkbox-grid"
                  :aria-label="`Select ${file.file_name}`"
                  @change="toggleSelection(file)"
                />
                <div
                  v-if="file.is_shared"
                  class="shared-indicator"
                  title="Shared file"
                >
                  <el-icon><Share /></el-icon>
                </div>
              </div>
              
              <div class="file-card-thumbnail">
                <FileIcon
                  :file="file"
                  size="large"
                />
                <FileThumbnail
                  v-if="canPreview(file)"
                  :file="file"
                />
              </div>
              
              <div class="file-card-info">
                <div
                  class="file-name"
                  :title="file.file_name"
                >
                  {{ file.file_name }}
                </div>
                <div class="file-meta">
                  <span class="file-size">{{ formatBytes(file.file_size) }}</span>
                  <span class="file-date">{{ formatDate(file.uploaded_at) }}</span>
                </div>
                <FileStatus :file="file" />
              </div>
              
              <div class="file-card-actions">
                <el-button 
                  v-if="canPreview(file)" 
                  size="small"
                  :aria-label="`Preview ${file.file_name}`"
                  @click.stop="previewFile(file)"
                >
                  <el-icon><View /></el-icon>
                </el-button>
                
                <el-button 
                  size="small" 
                  :aria-label="`Download ${file.file_name}`"
                  @click.stop="downloadFile(file)"
                >
                  <el-icon><Download /></el-icon>
                </el-button>
                
                <el-dropdown
                  trigger="click"
                  @command="handleFileAction"
                >
                  <el-button
                    size="small"
                    :aria-label="`More actions for ${file.file_name}`"
                  >
                    <el-icon><More /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item :command="{ action: 'share', file }">
                        <el-icon><Share /></el-icon>
                        Share
                      </el-dropdown-item>
                      <el-dropdown-item :command="{ action: 'info', file }">
                        <el-icon><InfoFilled /></el-icon>
                        Info
                      </el-dropdown-item>
                      <el-dropdown-item :command="{ action: 'rename', file }">
                        <el-icon><Edit /></el-icon>
                        Rename
                      </el-dropdown-item>
                      <el-dropdown-item 
                        :command="{ action: 'delete', file }" 
                        divided
                        class="danger-item"
                      >
                        <el-icon><Delete /></el-icon>
                        Delete
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>
          </template>
        </div>
      </template>
      
      <template #empty>
        <div class="empty-state">
          <el-empty description="No files found">
            <el-button
              type="primary"
              @click="$emit('upload-click')"
            >
              Upload Your First File
            </el-button>
          </el-empty>
        </div>
      </template>
    </VirtualScroller>
  </div>
</template>

<script>
import { ref, computed, watch } from 'vue'
import { useCachedPagination, useCachedSearch } from '@/composables/useCache'
import VirtualScroller from '@/components/common/VirtualScroller.vue'
import FileIcon from '@/components/files/FileIcon.vue'
import FileStatus from '@/components/files/FileStatus.vue'
import FileThumbnail from '@/components/files/FileThumbnail.vue'
import { 
  Upload, Refresh, Grid, List, Search, Sort, Download, Delete, 
  Share, More, View, InfoFilled, Edit, FolderOpened, CopyDocument 
} from '@element-plus/icons-vue'

export default {
  name: 'VirtualFileList',
  components: {
    VirtualScroller,
    FileIcon,
    FileStatus,
    FileThumbnail,
    Upload,
    Refresh,
    Grid,
    List,
    Search,
    Sort,
    Download,
    Delete,
    Share,
    More,
    View,
    InfoFilled,
    Edit,
    FolderOpened,
    CopyDocument
  },
  props: {
    files: {
      type: Array,
      required: true
    },
    loading: {
      type: Boolean,
      default: false
    },
    totalFiles: {
      type: Number,
      default: 0
    },
    totalSize: {
      type: Number,
      default: 0
    }
  },
  emits: [
    'upload-click',
    'refresh',
    'download',
    'share',
    'delete',
    'preview',
    'load-more',
    'search',
    'filter',
    'sort'
  ],
  setup(props, { emit }) {
    // Reactive state
    const viewMode = ref('list')
    const searchQuery = ref('')
    const filterType = ref('')
    const sortBy = ref('name')
    const sortOrder = ref('asc')
    const selectedFiles = ref([])
    
    // Cached search functionality
    const { 
      results: searchResults, 
      loading: searchLoading, 
      error: searchError,
      search: performSearch 
    } = useCachedSearch(async (query) => {
      // Emit search event to parent component
      emit('search', query)
      
      // Return filtered results based on current files
      return props.files.filter(file => 
        file.file_name.toLowerCase().includes(query.toLowerCase()) ||
        getFileType(file.file_name).toLowerCase().includes(query.toLowerCase())
      )
    })
    
    // File filtering and sorting
    const filteredFiles = computed(() => {
      // Use cached search results if available
      let result = searchQuery.value && searchResults.value.length > 0 
        ? [...searchResults.value] 
        : [...props.files]
      
      // Type filter
      if (filterType.value) {
        result = result.filter(file => {
          const type = getFileType(file.file_name)
          return getFileCategory(type) === filterType.value
        })
      }
      
      // Sort
      result.sort((a, b) => {
        let aVal, bVal
        
        switch (sortBy.value) {
          case 'name':
            aVal = a.file_name.toLowerCase()
            bVal = b.file_name.toLowerCase()
            break
          case 'size':
            aVal = a.file_size
            bVal = b.file_size
            break
          case 'date':
            aVal = new Date(a.uploaded_at)
            bVal = new Date(b.uploaded_at)
            break
          case 'type':
            aVal = getFileType(a.file_name)
            bVal = getFileType(b.file_name)
            break
          default:
            return 0
        }
        
        if (aVal < bVal) return sortOrder.value === 'asc' ? -1 : 1
        if (aVal > bVal) return sortOrder.value === 'asc' ? 1 : -1
        return 0
      })
      
      return result
    })
    
    // Methods
    const toggleView = () => {
      viewMode.value = viewMode.value === 'list' ? 'grid' : 'list'
    }
    
    const toggleSortOrder = () => {
      sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
      handleSort()
    }
    
    const handleSearch = () => {
      emit('search', searchQuery.value)
    }
    
    const handleFilter = () => {
      emit('filter', filterType.value)
    }
    
    const handleSort = () => {
      emit('sort', { by: sortBy.value, order: sortOrder.value })
    }
    
    const isSelected = (file) => {
      return selectedFiles.value.some(f => f.file_key === file.file_key)
    }
    
    const toggleSelection = (file) => {
      const index = selectedFiles.value.findIndex(f => f.file_key === file.file_key)
      if (index >= 0) {
        selectedFiles.value.splice(index, 1)
      } else {
        selectedFiles.value.push(file)
      }
    }
    
    const handleItemClick = ({ item, index }) => {
      toggleSelection(item)
    }
    
    const handleSelectionChange = ({ item, index }) => {
      // Handle keyboard navigation selection
    }
    
    const handleKeydown = (event, file) => {
      switch (event.key) {
        case 'Enter':
        case ' ':
          event.preventDefault()
          toggleSelection(file)
          break
        case 'Delete':
          event.preventDefault()
          if (isSelected(file)) {
            deleteSelected()
          } else {
            emit('delete', [file])
          }
          break
      }
    }
    
    const openFile = (file) => {
      if (canPreview(file)) {
        previewFile(file)
      } else {
        downloadFile(file)
      }
    }
    
    const previewFile = (file) => {
      emit('preview', file)
    }
    
    const downloadFile = (file) => {
      emit('download', file)
    }
    
    const shareFile = (file) => {
      emit('share', file)
    }
    
    const downloadSelected = () => {
      emit('download', selectedFiles.value)
    }
    
    const deleteSelected = () => {
      emit('delete', selectedFiles.value)
      selectedFiles.value = []
    }
    
    const handleFileAction = (command) => {
      const { action, file } = command
      
      switch (action) {
        case 'info':
          // Show file info dialog
          break
        case 'rename':
          // Show rename dialog
          break
        case 'move':
          // Show move dialog
          break
        case 'copy':
          // Copy file link to clipboard
          break
        case 'share':
          shareFile(file)
          break
        case 'delete':
          emit('delete', [file])
          break
      }
    }
    
    const showContextMenu = (file, event) => {
      // Show context menu at cursor position
      // Implementation would depend on your context menu component
    }
    
    const loadMore = () => {
      emit('load-more')
    }
    
    // Utility functions
    const getFileType = (filename) => {
      const ext = filename.split('.').pop()?.toLowerCase()
      return ext || 'unknown'
    }
    
    const getFileCategory = (type) => {
      const categories = {
        image: ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp'],
        document: ['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt'],
        video: ['mp4', 'avi', 'mov', 'wmv', 'flv', 'webm'],
        audio: ['mp3', 'wav', 'flac', 'aac', 'ogg'],
        archive: ['zip', 'rar', '7z', 'tar', 'gz'],
        code: ['js', 'ts', 'py', 'java', 'cpp', 'html', 'css']
      }
      
      for (const [category, extensions] of Object.entries(categories)) {
        if (extensions.includes(type)) {
          return category
        }
      }
      
      return 'other'
    }
    
    const canPreview = (file) => {
      const type = getFileType(file.file_name)
      const previewableTypes = ['jpg', 'jpeg', 'png', 'gif', 'svg', 'pdf', 'txt', 'mp4', 'mp3']
      return previewableTypes.includes(type)
    }
    
    const formatBytes = (bytes) => {
      if (bytes === 0) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }
    
    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleDateString()
    }
    
    return {
      // State
      viewMode,
      searchQuery,
      filterType,
      sortBy,
      sortOrder,
      selectedFiles,
      
      // Cached search
      searchResults,
      searchLoading,
      searchError,
      performSearch,
      
      // Computed
      filteredFiles,
      
      // Methods
      toggleView,
      toggleSortOrder,
      handleSearch,
      handleFilter,
      handleSort,
      isSelected,
      toggleSelection,
      handleItemClick,
      handleSelectionChange,
      handleKeydown,
      openFile,
      previewFile,
      downloadFile,
      shareFile,
      downloadSelected,
      deleteSelected,
      handleFileAction,
      showContextMenu,
      loadMore,
      getFileType,
      getFileCategory,
      canPreview,
      formatBytes,
      formatDate
    }
  }
}
</script>

<style scoped>
.virtual-file-list {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.file-list-header {
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.selection-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  padding: 0 12px;
  border-left: 1px solid var(--el-border-color);
}

.header-filters {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.search-input {
  min-width: 200px;
  flex: 1;
}

.type-filter,
.sort-select {
  min-width: 120px;
}

.file-stats {
  padding: 8px 16px;
  background: var(--el-fill-color-lighter);
  border-bottom: 1px solid var(--el-border-color-lighter);
  font-size: 14px;
  color: var(--el-text-color-secondary);
  display: flex;
  justify-content: space-between;
}

/* File item styles */
.file-item {
  border-bottom: 1px solid var(--el-border-color-lighter);
  transition: all 0.2s ease;
  cursor: pointer;
}

.file-item:hover {
  background: var(--el-fill-color-light);
}

.file-item:focus {
  outline: 2px solid var(--el-color-primary);
  outline-offset: -2px;
}

.file-item-selected {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-5);
}

/* List view styles */
.file-item-list {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  gap: 16px;
  min-height: 60px;
}

.file-checkbox {
  flex-shrink: 0;
}

.file-icon-container {
  position: relative;
  flex-shrink: 0;
}

.shared-indicator {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 16px;
  height: 16px;
  background: var(--el-color-success);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: white;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.file-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.file-status {
  flex-shrink: 0;
}

.file-actions {
  flex-shrink: 0;
}

/* Grid view styles */
.file-item-grid {
  padding: 8px;
}

.file-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 12px;
  height: 100%;
  display: flex;
  flex-direction: column;
  transition: all 0.2s ease;
}

.file-card:hover {
  border-color: var(--el-color-primary-light-5);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.file-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.file-checkbox-grid {
  flex-shrink: 0;
}

.file-card-thumbnail {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  position: relative;
}

.file-card-info {
  margin-bottom: 8px;
}

.file-card-info .file-name {
  font-size: 14px;
  margin-bottom: 4px;
}

.file-card-info .file-meta {
  font-size: 11px;
  flex-direction: column;
  gap: 2px;
}

.file-card-actions {
  display: flex;
  justify-content: center;
  gap: 4px;
  margin-top: auto;
}

/* Empty state */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 200px;
}

/* Dropdown styles */
.danger-item {
  color: var(--el-color-danger);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .header-actions,
  .header-filters {
    flex-direction: column;
    align-items: stretch;
  }
  
  .search-input {
    min-width: auto;
  }
  
  .file-item-list {
    padding: 8px 12px;
    gap: 8px;
  }
  
  .file-meta {
    flex-direction: column;
    gap: 2px;
  }
  
  .file-actions .el-button-group {
    flex-direction: column;
  }
  
  .selection-actions {
    margin-left: 0;
    padding-left: 0;
    border-left: none;
    border-top: 1px solid var(--el-border-color);
    padding-top: 8px;
    margin-top: 8px;
  }
}

/* Dark mode adjustments */
.dark .file-item:hover {
  background: var(--el-fill-color-dark);
}

.dark .file-card {
  background: var(--el-bg-color-overlay);
}

.dark .file-card:hover {
  border-color: var(--el-color-primary);
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .file-item {
    border-bottom-width: 2px;
  }
  
  .file-card {
    border-width: 2px;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .file-item,
  .file-card {
    transition: none;
  }
}
</style>