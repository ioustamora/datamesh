<template>
  <div class="quick-actions">
    <el-popover
      placement="bottom-end"
      :width="320"
      trigger="click"
      popper-class="quick-actions-popover"
    >
      <template #reference>
        <el-button
          type="primary"
          circle
          size="large"
          class="quick-actions-trigger"
          aria-label="Quick Actions"
          :icon="Plus"
        />
      </template>
      
      <div class="quick-actions-content">
        <h4 class="quick-actions-title">Quick Actions</h4>
        
        <div class="quick-actions-grid">
          <el-button
            v-for="action in quickActions"
            :key="action.id"
            :type="action.type"
            :icon="action.icon"
            class="quick-action-btn"
            @click="handleAction(action)"
          >
            {{ action.label }}
          </el-button>
        </div>
        
        <el-divider />
        
        <div class="recent-actions">
          <h5>Recent Actions</h5>
          <div class="recent-actions-list">
            <div
              v-for="recent in recentActions"
              :key="recent.id"
              class="recent-action-item"
              @click="handleRecentAction(recent)"
            >
              <el-icon class="recent-action-icon">
                <component :is="recent.icon" />
              </el-icon>
              <span class="recent-action-text">{{ recent.label }}</span>
              <span class="recent-action-time">{{ formatTime(recent.timestamp) }}</span>
            </div>
          </div>
        </div>
      </div>
    </el-popover>
  </div>
</template>

<script>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Upload, Download, Search, Setting, Document, Folder } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useFilesStore } from '../../store/files'

export default {
  name: 'QuickActions',
  setup() {
    const router = useRouter()
    const filesStore = useFilesStore()
    
    const recentActions = ref([])
    
    const quickActions = computed(() => [
      {
        id: 'upload',
        label: 'Upload File',
        icon: Upload,
        type: 'primary',
        action: () => router.push('/files?action=upload')
      },
      {
        id: 'new-folder',
        label: 'New Folder',
        icon: Folder,
        type: 'success',
        action: () => createNewFolder()
      },
      {
        id: 'search',
        label: 'Search',
        icon: Search,
        type: 'info',
        action: () => openGlobalSearch()
      },
      {
        id: 'backup',
        label: 'Create Backup',
        icon: Document,
        type: 'warning',
        action: () => router.push('/files?action=backup')
      }
    ])
    
    const handleAction = (action) => {
      addToRecentActions(action)
      action.action()
    }
    
    const handleRecentAction = (recent) => {
      const action = quickActions.value.find(a => a.id === recent.actionId)
      if (action) {
        action.action()
      }
    }
    
    const addToRecentActions = (action) => {
      const recent = {
        id: Date.now(),
        actionId: action.id,
        label: action.label,
        icon: action.icon,
        timestamp: new Date()
      }
      
      recentActions.value.unshift(recent)
      if (recentActions.value.length > 5) {
        recentActions.value.pop()
      }
      
      // Store in localStorage
      localStorage.setItem('datamesh-recent-actions', JSON.stringify(recentActions.value))
    }
    
    const createNewFolder = () => {
      ElMessage.info('Opening new folder dialog...')
      // Implement folder creation logic
    }
    
    const openGlobalSearch = () => {
      // Emit global search event or navigate to search
      document.dispatchEvent(new CustomEvent('open-global-search'))
    }
    
    const formatTime = (timestamp) => {
      const now = new Date()
      const diff = now - timestamp
      
      if (diff < 60000) return 'Just now'
      if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
      if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
      return `${Math.floor(diff / 86400000)}d ago`
    }
    
    onMounted(() => {
      // Load recent actions from localStorage
      const stored = localStorage.getItem('datamesh-recent-actions')
      if (stored) {
        try {
          recentActions.value = JSON.parse(stored).map(item => ({
            ...item,
            timestamp: new Date(item.timestamp)
          }))
        } catch (error) {
          console.error('Failed to parse recent actions:', error)
        }
      }
    })
    
    return {
      Plus,
      quickActions,
      recentActions,
      handleAction,
      handleRecentAction,
      formatTime
    }
  }
}
</script>

<style scoped>
.quick-actions {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 1000;
}

.quick-actions-trigger {
  width: 56px;
  height: 56px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transition: all 0.3s ease;
}

.quick-actions-trigger:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.quick-actions-content {
  padding: 16px;
}

.quick-actions-title {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.quick-actions-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 16px;
}

.quick-action-btn {
  width: 100%;
  height: 40px;
  font-size: 12px;
}

.recent-actions h5 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-secondary);
}

.recent-actions-list {
  max-height: 200px;
  overflow-y: auto;
}

.recent-action-item {
  display: flex;
  align-items: center;
  padding: 8px 0;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.recent-action-item:hover {
  background-color: var(--el-fill-color-light);
}

.recent-action-icon {
  margin-right: 8px;
  color: var(--el-text-color-secondary);
}

.recent-action-text {
  flex: 1;
  font-size: 13px;
  color: var(--el-text-color-primary);
}

.recent-action-time {
  font-size: 11px;
  color: var(--el-text-color-placeholder);
}

@media (max-width: 768px) {
  .quick-actions {
    bottom: 16px;
    right: 16px;
  }
  
  .quick-actions-trigger {
    width: 48px;
    height: 48px;
  }
}
</style>