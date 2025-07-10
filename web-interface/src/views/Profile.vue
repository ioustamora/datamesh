<template>
  <div class="profile-container">
    <div class="profile-header">
      <div class="profile-banner">
        <div class="banner-content">
          <div class="profile-avatar">
            <el-avatar
              :size="120"
              :src="profileData.avatar"
            >
              <el-icon><User /></el-icon>
            </el-avatar>
            <div class="avatar-status">
              <el-tag
                :type="getStatusType(profileData.status)"
                size="small"
              >
                {{ profileData.status }}
              </el-tag>
            </div>
          </div>
          
          <div class="profile-info">
            <h1 class="profile-name">
              {{ profileData.displayName || profileData.email }}
            </h1>
            <p class="profile-email">
              {{ profileData.email }}
            </p>
            <p class="profile-bio">
              {{ profileData.bio || 'No bio provided' }}
            </p>
            
            <div class="profile-stats">
              <div class="stat-item">
                <div class="stat-value">
                  {{ profileData.stats.filesUploaded }}
                </div>
                <div class="stat-label">
                  Files Uploaded
                </div>
              </div>
              <div class="stat-item">
                <div class="stat-value">
                  {{ profileData.stats.totalStorage }}
                </div>
                <div class="stat-label">
                  Total Storage
                </div>
              </div>
              <div class="stat-item">
                <div class="stat-value">
                  {{ profileData.stats.memberSince }}
                </div>
                <div class="stat-label">
                  Member Since
                </div>
              </div>
            </div>
          </div>
          
          <div class="profile-actions">
            <el-button
              type="primary"
              @click="editProfile"
            >
              <el-icon><Edit /></el-icon>
              Edit Profile
            </el-button>
            <el-button @click="shareProfile">
              <el-icon><Share /></el-icon>
              Share Profile
            </el-button>
          </div>
        </div>
      </div>
    </div>
    
    <div class="profile-content">
      <div class="profile-tabs">
        <el-tabs
          v-model="activeTab"
          @tab-click="handleTabClick"
        >
          <el-tab-pane
            label="Activity"
            name="activity"
          >
            <div class="activity-feed">
              <div class="activity-header">
                <h3>Recent Activity</h3>
                <div class="activity-filters">
                  <el-select
                    v-model="activityFilter"
                    placeholder="Filter by type"
                    size="small"
                  >
                    <el-option
                      label="All"
                      value="all"
                    />
                    <el-option
                      label="Uploads"
                      value="upload"
                    />
                    <el-option
                      label="Downloads"
                      value="download"
                    />
                    <el-option
                      label="Shares"
                      value="share"
                    />
                    <el-option
                      label="Deletes"
                      value="delete"
                    />
                  </el-select>
                </div>
              </div>
              
              <div class="activity-list">
                <div
                  v-for="activity in filteredActivities"
                  :key="activity.id"
                  class="activity-item"
                >
                  <div class="activity-icon">
                    <el-icon :class="getActivityIconClass(activity.type)">
                      <component :is="getActivityIcon(activity.type)" />
                    </el-icon>
                  </div>
                  
                  <div class="activity-content">
                    <div class="activity-description">
                      {{ activity.description }}
                    </div>
                    <div class="activity-details">
                      <span class="activity-file">{{ activity.fileName }}</span>
                      <span class="activity-time">{{ formatTimeAgo(activity.timestamp) }}</span>
                    </div>
                  </div>
                  
                  <div class="activity-actions">
                    <el-button
                      v-if="activity.type === 'upload'"
                      size="small"
                      @click="downloadFile(activity.fileId)"
                    >
                      <el-icon><Download /></el-icon>
                    </el-button>
                    <el-button
                      v-if="activity.type === 'share'"
                      size="small"
                      @click="viewShare(activity.shareId)"
                    >
                      <el-icon><View /></el-icon>
                    </el-button>
                  </div>
                </div>
              </div>
              
              <div class="activity-pagination">
                <el-pagination
                  v-model:current-page="currentPage"
                  :page-size="pageSize"
                  :total="totalActivities"
                  layout="prev, pager, next"
                  @current-change="handlePageChange"
                />
              </div>
            </div>
          </el-tab-pane>
          
          <el-tab-pane
            label="Files"
            name="files"
          >
            <div class="files-overview">
              <div class="files-header">
                <h3>File Overview</h3>
                <div class="files-actions">
                  <el-button @click="goToFileManager">
                    <el-icon><FolderOpened /></el-icon>
                    Open File Manager
                  </el-button>
                </div>
              </div>
              
              <div class="files-stats">
                <div class="stats-grid">
                  <div class="stat-card">
                    <div class="stat-icon">
                      <el-icon><Files /></el-icon>
                    </div>
                    <div class="stat-info">
                      <div class="stat-number">
                        {{ filesData.totalFiles }}
                      </div>
                      <div class="stat-text">
                        Total Files
                      </div>
                    </div>
                  </div>
                  
                  <div class="stat-card">
                    <div class="stat-icon">
                      <el-icon><Upload /></el-icon>
                    </div>
                    <div class="stat-info">
                      <div class="stat-number">
                        {{ filesData.uploadedThisMonth }}
                      </div>
                      <div class="stat-text">
                        Uploaded This Month
                      </div>
                    </div>
                  </div>
                  
                  <div class="stat-card">
                    <div class="stat-icon">
                      <el-icon><Share /></el-icon>
                    </div>
                    <div class="stat-info">
                      <div class="stat-number">
                        {{ filesData.sharedFiles }}
                      </div>
                      <div class="stat-text">
                        Shared Files
                      </div>
                    </div>
                  </div>
                  
                  <div class="stat-card">
                    <div class="stat-icon">
                      <el-icon><Download /></el-icon>
                    </div>
                    <div class="stat-info">
                      <div class="stat-number">
                        {{ filesData.totalDownloads }}
                      </div>
                      <div class="stat-text">
                        Total Downloads
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              
              <div class="files-chart">
                <h4>Storage Usage Over Time</h4>
                <div class="chart-container">
                  <canvas
                    ref="storageChart"
                    height="200"
                  />
                </div>
              </div>
              
              <div class="files-breakdown">
                <h4>File Types</h4>
                <div class="breakdown-list">
                  <div
                    v-for="type in filesData.typeBreakdown"
                    :key="type.type"
                    class="breakdown-item"
                  >
                    <div class="breakdown-info">
                      <div class="breakdown-icon">
                        <el-icon :class="getFileTypeIconClass(type.type)">
                          <component :is="getFileTypeIcon(type.type)" />
                        </el-icon>
                      </div>
                      <div class="breakdown-details">
                        <div class="breakdown-type">
                          {{ type.type }}
                        </div>
                        <div class="breakdown-count">
                          {{ type.count }} files
                        </div>
                      </div>
                    </div>
                    <div class="breakdown-size">
                      {{ type.size }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
          
          <el-tab-pane
            label="Storage"
            name="storage"
          >
            <div class="storage-overview">
              <div class="storage-header">
                <h3>Storage Usage</h3>
                <div class="storage-actions">
                  <el-button @click="optimizeStorage">
                    <el-icon><Setting /></el-icon>
                    Optimize Storage
                  </el-button>
                </div>
              </div>
              
              <div class="storage-usage">
                <div class="usage-chart">
                  <div class="usage-circle">
                    <el-progress
                      type="circle"
                      :percentage="storageData.usagePercent"
                      :width="200"
                      :stroke-width="12"
                    >
                      <div class="usage-info">
                        <div class="usage-used">
                          {{ storageData.usedStorage }}
                        </div>
                        <div class="usage-total">
                          of {{ storageData.totalStorage }}
                        </div>
                      </div>
                    </el-progress>
                  </div>
                  
                  <div class="usage-details">
                    <div class="usage-breakdown">
                      <div class="breakdown-category">
                        <div class="category-indicator active" />
                        <div class="category-info">
                          <div class="category-name">
                            Files
                          </div>
                          <div class="category-size">
                            {{ storageData.filesSize }}
                          </div>
                        </div>
                      </div>
                      
                      <div class="breakdown-category">
                        <div class="category-indicator cached" />
                        <div class="category-info">
                          <div class="category-name">
                            Cache
                          </div>
                          <div class="category-size">
                            {{ storageData.cacheSize }}
                          </div>
                        </div>
                      </div>
                      
                      <div class="breakdown-category">
                        <div class="category-indicator temp" />
                        <div class="category-info">
                          <div class="category-name">
                            Temporary
                          </div>
                          <div class="category-size">
                            {{ storageData.tempSize }}
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              
              <div class="storage-recommendations">
                <h4>Storage Recommendations</h4>
                <div class="recommendations-list">
                  <div
                    v-for="recommendation in storageData.recommendations"
                    :key="recommendation.id"
                    class="recommendation-item"
                  >
                    <div class="recommendation-icon">
                      <el-icon :class="recommendation.type">
                        <component :is="recommendation.icon" />
                      </el-icon>
                    </div>
                    <div class="recommendation-content">
                      <div class="recommendation-title">
                        {{ recommendation.title }}
                      </div>
                      <div class="recommendation-description">
                        {{ recommendation.description }}
                      </div>
                    </div>
                    <div class="recommendation-action">
                      <el-button
                        size="small"
                        @click="executeRecommendation(recommendation.id)"
                      >
                        {{ recommendation.actionText }}
                      </el-button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
    
    <!-- Edit Profile Dialog -->
    <el-dialog
      v-model="showEditDialog"
      title="Edit Profile"
      width="500px"
    >
      <el-form
        ref="editForm"
        :model="editData"
        :rules="editRules"
        label-position="top"
      >
        <el-form-item
          label="Display Name"
          prop="displayName"
        >
          <el-input
            v-model="editData.displayName"
            placeholder="Enter your display name"
          />
        </el-form-item>
        
        <el-form-item
          label="Bio"
          prop="bio"
        >
          <el-input
            v-model="editData.bio"
            type="textarea"
            :rows="3"
            placeholder="Tell us about yourself"
          />
        </el-form-item>
        
        <el-form-item label="Avatar">
          <div class="avatar-upload">
            <el-avatar
              :size="80"
              :src="editData.avatar"
            >
              <el-icon><User /></el-icon>
            </el-avatar>
            <div class="upload-actions">
              <el-button
                size="small"
                @click="uploadAvatar"
              >
                Upload New
              </el-button>
              <el-button
                size="small"
                @click="removeAvatar"
              >
                Remove
              </el-button>
            </div>
          </div>
        </el-form-item>
      </el-form>
      
      <template #footer>
        <el-button @click="showEditDialog = false">
          Cancel
        </el-button>
        <el-button
          type="primary"
          :loading="saving"
          @click="saveProfile"
        >
          Save Changes
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useFilesStore } from '@/store/files'
import { ElMessage } from 'element-plus'
import {
  User,
  Edit,
  Share,
  Upload,
  Download,
  Delete,
  View,
  FolderOpened,
  Files,
  Setting,
  Picture,
  Document,
  VideoPlay,
  Folder,
  Warning,
  InfoFilled
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

export default {
  name: 'Profile',
  components: {
    User,
    Edit,
    Share,
    Upload,
    Download,
    Delete,
    View,
    FolderOpened,
    Files,
    Setting,
    Picture,
    Document,
    VideoPlay,
    Folder,
    Warning,
    InfoFilled
  },
  setup() {
    const router = useRouter()
    const authStore = useAuthStore()
    const filesStore = useFilesStore()
    
    const activeTab = ref('activity')
    const activityFilter = ref('all')
    const currentPage = ref(1)
    const pageSize = ref(10)
    const totalActivities = ref(0)
    const showEditDialog = ref(false)
    const saving = ref(false)
    const editForm = ref()
    const storageChart = ref()
    
    const profileData = reactive({
      displayName: 'John Doe',
      email: 'john.doe@example.com',
      bio: 'Passionate developer and data enthusiast. Love working with distributed systems.',
      avatar: '',
      status: 'Active',
      stats: {
        filesUploaded: 1247,
        totalStorage: '2.5GB',
        memberSince: 'Jan 2023'
      }
    })
    
    const activities = reactive([
      {
        id: 1,
        type: 'upload',
        description: 'Uploaded a new file',
        fileName: 'project-proposal.pdf',
        fileId: 'file-123',
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000)
      },
      {
        id: 2,
        type: 'share',
        description: 'Shared a file',
        fileName: 'meeting-notes.docx',
        shareId: 'share-456',
        timestamp: new Date(Date.now() - 6 * 60 * 60 * 1000)
      },
      {
        id: 3,
        type: 'download',
        description: 'Downloaded a file',
        fileName: 'backup-data.zip',
        timestamp: new Date(Date.now() - 24 * 60 * 60 * 1000)
      }
    ])
    
    const filesData = reactive({
      totalFiles: 1247,
      uploadedThisMonth: 89,
      sharedFiles: 34,
      totalDownloads: 5623,
      typeBreakdown: [
        {
          type: 'Documents',
          count: 445,
          size: '1.2GB',
          icon: Document
        },
        {
          type: 'Images',
          count: 332,
          size: '890MB',
          icon: Picture
        },
        {
          type: 'Videos',
          count: 78,
          size: '2.1GB',
          icon: VideoPlay
        },
        {
          type: 'Audio',
          count: 156,
          size: '567MB',
          icon: Headphones
        },
        {
          type: 'Archives',
          count: 89,
          size: '1.8GB',
          icon: Folder
        }
      ]
    })
    
    const storageData = reactive({
      usedStorage: '4.2GB',
      totalStorage: '10GB',
      usagePercent: 42,
      filesSize: '3.1GB',
      cacheSize: '890MB',
      tempSize: '210MB',
      recommendations: [
        {
          id: 1,
          type: 'warning',
          icon: Warning,
          title: 'Clean up temporary files',
          description: 'You have 210MB of temporary files that can be safely removed',
          actionText: 'Clean Now'
        },
        {
          id: 2,
          type: 'info',
          icon: InfoFilled,
          title: 'Enable compression',
          description: 'Compress files to save up to 30% storage space',
          actionText: 'Enable'
        }
      ]
    })
    
    const editData = reactive({
      displayName: '',
      bio: '',
      avatar: ''
    })
    
    const editRules = {
      displayName: [
        { required: true, message: 'Please enter your display name', trigger: 'blur' }
      ]
    }
    
    const filteredActivities = computed(() => {
      if (activityFilter.value === 'all') {
        return activities
      }
      return activities.filter(activity => activity.type === activityFilter.value)
    })
    
    const handleTabClick = (tab) => {
      activeTab.value = tab.name
    }
    
    const handlePageChange = (page) => {
      currentPage.value = page
      // Load more activities
    }
    
    const getStatusType = (status) => {
      const typeMap = {
        'Active': 'success',
        'Inactive': 'warning',
        'Suspended': 'danger'
      }
      return typeMap[status] || 'info'
    }
    
    const getActivityIcon = (type) => {
      const iconMap = {
        'upload': Upload,
        'download': Download,
        'share': Share,
        'delete': Delete
      }
      return iconMap[type] || Files
    }
    
    const getActivityIconClass = (type) => {
      const classMap = {
        'upload': 'activity-icon-upload',
        'download': 'activity-icon-download',
        'share': 'activity-icon-share',
        'delete': 'activity-icon-delete'
      }
      return classMap[type] || 'activity-icon-default'
    }
    
    const getFileTypeIcon = (type) => {
      const iconMap = {
        'Documents': Document,
        'Images': Picture,
        'Videos': VideoPlay,
        'Audio': Headphones,
        'Archives': Folder
      }
      return iconMap[type] || Files
    }
    
    const getFileTypeIconClass = (type) => {
      const classMap = {
        'Documents': 'file-type-document',
        'Images': 'file-type-image',
        'Videos': 'file-type-video',
        'Audio': 'file-type-audio',
        'Archives': 'file-type-archive'
      }
      return classMap[type] || 'file-type-default'
    }
    
    const formatTimeAgo = (timestamp) => {
      return dayjs(timestamp).fromNow()
    }
    
    const editProfile = () => {
      editData.displayName = profileData.displayName
      editData.bio = profileData.bio
      editData.avatar = profileData.avatar
      showEditDialog.value = true
    }
    
    const saveProfile = async () => {
      try {
        const valid = await editForm.value.validate()
        if (!valid) return
        
        saving.value = true
        
        // Update profile data
        profileData.displayName = editData.displayName
        profileData.bio = editData.bio
        profileData.avatar = editData.avatar
        
        await authStore.updateProfile(editData)
        
        ElMessage.success('Profile updated successfully')
        showEditDialog.value = false
      } catch (error) {
        ElMessage.error(error.message || 'Failed to update profile')
      } finally {
        saving.value = false
      }
    }
    
    const shareProfile = () => {
      const profileUrl = `${window.location.origin}/profile/${profileData.email}`
      navigator.clipboard.writeText(profileUrl)
      ElMessage.success('Profile link copied to clipboard')
    }
    
    const downloadFile = async (fileId) => {
      try {
        await filesStore.downloadFile(fileId)
        ElMessage.success('File downloaded successfully')
      } catch (error) {
        ElMessage.error('Failed to download file')
      }
    }
    
    const viewShare = (shareId) => {
      router.push(`/share/${shareId}`)
    }
    
    const goToFileManager = () => {
      router.push('/files')
    }
    
    const optimizeStorage = () => {
      ElMessage.success('Storage optimization started')
    }
    
    const executeRecommendation = (id) => {
      const recommendation = storageData.recommendations.find(r => r.id === id)
      if (recommendation) {
        ElMessage.success(`Executing: ${recommendation.title}`)
        // Remove the recommendation after execution
        storageData.recommendations = storageData.recommendations.filter(r => r.id !== id)
      }
    }
    
    const uploadAvatar = () => {
      // Implement avatar upload
      ElMessage.info('Avatar upload functionality to be implemented')
    }
    
    const removeAvatar = () => {
      editData.avatar = ''
      ElMessage.success('Avatar removed')
    }
    
    const loadProfileData = async () => {
      try {
        const user = await authStore.getCurrentUser()
        profileData.displayName = user.displayName || user.email
        profileData.email = user.email
        profileData.bio = user.bio || 'No bio provided'
        profileData.avatar = user.avatar || ''
      } catch (error) {
        ElMessage.error('Failed to load profile data')
      }
    }
    
    onMounted(() => {
      loadProfileData()
      totalActivities.value = activities.length
    })
    
    return {
      activeTab,
      activityFilter,
      currentPage,
      pageSize,
      totalActivities,
      showEditDialog,
      saving,
      editForm,
      storageChart,
      profileData,
      activities,
      filesData,
      storageData,
      editData,
      editRules,
      filteredActivities,
      handleTabClick,
      handlePageChange,
      getStatusType,
      getActivityIcon,
      getActivityIconClass,
      getFileTypeIcon,
      getFileTypeIconClass,
      formatTimeAgo,
      editProfile,
      saveProfile,
      shareProfile,
      downloadFile,
      viewShare,
      goToFileManager,
      optimizeStorage,
      executeRecommendation,
      uploadAvatar,
      removeAvatar
    }
  }
}
</script>

<style scoped>
.profile-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.profile-header {
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.profile-banner {
  background: linear-gradient(135deg, var(--el-color-primary) 0%, var(--el-color-primary-light-3) 100%);
  color: white;
  padding: 40px 24px;
}

.banner-content {
  display: flex;
  align-items: center;
  gap: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.profile-avatar {
  position: relative;
  flex-shrink: 0;
}

.avatar-status {
  position: absolute;
  bottom: 0;
  right: 0;
}

.profile-info {
  flex: 1;
}

.profile-name {
  font-size: 32px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.profile-email {
  font-size: 18px;
  opacity: 0.9;
  margin: 0 0 12px 0;
}

.profile-bio {
  font-size: 16px;
  opacity: 0.8;
  margin: 0 0 20px 0;
  line-height: 1.5;
}

.profile-stats {
  display: flex;
  gap: 32px;
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  opacity: 0.8;
}

.profile-actions {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.profile-content {
  flex: 1;
  overflow: hidden;
}

.profile-tabs {
  height: 100%;
  padding: 24px;
}

.profile-tabs :deep(.el-tabs__content) {
  height: calc(100% - 60px);
  overflow: auto;
}

.activity-feed {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.activity-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.activity-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.activity-list {
  flex: 1;
  overflow: auto;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.activity-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color-light);
  flex-shrink: 0;
}

.activity-icon-upload {
  color: var(--el-color-success);
}

.activity-icon-download {
  color: var(--el-color-primary);
}

.activity-icon-share {
  color: var(--el-color-warning);
}

.activity-icon-delete {
  color: var(--el-color-danger);
}

.activity-content {
  flex: 1;
}

.activity-description {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.activity-details {
  display: flex;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.activity-file {
  font-style: italic;
}

.activity-actions {
  display: flex;
  gap: 8px;
}

.activity-pagination {
  margin-top: 24px;
  display: flex;
  justify-content: center;
}

.files-overview {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.files-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.files-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.files-stats {
  margin-bottom: 32px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.stat-card {
  background: var(--el-fill-color-light);
  padding: 20px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--el-color-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.stat-number {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.stat-text {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.files-chart {
  margin-bottom: 32px;
}

.files-chart h4 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.chart-container {
  height: 200px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--el-text-color-secondary);
}

.files-breakdown h4 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.breakdown-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.breakdown-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.breakdown-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--el-fill-color-light);
  display: flex;
  align-items: center;
  justify-content: center;
}

.breakdown-type {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.breakdown-count {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.breakdown-size {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.storage-overview {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.storage-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.storage-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.storage-usage {
  margin-bottom: 32px;
}

.usage-chart {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 48px;
  padding: 32px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.usage-info {
  text-align: center;
}

.usage-used {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.usage-total {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.usage-breakdown {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.breakdown-category {
  display: flex;
  align-items: center;
  gap: 12px;
}

.category-indicator {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.category-indicator.active {
  background: var(--el-color-primary);
}

.category-indicator.cached {
  background: var(--el-color-success);
}

.category-indicator.temp {
  background: var(--el-color-warning);
}

.category-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.category-size {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.storage-recommendations h4 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.recommendation-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 12px;
}

.recommendation-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--el-fill-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.recommendation-content {
  flex: 1;
}

.recommendation-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.recommendation-description {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.avatar-upload {
  display: flex;
  align-items: center;
  gap: 16px;
}

.upload-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

@media (max-width: 768px) {
  .banner-content {
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }
  
  .profile-stats {
    justify-content: center;
  }
  
  .profile-actions {
    flex-direction: row;
    justify-content: center;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
  
  .usage-chart {
    flex-direction: column;
    gap: 24px;
  }
  
  .activity-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
}
</style>