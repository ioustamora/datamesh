<template>
  <div class="network-health-view">
    <div class="page-header">
      <h1>Network Health</h1>
      <p>Monitor network status and performance</p>
    </div>

    <div class="health-content">
      <div class="health-grid">
        <el-card class="health-card">
          <template #header>
            <h3>Network Status</h3>
          </template>
          <div class="health-indicator">
            <div class="health-score">{{ networkScore }}%</div>
            <div class="health-status">{{ networkStatus }}</div>
          </div>
        </el-card>

        <el-card class="health-card">
          <template #header>
            <h3>Node Statistics</h3>
          </template>
          <div class="stats-list">
            <div class="stat-item">
              <span>Active Nodes:</span>
              <span>{{ activeNodes }}</span>
            </div>
            <div class="stat-item">
              <span>Total Nodes:</span>
              <span>{{ totalNodes }}</span>
            </div>
            <div class="stat-item">
              <span>Avg Response Time:</span>
              <span>{{ avgResponseTime }}ms</span>
            </div>
          </div>
        </el-card>

        <el-card class="health-card">
          <template #header>
            <h3>Performance Metrics</h3>
          </template>
          <div class="metrics-list">
            <div class="metric-item">
              <span>Throughput:</span>
              <span>{{ throughput }} TPS</span>
            </div>
            <div class="metric-item">
              <span>Latency:</span>
              <span>{{ latency }}ms</span>
            </div>
            <div class="metric-item">
              <span>Success Rate:</span>
              <span>{{ successRate }}%</span>
            </div>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue'

export default {
  name: 'NetworkHealth',
  setup() {
    const networkScore = ref(98)
    const networkStatus = ref('Healthy')
    const activeNodes = ref(45)
    const totalNodes = ref(50)
    const avgResponseTime = ref(125)
    const throughput = ref(2500)
    const latency = ref(45)
    const successRate = ref(99.8)

    return {
      networkScore,
      networkStatus,
      activeNodes,
      totalNodes,
      avgResponseTime,
      throughput,
      latency,
      successRate
    }
  }
}
</script>

<style scoped>
.network-health-view {
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

.health-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.health-card h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.health-indicator {
  text-align: center;
  padding: 20px 0;
}

.health-score {
  font-size: 48px;
  font-weight: bold;
  color: var(--el-color-success);
  margin-bottom: 8px;
}

.health-status {
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.stats-list,
.metrics-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stat-item,
.metric-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.stat-item:last-child,
.metric-item:last-child {
  border-bottom: none;
}
</style>