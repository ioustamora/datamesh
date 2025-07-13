import { useEffect, useRef, useState } from 'react'
import { WSMessage } from '@/types'
import toast from 'react-hot-toast'

interface UseWebSocketReturn {
  socket: WebSocket | null
  isConnected: boolean
  lastMessage: WSMessage | null
  sendMessage: (message: any) => void
  disconnect: () => void
}

export const useWebSocket = (url: string | null): UseWebSocketReturn => {
  const [socket, setSocket] = useState<WebSocket | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const [lastMessage, setLastMessage] = useState<WSMessage | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>()
  const reconnectAttemptsRef = useRef(0)
  const maxReconnectAttempts = 5

  const connect = (wsUrl: string) => {
    try {
      const ws = new WebSocket(wsUrl)
      
      ws.onopen = () => {
        console.log('WebSocket connected')
        setIsConnected(true)
        reconnectAttemptsRef.current = 0
        
        // Send ping to keep connection alive
        const ping = { type: 'ping', timestamp: new Date().toISOString() }
        ws.send(JSON.stringify(ping))
      }
      
      ws.onmessage = (event) => {
        try {
          const message: WSMessage = JSON.parse(event.data)
          setLastMessage(message)
          
          // Handle different message types
          switch (message.type) {
            case 'FileUploadProgress':
              // Could show toast or update progress UI
              break
              
            case 'FileDownloadProgress':
              // Could show toast or update progress UI
              break
              
            case 'SystemStatus':
              if (message.status === 'error') {
                toast.error(message.message)
              } else if (message.status === 'warning') {
                toast(message.message, { icon: '⚠️' })
              }
              break
              
            case 'NetworkHealth':
              // Update network status indicators
              break
              
            case 'GovernanceUpdate':
              toast.success(`Governance: ${message.event_type}`)
              break
              
            case 'CacheStats':
              // Update cache statistics
              break
              
            default:
              console.log('Unknown WebSocket message:', message)
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }
      
      ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason)
        setIsConnected(false)
        setSocket(null)
        
        // Attempt to reconnect if not intentionally closed
        if (event.code !== 1000 && reconnectAttemptsRef.current < maxReconnectAttempts) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000)
          console.log(`Attempting to reconnect in ${delay}ms (attempt ${reconnectAttemptsRef.current + 1}/${maxReconnectAttempts})`)
          
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttemptsRef.current++
            connect(wsUrl)
          }, delay)
        }
      }
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        toast.error('Connection error')
      }
      
      setSocket(ws)
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error)
      toast.error('Failed to connect to server')
    }
  }

  const sendMessage = (message: any) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket is not connected')
    }
  }

  const disconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    
    if (socket) {
      socket.close(1000, 'Client disconnect')
    }
    
    setSocket(null)
    setIsConnected(false)
    reconnectAttemptsRef.current = maxReconnectAttempts // Prevent reconnection
  }

  useEffect(() => {
    if (url) {
      connect(url)
    }
    
    return () => {
      disconnect()
    }
  }, [url])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect()
    }
  }, [])

  // Send periodic ping to keep connection alive
  useEffect(() => {
    if (isConnected && socket) {
      const pingInterval = setInterval(() => {
        sendMessage({ type: 'ping', timestamp: new Date().toISOString() })
      }, 30000) // Ping every 30 seconds
      
      return () => clearInterval(pingInterval)
    }
  }, [isConnected, socket])

  return {
    socket,
    isConnected,
    lastMessage,
    sendMessage,
    disconnect,
  }
}