<template>
  <div class="admin-overview">
    <div class="page-header">
      <h1>Administration Overview</h1>
      <p>System administration and management</p>
    </div>

    <div class="admin-content">
      <div class="admin-grid">
        <el-card class="admin-card">
          <template #header>
            <h3>System Status</h3>
          </template>
          <div class="status-list">
            <div class="status-item">
              <span>Server Status:</span>
              <el-tag type="success">Online</el-tag>
            </div>
            <div class="status-item">
              <span>Database:</span>
              <el-tag type="success">Connected</el-tag>
            </div>
            <div class="status-item">
              <span>Storage:</span>
              <el-tag type="warning">75% Full</el-tag>
            </div>
          </div>
        </el-card>

        <el-card class="admin-card">
          <template #header>
            <h3>User Statistics</h3>
          </template>
          <div class="stats-grid">
            <div class="stat-box">
              <div class="stat-number">{{ totalUsers }}</div>
              <div class="stat-label">Total Users</div>
            </div>
            <div class="stat-box">
              <div class="stat-number">{{ activeUsers }}</div>
              <div class="stat-label">Active Users</div>
            </div>
          </div>
        </el-card>

        <el-card class="admin-card">
          <template #header>
            <h3>Quick Actions</h3>
          </template>
          <div class="action-buttons">
            <el-button @click="$router.push('/administration/users')">
              Manage Users
            </el-button>
            <el-button @click="$router.push('/administration/system')">
              System Settings
            </el-button>
            <el-button @click="$router.push('/administration/audit')">
              View Audit Logs
            </el-button>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'

export default {
  name: 'AdminOverview',
  setup() {
    const totalUsers = ref(1250)
    const activeUsers = ref(890)

    return {
      totalUsers,
      activeUsers
    }
  }
}
</script>

<style scoped>
.admin-overview {
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

.admin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.admin-card h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.status-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.stat-box {
  text-align: center;
  padding: 16px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
}

.stat-number {
  font-size: 24px;
  font-weight: bold;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-buttons .el-button {
  width: 100%;
}
</style>