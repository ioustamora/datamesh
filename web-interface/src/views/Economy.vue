<template>
  <div class="economy-view">
    <EconomyDashboard />
  </div>
</template>

<script setup>
import EconomyDashboard from '../components/dashboard/EconomyDashboard.vue'
</script>

<style scoped>
.economy-view {
  min-height: 100vh;
  background-color: #f5f7fa;
}
</style>