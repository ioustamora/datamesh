<template>
  <div class="voting-view">
    <div class="page-header">
      <h1>Voting</h1>
      <p>Participate in governance voting</p>
    </div>

    <div class="voting-content">
      <el-card>
        <template #header>
          <h3>Current Votes</h3>
        </template>

        <div class="voting-list">
          <div v-for="vote in votes" :key="vote.id" class="vote-item">
            <div class="vote-info">
              <h4>{{ vote.proposal }}</h4>
              <div class="vote-progress">
                <el-progress :percentage="vote.yesPercentage" :show-text="false" />
                <div class="vote-results">
                  <span class="vote-yes">Yes: {{ vote.yes }}</span>
                  <span class="vote-no">No: {{ vote.no }}</span>
                </div>
              </div>
            </div>
            <div class="vote-actions">
              <el-button type="success" size="small">Vote Yes</el-button>
              <el-button type="danger" size="small">Vote No</el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'

export default {
  name: 'Voting',
  setup() {
    const votes = ref([
      {
        id: 1,
        proposal: 'Network Upgrade Proposal',
        yes: 120,
        no: 30,
        yesPercentage: 80
      },
      {
        id: 2,
        proposal: 'Storage Allocation Change',
        yes: 35,
        no: 10,
        yesPercentage: 78
      }
    ])

    return {
      votes
    }
  }
}
</script>

<style scoped>
.voting-view {
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

.voting-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.vote-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-bg-color);
}

.vote-info {
  flex: 1;
  margin-right: 20px;
}

.vote-info h4 {
  margin: 0 0 12px 0;
  color: var(--el-text-color-primary);
}

.vote-progress {
  width: 100%;
}

.vote-results {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: 14px;
}

.vote-yes {
  color: var(--el-color-success);
}

.vote-no {
  color: var(--el-color-danger);
}

.vote-actions {
  display: flex;
  gap: 8px;
}
</style>