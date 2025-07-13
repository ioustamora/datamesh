<template>
  <el-dialog
    v-model="visible"
    title="All Economy Transactions"
    width="800px"
    :before-close="handleClose"
    append-to-body
  >
    <div class="transactions">
      <div class="transactions-filters">
        <el-row :gutter="20">
          <el-col :span="8">
            <el-select v-model="filters.type" placeholder="Transaction Type" clearable>
              <el-option label="All Types" value="" />
              <el-option label="Contribution" value="contribution" />
              <el-option label="Upgrade" value="upgrade" />
              <el-option label="Verification" value="verification" />
            </el-select>
          </el-col>
          <el-col :span="8">
            <el-select v-model="filters.status" placeholder="Status" clearable>
              <el-option label="All Status" value="" />
              <el-option label="Completed" value="completed" />
              <el-option label="Pending" value="pending" />
              <el-option label="Failed" value="failed" />
            </el-select>
          </el-col>
          <el-col :span="8">
            <el-date-picker
              v-model="filters.dateRange"
              type="daterange"
              range-separator="To"
              start-placeholder="Start date"
              end-placeholder="End date"
              format="YYYY-MM-DD"
              value-format="YYYY-MM-DD"
            />
          </el-col>
        </el-row>
      </div>
      
      <el-table :data="filteredTransactions" style="width: 100%">
        <el-table-column prop="timestamp" label="Date" width="180">
          <template #default="scope">
            {{ formatDate(scope.row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column prop="transaction_type" label="Type" width="120">
          <template #default="scope">
            <el-tag :type="getTypeTagType(scope.row.transaction_type)">
              {{ scope.row.transaction_type }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="Description" />
        <el-table-column prop="amount" label="Amount" width="120">
          <template #default="scope">
            {{ formatSize(scope.row.amount) }}
          </template>
        </el-table-column>
        <el-table-column prop="status" label="Status" width="100">
          <template #default="scope">
            <el-tag :type="getStatusTagType(scope.row.status)">
              {{ scope.row.status }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
      
      <div class="pagination">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="totalTransactions"
          layout="total, sizes, prev, pager, next, jumper"
        />
      </div>
    </div>
    
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">Close</el-button>
        <el-button type="primary" @click="exportTransactions">
          Export CSV
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch, computed } from 'vue'

const emit = defineEmits(['update:modelValue'])
const props = defineProps({
  modelValue: Boolean
})

const visible = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const totalTransactions = ref(0)

const filters = ref({
  type: '',
  status: '',
  dateRange: null
})

// Mock transaction data
const mockTransactions = ref([
  {
    transaction_id: 'tx001',
    transaction_type: 'contribution',
    description: 'Started storage contribution',
    amount: 4294967296, // 4GB
    timestamp: '2024-01-15T10:30:00Z',
    status: 'completed'
  },
  {
    transaction_id: 'tx002',
    transaction_type: 'verification',
    description: 'Verification challenge completed',
    amount: 0,
    timestamp: '2024-01-14T15:45:00Z',
    status: 'completed'
  },
  {
    transaction_id: 'tx003',
    transaction_type: 'upgrade',
    description: 'Upgraded to Premium tier',
    amount: 107374182400, // 100GB
    timestamp: '2024-01-10T09:20:00Z',
    status: 'completed'
  }
])

const filteredTransactions = computed(() => {
  let filtered = mockTransactions.value
  
  if (filters.value.type) {
    filtered = filtered.filter(t => t.transaction_type === filters.value.type)
  }
  
  if (filters.value.status) {
    filtered = filtered.filter(t => t.status === filters.value.status)
  }
  
  if (filters.value.dateRange && filters.value.dateRange.length === 2) {
    const [start, end] = filters.value.dateRange
    filtered = filtered.filter(t => {
      const date = new Date(t.timestamp).toISOString().split('T')[0]
      return date >= start && date <= end
    })
  }
  
  totalTransactions.value = filtered.length
  return filtered
})

watch(() => props.modelValue, (val) => {
  visible.value = val
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

const formatDate = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString()
}

const formatSize = (bytes) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const getTypeTagType = (type) => {
  const types = {
    contribution: 'success',
    upgrade: 'primary',
    verification: 'warning'
  }
  return types[type] || 'info'
}

const getStatusTagType = (status) => {
  const statuses = {
    completed: 'success',
    pending: 'warning',
    failed: 'danger'
  }
  return statuses[status] || 'info'
}

const exportTransactions = () => {
  // Mock CSV export
  console.log('Exporting transactions to CSV...')
}

const handleClose = () => {
  visible.value = false
}
</script>

<style scoped>
.transactions {
  padding: 20px 0;
}

.transactions-filters {
  margin-bottom: 20px;
}

.pagination {
  margin-top: 20px;
  text-align: center;
}
</style>