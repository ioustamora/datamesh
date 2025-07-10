<template>
  <div class="user-management">
    <div class="page-header">
      <h1>User Management</h1>
      <p>Manage user accounts and permissions</p>
    </div>

    <div class="user-content">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>Users</h3>
            <el-button type="primary">
              Add User
            </el-button>
          </div>
        </template>

        <el-table
          :data="users"
          style="width: 100%"
        >
          <el-table-column
            prop="name"
            label="Name"
          />
          <el-table-column
            prop="email"
            label="Email"
          />
          <el-table-column
            prop="role"
            label="Role"
          />
          <el-table-column
            prop="status"
            label="Status"
          >
            <template #default="scope">
              <el-tag :type="scope.row.status === 'Active' ? 'success' : 'danger'">
                {{ scope.row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="Actions">
            <template #default="scope">
              <el-button size="small">
                Edit
              </el-button>
              <el-button
                size="small"
                type="danger"
              >
                Delete
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
  name: 'UserManagement',
  setup() {
    const users = ref([
      { id: 1, name: 'John Doe', email: 'john@example.com', role: 'Admin', status: 'Active' },
      { id: 2, name: 'Jane Smith', email: 'jane@example.com', role: 'User', status: 'Active' },
      { id: 3, name: 'Bob Johnson', email: 'bob@example.com', role: 'User', status: 'Inactive' }
    ])

    return {
      users
    }
  }
}
</script>

<style scoped>
.user-management {
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