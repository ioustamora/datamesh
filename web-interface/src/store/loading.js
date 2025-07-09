import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useLoadingStore = defineStore('loading', () => {
  // State
  const globalLoading = ref(false)
  const loadingTasks = ref(new Map())
  const loadingMessage = ref('')
  const loadingProgress = ref(0)
  
  // Getters
  const isLoading = computed(() => globalLoading.value || loadingTasks.value.size > 0)
  const taskCount = computed(() => loadingTasks.value.size)
  const hasMessage = computed(() => !!loadingMessage.value)
  const hasProgress = computed(() => loadingProgress.value > 0 && loadingProgress.value < 100)
  
  // Actions
  const setLoading = (loading, message = '') => {
    globalLoading.value = loading
    loadingMessage.value = message
    if (!loading) {
      loadingProgress.value = 0
    }
  }
  
  const setProgress = (progress) => {
    loadingProgress.value = Math.max(0, Math.min(100, progress))
  }
  
  const addTask = (taskId, message = '') => {
    loadingTasks.value.set(taskId, {
      id: taskId,
      message,
      startTime: Date.now(),
      progress: 0
    })
  }
  
  const removeTask = (taskId) => {
    loadingTasks.value.delete(taskId)
  }
  
  const updateTask = (taskId, updates) => {
    const task = loadingTasks.value.get(taskId)
    if (task) {
      loadingTasks.value.set(taskId, { ...task, ...updates })
    }
  }
  
  const setTaskProgress = (taskId, progress) => {
    updateTask(taskId, { progress })
  }
  
  const clearAllTasks = () => {
    loadingTasks.value.clear()
  }
  
  const getTask = (taskId) => {
    return loadingTasks.value.get(taskId)
  }
  
  const getAllTasks = () => {
    return Array.from(loadingTasks.value.values())
  }
  
  const getTasksByMessage = (message) => {
    return Array.from(loadingTasks.value.values()).filter(task => 
      task.message.toLowerCase().includes(message.toLowerCase())
    )
  }
  
  // Utility functions
  const withLoading = async (asyncFn, message = '') => {
    setLoading(true, message)
    try {
      const result = await asyncFn()
      return result
    } finally {
      setLoading(false)
    }
  }
  
  const withTask = async (taskId, asyncFn, message = '') => {
    addTask(taskId, message)
    try {
      const result = await asyncFn((progress) => {
        setTaskProgress(taskId, progress)
      })
      return result
    } finally {
      removeTask(taskId)
    }
  }
  
  return {
    // State
    globalLoading,
    loadingTasks,
    loadingMessage,
    loadingProgress,
    
    // Getters
    isLoading,
    taskCount,
    hasMessage,
    hasProgress,
    
    // Actions
    setLoading,
    setProgress,
    addTask,
    removeTask,
    updateTask,
    setTaskProgress,
    clearAllTasks,
    getTask,
    getAllTasks,
    getTasksByMessage,
    withLoading,
    withTask
  }
})