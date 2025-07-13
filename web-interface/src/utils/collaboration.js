/**
 * Real-time Collaboration System
 */

/**
 * WebSocket-based real-time collaboration
 */
export class CollaborationManager {
  constructor(options = {}) {
    this.options = {
      wsUrl: options.wsUrl || `ws://${window.location.host}/ws`,
      reconnectInterval: options.reconnectInterval || 3000,
      maxReconnectAttempts: options.maxReconnectAttempts || 5,
      heartbeatInterval: options.heartbeatInterval || 30000,
      presence: options.presence || true,
      ...options
    }

    this.ws = null
    this.reconnectAttempts = 0
    this.isConnected = false
    this.isReconnecting = false
    this.heartbeatTimer = null
    this.presenceTimer = null

    // Event listeners
    this.eventListeners = new Map()
    
    // Active sessions and users
    this.sessions = new Map()
    this.users = new Map()
    this.currentUser = null

    // Document synchronization
    this.documentState = new Map()
    this.operationQueue = []
    this.acknowledgedOperations = new Set()

    // Presence tracking
    this.presenceData = new Map()
    this.userCursors = new Map()
    
    this.init()
  }

  /**
   * Initialize collaboration system
   */
  async init() {
    await this.connect()
    this.setupEventListeners()
    
    if (this.options.presence) {
      this.startPresenceTracking()
    }
  }

  /**
   * Connect to WebSocket server
   */
  async connect() {
    if (this.isConnected || this.isReconnecting) return

    this.isReconnecting = true

    try {
      this.ws = new WebSocket(this.options.wsUrl)
      
      this.ws.onopen = () => {
        console.log('🔗 Collaboration: Connected to server')
        this.isConnected = true
        this.isReconnecting = false
        this.reconnectAttempts = 0
        
        this.startHeartbeat()
        this.emit('connected')
        this.sendPendingOperations()
      }

      this.ws.onmessage = (event) => {
        this.handleMessage(JSON.parse(event.data))
      }

      this.ws.onclose = () => {
        console.log('🔌 Collaboration: Disconnected from server')
        this.isConnected = false
        this.stopHeartbeat()
        
        this.emit('disconnected')
        this.scheduleReconnect()
      }

      this.ws.onerror = (error) => {
        console.error('🚫 Collaboration: WebSocket error:', error)
        this.emit('error', error)
      }

    } catch (error) {
      console.error('🚫 Collaboration: Connection failed:', error)
      this.isReconnecting = false
      this.scheduleReconnect()
    }
  }

  /**
   * Schedule reconnection attempt
   */
  scheduleReconnect() {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('🚫 Collaboration: Max reconnection attempts reached')
      this.emit('maxReconnectAttemptsReached')
      return
    }

    setTimeout(() => {
      this.reconnectAttempts++
      console.log(`🔄 Collaboration: Reconnecting attempt ${this.reconnectAttempts}`)
      this.connect()
    }, this.options.reconnectInterval)
  }

  /**
   * Handle incoming WebSocket messages
   */
  handleMessage(message) {
    const { type, data, sessionId, userId } = message

    switch (type) {
      case 'operation':
        this.handleOperation(data, sessionId, userId)
        break
      
      case 'presence':
        this.handlePresence(data, userId)
        break
      
      case 'userJoined':
        this.handleUserJoined(data)
        break
      
      case 'userLeft':
        this.handleUserLeft(data)
        break
      
      case 'documentState':
        this.handleDocumentState(data)
        break
      
      case 'operationAck':
        this.handleOperationAck(data)
        break
      
      case 'conflict':
        this.handleConflict(data)
        break
      
      case 'heartbeat':
        this.handleHeartbeat()
        break
      
      default:
        console.warn('🤔 Collaboration: Unknown message type:', type)
    }
  }

  /**
   * Handle document operations
   */
  handleOperation(operation, sessionId, userId) {
    if (this.acknowledgedOperations.has(operation.id)) {
      return // Already processed
    }

    this.acknowledgedOperations.add(operation.id)
    
    // Apply operation to document
    this.applyOperation(operation)
    
    // Update document state
    this.updateDocumentState(operation)
    
    // Emit operation event
    this.emit('operation', {
      operation,
      sessionId,
      userId,
      user: this.users.get(userId)
    })
  }

  /**
   * Handle presence updates
   */
  handlePresence(presence, userId) {
    this.presenceData.set(userId, {
      ...presence,
      timestamp: Date.now()
    })
    
    // Update user cursor position
    if (presence.cursor) {
      this.userCursors.set(userId, presence.cursor)
    }
    
    this.emit('presence', {
      userId,
      presence,
      user: this.users.get(userId)
    })
  }

  /**
   * Handle user joined
   */
  handleUserJoined(user) {
    this.users.set(user.id, user)
    this.emit('userJoined', user)
  }

  /**
   * Handle user left
   */
  handleUserLeft(userId) {
    const user = this.users.get(userId)
    this.users.delete(userId)
    this.presenceData.delete(userId)
    this.userCursors.delete(userId)
    
    this.emit('userLeft', { userId, user })
  }

  /**
   * Handle document state synchronization
   */
  handleDocumentState(state) {
    this.documentState.set(state.documentId, state)
    this.emit('documentState', state)
  }

  /**
   * Handle operation acknowledgment
   */
  handleOperationAck(ackData) {
    const { operationId, success, error } = ackData
    
    if (success) {
      this.acknowledgedOperations.add(operationId)
      // Remove from queue if present
      this.operationQueue = this.operationQueue.filter(op => op.id !== operationId)
    } else {
      console.error('🚫 Collaboration: Operation failed:', error)
      this.emit('operationError', { operationId, error })
    }
  }

  /**
   * Handle conflicts
   */
  handleConflict(conflictData) {
    console.warn('⚠️ Collaboration: Conflict detected:', conflictData)
    
    // Implement conflict resolution strategy
    this.resolveConflict(conflictData)
    
    this.emit('conflict', conflictData)
  }

  /**
   * Send operation to server
   */
  sendOperation(operation) {
    if (!this.isConnected) {
      // Queue operation for later
      this.operationQueue.push(operation)
      return
    }

    this.send({
      type: 'operation',
      data: operation
    })
  }

  /**
   * Send presence update
   */
  sendPresence(presenceData) {
    if (!this.isConnected) return

    this.send({
      type: 'presence',
      data: presenceData
    })
  }

  /**
   * Send message to server
   */
  send(message) {
    if (this.ws && this.isConnected) {
      this.ws.send(JSON.stringify(message))
    }
  }

  /**
   * Send pending operations
   */
  sendPendingOperations() {
    if (this.operationQueue.length === 0) return

    console.log(`📤 Collaboration: Sending ${this.operationQueue.length} pending operations`)
    
    this.operationQueue.forEach(operation => {
      this.sendOperation(operation)
    })
  }

  /**
   * Apply operation to document
   */
  applyOperation(operation) {
    const { type, path, value, oldValue } = operation

    switch (type) {
      case 'insert':
        this.insertText(path, value)
        break
      
      case 'delete':
        this.deleteText(path, oldValue.length)
        break
      
      case 'replace':
        this.replaceText(path, value, oldValue)
        break
      
      case 'move':
        this.moveText(path, operation.targetPath)
        break
      
      default:
        console.warn('🤔 Collaboration: Unknown operation type:', type)
    }
  }

  /**
   * Insert text at path
   */
  insertText(path, text) {
    // Implementation depends on document structure
    // This is a placeholder for actual text insertion logic
    console.log('📝 Collaboration: Insert text at', path, ':', text)
  }

  /**
   * Delete text at path
   */
  deleteText(path, length) {
    console.log('🗑️ Collaboration: Delete text at', path, ', length:', length)
  }

  /**
   * Replace text at path
   */
  replaceText(path, newText, oldText) {
    console.log('🔄 Collaboration: Replace text at', path, ':', oldText, '->', newText)
  }

  /**
   * Move text from source to target
   */
  moveText(sourcePath, targetPath) {
    console.log('📦 Collaboration: Move text from', sourcePath, 'to', targetPath)
  }

  /**
   * Update document state
   */
  updateDocumentState(operation) {
    // Update local document state based on operation
    // This maintains consistency across all clients
  }

  /**
   * Resolve conflicts using operational transformation
   */
  resolveConflict(conflictData) {
    // Implement operational transformation algorithms
    // This is a complex topic that requires careful implementation
    console.log('🔧 Collaboration: Resolving conflict:', conflictData)
  }

  /**
   * Start heartbeat to maintain connection
   */
  startHeartbeat() {
    this.heartbeatTimer = setInterval(() => {
      this.send({ type: 'heartbeat' })
    }, this.options.heartbeatInterval)
  }

  /**
   * Stop heartbeat
   */
  stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  /**
   * Handle heartbeat response
   */
  handleHeartbeat() {
    // Server is alive, connection is good
  }

  /**
   * Start presence tracking
   */
  startPresenceTracking() {
    this.presenceTimer = setInterval(() => {
      this.sendPresence({
        timestamp: Date.now(),
        active: document.hasFocus(),
        cursor: this.getCurrentCursor()
      })
    }, 2000) // Send presence every 2 seconds
  }

  /**
   * Stop presence tracking
   */
  stopPresenceTracking() {
    if (this.presenceTimer) {
      clearInterval(this.presenceTimer)
      this.presenceTimer = null
    }
  }

  /**
   * Get current cursor position
   */
  getCurrentCursor() {
    const selection = window.getSelection()
    if (selection.rangeCount === 0) return null

    const range = selection.getRangeAt(0)
    return {
      start: range.startOffset,
      end: range.endOffset,
      container: this.getElementPath(range.startContainer)
    }
  }

  /**
   * Get DOM element path
   */
  getElementPath(element) {
    const path = []
    let current = element

    while (current && current !== document.body) {
      if (current.nodeType === Node.ELEMENT_NODE) {
        path.unshift(current.tagName.toLowerCase())
      }
      current = current.parentNode
    }

    return path.join(' > ')
  }

  /**
   * Set current user
   */
  setCurrentUser(user) {
    this.currentUser = user
    this.users.set(user.id, user)
  }

  /**
   * Get online users
   */
  getOnlineUsers() {
    return Array.from(this.users.values())
  }

  /**
   * Get user presence
   */
  getUserPresence(userId) {
    return this.presenceData.get(userId)
  }

  /**
   * Get user cursor
   */
  getUserCursor(userId) {
    return this.userCursors.get(userId)
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    // Handle page visibility changes
    document.addEventListener('visibilitychange', () => {
      if (this.options.presence) {
        this.sendPresence({
          timestamp: Date.now(),
          active: !document.hidden
        })
      }
    })

    // Handle beforeunload
    window.addEventListener('beforeunload', () => {
      this.disconnect()
    })
  }

  /**
   * Add event listener
   */
  on(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, [])
    }
    this.eventListeners.get(event).push(callback)
  }

  /**
   * Remove event listener
   */
  off(event, callback) {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      const index = listeners.indexOf(callback)
      if (index !== -1) {
        listeners.splice(index, 1)
      }
    }
  }

  /**
   * Emit event
   */
  emit(event, data) {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      listeners.forEach(callback => callback(data))
    }
  }

  /**
   * Disconnect from server
   */
  disconnect() {
    this.stopHeartbeat()
    this.stopPresenceTracking()
    
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    
    this.isConnected = false
  }

  /**
   * Get connection status
   */
  isOnline() {
    return this.isConnected
  }

  /**
   * Get statistics
   */
  getStats() {
    return {
      connected: this.isConnected,
      users: this.users.size,
      operations: this.operationQueue.length,
      acknowledged: this.acknowledgedOperations.size,
      presenceData: this.presenceData.size
    }
  }
}

/**
 * Shared document manager
 */
export class SharedDocument {
  constructor(documentId, collaborationManager) {
    this.documentId = documentId
    this.collaboration = collaborationManager
    this.content = ''
    this.version = 0
    this.operations = []
    
    this.setupEventListeners()
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    this.collaboration.on('operation', (event) => {
      if (event.operation.documentId === this.documentId) {
        this.handleRemoteOperation(event.operation)
      }
    })
  }

  /**
   * Handle remote operation
   */
  handleRemoteOperation(operation) {
    this.applyOperation(operation)
    this.version++
    this.operations.push(operation)
  }

  /**
   * Apply local operation
   */
  applyLocalOperation(operation) {
    operation.documentId = this.documentId
    operation.version = this.version
    operation.timestamp = Date.now()
    
    this.applyOperation(operation)
    this.collaboration.sendOperation(operation)
    
    this.version++
    this.operations.push(operation)
  }

  /**
   * Apply operation to document
   */
  applyOperation(operation) {
    // Apply operation to document content
    // This depends on the document structure and operation type
  }

  /**
   * Insert text
   */
  insertText(position, text) {
    this.applyLocalOperation({
      type: 'insert',
      position,
      text,
      id: this.generateOperationId()
    })
  }

  /**
   * Delete text
   */
  deleteText(position, length) {
    this.applyLocalOperation({
      type: 'delete',
      position,
      length,
      id: this.generateOperationId()
    })
  }

  /**
   * Generate unique operation ID
   */
  generateOperationId() {
    return `${this.documentId}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }
}

export const collaboration = new CollaborationManager()

export default {
  CollaborationManager,
  SharedDocument,
  collaboration
}
