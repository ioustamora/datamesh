<template>
  <div class="operator-management">
    <div class="page-header">
      <h1>Operator Management</h1>
      <p>Manage system operators and permissions</p>
    </div>

    <div class="operator-content">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>System Operators</h3>
            <el-button type="primary">
              Add Operator
            </el-button>
          </div>
        </template>

        <el-table
          :data="operators"
          style="width: 100%"
        >
          <el-table-column
            prop="name"
            label="Name"
          />
          <el-table-column
            prop="type"
            label="Type"
          />
          <el-table-column
            prop="permissions"
            label="Permissions"
          />
          <el-table-column
            prop="lastActive"
            label="Last Active"
          />
          <el-table-column label="Actions">
            <template #default="scope">
              <el-button size="small">
                Edit
              </el-button>
              <el-button
                size="small"
                type="danger"
              >
                Remove
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'

export default {
  name: 'OperatorManagement',
  setup() {
    const operators = ref([
      { id: 1, name: 'System Admin', type: 'System', permissions: 'Full', lastActive: '2024-01-10' },
      { id: 2, name: 'File Manager', type: 'Service', permissions: 'Files', lastActive: '2024-01-10' },
      { id: 3, name: 'Network Monitor', type: 'Service', permissions: 'Read-only', lastActive: '2024-01-09' }
    ])

    return {
      operators
    }
  }
}
</script>

<style scoped>
.operator-management {
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