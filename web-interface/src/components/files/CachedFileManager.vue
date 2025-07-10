<template>
  <div class="cached-file-manager">
    <!-- Cache status indicator -->
    <div
      v-if="showCacheStatus"
      class="cache-status"
    >
      <el-tag
        :type="fromCache ? 'success' : 'info'"
        size="small"
      >
        {{ fromCache ? (fresh ? 'Cache Hit (Fresh)' : 'Cache Hit (Stale)') : 'Network Request' }}
      </el-tag>
    </div>
    
    <!-- Search with deduplication -->
    <div class="search-section">
      <el-input
        v-model="searchQuery"
        placeholder="Search files with intelligent caching..."
        clearable
        size="large"
        :loading="searchLoading"
        @keyup.enter="performSearch"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
        <template #append>
          <el-button
            :loading="searchLoading"
            @click="performSearch"
          >
            Search
          </el-button>
        </template>
      </el-input>
      
      <div
        v-if="searchError"
        class="search-error"
      >
        <el-alert
          :title="searchError.message"
          type="error"
          :closable="false"
        />
      </div>
    </div>

    <!-- File list with cached pagination -->
    <div class="file-list-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>Files</h3>
            <div class="header-actions">
              <el-button
                :loading="loading"
                @click="refreshData"
              >
                <el-icon><Refresh /></el-icon>
                Refresh
              </el-button>
              <el-button @click="clearCache">
                <el-icon><Delete /></el-icon>
                Clear Cache
              </el-button>
            </div>
          </div>
        </template>

        <!-- Loading state -->
        <div
          v-if="loading"
          class="loading-container"
        >
          <el-skeleton
            :rows="5"
            animated
          />
        </div>

        <!-- Error state -->
        <div
          v-else-if="error"
          class="error-container"
        >
          <el-alert
            :title="error.message"
            type="error"
            show-icon
          >
            <template #default>
              <p>Failed to load files. Please try again.</p>
              <el-button
                type="primary"
                size="small"
                @click="fetchData"
              >
                Retry
              </el-button>
            </template>
          </el-alert>
        </div>

        <!-- File list -->
        <div
          v-else
          class="file-list"
        >
          <div
            v-for="file in data"
            :key="file.file_key"
            class="file-item"
            @click="selectFile(file)"
          >
            <div class="file-icon">
              <FileIcon
                :file="file"
                size="medium"
              />
            </div>
            <div class="file-info">
              <div class="file-name">
                {{ file.file_name }}
              </div>
              <div class="file-meta">
                <span class="file-size">{{ formatBytes(file.file_size) }}</span>
                <span class="file-date">{{ formatDate(file.uploaded_at) }}</span>
              </div>
            </div>
            <div class="file-status">
              <FileStatus :file="file" />
            </div>
          </div>

          <!-- Empty state -->
          <div
            v-if="data.length === 0"
            class="empty-state"
          >
            <el-empty description="No files found" />
          </div>
        </div>

        <!-- Pagination -->
        <div class="pagination-container">
          <el-pagination
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
            :total="totalItems"
            layout="total, sizes, prev, pager, next, jumper"
            :page-sizes="[10, 20, 50, 100]"
            @size-change="handleSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </el-card>
    </div>

    <!-- Cache statistics -->
    <div
      v-if="showCacheStats"
      class="cache-stats"
    >
      <el-card>
        <template #header>
          <h4>Cache Statistics</h4>
        </template>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">Cache Size:</span>
            <span class="stat-value">{{ cacheStats.cacheSize }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Pending Requests:</span>
            <span class="stat-value">{{ cacheStats.pendingRequests }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Hit Rate:</span>
            <span class="stat-value">{{ hitRate }}%</span>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref, computed } from 'vue'
import { useCachedPagination, useCachedSearch, useCache } from '@/composables/useCache'
import { Search, Refresh, Delete } from '@element-plus/icons-vue'
import FileIcon from './FileIcon.vue'
import FileStatus from './FileStatus.vue'

export default {
  name: 'CachedFileManager',
  components: {
    Search,
    Refresh,
    Delete,
    FileIcon,
    FileStatus
  },
  props: {
    showCacheStatus: {
      type: Boolean,
      default: true
    },
    showCacheStats: {
      type: Boolean,
      default: true
    }
  },
  setup() {
    // Cache management
    const { cacheStats, clearCache: clearAllCache } = useCache()
    
    // Cached pagination for files
    const {
      data,
      loading,
      error,
      currentPage,
      pageSize,
      totalPages,
      totalItems,
      goToPage,
      refresh: refreshData,
      fetchData
    } = useCachedPagination('/api/v1/files', {
      pageSize: 20,
      prefetch: true, // Enable prefetching of adjacent pages
      fetchOptions: {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
        }
      }
    })

    // Search with deduplication
    const searchQuery = ref('')
    const {
      results: searchResults,
      loading: searchLoading,
      error: searchError,
      search: performSearch
    } = useCachedSearch(async (query) => {
      // Simulate API call for search
      const response = await fetch(`/api/v1/files/search?q=${encodeURIComponent(query)}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
        }
      })
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      
      return response.json()
    })

    // Cache hit tracking
    const cacheHits = ref(0)
    const totalRequests = ref(0)
    const fromCache = ref(false)
    const fresh = ref(true)

    const hitRate = computed(() => {
      return totalRequests.value > 0 
        ? Math.round((cacheHits.value / totalRequests.value) * 100)
        : 0
    })

    // Utility functions
    const formatBytes = (bytes) => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleDateString()
    }

    const selectFile = (file) => {
      console.log('Selected file:', file)
    }

    const handlePageChange = (page) => {
      goToPage(page)
    }

    const handleSizeChange = (size) => {
      pageSize.value = size
      currentPage.value = 1
    }

    const clearCache = () => {
      clearAllCache()
      // Reset hit tracking
      cacheHits.value = 0
      totalRequests.value = 0
    }

    return {
      // Pagination
      data,
      loading,
      error,
      currentPage,
      pageSize,
      totalPages,
      totalItems,
      refreshData,
      fetchData,
      
      // Search
      searchQuery,
      searchResults,
      searchLoading,
      searchError,
      performSearch,
      
      // Cache
      cacheStats,
      hitRate,
      fromCache,
      fresh,
      clearCache,
      
      // Utils
      formatBytes,
      formatDate,
      selectFile,
      handlePageChange,
      handleSizeChange
    }
  }
}
</script>

<style scoped>
.cached-file-manager {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.cache-status {
  margin-bottom: 16px;
  text-align: right;
}

.search-section {
  margin-bottom: 20px;
}

.search-error {
  margin-top: 12px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.loading-container,
.error-container {
  padding: 20px 0;
}

.file-list {
  min-height: 400px;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.file-item:hover {
  background-color: var(--el-fill-color-lighter);
}

.file-item:last-child {
  border-bottom: none;
}

.file-icon {
  margin-right: 12px;
}

.file-info {
  flex: 1;
}

.file-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.file-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.file-status {
  margin-left: 12px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}

.empty-state {
  padding: 60px 0;
  text-align: center;
}

.cache-stats {
  margin-top: 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.stat-label {
  font-weight: 500;
  color: var(--el-text-color-regular);
}

.stat-value {
  font-weight: bold;
  color: var(--el-color-primary);
}

/* Dark mode adjustments */
.dark .file-item:hover {
  background-color: var(--el-fill-color-dark);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .cached-file-manager {
    padding: 12px;
  }
  
  .card-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }
  
  .file-item {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  
  .file-meta {
    flex-direction: column;
    gap: 4px;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
}
</style>