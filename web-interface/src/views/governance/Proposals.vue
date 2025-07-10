<template>
  <div class="proposals-view">
    <div class="page-header">
      <h1>Proposals</h1>
      <p>Governance proposals and voting</p>
    </div>

    <div class="proposals-content">
      <el-card>
        <template #header>
          <div class="card-header">
            <h3>Active Proposals</h3>
            <el-button type="primary">
              <el-icon><Plus /></el-icon>
              Create Proposal
            </el-button>
          </div>
        </template>

        <div class="proposals-list">
          <div
            v-for="proposal in proposals"
            :key="proposal.id"
            class="proposal-item"
          >
            <div class="proposal-header">
              <h4>{{ proposal.title }}</h4>
              <el-tag :type="getProposalType(proposal.status)">
                {{ proposal.status }}
              </el-tag>
            </div>
            <p class="proposal-description">
              {{ proposal.description }}
            </p>
            <div class="proposal-meta">
              <span>Voting ends: {{ formatDate(proposal.endDate) }}</span>
              <span>Votes: {{ proposal.votes }}</span>
            </div>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'
import { Plus } from '@element-plus/icons-vue'

export default {
  name: 'Proposals',
  components: {
    Plus
  },
  setup() {
    const proposals = ref([
      {
        id: 1,
        title: 'Network Upgrade Proposal',
        description: 'Upgrade network to version 2.0 with improved performance',
        status: 'Active',
        endDate: new Date(Date.now() + 1000 * 60 * 60 * 24 * 7),
        votes: 150
      },
      {
        id: 2,
        title: 'Storage Allocation Change',
        description: 'Modify storage allocation algorithm for better efficiency',
        status: 'Pending',
        endDate: new Date(Date.now() + 1000 * 60 * 60 * 24 * 14),
        votes: 45
      }
    ])

    const getProposalType = (status) => {
      switch (status) {
        case 'Active': return 'success'
        case 'Pending': return 'warning'
        case 'Completed': return 'info'
        default: return 'danger'
      }
    }

    const formatDate = (date) => {
      return new Date(date).toLocaleDateString()
    }

    return {
      proposals,
      getProposalType,
      formatDate
    }
  }
}
</script>

<style scoped>
.proposals-view {
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

.proposals-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.proposal-item {
  padding: 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-bg-color);
}

.proposal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.proposal-header h4 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.proposal-description {
  margin: 8px 0;
  color: var(--el-text-color-regular);
}

.proposal-meta {
  display: flex;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}
</style>