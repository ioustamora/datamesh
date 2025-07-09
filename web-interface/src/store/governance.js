import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { governanceAPI } from '../services/api'

export const useGovernanceStore = defineStore('governance', () => {
  // State
  const status = ref({
    enabled: false,
    total_operators: 0,
    active_operators: 0,
    network_healthy: false,
    can_reach_consensus: false
  })
  
  const operators = ref([])
  const networkHealth = ref({
    total_operators: 0,
    online_operators: 0,
    online_percentage: 0,
    total_governance_weight: 0,
    online_governance_weight: 0,
    can_reach_consensus: false
  })
  
  const proposals = ref([])
  const votes = ref([])
  const loading = ref(false)
  const error = ref(null)
  
  // Getters
  const isGovernanceEnabled = computed(() => status.value.enabled)
  const isNetworkHealthy = computed(() => status.value.network_healthy)
  const canReachConsensus = computed(() => status.value.can_reach_consensus)
  const activeOperators = computed(() => operators.value.filter(op => op.status === 'active'))
  const totalGovernanceWeight = computed(() => networkHealth.value.total_governance_weight)
  const onlineGovernanceWeight = computed(() => networkHealth.value.online_governance_weight)
  const consensusThreshold = computed(() => networkHealth.value.total_governance_weight * 0.51)
  
  // Proposals computed
  const activeProposals = computed(() => proposals.value.filter(p => p.status === 'active'))
  const pendingProposals = computed(() => proposals.value.filter(p => p.status === 'pending'))
  const completedProposals = computed(() => proposals.value.filter(p => ['approved', 'rejected'].includes(p.status)))
  
  // Actions
  const fetchGovernanceStatus = async () => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getStatus()
      status.value = response.data
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchOperators = async () => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getOperators()
      operators.value = response.data
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchOperator = async (operatorId) => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getOperator(operatorId)
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchOperatorDashboard = async (operatorId) => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getOperatorDashboard(operatorId)
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchNetworkHealth = async () => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getNetworkHealth()
      networkHealth.value = response.data
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const registerOperator = async (operatorData) => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.registerOperator(operatorData)
      // Add new operator to list
      operators.value.push(response.data)
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const registerService = async (operatorId, serviceData) => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.registerService(operatorId, serviceData)
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const updateServiceHeartbeat = async (operatorId, serviceId) => {
    try {
      const response = await governanceAPI.updateServiceHeartbeat(operatorId, serviceId)
      return response.data
    } catch (err) {
      console.error('Heartbeat update failed:', err)
      throw err
    }
  }
  
  const executeAdminAction = async (actionData) => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.executeAdminAction(actionData)
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const fetchAdminActions = async () => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.getAdminActions()
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  const cleanupInactiveOperators = async () => {
    loading.value = true
    error.value = null
    try {
      const response = await governanceAPI.cleanupInactiveOperators()
      // Refresh operators list
      await fetchOperators()
      return response.data
    } catch (err) {
      error.value = err.message
      throw err
    } finally {
      loading.value = false
    }
  }
  
  // Mock methods for future implementation
  const fetchProposals = async () => {
    // TODO: Implement when proposal API is available
    proposals.value = []
    return []
  }
  
  const createProposal = async (proposalData) => {
    // TODO: Implement when proposal API is available
    console.log('Creating proposal:', proposalData)
    return null
  }
  
  const voteOnProposal = async (proposalId, voteData) => {
    // TODO: Implement when voting API is available
    console.log('Voting on proposal:', proposalId, voteData)
    return null
  }
  
  const fetchVotes = async (proposalId) => {
    // TODO: Implement when voting API is available
    votes.value = []
    return []
  }
  
  // Real-time updates
  const subscribeToUpdates = () => {
    // TODO: Implement WebSocket subscriptions for real-time updates
    console.log('Subscribing to governance updates')
  }
  
  const unsubscribeFromUpdates = () => {
    // TODO: Implement WebSocket unsubscription
    console.log('Unsubscribing from governance updates')
  }
  
  // Initialize store
  const init = async () => {
    try {
      await Promise.all([
        fetchGovernanceStatus(),
        fetchOperators(),
        fetchNetworkHealth()
      ])
    } catch (error) {
      console.error('Failed to initialize governance store:', error)
    }
  }
  
  return {
    // State
    status,
    operators,
    networkHealth,
    proposals,
    votes,
    loading,
    error,
    
    // Getters
    isGovernanceEnabled,
    isNetworkHealthy,
    canReachConsensus,
    activeOperators,
    totalGovernanceWeight,
    onlineGovernanceWeight,
    consensusThreshold,
    activeProposals,
    pendingProposals,
    completedProposals,
    
    // Actions
    fetchGovernanceStatus,
    fetchOperators,
    fetchOperator,
    fetchOperatorDashboard,
    fetchNetworkHealth,
    registerOperator,
    registerService,
    updateServiceHeartbeat,
    executeAdminAction,
    fetchAdminActions,
    cleanupInactiveOperators,
    fetchProposals,
    createProposal,
    voteOnProposal,
    fetchVotes,
    subscribeToUpdates,
    unsubscribeFromUpdates,
    init
  }
})