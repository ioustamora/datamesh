<template>
  <div class="audit-logs">
    <div class="page-header">
      <h1>Audit Logs</h1>
      <p>System activity and security audit logs</p>
    </div>

    <div class="logs-content">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>Recent Activity</h3>
            <el-button @click="refreshLogs">
              Refresh
            </el-button>
          </div>
        </template>

        <el-table
          :data="logs"
          style="width: 100%"
        >
          <el-table-column
            prop="timestamp"
            label="Timestamp"
            width="180"
          />
          <el-table-column
            prop="user"
            label="User"
            width="120"
          />
          <el-table-column
            prop="action"
            label="Action"
          />
          <el-table-column
            prop="resource"
            label="Resource"
          />
          <el-table-column
            prop="status"
            label="Status"
            width="100"
          >
            <template #default="scope">
              <el-tag :type="scope.row.status === 'Success' ? 'success' : 'danger'">
                {{ scope.row.status }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { ElMessage } from 'element-plus'

export default {
  name: 'AuditLogs',
  setup() {
    const logs = ref([
      {
        id: 1,
        timestamp: '2024-01-10 14:30:25',
        user: 'admin@example.com',
        action: 'User Login',
        resource: 'Authentication',
        status: 'Success'
      },
      {
        id: 2,
        timestamp: '2024-01-10 14:25:12',
        user: 'user@example.com',
        action: 'File Upload',
        resource: '/files/document.pdf',
        status: 'Success'
      },
      {
        id: 3,
        timestamp: '2024-01-10 14:20:45',
        user: 'user@example.com',
        action: 'Failed Login',
        resource: 'Authentication',
        status: 'Failed'
      }
    ])

    const refreshLogs = () => {
      ElMessage.success('Logs refreshed')
    }

    return {
      logs,
      refreshLogs
    }
  }
}
</script>

<style scoped>
.audit-logs {
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

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}
</style>