<template>
  <div class="file-status">
    <el-tag 
      :type="statusType" 
      :size="size"
      :effect="effect"
      :icon="statusIcon"
    >
      {{ statusText }}
    </el-tag>
    
    <!-- Progress indicator for uploads/downloads -->
    <div v-if="showProgress" class="status-progress">
      <el-progress
        :percentage="progressPercentage"
        :status="progressStatus"
        :stroke-width="4"
        :show-text="false"
      />
    </div>
    
    <!-- Additional status indicators -->
    <div v-if="file.is_encrypted" class="status-indicator encrypted" title="File is encrypted">
      <el-icon><Lock /></el-icon>
    </div>
    
    <div v-if="file.is_shared" class="status-indicator shared" title="File is shared">
      <el-icon><Share /></el-icon>
    </div>
    
    <div v-if="file.has_versions" class="status-indicator versioned" title="File has multiple versions">
      <el-icon><Clock /></el-icon>
    </div>
    
    <div v-if="file.is_favorite" class="status-indicator favorite" title="File is favorited">
      <el-icon><Star /></el-icon>
    </div>
  </div>
</template>

<script>
import { computed } from 'vue'
import { Lock, Share, Clock, Star, CircleCheck, Warning, Loading, CircleClose } from '@element-plus/icons-vue'

export default {
  name: 'FileStatus',
  components: {
    Lock,
    Share,
    Clock,
    Star,
    CircleCheck,
    Warning,
    Loading,
    CircleClose
  },
  props: {
    file: {
      type: Object,
      required: true
    },
    size: {
      type: String,
      default: 'small',
      validator: value => ['large', 'default', 'small'].includes(value)
    },
    effect: {
      type: String,
      default: 'light',
      validator: value => ['dark', 'light', 'plain'].includes(value)
    },
    showProgress: {
      type: Boolean,
      default: false
    }
  },
  setup(props) {
    const fileStatus = computed(() => {
      // Check for various file states
      if (props.file.upload_progress !== undefined && props.file.upload_progress < 100) {
        return 'uploading'
      }
      
      if (props.file.download_progress !== undefined && props.file.download_progress < 100) {
        return 'downloading'
      }
      
      if (props.file.status === 'processing') {
        return 'processing'
      }
      
      if (props.file.status === 'failed' || props.file.has_error) {
        return 'error'
      }
      
      if (props.file.status === 'corrupted') {
        return 'corrupted'
      }
      
      if (props.file.status === 'quarantined') {
        return 'quarantined'
      }
      
      if (props.file.sync_status === 'syncing') {
        return 'syncing'
      }
      
      if (props.file.sync_status === 'sync_failed') {
        return 'sync_failed'
      }
      
      if (props.file.is_verified === false) {
        return 'unverified'
      }
      
      // Default to completed/ready state
      return 'ready'
    })

    const statusType = computed(() => {
      switch (fileStatus.value) {
        case 'ready':
          return 'success'
        case 'uploading':
        case 'downloading':
        case 'processing':
        case 'syncing':
          return 'primary'
        case 'unverified':
          return 'warning'
        case 'error':
        case 'corrupted':
        case 'quarantined':
        case 'sync_failed':
          return 'danger'
        default:
          return 'info'
      }
    })

    const statusText = computed(() => {
      switch (fileStatus.value) {
        case 'ready':
          return 'Ready'
        case 'uploading':
          return 'Uploading'
        case 'downloading':
          return 'Downloading'
        case 'processing':
          return 'Processing'
        case 'syncing':
          return 'Syncing'
        case 'unverified':
          return 'Unverified'
        case 'error':
          return 'Error'
        case 'corrupted':
          return 'Corrupted'
        case 'quarantined':
          return 'Quarantined'
        case 'sync_failed':
          return 'Sync Failed'
        default:
          return 'Unknown'
      }
    })

    const statusIcon = computed(() => {
      switch (fileStatus.value) {
        case 'ready':
          return CircleCheck
        case 'uploading':
        case 'downloading':
        case 'processing':
        case 'syncing':
          return Loading
        case 'unverified':
          return Warning
        case 'error':
        case 'corrupted':
        case 'quarantined':
        case 'sync_failed':
          return CircleClose
        default:
          return null
      }
    })

    const progressPercentage = computed(() => {
      if (props.file.upload_progress !== undefined) {
        return props.file.upload_progress
      }
      if (props.file.download_progress !== undefined) {
        return props.file.download_progress
      }
      if (props.file.processing_progress !== undefined) {
        return props.file.processing_progress
      }
      return 0
    })

    const progressStatus = computed(() => {
      if (progressPercentage.value === 100) {
        return 'success'
      }
      if (fileStatus.value === 'error' || fileStatus.value === 'sync_failed') {
        return 'exception'
      }
      return null
    })

    return {
      fileStatus,
      statusType,
      statusText,
      statusIcon,
      progressPercentage,
      progressStatus
    }
  }
}
</script>

<style scoped>
.file-status {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-progress {
  min-width: 60px;
  margin-left: 8px;
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 10px;
}

.status-indicator.encrypted {
  background: var(--el-color-warning-light-8);
  color: var(--el-color-warning);
}

.status-indicator.shared {
  background: var(--el-color-success-light-8);
  color: var(--el-color-success);
}

.status-indicator.versioned {
  background: var(--el-color-info-light-8);
  color: var(--el-color-info);
}

.status-indicator.favorite {
  background: var(--el-color-primary-light-8);
  color: var(--el-color-primary);
}

/* Animation for loading states */
.el-tag.el-tag--primary .el-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Dark mode adjustments */
.dark .status-indicator.encrypted {
  background: var(--el-color-warning-dark-2);
}

.dark .status-indicator.shared {
  background: var(--el-color-success-dark-2);
}

.dark .status-indicator.versioned {
  background: var(--el-color-info-dark-2);
}

.dark .status-indicator.favorite {
  background: var(--el-color-primary-dark-2);
}
</style>