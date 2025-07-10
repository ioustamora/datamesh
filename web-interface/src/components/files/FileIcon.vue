<template>
  <div class="file-icon" :class="[sizeClass, iconClass]">
    <el-icon :size="iconSize">
      <component :is="iconComponent" />
    </el-icon>
    <div v-if="showExtension && fileExtension" class="file-extension">
      {{ fileExtension }}
    </div>
  </div>
</template>

<script>
import { computed } from 'vue'
import {
  Document,
  Picture,
  VideoPlay,
  Headphone,
  FolderOpened,
  Files,
  DocumentCopy,
  Notebook,
  Monitor,
  Setting
} from '@element-plus/icons-vue'

export default {
  name: 'FileIcon',
  components: {
    Document,
    Picture,
    VideoPlay,
    Headphone,
    FolderOpened,
    Files,
    DocumentCopy,
    Notebook,
    Monitor,
    Setting
  },
  props: {
    file: {
      type: Object,
      required: true
    },
    size: {
      type: String,
      default: 'medium',
      validator: value => ['small', 'medium', 'large', 'xl'].includes(value)
    },
    showExtension: {
      type: Boolean,
      default: false
    }
  },
  setup(props) {
    const fileExtension = computed(() => {
      const parts = props.file.file_name.split('.')
      return parts.length > 1 ? parts.pop().toLowerCase() : ''
    })

    const fileType = computed(() => {
      const ext = fileExtension.value
      
      // Image files
      if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext)) {
        return 'image'
      }
      
      // Video files
      if (['mp4', 'avi', 'mov', 'wmv', 'flv', 'webm', 'mkv', '3gp'].includes(ext)) {
        return 'video'
      }
      
      // Audio files
      if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'wma'].includes(ext)) {
        return 'audio'
      }
      
      // Archive files
      if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz'].includes(ext)) {
        return 'archive'
      }
      
      // Document files
      if (['pdf', 'doc', 'docx', 'odt', 'rtf'].includes(ext)) {
        return 'document'
      }
      
      // Spreadsheet files
      if (['xls', 'xlsx', 'ods', 'csv'].includes(ext)) {
        return 'spreadsheet'
      }
      
      // Presentation files
      if (['ppt', 'pptx', 'odp'].includes(ext)) {
        return 'presentation'
      }
      
      // Text files
      if (['txt', 'md', 'readme', 'log', 'yml', 'yaml', 'json', 'xml'].includes(ext)) {
        return 'text'
      }
      
      // Code files
      if (['js', 'ts', 'jsx', 'tsx', 'vue', 'py', 'java', 'cpp', 'c', 'h', 'css', 'scss', 'html', 'php', 'rb', 'go', 'rs', 'swift'].includes(ext)) {
        return 'code'
      }
      
      // Executable files
      if (['exe', 'msi', 'dmg', 'pkg', 'deb', 'rpm', 'app'].includes(ext)) {
        return 'executable'
      }
      
      return 'unknown'
    })

    const iconComponent = computed(() => {
      switch (fileType.value) {
        case 'image':
          return 'Picture'
        case 'video':
          return 'VideoPlay'
        case 'audio':
          return 'Headphone'
        case 'archive':
          return 'FolderOpened'
        case 'document':
        case 'text':
          return 'Document'
        case 'spreadsheet':
          return 'DocumentCopy'
        case 'presentation':
          return 'Monitor'
        case 'code':
          return 'Notebook'
        case 'executable':
          return 'Setting'
        default:
          return 'Files'
      }
    })

    const iconClass = computed(() => {
      return `file-icon-${fileType.value}`
    })

    const sizeClass = computed(() => {
      return `file-icon-${props.size}`
    })

    const iconSize = computed(() => {
      switch (props.size) {
        case 'small':
          return 16
        case 'medium':
          return 24
        case 'large':
          return 32
        case 'xl':
          return 48
        default:
          return 24
      }
    })

    return {
      fileExtension,
      fileType,
      iconComponent,
      iconClass,
      sizeClass,
      iconSize
    }
  }
}
</script>

<style scoped>
.file-icon {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.file-extension {
  position: absolute;
  bottom: -2px;
  right: -2px;
  background: var(--el-color-primary);
  color: white;
  font-size: 8px;
  padding: 1px 3px;
  border-radius: 2px;
  font-weight: bold;
  text-transform: uppercase;
  line-height: 1;
}

/* Size classes */
.file-icon-small {
  width: 20px;
  height: 20px;
}

.file-icon-medium {
  width: 28px;
  height: 28px;
}

.file-icon-large {
  width: 36px;
  height: 36px;
}

.file-icon-xl {
  width: 52px;
  height: 52px;
}

/* Type-specific colors */
.file-icon-image {
  color: var(--el-color-primary);
}

.file-icon-video {
  color: var(--el-color-danger);
}

.file-icon-audio {
  color: var(--el-color-success);
}

.file-icon-archive {
  color: var(--el-color-warning);
}

.file-icon-document {
  color: var(--el-color-info);
}

.file-icon-spreadsheet {
  color: var(--el-color-success);
}

.file-icon-presentation {
  color: var(--el-color-warning);
}

.file-icon-text {
  color: var(--el-text-color-primary);
}

.file-icon-code {
  color: var(--el-color-primary);
}

.file-icon-executable {
  color: var(--el-color-danger);
}

.file-icon-unknown {
  color: var(--el-text-color-secondary);
}

/* Extension badge for larger sizes */
.file-icon-large .file-extension,
.file-icon-xl .file-extension {
  font-size: 10px;
  padding: 2px 4px;
  border-radius: 3px;
  bottom: -4px;
  right: -4px;
}
</style>