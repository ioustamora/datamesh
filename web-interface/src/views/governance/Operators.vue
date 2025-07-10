<template>
  <div class="operators-view">
    <div class="page-header">
      <h1>Operators</h1>
      <p>Manage and monitor network operators</p>
    </div>

    <div class="operators-content">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>Network Operators</h3>
            <el-button type="primary">
              <el-icon><Plus /></el-icon>
              Add Operator
            </el-button>
          </div>
        </template>

        <el-table :data="operators" style="width: 100%">
          <el-table-column prop="name" label="Name" />
          <el-table-column prop="address" label="Address" />
          <el-table-column prop="status" label="Status">
            <template #default="scope">
              <el-tag :type="getStatusType(scope.row.status)">
                {{ scope.row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="stake" label="Stake" />
          <el-table-column prop="uptime" label="Uptime" />
          <el-table-column label="Actions">
            <template #default="scope">
              <el-button size="small" @click="viewOperator(scope.row)">View</el-button>
              <el-button size="small" type="warning" @click="editOperator(scope.row)">Edit</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { Plus } from '@element-plus/icons-vue'

export default {
  name: 'Operators',
  components: {
    Plus
  },
  setup() {
    const operators = ref([
      { id: 1, name: 'Operator 1', address: '0x123...', status: 'Active', stake: '100K', uptime: '99.9%' },
      { id: 2, name: 'Operator 2', address: '0x456...', status: 'Inactive', stake: '75K', uptime: '95.2%' }
    ])

    const getStatusType = (status) => {
      return status === 'Active' ? 'success' : 'danger'
    }

    const viewOperator = (operator) => {
      console.log('View operator:', operator)
    }

    const editOperator = (operator) => {
      console.log('Edit operator:', operator)
    }

    return {
      operators,
      getStatusType,
      viewOperator,
      editOperator
    }
  }
}
</script>

<style scoped>
.operators-view {
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