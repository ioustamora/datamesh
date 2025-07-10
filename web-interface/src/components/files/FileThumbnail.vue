<template>
  <div
    class="file-thumbnail"
    :class="{ 'loading': isLoading }"
  >
    <!-- Image thumbnails -->
    <img 
      v-if="thumbnailType === 'image'" 
      :src="thumbnailUrl" 
      :alt="`Thumbnail of ${file.file_name}`"
      class="thumbnail-image"
      @load="onThumbnailLoad"
      @error="onThumbnailError"
    >
    
    <!-- Video thumbnails -->
    <div
      v-else-if="thumbnailType === 'video'"
      class="video-thumbnail"
    >
      <img 
        v-if="thumbnailUrl"
        :src="thumbnailUrl" 
        :alt="`Video thumbnail of ${file.file_name}`"
        class="thumbnail-image"
        @load="onThumbnailLoad"
        @error="onThumbnailError"
      >
      <div class="video-overlay">
        <el-icon class="play-icon">
          <VideoPlay />
        </el-icon>
        <span
          v-if="file.duration"
          class="video-duration"
        >{{ formatDuration(file.duration) }}</span>
      </div>
    </div>
    
    <!-- PDF thumbnails -->
    <div
      v-else-if="thumbnailType === 'pdf'"
      class="pdf-thumbnail"
    >
      <img 
        v-if="thumbnailUrl"
        :src="thumbnailUrl" 
        :alt="`PDF thumbnail of ${file.file_name}`"
        class="thumbnail-image"
        @load="onThumbnailLoad"
        @error="onThumbnailError"
      >
      <div class="pdf-overlay">
        <el-icon class="pdf-icon">
          <Document />
        </el-icon>
        <span
          v-if="file.page_count"
          class="page-count"
        >{{ file.page_count }} pages</span>
      </div>
    </div>
    
    <!-- Audio thumbnails -->
    <div
      v-else-if="thumbnailType === 'audio'"
      class="audio-thumbnail"
    >
      <div class="audio-visualizer">
        <div class="waveform">
          <div 
            v-for="i in 20" 
            :key="i" 
            class="waveform-bar"
            :style="{ height: getWaveformHeight(i) + '%' }"
          />
        </div>
        <el-icon class="audio-icon">
          <Headphone />
        </el-icon>
        <span
          v-if="file.duration"
          class="audio-duration"
        >{{ formatDuration(file.duration) }}</span>
      </div>
    </div>
    
    <!-- Text file previews -->
    <div
      v-else-if="thumbnailType === 'text'"
      class="text-thumbnail"
    >
      <div
        v-if="textPreview"
        class="text-content"
      >
        {{ textPreview }}
      </div>
      <div
        v-else
        class="text-placeholder"
      >
        <el-icon class="text-icon">
          <Document />
        </el-icon>
        <span>Text File</span>
      </div>
    </div>
    
    <!-- Archive thumbnails -->
    <div
      v-else-if="thumbnailType === 'archive'"
      class="archive-thumbnail"
    >
      <el-icon class="archive-icon">
        <FolderOpened />
      </el-icon>
      <span
        v-if="file.file_count"
        class="file-count"
      >{{ file.file_count }} files</span>
    </div>
    
    <!-- Code file thumbnails -->
    <div
      v-else-if="thumbnailType === 'code'"
      class="code-thumbnail"
    >
      <div
        v-if="codePreview"
        class="code-content"
      >
        <pre><code>{{ codePreview }}</code></pre>
      </div>
      <div
        v-else
        class="code-placeholder"
      >
        <el-icon class="code-icon">
          <Notebook />
        </el-icon>
        <span>{{ getFileExtension(file.file_name).toUpperCase() }}</span>
      </div>
    </div>
    
    <!-- Loading state -->
    <div
      v-if="isLoading"
      class="thumbnail-loading"
    >
      <el-skeleton-item
        variant="image"
        class="thumbnail-skeleton"
      />
    </div>
    
    <!-- Error state -->
    <div
      v-if="hasError"
      class="thumbnail-error"
    >
      <el-icon class="error-icon">
        <Warning />
      </el-icon>
      <span>Preview unavailable</span>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted } from 'vue'
import { 
  VideoPlay, 
  Document, 
  Headphone, 
  FolderOpened, 
  Notebook, 
  Warning 
} from '@element-plus/icons-vue'

export default {
  name: 'FileThumbnail',
  components: {
    VideoPlay,
    Document,
    Headphone,
    FolderOpened,
    Notebook,
    Warning
  },
  props: {
    file: {
      type: Object,
      required: true
    },
    size: {
      type: String,
      default: 'medium',
      validator: value => ['small', 'medium', 'large'].includes(value)
    },
    loadImmediately: {
      type: Boolean,
      default: true
    }
  },
  setup(props) {
    const isLoading = ref(false)
    const hasError = ref(false)
    const thumbnailUrl = ref('')
    const textPreview = ref('')
    const codePreview = ref('')

    const fileExtension = computed(() => {
      return getFileExtension(props.file.file_name)
    })

    const thumbnailType = computed(() => {
      const ext = fileExtension.value.toLowerCase()
      
      if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp'].includes(ext)) {
        return 'image'
      }
      if (['mp4', 'avi', 'mov', 'wmv', 'webm'].includes(ext)) {
        return 'video'
      }
      if (['pdf'].includes(ext)) {
        return 'pdf'
      }
      if (['mp3', 'wav', 'flac', 'aac', 'ogg'].includes(ext)) {
        return 'audio'
      }
      if (['txt', 'md', 'readme', 'log'].includes(ext)) {
        return 'text'
      }
      if (['js', 'ts', 'py', 'java', 'cpp', 'css', 'html', 'json'].includes(ext)) {
        return 'code'
      }
      if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) {
        return 'archive'
      }
      
      return 'unknown'
    })

    const getFileExtension = (filename) => {
      const parts = filename.split('.')
      return parts.length > 1 ? parts.pop() : ''
    }

    const loadThumbnail = async () => {
      if (!props.loadImmediately) return

      isLoading.value = true
      hasError.value = false

      try {
        // Generate thumbnail URL based on file type and size
        const baseUrl = `/api/v1/files/${props.file.file_key}/thumbnail`
        const sizeParam = `?size=${props.size}`
        
        thumbnailUrl.value = baseUrl + sizeParam

        // For text and code files, load preview content
        if (thumbnailType.value === 'text' || thumbnailType.value === 'code') {
          await loadTextPreview()
        }

      } catch (error) {
        console.error('Failed to load thumbnail:', error)
        hasError.value = true
      } finally {
        isLoading.value = false
      }
    }

    const loadTextPreview = async () => {
      try {
        // Load first few lines of text/code files
        const response = await fetch(`/api/v1/files/${props.file.file_key}/preview?lines=10`)
        const content = await response.text()
        
        if (thumbnailType.value === 'text') {
          textPreview.value = content.substring(0, 200) + (content.length > 200 ? '...' : '')
        } else if (thumbnailType.value === 'code') {
          codePreview.value = content.substring(0, 300) + (content.length > 300 ? '...' : '')
        }
      } catch (error) {
        console.error('Failed to load text preview:', error)
      }
    }

    const onThumbnailLoad = () => {
      isLoading.value = false
      hasError.value = false
    }

    const onThumbnailError = () => {
      isLoading.value = false
      hasError.value = true
    }

    const formatDuration = (seconds) => {
      if (!seconds) return ''
      
      const hours = Math.floor(seconds / 3600)
      const minutes = Math.floor((seconds % 3600) / 60)
      const secs = Math.floor(seconds % 60)
      
      if (hours > 0) {
        return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
      }
      return `${minutes}:${secs.toString().padStart(2, '0')}`
    }

    const getWaveformHeight = (index) => {
      // Generate pseudo-random waveform visualization
      const seed = props.file.file_key ? props.file.file_key.charCodeAt(index % props.file.file_key.length) : index
      return 20 + (seed % 60)
    }

    onMounted(() => {
      if (props.loadImmediately) {
        loadThumbnail()
      }
    })

    return {
      isLoading,
      hasError,
      thumbnailUrl,
      textPreview,
      codePreview,
      thumbnailType,
      getFileExtension,
      onThumbnailLoad,
      onThumbnailError,
      formatDuration,
      getWaveformHeight,
      loadThumbnail
    }
  }
}
</script>

<style scoped>
.file-thumbnail {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
  overflow: hidden;
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 4px;
}

/* Video thumbnail */
.video-thumbnail {
  position: relative;
  width: 100%;
  height: 100%;
}

.video-overlay {
  position: absolute;
  bottom: 4px;
  left: 4px;
  right: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.play-icon {
  font-size: 16px;
}

.video-duration {
  background: rgba(0, 0, 0, 0.8);
  padding: 2px 4px;
  border-radius: 2px;
  font-size: 10px;
}

/* PDF thumbnail */
.pdf-thumbnail {
  position: relative;
  width: 100%;
  height: 100%;
}

.pdf-overlay {
  position: absolute;
  bottom: 4px;
  left: 4px;
  right: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(255, 255, 255, 0.9);
  color: var(--el-text-color-primary);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.pdf-icon {
  color: var(--el-color-danger);
  font-size: 16px;
}

.page-count {
  background: var(--el-color-info);
  color: white;
  padding: 2px 4px;
  border-radius: 2px;
  font-size: 10px;
}

/* Audio thumbnail */
.audio-thumbnail {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--el-color-success-light-8), var(--el-color-success-light-6));
}

.audio-visualizer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.waveform {
  display: flex;
  align-items: end;
  gap: 2px;
  height: 30px;
}

.waveform-bar {
  width: 3px;
  background: var(--el-color-success);
  border-radius: 1px;
  animation: waveform 2s ease-in-out infinite;
}

.waveform-bar:nth-child(even) {
  animation-delay: 0.1s;
}

.waveform-bar:nth-child(3n) {
  animation-delay: 0.2s;
}

@keyframes waveform {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

.audio-icon {
  font-size: 20px;
  color: var(--el-color-success);
}

.audio-duration {
  font-size: 10px;
  color: var(--el-text-color-secondary);
}

/* Text thumbnail */
.text-thumbnail {
  width: 100%;
  height: 100%;
  padding: 8px;
  background: var(--el-bg-color);
}

.text-content {
  font-size: 10px;
  line-height: 1.2;
  color: var(--el-text-color-primary);
  overflow: hidden;
  white-space: pre-wrap;
  font-family: monospace;
}

.text-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 4px;
  color: var(--el-text-color-secondary);
}

.text-icon {
  font-size: 24px;
}

/* Code thumbnail */
.code-thumbnail {
  width: 100%;
  height: 100%;
  background: #1e1e1e;
  color: #d4d4d4;
}

.code-content {
  padding: 8px;
  height: 100%;
  overflow: hidden;
}

.code-content pre {
  margin: 0;
  font-size: 8px;
  line-height: 1.2;
  font-family: 'Courier New', monospace;
}

.code-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 4px;
}

.code-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

/* Archive thumbnail */
.archive-thumbnail {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  background: var(--el-color-warning-light-8);
  color: var(--el-color-warning);
}

.archive-icon {
  font-size: 24px;
}

.file-count {
  font-size: 10px;
  color: var(--el-text-color-secondary);
}

/* Loading and error states */
.thumbnail-loading {
  width: 100%;
  height: 100%;
}

.thumbnail-skeleton {
  width: 100%;
  height: 100%;
}

.thumbnail-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: var(--el-color-danger);
  font-size: 12px;
}

.error-icon {
  font-size: 20px;
}

/* Size variations */
.file-thumbnail.loading {
  background: var(--el-fill-color);
}

/* Dark mode adjustments */
.dark .code-thumbnail {
  background: #0d1117;
  color: #c9d1d9;
}

.dark .text-thumbnail {
  background: var(--el-bg-color-overlay);
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .video-overlay,
  .pdf-overlay {
    background: rgba(0, 0, 0, 0.9);
  }
  
  .waveform-bar {
    background: currentColor;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .waveform-bar {
    animation: none;
  }
}
</style>