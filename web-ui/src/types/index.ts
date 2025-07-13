// API Types
export interface User {
  user_id: string
  email: string
  account_type: 'free' | 'premium' | 'enterprise'
  verification_status: 'unverified' | 'email_verified' | 'identity_verified'
  registration_date: string
}

export interface AuthResponse {
  access_token: string
  token_type: string
  expires_in: number
  user: User
}

export interface FileMetadata {
  file_key: string
  file_name: string
  original_name: string
  file_size: number
  uploaded_at: string
  tags: string[]
  public_key: string
}

export interface FileListResponse {
  files: FileMetadata[]
  total: number
  page: number
  page_size: number
}

export interface FileUploadResponse {
  file_key: string
  file_name: string
  file_size: number
  uploaded_at: string
  message: string
}

export interface SystemMetrics {
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  network_throughput: number
  active_connections: number
  timestamp: string
}

export interface StorageMetrics {
  total_capacity: number
  used_storage: number
  available_storage: number
  files_count: number
  average_file_size: number
  timestamp: string
}

export interface NetworkMetrics {
  total_peers: number
  connected_peers: number
  network_latency: number
  bandwidth_usage: number
  packet_loss_rate: number
  timestamp: string
}

export interface Proposal {
  id: string
  title: string
  description: string
  proposal_type: string
  status: string
  votes_for: number
  votes_against: number
  created_at: string
  expires_at: string
}

export interface UserSettings {
  theme: 'light' | 'dark' | 'system'
  language: string
  notifications_enabled: boolean
  email_notifications: boolean
  auto_delete_days: number
  privacy_mode: boolean
}

// WebSocket Message Types
export type WSMessage = 
  | {
      type: 'FileUploadProgress'
      file_key: string
      progress: number
      status: string
    }
  | {
      type: 'FileDownloadProgress'
      file_key: string
      progress: number
      status: string
    }
  | {
      type: 'SystemStatus'
      status: string
      message: string
    }
  | {
      type: 'NetworkHealth'
      total_operators: number
      online_operators: number
      online_percentage: number
      can_reach_consensus: boolean
    }
  | {
      type: 'GovernanceUpdate'
      event_type: string
      data: any
    }
  | {
      type: 'CacheStats'
      hit_ratio: number
      cache_size: number
    }

// UI Types
export interface ToastOptions {
  type?: 'success' | 'error' | 'warning' | 'info'
  duration?: number
}

export interface UploadFile {
  file: File
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  file_key?: string
}

export interface SearchFilters {
  query?: string
  tags?: string[]
  file_type?: string
  size_range?: [number, number]
  date_range?: [Date, Date]
}

export interface NetworkNode {
  id: string
  address: string
  peer_id: string
  status: 'online' | 'offline' | 'connecting'
  latency?: number
  last_seen: string
  services: string[]
}

export interface NetworkHealth {
  overall_status: 'healthy' | 'degraded' | 'critical'
  connected_nodes: number
  total_nodes: number
  network_latency: number
  consensus_status: boolean
}

// Error Types
export interface ApiError {
  code: string
  message: string
  details?: string
  request_id?: string
}

// Form Types
export interface LoginForm {
  email: string
  password: string
}

export interface RegisterForm {
  email: string
  password: string
  confirmPassword: string
  public_key?: string
}

export interface UploadForm {
  files: File[]
  tags?: string
  public_key?: string
}

export interface ProposalForm {
  title: string
  description: string
  proposal_type: string
  data: any
}

// Utility Types
export type Theme = 'light' | 'dark' | 'system'
export type LoadingState = 'idle' | 'loading' | 'success' | 'error'
export type SortDirection = 'asc' | 'desc'

export interface SortConfig {
  field: string
  direction: SortDirection
}

export interface PaginationConfig {
  page: number
  pageSize: number
  total: number
}

// Chart Data Types
export interface ChartDataPoint {
  timestamp: string
  value: number
  label?: string
}

export interface MetricChart {
  name: string
  data: ChartDataPoint[]
  color: string
  unit?: string
}