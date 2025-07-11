<template>
  <div class="feature-tour">
    <!-- Tour Step Overlay -->
    <div
      v-if="isActive && currentStep"
      class="tour-overlay"
      @click="handleOverlayClick"
    >
      <!-- Spotlight Effect -->
      <div
        class="tour-spotlight"
        :style="spotlightStyle"
      />
      
      <!-- Tour Tooltip -->
      <div
        class="tour-tooltip"
        :style="tooltipStyle"
        :class="[`tour-tooltip--${currentStep.placement}`]"
      >
        <!-- Progress Bar -->
        <div class="tour-progress">
          <div
            class="tour-progress-bar"
            :style="{ width: progressPercentage + '%' }"
          />
        </div>
        
        <!-- Step Content -->
        <div class="tour-content">
          <div class="tour-header">
            <h3 class="tour-title">{{ currentStep.title }}</h3>
            <el-button
              text
              :icon="Close"
              @click="closeTour"
              class="tour-close"
              aria-label="Close tour"
            />
          </div>
          
          <div class="tour-body">
            <p class="tour-description">{{ currentStep.description }}</p>
            
            <!-- Optional Image/GIF -->
            <div v-if="currentStep.image" class="tour-media">
              <img
                :src="currentStep.image"
                :alt="currentStep.title"
                class="tour-image"
              />
            </div>
            
            <!-- Interactive Elements -->
            <div v-if="currentStep.interactive" class="tour-interactive">
              <component
                :is="currentStep.interactive.component"
                v-bind="currentStep.interactive.props"
                @completed="handleInteractiveCompleted"
              />
            </div>
          </div>
          
          <div class="tour-footer">
            <div class="tour-step-info">
              Step {{ currentStepIndex + 1 }} of {{ steps.length }}
            </div>
            
            <div class="tour-actions">
              <el-button
                v-if="currentStepIndex > 0"
                @click="previousStep"
                size="small"
              >
                Previous
              </el-button>
              
              <el-button
                v-if="!isLastStep"
                type="primary"
                @click="nextStep"
                size="small"
                :disabled="currentStep.interactive && !interactiveCompleted"
              >
                {{ currentStep.interactive && !interactiveCompleted ? 'Try it!' : 'Next' }}
              </el-button>
              
              <el-button
                v-else
                type="primary"
                @click="completeTour"
                size="small"
              >
                Finish Tour
              </el-button>
              
              <el-button
                text
                @click="skipTour"
                size="small"
                class="tour-skip"
              >
                Skip Tour
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Tour Trigger Button -->
    <el-button
      v-if="!isActive && showTrigger"
      @click="startTour"
      type="primary"
      :icon="QuestionFilled"
      class="tour-trigger"
    >
      Take a Tour
    </el-button>
  </div>
</template>

<script>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { Close, QuestionFilled } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

export default {
  name: 'FeatureTour',
  props: {
    tourId: {
      type: String,
      required: true
    },
    steps: {
      type: Array,
      required: true
    },
    autoStart: {
      type: Boolean,
      default: false
    },
    showTrigger: {
      type: Boolean,
      default: true
    }
  },
  emits: ['tour-started', 'tour-completed', 'tour-skipped', 'step-changed'],
  setup(props, { emit }) {
    const isActive = ref(false)
    const currentStepIndex = ref(0)
    const interactiveCompleted = ref(false)
    const spotlightStyle = ref({})
    const tooltipStyle = ref({})
    
    const currentStep = computed(() => {
      return props.steps[currentStepIndex.value]
    })
    
    const isLastStep = computed(() => {
      return currentStepIndex.value === props.steps.length - 1
    })
    
    const progressPercentage = computed(() => {
      return ((currentStepIndex.value + 1) / props.steps.length) * 100
    })
    
    const startTour = () => {
      if (hasCompletedTour()) {
        // Ask if user wants to retake the tour
        ElMessage.confirm(
          'You have already completed this tour. Would you like to take it again?',
          'Retake Tour',
          {
            type: 'info',
            confirmButtonText: 'Yes',
            cancelButtonText: 'No'
          }
        ).then(() => {
          initiateTour()
        }).catch(() => {
          // User cancelled
        })
      } else {
        initiateTour()
      }
    }
    
    const initiateTour = () => {
      isActive.value = true
      currentStepIndex.value = 0
      interactiveCompleted.value = false
      updateStepPosition()
      emit('tour-started', { tourId: props.tourId })
      
      // Disable page interactions
      document.body.style.overflow = 'hidden'
    }
    
    const nextStep = () => {
      if (currentStepIndex.value < props.steps.length - 1) {
        currentStepIndex.value++
        interactiveCompleted.value = false
        updateStepPosition()
        emit('step-changed', {
          step: currentStep.value,
          index: currentStepIndex.value
        })
      }
    }
    
    const previousStep = () => {
      if (currentStepIndex.value > 0) {
        currentStepIndex.value--
        interactiveCompleted.value = false
        updateStepPosition()
        emit('step-changed', {
          step: currentStep.value,
          index: currentStepIndex.value
        })
      }
    }
    
    const completeTour = () => {
      markTourAsCompleted()
      closeTour()
      emit('tour-completed', { tourId: props.tourId })
      ElMessage.success('Tour completed! You can retake it anytime from the help menu.')
    }
    
    const skipTour = () => {
      closeTour()
      emit('tour-skipped', { tourId: props.tourId })
    }
    
    const closeTour = () => {
      isActive.value = false
      document.body.style.overflow = ''
    }
    
    const updateStepPosition = async () => {
      await nextTick()
      
      if (!currentStep.value?.selector) {
        // Center the tooltip if no target element
        tooltipStyle.value = {
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          zIndex: 10001
        }
        spotlightStyle.value = {}
        return
      }
      
      const targetElement = document.querySelector(currentStep.value.selector)
      if (!targetElement) {
        console.warn(`Tour target element not found: ${currentStep.value.selector}`)
        return
      }
      
      const rect = targetElement.getBoundingClientRect()
      const scrollTop = window.pageYOffset || document.documentElement.scrollTop
      const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft
      
      // Create spotlight effect
      const padding = 8
      spotlightStyle.value = {
        position: 'absolute',
        top: rect.top + scrollTop - padding + 'px',
        left: rect.left + scrollLeft - padding + 'px',
        width: rect.width + (padding * 2) + 'px',
        height: rect.height + (padding * 2) + 'px',
        borderRadius: '8px',
        boxShadow: '0 0 0 4px rgba(255, 255, 255, 0.8), 0 0 0 9999px rgba(0, 0, 0, 0.5)',
        zIndex: 10000
      }
      
      // Position tooltip
      const tooltipWidth = 350
      const tooltipHeight = 200 // Approximate
      const placement = currentStep.value.placement || 'bottom'
      
      let top, left
      
      switch (placement) {
        case 'top':
          top = rect.top + scrollTop - tooltipHeight - 16
          left = rect.left + scrollLeft + (rect.width / 2) - (tooltipWidth / 2)
          break
        case 'bottom':
          top = rect.bottom + scrollTop + 16
          left = rect.left + scrollLeft + (rect.width / 2) - (tooltipWidth / 2)
          break
        case 'left':
          top = rect.top + scrollTop + (rect.height / 2) - (tooltipHeight / 2)
          left = rect.left + scrollLeft - tooltipWidth - 16
          break
        case 'right':
          top = rect.top + scrollTop + (rect.height / 2) - (tooltipHeight / 2)
          left = rect.right + scrollLeft + 16
          break
        default:
          top = rect.bottom + scrollTop + 16
          left = rect.left + scrollLeft + (rect.width / 2) - (tooltipWidth / 2)
      }
      
      // Keep tooltip within viewport
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight
      
      if (left < 16) left = 16
      if (left + tooltipWidth > viewportWidth - 16) left = viewportWidth - tooltipWidth - 16
      if (top < 16) top = 16
      if (top + tooltipHeight > viewportHeight - 16) top = viewportHeight - tooltipHeight - 16
      
      tooltipStyle.value = {
        position: 'absolute',
        top: top + 'px',
        left: left + 'px',
        width: tooltipWidth + 'px',
        zIndex: 10001
      }
      
      // Scroll element into view if needed
      targetElement.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'center'
      })
    }
    
    const handleInteractiveCompleted = () => {
      interactiveCompleted.value = true
    }
    
    const handleOverlayClick = (event) => {
      // Only close if clicking on the overlay itself, not the tooltip
      if (event.target.classList.contains('tour-overlay')) {
        skipTour()
      }
    }
    
    const hasCompletedTour = () => {
      const completed = localStorage.getItem(`datamesh-tour-${props.tourId}`)
      return completed === 'true'
    }
    
    const markTourAsCompleted = () => {
      localStorage.setItem(`datamesh-tour-${props.tourId}`, 'true')
    }
    
    const handleKeydown = (event) => {
      if (!isActive.value) return
      
      switch (event.key) {
        case 'Escape':
          skipTour()
          break
        case 'ArrowRight':
        case 'ArrowDown':
          if (!isLastStep.value) nextStep()
          break
        case 'ArrowLeft':
        case 'ArrowUp':
          if (currentStepIndex.value > 0) previousStep()
          break
        case 'Enter':
          if (isLastStep.value) {
            completeTour()
          } else if (!currentStep.value.interactive || interactiveCompleted.value) {
            nextStep()
          }
          break
      }
    }
    
    // Watch for step changes to update positioning
    watch(currentStepIndex, () => {
      updateStepPosition()
    })
    
    // Handle window resize
    const handleResize = () => {
      if (isActive.value) {
        updateStepPosition()
      }
    }
    
    onMounted(() => {
      if (props.autoStart && !hasCompletedTour()) {
        setTimeout(startTour, 1000) // Small delay to let page load
      }
      
      document.addEventListener('keydown', handleKeydown)
      window.addEventListener('resize', handleResize)
    })
    
    onUnmounted(() => {
      document.removeEventListener('keydown', handleKeydown)
      window.removeEventListener('resize', handleResize)
      document.body.style.overflow = ''
    })
    
    return {
      isActive,
      currentStepIndex,
      currentStep,
      isLastStep,
      progressPercentage,
      interactiveCompleted,
      spotlightStyle,
      tooltipStyle,
      startTour,
      nextStep,
      previousStep,
      completeTour,
      skipTour,
      closeTour,
      handleInteractiveCompleted,
      handleOverlayClick,
      Close,
      QuestionFilled
    }
  }
}
</script>

<style scoped>
.tour-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 9999;
  pointer-events: auto;
}

.tour-spotlight {
  pointer-events: none;
}

.tour-tooltip {
  background: white;
  border-radius: 12px;
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
  border: 1px solid var(--el-border-color-lighter);
  overflow: hidden;
  min-height: 200px;
  display: flex;
  flex-direction: column;
}

.tour-progress {
  height: 3px;
  background-color: var(--el-fill-color-light);
}

.tour-progress-bar {
  height: 100%;
  background-color: var(--el-color-primary);
  transition: width 0.3s ease;
}

.tour-content {
  padding: 20px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.tour-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.tour-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  flex: 1;
}

.tour-close {
  color: var(--el-text-color-secondary);
  margin-left: 8px;
}

.tour-body {
  flex: 1;
  margin-bottom: 20px;
}

.tour-description {
  margin: 0 0 16px 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--el-text-color-regular);
}

.tour-media {
  margin-bottom: 16px;
}

.tour-image {
  width: 100%;
  max-height: 150px;
  object-fit: cover;
  border-radius: 8px;
  border: 1px solid var(--el-border-color-lighter);
}

.tour-interactive {
  padding: 16px;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  border: 1px dashed var(--el-border-color);
}

.tour-footer {
  border-top: 1px solid var(--el-border-color-lighter);
  padding-top: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tour-step-info {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.tour-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.tour-skip {
  color: var(--el-text-color-secondary);
}

.tour-trigger {
  position: fixed;
  bottom: 20px;
  left: 20px;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Tooltip arrow styles */
.tour-tooltip--top::before {
  content: '';
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-top: 8px solid white;
}

.tour-tooltip--bottom::before {
  content: '';
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid white;
}

.tour-tooltip--left::before {
  content: '';
  position: absolute;
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-top: 8px solid transparent;
  border-bottom: 8px solid transparent;
  border-left: 8px solid white;
}

.tour-tooltip--right::before {
  content: '';
  position: absolute;
  right: 100%;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-top: 8px solid transparent;
  border-bottom: 8px solid transparent;
  border-right: 8px solid white;
}

/* Dark theme support */
.dark .tour-tooltip {
  background: var(--el-bg-color);
  border-color: var(--el-border-color);
}

.dark .tour-tooltip--top::before,
.dark .tour-tooltip--bottom::before,
.dark .tour-tooltip--left::before,
.dark .tour-tooltip--right::before {
  border-color: var(--el-bg-color);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .tour-tooltip {
    width: calc(100vw - 32px) !important;
    max-width: 350px;
  }
  
  .tour-trigger {
    bottom: 16px;
    left: 16px;
  }
  
  .tour-content {
    padding: 16px;
  }
  
  .tour-actions {
    flex-wrap: wrap;
    gap: 4px;
  }
  
  .tour-actions .el-button {
    font-size: 12px;
    padding: 6px 12px;
  }
}
</style>