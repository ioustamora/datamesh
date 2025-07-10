<template>
  <div 
    ref="scrollerRef" 
    class="virtual-scroller" 
    :style="{ height: containerHeight }"
    @scroll="handleScroll"
    @wheel="handleWheel"
    :aria-label="ariaLabel || 'Virtual scrolling list'"
    role="region"
    tabindex="0"
    @keydown="handleKeydown"
  >
    <!-- Spacer for items before visible area -->
    <div 
      class="virtual-spacer-top" 
      :style="{ height: offsetY + 'px' }"
      aria-hidden="true"
    />
    
    <!-- Visible items -->
    <div 
      class="virtual-items-container"
      :style="{ 
        transform: `translateY(${translateY}px)`,
        paddingBottom: paddingBottom + 'px'
      }"
    >
      <div
        v-for="(item, index) in visibleItems"
        :key="getItemKey(item, startIndex + index)"
        class="virtual-item"
        :style="{ 
          height: getItemHeight(item, startIndex + index) + 'px',
          minHeight: getItemHeight(item, startIndex + index) + 'px'
        }"
        :data-index="startIndex + index"
        :aria-setsize="totalItems"
        :aria-posinset="startIndex + index + 1"
        role="listitem"
      >
        <slot 
          :item="item" 
          :index="startIndex + index" 
          :is-visible="true"
        >
          <div class="default-item">
            {{ item }}
          </div>
        </slot>
      </div>
    </div>
    
    <!-- Spacer for items after visible area -->
    <div 
      class="virtual-spacer-bottom" 
      :style="{ height: (totalHeight - offsetY - visibleHeight) + 'px' }"
      aria-hidden="true"
    />
    
    <!-- Loading indicator -->
    <div v-if="loading" class="virtual-loading">
      <el-skeleton :rows="Math.min(5, itemsPerPage)" animated />
    </div>
    
    <!-- Empty state -->
    <div v-if="!loading && totalItems === 0" class="virtual-empty">
      <slot name="empty">
        <el-empty description="No items to display" />
      </slot>
    </div>
    
    <!-- Scroll indicators -->
    <div v-if="showScrollIndicators" class="scroll-indicators">
      <div 
        v-if="canScrollUp" 
        class="scroll-indicator scroll-up"
        aria-hidden="true"
      >
        <el-icon><ArrowUp /></el-icon>
      </div>
      <div 
        v-if="canScrollDown" 
        class="scroll-indicator scroll-down"
        aria-hidden="true"
      >
        <el-icon><ArrowDown /></el-icon>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useEventListener, useTimeout } from '@/composables/useWebSocket'
import { ArrowUp, ArrowDown } from '@element-plus/icons-vue'

export default {
  name: 'VirtualScroller',
  components: {
    ArrowUp,
    ArrowDown
  },
  props: {
    // Data items to display
    items: {
      type: Array,
      required: true
    },
    
    // Height of each item (can be function for dynamic heights)
    itemHeight: {
      type: [Number, Function],
      default: 60
    },
    
    // Container height
    containerHeight: {
      type: String,
      default: '400px'
    },
    
    // Buffer size (number of items to render outside visible area)
    buffer: {
      type: Number,
      default: 5
    },
    
    // Key function for item identification
    itemKey: {
      type: [String, Function],
      default: 'id'
    },
    
    // Loading state
    loading: {
      type: Boolean,
      default: false
    },
    
    // Enable infinite scrolling
    infiniteScroll: {
      type: Boolean,
      default: false
    },
    
    // Threshold for infinite scroll trigger (pixels from bottom)
    infiniteScrollThreshold: {
      type: Number,
      default: 100
    },
    
    // Show scroll indicators
    showScrollIndicators: {
      type: Boolean,
      default: true
    },
    
    // Accessibility label
    ariaLabel: {
      type: String,
      default: ''
    },
    
    // Enable keyboard navigation
    keyboardNavigation: {
      type: Boolean,
      default: true
    },
    
    // Scroll behavior
    scrollBehavior: {
      type: String,
      default: 'smooth',
      validator: value => ['auto', 'smooth'].includes(value)
    }
  ],
  emits: [
    'scroll',
    'item-click',
    'load-more',
    'selection-change',
    'visible-range-change'
  ],
  setup(props, { emit }) {
    const scrollerRef = ref(null)
    const { addEventListener } = useEventListener()
    const { setTimeout } = useTimeout()
    
    // Reactive state
    const scrollTop = ref(0)
    const containerClientHeight = ref(0)
    const isScrolling = ref(false)
    const selectedIndex = ref(-1)
    const resizeObserver = ref(null)
    
    // Cached item heights for performance
    const itemHeightCache = new Map()
    
    // Computed properties
    const totalItems = computed(() => props.items.length)
    
    const getItemHeight = (item, index) => {
      if (typeof props.itemHeight === 'function') {
        // Check cache first
        const cacheKey = getItemKey(item, index)
        if (itemHeightCache.has(cacheKey)) {
          return itemHeightCache.get(cacheKey)
        }
        
        const height = props.itemHeight(item, index)
        itemHeightCache.set(cacheKey, height)
        return height
      }
      return props.itemHeight
    }
    
    const getItemKey = (item, index) => {
      if (typeof props.itemKey === 'function') {
        return props.itemKey(item, index)
      }
      return item[props.itemKey] || index
    }
    
    const totalHeight = computed(() => {
      if (typeof props.itemHeight === 'number') {
        return totalItems.value * props.itemHeight
      }
      
      // Calculate total height for dynamic heights
      let height = 0
      for (let i = 0; i < totalItems.value; i++) {
        height += getItemHeight(props.items[i], i)
      }
      return height
    })
    
    const itemsPerPage = computed(() => {
      return Math.ceil(containerClientHeight.value / (props.itemHeight || 60)) + props.buffer * 2
    })
    
    const startIndex = computed(() => {
      if (typeof props.itemHeight === 'number') {
        return Math.floor(scrollTop.value / props.itemHeight)
      }
      
      // Binary search for dynamic heights
      let top = 0
      for (let i = 0; i < totalItems.value; i++) {
        const itemHeight = getItemHeight(props.items[i], i)
        if (top + itemHeight > scrollTop.value) {
          return Math.max(0, i - props.buffer)
        }
        top += itemHeight
      }
      return 0
    })
    
    const endIndex = computed(() => {
      return Math.min(totalItems.value - 1, startIndex.value + itemsPerPage.value)
    })
    
    const visibleItems = computed(() => {
      return props.items.slice(startIndex.value, endIndex.value + 1)
    })
    
    const offsetY = computed(() => {
      if (typeof props.itemHeight === 'number') {
        return startIndex.value * props.itemHeight
      }
      
      // Calculate offset for dynamic heights
      let offset = 0
      for (let i = 0; i < startIndex.value; i++) {
        offset += getItemHeight(props.items[i], i)
      }
      return offset
    })
    
    const visibleHeight = computed(() => {
      let height = 0
      for (let i = startIndex.value; i <= endIndex.value; i++) {
        if (i < totalItems.value) {
          height += getItemHeight(props.items[i], i)
        }
      }
      return height
    })
    
    const translateY = ref(0)
    const paddingBottom = ref(0)
    
    const canScrollUp = computed(() => scrollTop.value > 0)
    const canScrollDown = computed(() => 
      scrollTop.value < totalHeight.value - containerClientHeight.value
    )
    
    // Methods
    const updateContainerHeight = () => {
      if (scrollerRef.value) {
        containerClientHeight.value = scrollerRef.value.clientHeight
      }
    }
    
    const handleScroll = (event) => {
      scrollTop.value = event.target.scrollTop
      
      // Set scrolling state
      isScrolling.value = true
      setTimeout(() => {
        isScrolling.value = false
      }, 150)
      
      emit('scroll', {
        scrollTop: scrollTop.value,
        scrollHeight: totalHeight.value,
        clientHeight: containerClientHeight.value,
        isScrolling: isScrolling.value
      })
      
      // Infinite scroll check
      if (props.infiniteScroll && !props.loading) {
        const { scrollTop: top, scrollHeight, clientHeight } = event.target
        const distanceFromBottom = scrollHeight - (top + clientHeight)
        
        if (distanceFromBottom <= props.infiniteScrollThreshold) {
          emit('load-more')
        }
      }
      
      // Emit visible range change
      emit('visible-range-change', {
        startIndex: startIndex.value,
        endIndex: endIndex.value,
        visibleItems: visibleItems.value
      })
    }
    
    const handleWheel = (event) => {
      // Prevent horizontal scrolling
      if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) {
        event.preventDefault()
      }
    }
    
    const handleKeydown = (event) => {
      if (!props.keyboardNavigation) return
      
      const { key } = event
      const currentIndex = selectedIndex.value
      
      switch (key) {
        case 'ArrowDown':
          event.preventDefault()
          selectItem(Math.min(totalItems.value - 1, currentIndex + 1))
          break
        
        case 'ArrowUp':
          event.preventDefault()
          selectItem(Math.max(0, currentIndex - 1))
          break
        
        case 'PageDown':
          event.preventDefault()
          selectItem(Math.min(totalItems.value - 1, currentIndex + itemsPerPage.value))
          break
        
        case 'PageUp':
          event.preventDefault()
          selectItem(Math.max(0, currentIndex - itemsPerPage.value))
          break
        
        case 'Home':
          event.preventDefault()
          selectItem(0)
          break
        
        case 'End':
          event.preventDefault()
          selectItem(totalItems.value - 1)
          break
        
        case 'Enter':
        case ' ':
          event.preventDefault()
          if (currentIndex >= 0) {
            emit('item-click', {
              item: props.items[currentIndex],
              index: currentIndex
            })
          }
          break
      }
    }
    
    const selectItem = (index) => {
      selectedIndex.value = index
      scrollToItem(index)
      emit('selection-change', {
        item: props.items[index],
        index
      })
    }
    
    const scrollToItem = (index, behavior = props.scrollBehavior) => {
      if (!scrollerRef.value || index < 0 || index >= totalItems.value) return
      
      let targetScrollTop = 0
      
      if (typeof props.itemHeight === 'number') {
        targetScrollTop = index * props.itemHeight
      } else {
        // Calculate scroll position for dynamic heights
        for (let i = 0; i < index; i++) {
          targetScrollTop += getItemHeight(props.items[i], i)
        }
      }
      
      scrollerRef.value.scrollTo({
        top: targetScrollTop,
        behavior
      })
    }
    
    const scrollToTop = (behavior = props.scrollBehavior) => {
      if (scrollerRef.value) {
        scrollerRef.value.scrollTo({
          top: 0,
          behavior
        })
      }
    }
    
    const scrollToBottom = (behavior = props.scrollBehavior) => {
      if (scrollerRef.value) {
        scrollerRef.value.scrollTo({
          top: totalHeight.value,
          behavior
        })
      }
    }
    
    const refresh = async () => {
      // Clear height cache
      itemHeightCache.clear()
      
      // Force re-render
      await nextTick()
      updateContainerHeight()
    }
    
    // Watchers
    watch(() => props.items, () => {
      // Clear cache when items change
      itemHeightCache.clear()
      selectedIndex.value = -1
    }, { deep: true })
    
    watch([startIndex, endIndex], () => {
      emit('visible-range-change', {
        startIndex: startIndex.value,
        endIndex: endIndex.value,
        visibleItems: visibleItems.value
      })
    })
    
    // Lifecycle
    onMounted(() => {
      updateContainerHeight()
      
      // Set up resize observer
      if (window.ResizeObserver) {
        resizeObserver.value = new ResizeObserver(() => {
          updateContainerHeight()
        })
        resizeObserver.value.observe(scrollerRef.value)
      }
      
      // Fallback resize listener
      addEventListener(window, 'resize', updateContainerHeight)
    })
    
    onUnmounted(() => {
      if (resizeObserver.value) {
        resizeObserver.value.disconnect()
      }
    })
    
    return {
      // Template refs
      scrollerRef,
      
      // State
      scrollTop,
      isScrolling,
      selectedIndex,
      
      // Computed
      totalItems,
      totalHeight,
      startIndex,
      endIndex,
      visibleItems,
      offsetY,
      visibleHeight,
      translateY,
      paddingBottom,
      canScrollUp,
      canScrollDown,
      
      // Methods
      getItemKey,
      getItemHeight,
      handleScroll,
      handleWheel,
      handleKeydown,
      selectItem,
      scrollToItem,
      scrollToTop,
      scrollToBottom,
      refresh
    }
  }
}
</script>

<style scoped>
.virtual-scroller {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
  will-change: scroll-position;
}

.virtual-scroller:focus {
  outline: 2px solid var(--el-color-primary);
  outline-offset: -2px;
}

.virtual-items-container {
  position: relative;
  will-change: transform;
}

.virtual-item {
  position: relative;
  box-sizing: border-box;
  overflow: hidden;
}

.virtual-spacer-top,
.virtual-spacer-bottom {
  flex-shrink: 0;
}

.virtual-loading {
  padding: 16px;
  background: var(--el-bg-color);
}

.virtual-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 200px;
}

.default-item {
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color);
}

.scroll-indicators {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 10;
  pointer-events: none;
}

.scroll-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color);
  border-radius: 50%;
  margin: 4px 0;
  opacity: 0.7;
  transition: opacity 0.3s ease;
  backdrop-filter: blur(4px);
}

.scroll-indicator.scroll-up {
  animation: bounce-up 1.5s infinite;
}

.scroll-indicator.scroll-down {
  animation: bounce-down 1.5s infinite;
}

@keyframes bounce-up {
  0%, 20%, 50%, 80%, 100% {
    transform: translateY(0);
  }
  40% {
    transform: translateY(-4px);
  }
  60% {
    transform: translateY(-2px);
  }
}

@keyframes bounce-down {
  0%, 20%, 50%, 80%, 100% {
    transform: translateY(0);
  }
  40% {
    transform: translateY(4px);
  }
  60% {
    transform: translateY(2px);
  }
}

/* Custom scrollbar */
.virtual-scroller::-webkit-scrollbar {
  width: 8px;
}

.virtual-scroller::-webkit-scrollbar-track {
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
}

.virtual-scroller::-webkit-scrollbar-thumb {
  background: var(--el-border-color-darker);
  border-radius: 4px;
  transition: background-color 0.3s ease;
}

.virtual-scroller::-webkit-scrollbar-thumb:hover {
  background: var(--el-border-color-dark);
}

/* Dark mode */
.dark .virtual-scroller::-webkit-scrollbar-track {
  background: var(--el-fill-color-dark);
}

.dark .virtual-scroller::-webkit-scrollbar-thumb {
  background: var(--el-border-color-light);
}

.dark .virtual-scroller::-webkit-scrollbar-thumb:hover {
  background: var(--el-border-color);
}

/* Performance optimizations */
.virtual-scroller * {
  box-sizing: border-box;
}

.virtual-item {
  contain: layout style paint;
}

/* Mobile optimizations */
@media (max-width: 768px) {
  .virtual-scroller {
    -webkit-overflow-scrolling: touch;
  }
  
  .scroll-indicators {
    display: none;
  }
  
  .virtual-scroller::-webkit-scrollbar {
    display: none;
  }
}

/* Accessibility improvements */
@media (prefers-reduced-motion: reduce) {
  .scroll-indicator {
    animation: none;
  }
  
  .virtual-scroller {
    scroll-behavior: auto !important;
  }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .virtual-item {
    border-bottom: 2px solid;
  }
  
  .scroll-indicator {
    border-width: 2px;
  }
}
</style>