<template>
  <div class="settings-container">
    <div class="settings-header">
      <h1 class="settings-title">Settings</h1>
      <p class="settings-subtitle">Manage your account preferences and system configuration</p>
    </div>
    
    <div class="settings-content">
      <div class="settings-sidebar">
        <el-menu
          :default-active="activeTab"
          class="settings-menu"
          @select="handleTabChange"
        >
          <el-menu-item index="profile">
            <el-icon><User /></el-icon>
            <span>Profile</span>
          </el-menu-item>
          <el-menu-item index="account">
            <el-icon><CreditCard /></el-icon>
            <span>Account & Billing</span>
          </el-menu-item>
          <el-menu-item index="security">
            <el-icon><Lock /></el-icon>
            <span>Security</span>
          </el-menu-item>
          <el-menu-item index="storage">
            <el-icon><FolderOpened /></el-icon>
            <span>Storage</span>
          </el-menu-item>
          <el-menu-item index="notifications">
            <el-icon><Bell /></el-icon>
            <span>Notifications</span>
          </el-menu-item>
          <el-menu-item index="advanced">
            <el-icon><Setting /></el-icon>
            <span>Advanced</span>
          </el-menu-item>
        </el-menu>
      </div>
      
      <div class="settings-main">
        <!-- Profile Settings -->
        <div v-if="activeTab === 'profile'" class="settings-section">
          <h2 class="section-title">Profile Information</h2>
          <p class="section-description">Update your personal information and profile settings</p>
          
          <el-form
            ref="profileForm"
            :model="profileData"
            :rules="profileRules"
            label-position="top"
            style="max-width: 600px"
          >
            <div class="profile-avatar">
              <el-avatar :size="100" :src="profileData.avatar">
                <el-icon><User /></el-icon>
              </el-avatar>
              <div class="avatar-actions">
                <el-button size="small" @click="changeAvatar">Change Avatar</el-button>
                <el-button size="small" @click="removeAvatar">Remove</el-button>
              </div>
            </div>
            
            <el-form-item label="Display Name" prop="displayName">
              <el-input
                v-model="profileData.displayName"
                placeholder="Enter your display name"
              />
            </el-form-item>
            
            <el-form-item label="Email Address" prop="email">
              <el-input
                v-model="profileData.email"
                type="email"
                placeholder="Enter your email address"
                disabled
              />
              <div class="form-help">
                Email cannot be changed. <el-link type="primary">Contact support</el-link> if needed.
              </div>
            </el-form-item>
            
            <el-form-item label="Bio" prop="bio">
              <el-input
                v-model="profileData.bio"
                type="textarea"
                :rows="3"
                placeholder="Tell us about yourself"
              />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="saveProfile" :loading="saving">
                Save Changes
              </el-button>
              <el-button @click="resetProfile">Reset</el-button>
            </el-form-item>
          </el-form>
        </div>
        
        <!-- Account & Billing -->
        <div v-if="activeTab === 'account'" class="settings-section">
          <h2 class="section-title">Account & Billing</h2>
          <p class="section-description">Manage your subscription and billing information</p>
          
          <div class="account-info">
            <el-card class="account-card">
              <div class="account-tier">
                <div class="tier-info">
                  <h3>{{ accountData.tier.name }}</h3>
                  <p class="tier-price">{{ accountData.tier.price }}</p>
                </div>
                <div class="tier-actions">
                  <el-button v-if="accountData.tier.value !== 'enterprise'" type="primary">
                    Upgrade
                  </el-button>
                  <el-button v-if="accountData.tier.value !== 'free'" type="danger">
                    Downgrade
                  </el-button>
                </div>
              </div>
              
              <div class="account-usage">
                <div class="usage-item">
                  <div class="usage-label">Storage Used</div>
                  <div class="usage-progress">
                    <el-progress
                      :percentage="accountData.usage.storagePercent"
                      :format="() => `${accountData.usage.storageUsed} / ${accountData.usage.storageLimit}`"
                    />
                  </div>
                </div>
                
                <div class="usage-item">
                  <div class="usage-label">Bandwidth Used This Month</div>
                  <div class="usage-progress">
                    <el-progress
                      :percentage="accountData.usage.bandwidthPercent"
                      :format="() => `${accountData.usage.bandwidthUsed} / ${accountData.usage.bandwidthLimit}`"
                    />
                  </div>
                </div>
              </div>
            </el-card>
            
            <el-card class="billing-card">
              <h3>Billing Information</h3>
              <div class="billing-info">
                <div class="billing-item">
                  <strong>Payment Method:</strong>
                  <span>{{ accountData.billing.paymentMethod }}</span>
                </div>
                <div class="billing-item">
                  <strong>Next Billing Date:</strong>
                  <span>{{ formatDate(accountData.billing.nextBilling) }}</span>
                </div>
                <div class="billing-item">
                  <strong>Billing Status:</strong>
                  <el-tag :type="getBillingStatusType(accountData.billing.status)">
                    {{ accountData.billing.status }}
                  </el-tag>
                </div>
              </div>
              <div class="billing-actions">
                <el-button>Update Payment Method</el-button>
                <el-button>View Billing History</el-button>
              </div>
            </el-card>
          </div>
        </div>
        
        <!-- Security Settings -->
        <div v-if="activeTab === 'security'" class="settings-section">
          <h2 class="section-title">Security</h2>
          <p class="section-description">Manage your account security and privacy settings</p>
          
          <div class="security-settings">
            <el-card class="security-card">
              <h3>Password</h3>
              <p>Change your password regularly to keep your account secure</p>
              <el-button @click="showPasswordDialog = true">Change Password</el-button>
            </el-card>
            
            <el-card class="security-card">
              <h3>Two-Factor Authentication</h3>
              <p>Add an extra layer of security to your account</p>
              <div class="2fa-status">
                <el-tag v-if="securityData.twoFactorEnabled" type="success">
                  Enabled
                </el-tag>
                <el-tag v-else type="warning">
                  Disabled
                </el-tag>
                <el-button 
                  :type="securityData.twoFactorEnabled ? 'danger' : 'primary'"
                  @click="toggle2FA"
                  style="margin-left: 12px"
                >
                  {{ securityData.twoFactorEnabled ? 'Disable' : 'Enable' }} 2FA
                </el-button>
              </div>
            </el-card>
            
            <el-card class="security-card">
              <h3>Active Sessions</h3>
              <p>Monitor and manage your active sessions</p>
              <div class="sessions-list">
                <div
                  v-for="session in securityData.activeSessions"
                  :key="session.id"
                  class="session-item"
                >
                  <div class="session-info">
                    <div class="session-device">{{ session.device }}</div>
                    <div class="session-details">
                      {{ session.location }} • {{ formatDate(session.lastActive) }}
                    </div>
                  </div>
                  <div class="session-actions">
                    <el-tag v-if="session.current" type="success">Current</el-tag>
                    <el-button v-else size="small" type="danger" @click="terminateSession(session.id)">
                      Terminate
                    </el-button>
                  </div>
                </div>
              </div>
            </el-card>
          </div>
        </div>
        
        <!-- Storage Settings -->
        <div v-if="activeTab === 'storage'" class="settings-section">
          <h2 class="section-title">Storage Settings</h2>
          <p class="section-description">Configure your storage preferences and manage files</p>
          
          <div class="storage-settings">
            <el-card class="storage-card">
              <h3>Storage Preferences</h3>
              <el-form label-position="top">
                <el-form-item label="Auto-delete files after">
                  <el-select v-model="storageData.autoDeleteDays" placeholder="Select retention period">
                    <el-option label="Never" value="0" />
                    <el-option label="30 days" value="30" />
                    <el-option label="90 days" value="90" />
                    <el-option label="1 year" value="365" />
                  </el-select>
                </el-form-item>
                
                <el-form-item label="File versioning">
                  <el-switch v-model="storageData.versioningEnabled" />
                  <div class="form-help">
                    Keep multiple versions of files when they are updated
                  </div>
                </el-form-item>
                
                <el-form-item label="Compression">
                  <el-switch v-model="storageData.compressionEnabled" />
                  <div class="form-help">
                    Automatically compress files to save storage space
                  </div>
                </el-form-item>
                
                <el-form-item>
                  <el-button type="primary" @click="saveStorageSettings">
                    Save Settings
                  </el-button>
                </el-form-item>
              </el-form>
            </el-card>
            
            <el-card class="storage-card">
              <h3>Storage Cleanup</h3>
              <p>Free up space by cleaning up old or unused files</p>
              <div class="cleanup-actions">
                <el-button @click="cleanupDuplicates">Remove Duplicates</el-button>
                <el-button @click="cleanupOldFiles">Clean Old Files</el-button>
                <el-button @click="cleanupTempFiles">Clean Temporary Files</el-button>
              </div>
            </el-card>
          </div>
        </div>
        
        <!-- Notifications -->
        <div v-if="activeTab === 'notifications'" class="settings-section">
          <h2 class="section-title">Notifications</h2>
          <p class="section-description">Configure how you receive notifications</p>
          
          <div class="notifications-settings">
            <el-card class="notification-card">
              <h3>Email Notifications</h3>
              <el-form label-position="top">
                <el-form-item v-for="notification in notificationData.email" :key="notification.key">
                  <div class="notification-item">
                    <div class="notification-info">
                      <div class="notification-label">{{ notification.label }}</div>
                      <div class="notification-description">{{ notification.description }}</div>
                    </div>
                    <el-switch v-model="notification.enabled" />
                  </div>
                </el-form-item>
              </el-form>
            </el-card>
            
            <el-card class="notification-card">
              <h3>Push Notifications</h3>
              <el-form label-position="top">
                <el-form-item v-for="notification in notificationData.push" :key="notification.key">
                  <div class="notification-item">
                    <div class="notification-info">
                      <div class="notification-label">{{ notification.label }}</div>
                      <div class="notification-description">{{ notification.description }}</div>
                    </div>
                    <el-switch v-model="notification.enabled" />
                  </div>
                </el-form-item>
              </el-form>
            </el-card>
          </div>
        </div>
        
        <!-- Advanced Settings -->
        <div v-if="activeTab === 'advanced'" class="settings-section">
          <h2 class="section-title">Advanced Settings</h2>
          <p class="section-description">Advanced configuration options for power users</p>
          
          <div class="advanced-settings">
            <el-card class="advanced-card">
              <h3>API Access</h3>
              <p>Manage your API keys and access tokens</p>
              <div class="api-keys">
                <div class="api-key-item">
                  <div class="api-key-info">
                    <div class="api-key-name">Primary API Key</div>
                    <div class="api-key-value">{{ maskApiKey(advancedData.primaryApiKey) }}</div>
                  </div>
                  <div class="api-key-actions">
                    <el-button size="small" @click="copyApiKey">Copy</el-button>
                    <el-button size="small" type="danger" @click="regenerateApiKey">Regenerate</el-button>
                  </div>
                </div>
              </div>
              <el-button @click="showCreateApiKeyDialog = true">Create New API Key</el-button>
            </el-card>
            
            <el-card class="advanced-card">
              <h3>Data Export</h3>
              <p>Export your data or delete your account</p>
              <div class="export-actions">
                <el-button @click="exportData">Export All Data</el-button>
                <el-button type="danger" @click="deleteAccount">Delete Account</el-button>
              </div>
            </el-card>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Password Change Dialog -->
    <el-dialog v-model="showPasswordDialog" title="Change Password" width="400px">
      <el-form ref="passwordForm" :model="passwordData" :rules="passwordRules" label-position="top">
        <el-form-item label="Current Password" prop="currentPassword">
          <el-input v-model="passwordData.currentPassword" type="password" show-password />
        </el-form-item>
        <el-form-item label="New Password" prop="newPassword">
          <el-input v-model="passwordData.newPassword" type="password" show-password />
        </el-form-item>
        <el-form-item label="Confirm New Password" prop="confirmPassword">
          <el-input v-model="passwordData.confirmPassword" type="password" show-password />
        </el-form-item>
      </el-form>
      
      <template #footer>
        <el-button @click="showPasswordDialog = false">Cancel</el-button>
        <el-button type="primary" @click="changePassword" :loading="changingPassword">
          Change Password
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, reactive, onMounted } from 'vue'
import { useAuthStore } from '@/store/auth'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  User,
  CreditCard,
  Lock,
  FolderOpened,
  Bell,
  Setting
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'

export default {
  name: 'Settings',
  components: {
    User,
    CreditCard,
    Lock,
    FolderOpened,
    Bell,
    Setting
  },
  setup() {
    const authStore = useAuthStore()
    
    const activeTab = ref('profile')
    const saving = ref(false)
    const showPasswordDialog = ref(false)
    const showCreateApiKeyDialog = ref(false)
    const changingPassword = ref(false)
    
    const profileForm = ref()
    const passwordForm = ref()
    
    const profileData = reactive({
      displayName: '',
      email: '',
      bio: '',
      avatar: ''
    })
    
    const accountData = reactive({
      tier: {
        name: 'Free',
        value: 'free',
        price: 'Free'
      },
      usage: {
        storageUsed: '2.5GB',
        storageLimit: '5GB',
        storagePercent: 50,
        bandwidthUsed: '15GB',
        bandwidthLimit: '50GB',
        bandwidthPercent: 30
      },
      billing: {
        paymentMethod: 'Not set',
        nextBilling: new Date(),
        status: 'Active'
      }
    })
    
    const securityData = reactive({
      twoFactorEnabled: false,
      activeSessions: [
        {
          id: '1',
          device: 'Chrome on Windows',
          location: 'New York, US',
          lastActive: new Date(),
          current: true
        },
        {
          id: '2',
          device: 'Safari on iPhone',
          location: 'New York, US',
          lastActive: new Date(Date.now() - 2 * 60 * 60 * 1000),
          current: false
        }
      ]
    })
    
    const storageData = reactive({
      autoDeleteDays: '0',
      versioningEnabled: false,
      compressionEnabled: true
    })
    
    const notificationData = reactive({
      email: [
        {
          key: 'upload_complete',
          label: 'Upload Complete',
          description: 'Get notified when file uploads are complete',
          enabled: true
        },
        {
          key: 'storage_full',
          label: 'Storage Full',
          description: 'Get notified when storage quota is reached',
          enabled: true
        },
        {
          key: 'security_alerts',
          label: 'Security Alerts',
          description: 'Get notified of security-related events',
          enabled: true
        }
      ],
      push: [
        {
          key: 'file_shared',
          label: 'File Shared',
          description: 'Get notified when files are shared with you',
          enabled: false
        },
        {
          key: 'system_maintenance',
          label: 'System Maintenance',
          description: 'Get notified of planned maintenance',
          enabled: true
        }
      ]
    })
    
    const advancedData = reactive({
      primaryApiKey: 'dmk_1234567890abcdefghijklmnopqrstuvwxyz'
    })
    
    const passwordData = reactive({
      currentPassword: '',
      newPassword: '',
      confirmPassword: ''
    })
    
    const profileRules = {
      displayName: [
        { required: true, message: 'Please enter your display name', trigger: 'blur' }
      ],
      email: [
        { required: true, message: 'Please enter your email', trigger: 'blur' },
        { type: 'email', message: 'Please enter a valid email', trigger: 'blur' }
      ]
    }
    
    const passwordRules = {
      currentPassword: [
        { required: true, message: 'Please enter your current password', trigger: 'blur' }
      ],
      newPassword: [
        { required: true, message: 'Please enter a new password', trigger: 'blur' },
        { min: 8, message: 'Password must be at least 8 characters', trigger: 'blur' }
      ],
      confirmPassword: [
        { required: true, message: 'Please confirm your new password', trigger: 'blur' },
        {
          validator: (rule, value, callback) => {
            if (value !== passwordData.newPassword) {
              callback(new Error('Passwords do not match'))
            } else {
              callback()
            }
          },
          trigger: 'blur'
        }
      ]
    }
    
    const handleTabChange = (key) => {
      activeTab.value = key
    }
    
    const saveProfile = async () => {
      try {
        const valid = await profileForm.value.validate()
        if (!valid) return
        
        saving.value = true
        
        // Save profile data
        await authStore.updateProfile(profileData)
        
        ElMessage.success('Profile updated successfully')
      } catch (error) {
        ElMessage.error(error.message || 'Failed to update profile')
      } finally {
        saving.value = false
      }
    }
    
    const resetProfile = () => {
      // Reset form to original values
      loadProfileData()
    }
    
    const changePassword = async () => {
      try {
        const valid = await passwordForm.value.validate()
        if (!valid) return
        
        changingPassword.value = true
        
        await authStore.changePassword(passwordData.currentPassword, passwordData.newPassword)
        
        ElMessage.success('Password changed successfully')
        showPasswordDialog.value = false
        
        // Reset form
        passwordData.currentPassword = ''
        passwordData.newPassword = ''
        passwordData.confirmPassword = ''
      } catch (error) {
        ElMessage.error(error.message || 'Failed to change password')
      } finally {
        changingPassword.value = false
      }
    }
    
    const toggle2FA = async () => {
      try {
        if (securityData.twoFactorEnabled) {
          await authStore.disable2FA()
          securityData.twoFactorEnabled = false
          ElMessage.success('Two-factor authentication disabled')
        } else {
          await authStore.enable2FA()
          securityData.twoFactorEnabled = true
          ElMessage.success('Two-factor authentication enabled')
        }
      } catch (error) {
        ElMessage.error(error.message || 'Failed to toggle 2FA')
      }
    }
    
    const terminateSession = async (sessionId) => {
      try {
        await authStore.terminateSession(sessionId)
        securityData.activeSessions = securityData.activeSessions.filter(s => s.id !== sessionId)
        ElMessage.success('Session terminated')
      } catch (error) {
        ElMessage.error(error.message || 'Failed to terminate session')
      }
    }
    
    const saveStorageSettings = async () => {
      try {
        // Save storage settings
        ElMessage.success('Storage settings saved')
      } catch (error) {
        ElMessage.error(error.message || 'Failed to save storage settings')
      }
    }
    
    const maskApiKey = (key) => {
      if (!key) return ''
      return key.substring(0, 8) + '...' + key.substring(key.length - 8)
    }
    
    const copyApiKey = () => {
      navigator.clipboard.writeText(advancedData.primaryApiKey)
      ElMessage.success('API key copied to clipboard')
    }
    
    const regenerateApiKey = async () => {
      try {
        await ElMessageBox.confirm(
          'Are you sure you want to regenerate your API key? This will invalidate the current key.',
          'Regenerate API Key',
          {
            confirmButtonText: 'Regenerate',
            cancelButtonText: 'Cancel',
            type: 'warning'
          }
        )
        
        // Regenerate API key
        advancedData.primaryApiKey = 'dmk_' + Math.random().toString(36).substr(2, 32)
        ElMessage.success('API key regenerated')
      } catch (error) {
        // User cancelled
      }
    }
    
    const exportData = async () => {
      try {
        // Export user data
        ElMessage.success('Data export started. You will receive an email when complete.')
      } catch (error) {
        ElMessage.error(error.message || 'Failed to export data')
      }
    }
    
    const deleteAccount = async () => {
      try {
        await ElMessageBox.confirm(
          'Are you sure you want to delete your account? This action cannot be undone.',
          'Delete Account',
          {
            confirmButtonText: 'Delete Account',
            cancelButtonText: 'Cancel',
            type: 'error'
          }
        )
        
        // Delete account
        await authStore.deleteAccount()
        ElMessage.success('Account deleted successfully')
      } catch (error) {
        if (error !== 'cancel') {
          ElMessage.error(error.message || 'Failed to delete account')
        }
      }
    }
    
    const formatDate = (date) => {
      return dayjs(date).format('YYYY-MM-DD HH:mm:ss')
    }
    
    const getBillingStatusType = (status) => {
      const typeMap = {
        'Active': 'success',
        'Expired': 'danger',
        'Pending': 'warning'
      }
      return typeMap[status] || 'info'
    }
    
    const loadProfileData = async () => {
      try {
        const user = await authStore.getCurrentUser()
        profileData.displayName = user.displayName || ''
        profileData.email = user.email || ''
        profileData.bio = user.bio || ''
        profileData.avatar = user.avatar || ''
      } catch (error) {
        ElMessage.error('Failed to load profile data')
      }
    }
    
    onMounted(() => {
      loadProfileData()
    })
    
    return {
      activeTab,
      saving,
      showPasswordDialog,
      showCreateApiKeyDialog,
      changingPassword,
      profileForm,
      passwordForm,
      profileData,
      accountData,
      securityData,
      storageData,
      notificationData,
      advancedData,
      passwordData,
      profileRules,
      passwordRules,
      handleTabChange,
      saveProfile,
      resetProfile,
      changePassword,
      toggle2FA,
      terminateSession,
      saveStorageSettings,
      maskApiKey,
      copyApiKey,
      regenerateApiKey,
      exportData,
      deleteAccount,
      formatDate,
      getBillingStatusType
    }
  }
}
</script>

<style scoped>
.settings-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.settings-header {
  padding: 24px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.settings-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px 0;
}

.settings-subtitle {
  color: var(--el-text-color-regular);
  margin: 0;
}

.settings-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.settings-sidebar {
  width: 250px;
  border-right: 1px solid var(--el-border-color-lighter);
  background: var(--el-fill-color-light);
}

.settings-menu {
  border: none;
  background: transparent;
}

.settings-main {
  flex: 1;
  overflow: auto;
  padding: 24px;
}

.settings-section {
  max-width: 800px;
}

.section-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px 0;
}

.section-description {
  color: var(--el-text-color-regular);
  margin: 0 0 24px 0;
}

.profile-avatar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
  padding: 20px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.avatar-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-help {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.account-info {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

.account-card,
.billing-card {
  padding: 20px;
}

.account-tier {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.tier-info h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.tier-price {
  margin: 4px 0 0 0;
  color: var(--el-text-color-secondary);
}

.tier-actions {
  display: flex;
  gap: 8px;
}

.usage-item {
  margin-bottom: 16px;
}

.usage-label {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
}

.usage-progress {
  width: 100%;
}

.billing-info {
  margin-bottom: 20px;
}

.billing-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.billing-actions {
  display: flex;
  gap: 8px;
}

.security-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.security-card {
  padding: 20px;
}

.security-card h3 {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.security-card p {
  margin: 0 0 16px 0;
  color: var(--el-text-color-regular);
}

.2fa-status {
  display: flex;
  align-items: center;
}

.sessions-list {
  margin-top: 16px;
}

.session-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.session-info {
  flex: 1;
}

.session-device {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.session-details {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.session-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.storage-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.storage-card {
  padding: 20px;
}

.storage-card h3 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.cleanup-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.notifications-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.notification-card {
  padding: 20px;
}

.notification-card h3 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.notification-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.notification-info {
  flex: 1;
}

.notification-label {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.notification-description {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.advanced-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.advanced-card {
  padding: 20px;
}

.advanced-card h3 {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.advanced-card p {
  margin: 0 0 16px 0;
  color: var(--el-text-color-regular);
}

.api-keys {
  margin-bottom: 16px;
}

.api-key-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 8px;
}

.api-key-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.api-key-value {
  font-family: monospace;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.api-key-actions {
  display: flex;
  gap: 8px;
}

.export-actions {
  display: flex;
  gap: 8px;
}

@media (max-width: 768px) {
  .settings-content {
    flex-direction: column;
  }
  
  .settings-sidebar {
    width: 100%;
    height: auto;
    border-right: none;
    border-bottom: 1px solid var(--el-border-color-lighter);
  }
  
  .settings-menu {
    display: flex;
    overflow-x: auto;
  }
  
  .account-info {
    grid-template-columns: 1fr;
  }
  
  .tier-actions {
    flex-direction: column;
  }
}
</style>