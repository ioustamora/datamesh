<template>
  <el-dialog
    v-model="visible"
    title="Verification Challenges"
    width="600px"
    :before-close="handleClose"
    append-to-body
  >
    <div class="challenges">
      <div class="challenge-info">
        <el-alert
          title="Verification Required"
          type="info"
          show-icon
          :closable="false"
        >
          Complete these challenges to maintain your storage contribution status.
        </el-alert>
      </div>
      
      <div class="challenge-list">
        <div v-for="challenge in mockChallenges" :key="challenge.id" class="challenge-item">
          <div class="challenge-header">
            <h4>{{ challenge.title }}</h4>
            <el-tag :type="challenge.status === 'pending' ? 'warning' : 'success'">
              {{ challenge.status }}
            </el-tag>
          </div>
          <p>{{ challenge.description }}</p>
          
          <div v-if="challenge.status === 'pending'" class="challenge-response">
            <el-input
              v-model="responses[challenge.id]"
              type="textarea"
              placeholder="Enter your response..."
              :rows="3"
            />
            <el-button 
              type="primary" 
              size="small" 
              @click="submitResponse(challenge.id)"
              :loading="submitting[challenge.id]"
              style="margin-top: 8px"
            >
              Submit Response
            </el-button>
          </div>
        </div>
      </div>
    </div>
    
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">Close</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch } from 'vue'

const emit = defineEmits(['update:modelValue', 'success'])
const props = defineProps({
  modelValue: Boolean
})

const visible = ref(false)
const responses = ref({})
const submitting = ref({})

const mockChallenges = ref([
  {
    id: 'ch001',
    title: 'Storage Proof Challenge',
    description: 'Provide proof that you still have access to block ID: 7a8b9c2d3e4f5g6h',
    status: 'pending'
  },
  {
    id: 'ch002',
    title: 'Availability Check',
    description: 'Confirm your storage is available for the next 24 hours',
    status: 'completed'
  }
])

watch(() => props.modelValue, (val) => {
  visible.value = val
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

const submitResponse = async (challengeId) => {
  submitting.value[challengeId] = true
  
  // Simulate API call
  setTimeout(() => {
    const challenge = mockChallenges.value.find(c => c.id === challengeId)
    if (challenge) {
      challenge.status = 'completed'
    }
    submitting.value[challengeId] = false
    emit('success')
  }, 1500)
}

const handleClose = () => {
  visible.value = false
}
</script>

<style scoped>
.challenges {
  padding: 20px 0;
}

.challenge-info {
  margin-bottom: 24px;
}

.challenge-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.challenge-item {
  padding: 20px;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
}

.challenge-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.challenge-header h4 {
  margin: 0;
  color: #303133;
}

.challenge-response {
  margin-top: 16px;
}
</style>