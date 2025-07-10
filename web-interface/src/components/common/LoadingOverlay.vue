<template>
  <div class="loading-overlay">
    <div class="loading-container">
      <div class="loading-spinner">
        <el-icon
          size="48"
          class="rotating"
        >
          <Loading />
        </el-icon>
      </div>
      <div
        v-if="message"
        class="loading-message"
      >
        {{ message }}
      </div>
      <div
        v-if="hasProgress"
        class="loading-progress"
      >
        <el-progress
          :percentage="progress"
          :stroke-width="6"
          :show-text="false"
          color="#409EFF"
        />
      </div>
    </div>
  </div>
</template>

<script>
import { useLoadingStore } from '../../store/loading'
import { computed } from 'vue'

export default {
  name: 'LoadingOverlay',
  setup() {
    const loadingStore = useLoadingStore()
    
    const message = computed(() => loadingStore.loadingMessage)
    const progress = computed(() => loadingStore.loadingProgress)
    const hasProgress = computed(() => loadingStore.hasProgress)
    
    return {
      message,
      progress,
      hasProgress
    }
  }
}
</script>

<style scoped>
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
}

.loading-container {
  background: var(--el-bg-color);
  border-radius: 12px;
  padding: 32px;
  text-align: center;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  min-width: 200px;
}

.loading-spinner {
  margin-bottom: 16px;
}

.rotating {
  animation: rotate 2s linear infinite;
  color: var(--el-color-primary);
}

@keyframes rotate {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-message {
  font-size: 16px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 16px;
}

.loading-progress {
  width: 200px;
  margin: 0 auto;
}
</style>