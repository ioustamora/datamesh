import { useQuery } from 'react-query'
import {
  ChartBarIcon,
  CloudIcon,
  GlobeAltIcon,
  UsersIcon,
  DocumentIcon,
  ServerIcon,
} from '@heroicons/react/24/outline'
import { api } from '@/utils/api'
import { formatFileSize } from '@/utils/api'
import { SystemMetrics, StorageMetrics, NetworkMetrics } from '@/types'

export function Dashboard() {
  const { data: systemMetrics } = useQuery<SystemMetrics>(
    'system-metrics',
    () => api.get('/analytics/system').then(res => res.data),
    { refetchInterval: 30000 }
  )

  const { data: storageMetrics } = useQuery<StorageMetrics>(
    'storage-metrics',
    () => api.get('/analytics/storage').then(res => res.data),
    { refetchInterval: 30000 }
  )

  const { data: networkMetrics } = useQuery<NetworkMetrics>(
    'network-metrics',
    () => api.get('/analytics/network').then(res => res.data),
    { refetchInterval: 30000 }
  )

  const stats = [
    {
      name: 'Total Files',
      value: storageMetrics?.files_count.toLocaleString() || '0',
      icon: DocumentIcon,
      color: 'text-blue-600',
      bgColor: 'bg-blue-100 dark:bg-blue-900/20',
    },
    {
      name: 'Storage Used',
      value: storageMetrics ? formatFileSize(storageMetrics.used_storage) : '0 B',
      icon: CloudIcon,
      color: 'text-green-600',
      bgColor: 'bg-green-100 dark:bg-green-900/20',
    },
    {
      name: 'Connected Peers',
      value: networkMetrics?.connected_peers.toString() || '0',
      icon: UsersIcon,
      color: 'text-purple-600',
      bgColor: 'bg-purple-100 dark:bg-purple-900/20',
    },
    {
      name: 'CPU Usage',
      value: systemMetrics ? `${systemMetrics.cpu_usage.toFixed(1)}%` : '0%',
      icon: ServerIcon,
      color: 'text-orange-600',
      bgColor: 'bg-orange-100 dark:bg-orange-900/20',
    },
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
          Welcome to your DataMesh control center
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <div key={stat.name} className="card">
            <div className="card-body">
              <div className="flex items-center">
                <div className={`flex-shrink-0 p-3 rounded-lg ${stat.bgColor}`}>
                  <stat.icon className={`h-6 w-6 ${stat.color}`} />
                </div>
                <div className="ml-4">
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    {stat.name}
                  </dt>
                  <dd className="text-lg font-semibold text-gray-900 dark:text-white">
                    {stat.value}
                  </dd>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* System Performance */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              System Performance
            </h3>
          </div>
          <div className="card-body">
            <div className="space-y-4">
              <div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600 dark:text-gray-400">CPU Usage</span>
                  <span className="font-medium">{systemMetrics?.cpu_usage.toFixed(1)}%</span>
                </div>
                <div className="progress-bar mt-2">
                  <div 
                    className="progress-fill bg-blue-600" 
                    style={{ width: `${systemMetrics?.cpu_usage || 0}%` }}
                  />
                </div>
              </div>
              
              <div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600 dark:text-gray-400">Memory Usage</span>
                  <span className="font-medium">{systemMetrics?.memory_usage.toFixed(1)}%</span>
                </div>
                <div className="progress-bar mt-2">
                  <div 
                    className="progress-fill bg-green-600" 
                    style={{ width: `${systemMetrics?.memory_usage || 0}%` }}
                  />
                </div>
              </div>
              
              <div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600 dark:text-gray-400">Disk Usage</span>
                  <span className="font-medium">{systemMetrics?.disk_usage.toFixed(1)}%</span>
                </div>
                <div className="progress-bar mt-2">
                  <div 
                    className="progress-fill bg-orange-600" 
                    style={{ width: `${systemMetrics?.disk_usage || 0}%` }}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Storage Overview */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Storage Overview
            </h3>
          </div>
          <div className="card-body">
            <div className="space-y-4">
              <div className="text-center">
                <div className="text-3xl font-bold text-gray-900 dark:text-white">
                  {storageMetrics ? formatFileSize(storageMetrics.used_storage) : '0 B'}
                </div>
                <div className="text-sm text-gray-600 dark:text-gray-400">
                  of {storageMetrics ? formatFileSize(storageMetrics.total_capacity) : '0 B'} used
                </div>
              </div>
              
              <div className="progress-bar">
                <div 
                  className="progress-fill bg-purple-600" 
                  style={{ 
                    width: storageMetrics 
                      ? `${(storageMetrics.used_storage / storageMetrics.total_capacity) * 100}%`
                      : '0%'
                  }}
                />
              </div>
              
              <div className="grid grid-cols-2 gap-4 text-center">
                <div>
                  <div className="text-lg font-semibold text-gray-900 dark:text-white">
                    {storageMetrics?.files_count.toLocaleString() || '0'}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">Files</div>
                </div>
                <div>
                  <div className="text-lg font-semibold text-gray-900 dark:text-white">
                    {storageMetrics ? formatFileSize(storageMetrics.average_file_size) : '0 B'}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">Avg Size</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Network Status */}
      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            Network Status
          </h3>
        </div>
        <div className="card-body">
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {networkMetrics?.connected_peers || 0}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Connected Peers</div>
              <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                of {networkMetrics?.total_peers || 0} total
              </div>
            </div>
            
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {networkMetrics?.network_latency.toFixed(0) || 0}ms
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Network Latency</div>
              <div className={`text-xs mt-1 ${
                (networkMetrics?.network_latency || 0) < 100 
                  ? 'text-green-600' 
                  : (networkMetrics?.network_latency || 0) < 300 
                    ? 'text-yellow-600' 
                    : 'text-red-600'
              }`}>
                {(networkMetrics?.network_latency || 0) < 100 ? 'Excellent' : 
                 (networkMetrics?.network_latency || 0) < 300 ? 'Good' : 'Poor'}
              </div>
            </div>
            
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {networkMetrics?.bandwidth_usage.toFixed(1) || 0}%
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Bandwidth Usage</div>
              <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Current utilization
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="card">
        <div className="card-header">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            Quick Actions
          </h3>
        </div>
        <div className="card-body">
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <button className="btn-primary">
              <CloudIcon className="h-5 w-5 mr-2" />
              Upload Files
            </button>
            <button className="btn-secondary">
              <DocumentIcon className="h-5 w-5 mr-2" />
              Browse Files
            </button>
            <button className="btn-secondary">
              <GlobeAltIcon className="h-5 w-5 mr-2" />
              Network Status
            </button>
            <button className="btn-secondary">
              <ChartBarIcon className="h-5 w-5 mr-2" />
              View Analytics
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}