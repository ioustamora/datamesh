<template>
  <el-dialog
    v-model="visible"
    title=""
    :width="800"
    :show-close="false"
    :modal="true"
    class="global-search-dialog"
    @opened="focusInput"
    @close="handleClose"
  >
    <div class="global-search">
      <!-- Search Input -->
      <div class="search-header">
        <el-input
          ref="searchInput"
          v-model="searchQuery"
          size="large"
          placeholder="Search files, folders, users, or type a command..."
          :prefix-icon="Search"
          clearable
          @input="handleSearch"
          @keydown.enter="handleEnter"
          @keydown.down="navigateDown"
          @keydown.up="navigateUp"
          @keydown.esc="handleClose"
        />
        <el-button
          text
          :icon="Close"
          @click="handleClose"
          class="close-btn"
          aria-label="Close search"
        />
      </div>
      
      <!-- Search Suggestions/Commands -->
      <div v-if="showSuggestions" class="search-suggestions">
        <div class="suggestion-section">
          <h4>Quick Commands</h4>
          <div
            v-for="(command, index) in filteredCommands"
            :key="command.id"
            :class="['suggestion-item', { active: selectedIndex === index }]"
            @click="executeCommand(command)"
            @mouseenter="selectedIndex = index"
          >
            <el-icon class="suggestion-icon">
              <component :is="command.icon" />
            </el-icon>
            <div class="suggestion-content">
              <div class="suggestion-title">{{ command.title }}</div>
              <div class="suggestion-description">{{ command.description }}</div>
            </div>
            <div class="suggestion-shortcut">{{ command.shortcut }}</div>
          </div>
        </div>
      </div>
      
      <!-- Search Results -->
      <div v-if="searchResults.length > 0" class="search-results">
        <div class="results-section">
          <h4>Search Results ({{ searchResults.length }})</h4>
          <div
            v-for="(result, index) in searchResults"
            :key="result.id"
            :class="['result-item', { active: selectedIndex === filteredCommands.length + index }]"
            @click="openResult(result)"
            @mouseenter="selectedIndex = filteredCommands.length + index"
          >
            <el-icon class="result-icon">
              <component :is="getResultIcon(result.type)" />
            </el-icon>
            <div class="result-content">
              <div class="result-title" v-html="highlightText(result.title, searchQuery)"></div>
              <div class="result-path">{{ result.path }}</div>
              <div class="result-meta">
                {{ formatFileSize(result.size) }} • {{ formatDate(result.modified) }}
              </div>
            </div>
            <div class="result-type">{{ result.type }}</div>
          </div>
        </div>
      </div>
      
      <!-- No Results -->
      <div v-if="searchQuery && searchResults.length === 0 && !isSearching" class="no-results">
        <el-empty
          description="No results found"
          :image-size="80"
        >
          <template #description>
            <p>Try searching with different keywords or check your spelling.</p>
            <el-button type="primary" @click="clearSearch">Clear Search</el-button>
          </template>
        </el-empty>
      </div>
      
      <!-- Loading State -->
      <div v-if="isSearching" class="search-loading">
        <el-skeleton :rows="5" animated />
      </div>
      
      <!-- Search Tips -->
      <div v-if="!searchQuery" class="search-tips">
        <h4>Search Tips</h4>
        <div class="tips-grid">
          <div class="tip-item">
            <el-icon class="tip-icon"><Document /></el-icon>
            <div>
              <div class="tip-title">Files & Folders</div>
              <div class="tip-description">Search by name, extension, or content</div>
            </div>
          </div>
          <div class="tip-item">
            <el-icon class="tip-icon"><User /></el-icon>
            <div>
              <div class="tip-title">Users</div>
              <div class="tip-description">Find users by name or email</div>
            </div>
          </div>
          <div class="tip-item">
            <el-icon class="tip-icon"><Setting /></el-icon>
            <div>
              <div class="tip-title">Commands</div>
              <div class="tip-description">Type commands like "create folder"</div>
            </div>
          </div>
          <div class="tip-item">
            <el-icon class="tip-icon"><Calendar /></el-icon>
            <div>
              <div class="tip-title">Filters</div>
              <div class="tip-description">Use "modified:today" or "size:>1MB"</div>
            </div>
          </div>
        </div>
        
        <div class="recent-searches" v-if="recentSearches.length > 0">
          <h5>Recent Searches</h5>
          <div class="recent-search-list">
            <el-tag
              v-for="recent in recentSearches"
              :key="recent"
              @click="searchQuery = recent"
              class="recent-search-tag"
              closable
              @close="removeRecentSearch(recent)"
            >
              {{ recent }}
            </el-tag>
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  Search, Close, Document, Folder, User, Setting, Calendar,
  Upload, Download, Picture, VideoCamera, Headphones
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useFilesStore } from '../../store/files'
import { debounce } from '../../utils/helpers'

export default {
  name: 'GlobalSearch',
  setup() {
    const router = useRouter()
    const filesStore = useFilesStore()
    
    const visible = ref(false)
    const searchQuery = ref('')
    const searchInput = ref(null)
    const selectedIndex = ref(0)
    const isSearching = ref(false)
    const searchResults = ref([])
    const recentSearches = ref([])
    
    const commands = [
      {
        id: 'upload',
        title: 'Upload File',
        description: 'Upload a new file to the network',
        icon: Upload,
        shortcut: 'Ctrl+U',
        action: () => router.push('/files?action=upload')
      },
      {
        id: 'new-folder',
        title: 'Create Folder',
        description: 'Create a new folder',
        icon: Folder,
        shortcut: 'Ctrl+Shift+N',
        action: () => createFolder()
      },
      {
        id: 'settings',
        title: 'Settings',
        description: 'Open application settings',
        icon: Setting,
        shortcut: 'Ctrl+,',
        action: () => router.push('/settings')
      },
      {
        id: 'profile',
        title: 'Profile',
        description: 'View and edit your profile',
        icon: User,
        shortcut: 'Ctrl+P',
        action: () => router.push('/profile')
      }
    ]
    
    const filteredCommands = computed(() => {
      if (!searchQuery.value) return commands
      
      return commands.filter(command =>
        command.title.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        command.description.toLowerCase().includes(searchQuery.value.toLowerCase())
      )
    })
    
    const showSuggestions = computed(() => {
      return !searchQuery.value || filteredCommands.value.length > 0
    })
    
    const totalItems = computed(() => {
      return filteredCommands.value.length + searchResults.value.length
    })
    
    // Debounced search function
    const debouncedSearch = debounce(async (query) => {
      if (!query.trim()) {
        searchResults.value = []
        return
      }
      
      isSearching.value = true
      
      try {
        // Simulate API call - replace with actual search API
        await new Promise(resolve => setTimeout(resolve, 300))
        
        // Mock search results
        searchResults.value = [
          {
            id: 1,
            title: 'Project Documentation.pdf',
            path: '/documents/projects/',
            type: 'file',
            size: 2456789,
            modified: new Date(2024, 0, 15),
            fileType: 'pdf'
          },
          {
            id: 2,
            title: 'Images',
            path: '/media/',
            type: 'folder',
            size: 0,
            modified: new Date(2024, 0, 20)
          }
        ].filter(item =>
          item.title.toLowerCase().includes(query.toLowerCase())
        )
        
        // Add to recent searches
        if (query.trim() && !recentSearches.value.includes(query)) {
          recentSearches.value.unshift(query)
          if (recentSearches.value.length > 5) {
            recentSearches.value.pop()
          }
          saveRecentSearches()
        }
      } catch (error) {
        console.error('Search error:', error)
        ElMessage.error('Search failed. Please try again.')
      } finally {
        isSearching.value = false
      }
    }, 300)
    
    const handleSearch = (query) => {
      selectedIndex.value = 0
      debouncedSearch(query)
    }
    
    const handleEnter = () => {
      if (totalItems.value === 0) return
      
      if (selectedIndex.value < filteredCommands.value.length) {
        executeCommand(filteredCommands.value[selectedIndex.value])
      } else {
        const resultIndex = selectedIndex.value - filteredCommands.value.length
        openResult(searchResults.value[resultIndex])
      }
    }
    
    const navigateDown = () => {
      if (selectedIndex.value < totalItems.value - 1) {
        selectedIndex.value++
      }
    }
    
    const navigateUp = () => {
      if (selectedIndex.value > 0) {
        selectedIndex.value--
      }
    }
    
    const executeCommand = (command) => {
      command.action()
      handleClose()
    }
    
    const openResult = (result) => {
      if (result.type === 'folder') {
        router.push(`/files?path=${encodeURIComponent(result.path)}`)
      } else {
        // Open file preview or download
        ElMessage.info(`Opening ${result.title}`)
      }
      handleClose()
    }
    
    const createFolder = () => {
      ElMessage.info('Creating new folder...')
      // Implement folder creation
    }
    
    const getResultIcon = (type) => {
      switch (type) {
        case 'folder': return Folder
        case 'image': return Picture
        case 'video': return VideoCamera
        case 'audio': return Headphones
        default: return Document
      }
    }
    
    const highlightText = (text, query) => {
      if (!query) return text
      
      const regex = new RegExp(`(${query})`, 'gi')
      return text.replace(regex, '<mark>$1</mark>')
    }
    
    const formatFileSize = (bytes) => {
      if (bytes === 0) return '0 B'
      
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      
      return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
    }
    
    const formatDate = (date) => {
      return new Intl.DateTimeFormat('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric'
      }).format(date)
    }
    
    const clearSearch = () => {
      searchQuery.value = ''
      searchResults.value = []
      selectedIndex.value = 0
    }
    
    const removeRecentSearch = (search) => {
      const index = recentSearches.value.indexOf(search)
      if (index > -1) {
        recentSearches.value.splice(index, 1)
        saveRecentSearches()
      }
    }
    
    const saveRecentSearches = () => {
      localStorage.setItem('datamesh-recent-searches', JSON.stringify(recentSearches.value))
    }
    
    const loadRecentSearches = () => {
      const stored = localStorage.getItem('datamesh-recent-searches')
      if (stored) {
        try {
          recentSearches.value = JSON.parse(stored)
        } catch (error) {
          console.error('Failed to load recent searches:', error)
        }
      }
    }
    
    const focusInput = () => {
      nextTick(() => {
        searchInput.value?.focus()
      })
    }
    
    const handleClose = () => {
      visible.value = false
      clearSearch()
    }
    
    const openSearch = () => {
      visible.value = true
    }
    
    // Global keyboard shortcut
    const handleGlobalKeydown = (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
        event.preventDefault()
        openSearch()
      }
    }
    
    // Global search event listener
    const handleGlobalSearchEvent = () => {
      openSearch()
    }
    
    onMounted(() => {
      loadRecentSearches()
      document.addEventListener('keydown', handleGlobalKeydown)
      document.addEventListener('open-global-search', handleGlobalSearchEvent)
    })
    
    onUnmounted(() => {
      document.removeEventListener('keydown', handleGlobalKeydown)
      document.removeEventListener('open-global-search', handleGlobalSearchEvent)
    })
    
    return {
      visible,
      searchQuery,
      searchInput,
      selectedIndex,
      isSearching,
      searchResults,
      recentSearches,
      filteredCommands,
      showSuggestions,
      handleSearch,
      handleEnter,
      navigateDown,
      navigateUp,
      executeCommand,
      openResult,
      getResultIcon,
      highlightText,
      formatFileSize,
      formatDate,
      clearSearch,
      removeRecentSearch,
      focusInput,
      handleClose,
      openSearch,
      Search,
      Close,
      Document,
      Folder,
      User,
      Setting,
      Calendar,
      Upload,
      Download,
      Picture,
      VideoCamera,
      Headphones
    }
  }
}
</script>

<style scoped>
.global-search-dialog {
  --el-dialog-padding-primary: 0;
}

.global-search {
  max-height: 70vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.search-header {
  display: flex;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.search-header .el-input {
  flex: 1;
  margin-right: 12px;
}

.close-btn {
  color: var(--el-text-color-secondary);
}

.search-suggestions,
.search-results,
.search-tips {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.suggestion-section,
.results-section {
  margin-bottom: 16px;
}

.suggestion-section h4,
.results-section h4,
.search-tips h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
}

.suggestion-item,
.result-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.suggestion-item:hover,
.result-item:hover,
.suggestion-item.active,
.result-item.active {
  background-color: var(--el-fill-color-light);
}

.suggestion-icon,
.result-icon {
  margin-right: 12px;
  color: var(--el-text-color-secondary);
  font-size: 20px;
}

.suggestion-content,
.result-content {
  flex: 1;
}

.suggestion-title,
.result-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.suggestion-description {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.result-path,
.result-meta {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 2px;
}

.suggestion-shortcut,
.result-type {
  font-size: 11px;
  color: var(--el-text-color-placeholder);
  background-color: var(--el-fill-color);
  padding: 2px 6px;
  border-radius: 4px;
}

.no-results {
  padding: 40px 20px;
  text-align: center;
}

.search-loading {
  padding: 20px;
}

.tips-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 24px;
}

.tip-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.tip-icon {
  margin-top: 2px;
  color: var(--el-color-primary);
}

.tip-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.tip-description {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.recent-searches h5 {
  margin: 0 0 8px 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
}

.recent-search-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.recent-search-tag {
  cursor: pointer;
}

/* Highlight text */
:deep(mark) {
  background-color: var(--el-color-primary-light-8);
  color: var(--el-color-primary);
  padding: 0 2px;
  border-radius: 2px;
}

@media (max-width: 768px) {
  .tips-grid {
    grid-template-columns: 1fr;
  }
}
</style>