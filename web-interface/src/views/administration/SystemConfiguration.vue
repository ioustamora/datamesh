<template>
  <div class="system-configuration">
    <div class="page-header">
      <h1>System Configuration</h1>
      <p>Configure system settings and parameters</p>
    </div>

    <div class="config-content">
      <el-card>
        <template #header>
          <h3>System Settings</h3>
        </template>

        <el-form :model="config" label-width="200px">
          <el-form-item label="Max File Size">
            <el-input v-model="config.maxFileSize" />
          </el-form-item>
          <el-form-item label="Session Timeout">
            <el-input v-model="config.sessionTimeout" />
          </el-form-item>
          <el-form-item label="Enable Debug Mode">
            <el-switch v-model="config.debugMode" />
          </el-form-item>
          <el-form-item label="Auto Backup">
            <el-switch v-model="config.autoBackup" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="saveConfig">Save Settings</el-button>
            <el-button @click="resetConfig">Reset to Default</el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { ElMessage } from 'element-plus'

export default {
  name: 'SystemConfiguration',
  setup() {
    const config = ref({
      maxFileSize: '100MB',
      sessionTimeout: '30 minutes',
      debugMode: false,
      autoBackup: true
    })

    const saveConfig = () => {
      ElMessage.success('Configuration saved successfully')
    }

    const resetConfig = () => {
      config.value = {
        maxFileSize: '100MB',
        sessionTimeout: '30 minutes',
        debugMode: false,
        autoBackup: true
      }
      ElMessage.info('Configuration reset to default values')
    }

    return {
      config,
      saveConfig,
      resetConfig
    }
  }
}
</script>

<style scoped>
.system-configuration {
  padding: 24px;
}

.page-header {
  margin-bottom: 24px;
}

.page-header h1 {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.page-header p {
  margin: 0;
  color: var(--el-text-color-secondary);
}
</style>