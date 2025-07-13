import React, { useEffect, useState } from 'react'
import { useEconomyStore } from '../stores/economyStore'
import { formatFileSize } from '../utils/api'
import { 
  Wallet, 
  RefreshCw, 
  HardDrive, 
  Award, 
  Star, 
  Share2, 
  Upload, 
  Download, 
  AlertTriangle, 
  Plus,
  TrendingUp,
  FileText,
  DollarSign,
  ChevronRight,
  CheckCircle,
  Clock,
  XCircle
} from 'lucide-react'

const Economy: React.FC = () => {
  const {
    economyStatus,
    userProfile,
    storageTiers,
    contributionStatus,
    verificationStatus,
    transactions,
    quotaStatus,
    loading,
    errors,
    fetchEconomyStatus,
    fetchUserProfile,
    fetchStorageTiers,
    fetchContributionStatus,
    fetchVerificationStatus,
    fetchTransactions,
    fetchQuotaStatus,
    initializeEconomyData,
    refreshData,
    getStorageUsagePercentage,
    getUploadQuotaPercentage,
    getDownloadQuotaPercentage,
    getCanUpgradeTier,
    getNextTier,
    getTierColor,
    getIsStorageLow,
    getIsQuotaLow,
  } = useEconomyStore()

  const [showQuotaDetails, setShowQuotaDetails] = useState(false)
  const [showContributionSetup, setShowContributionSetup] = useState(false)
  const [showTierUpgrade, setShowTierUpgrade] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)

  useEffect(() => {
    initializeEconomyData()
    
    // Set up auto-refresh every 30 seconds
    const interval = setInterval(() => {
      refreshData()
    }, 30000)
    
    return () => clearInterval(interval)
  }, [initializeEconomyData, refreshData])

  const handleRefresh = async () => {
    setIsRefreshing(true)
    try {
      await refreshData()
    } finally {
      setIsRefreshing(false)
    }
  }

  const formatDate = (dateString: string | null) => {
    if (!dateString) return 'Never'
    return new Date(dateString).toLocaleDateString() + ' ' + new Date(dateString).toLocaleTimeString()
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'text-green-600'
      case 'pending': return 'text-yellow-600'
      case 'failed': return 'text-red-600'
      default: return 'text-gray-600'
    }
  }

  const getTransactionIcon = (type: string) => {
    switch (type) {
      case 'contribution': return <Share2 className="w-4 h-4" />
      case 'upgrade': return <TrendingUp className="w-4 h-4" />
      case 'verification': return <FileText className="w-4 h-4" />
      default: return <DollarSign className="w-4 h-4" />
    }
  }

  const storageUsagePercentage = getStorageUsagePercentage()
  const uploadQuotaPercentage = getUploadQuotaPercentage()
  const downloadQuotaPercentage = getDownloadQuotaPercentage()
  const canUpgradeTier = getCanUpgradeTier()
  const nextTier = getNextTier()
  const isStorageLow = getIsStorageLow()
  const isQuotaLow = getIsQuotaLow()

  if (loading.userProfile && !userProfile) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading economy data...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      {/* Header */}
      <div className="flex justify-between items-center mb-8">
        <div className="flex items-center gap-3">
          <Wallet className="w-8 h-8 text-blue-600" />
          <h1 className="text-3xl font-bold text-gray-900">Storage Economy</h1>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isRefreshing}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        {/* Storage Usage */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-gradient-to-br from-purple-500 to-blue-600 rounded-lg">
              <HardDrive className="w-6 h-6 text-white" />
            </div>
            <div className="flex-1">
              <h3 className="text-sm font-medium text-gray-600">Storage Used</h3>
              <p className="text-2xl font-bold text-gray-900">
                {userProfile ? formatFileSize(userProfile.current_usage) : '0 B'}
              </p>
              <div className="mt-2">
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    className={`h-2 rounded-full ${storageUsagePercentage > 85 ? 'bg-red-500' : 'bg-green-500'}`}
                    style={{ width: `${storageUsagePercentage}%` }}
                  ></div>
                </div>
                <p className="text-xs text-gray-500 mt-1">{storageUsagePercentage}% used</p>
              </div>
            </div>
          </div>
        </div>

        {/* Current Tier */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div 
              className="p-3 rounded-lg"
              style={{ backgroundColor: userProfile ? getTierColor(userProfile.tier) : '#9CA3AF' }}
            >
              <Award className="w-6 h-6 text-white" />
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-600">Current Tier</h3>
              <p className="text-2xl font-bold text-gray-900">
                {userProfile?.tier || 'Free'}
              </p>
              <p className="text-xs text-gray-500">
                {userProfile ? formatFileSize(userProfile.max_storage) : '0 B'} limit
              </p>
            </div>
          </div>
        </div>

        {/* Reputation Score */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-gradient-to-br from-blue-400 to-cyan-500 rounded-lg">
              <Star className="w-6 h-6 text-white" />
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-600">Reputation</h3>
              <p className="text-2xl font-bold text-gray-900">
                {userProfile?.reputation_score.toFixed(1) || '0.0'}
              </p>
              <p className={`text-xs ${userProfile?.can_contribute ? 'text-green-500' : 'text-gray-500'}`}>
                {userProfile?.can_contribute ? 'Can Contribute' : 'Cannot Contribute'}
              </p>
            </div>
          </div>
        </div>

        {/* Contribution Status */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-gradient-to-br from-green-400 to-emerald-500 rounded-lg">
              <Share2 className="w-6 h-6 text-white" />
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-600">Contributing</h3>
              <p className="text-2xl font-bold text-gray-900">
                {contributionStatus ? formatFileSize(contributionStatus.contributed_amount) : '0 B'}
              </p>
              <p className={`text-xs ${contributionStatus?.active ? 'text-green-500' : 'text-gray-500'}`}>
                {contributionStatus?.active ? 'Active' : 'Inactive'}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Storage Quotas */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-bold text-gray-900">Storage Quotas</h2>
            <button
              onClick={() => setShowQuotaDetails(!showQuotaDetails)}
              className="text-sm text-blue-600 hover:text-blue-700"
            >
              {showQuotaDetails ? 'Hide Details' : 'Show Details'}
            </button>
          </div>

          <div className="space-y-6">
            {/* Storage Usage */}
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700">Storage Usage</span>
                <span className="text-sm text-gray-600">
                  {userProfile ? formatFileSize(userProfile.current_usage) : '0 B'} / {userProfile ? formatFileSize(userProfile.max_storage) : '0 B'}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-3">
                <div 
                  className={`h-3 rounded-full transition-all duration-300 ${storageUsagePercentage > 85 ? 'bg-red-500' : 'bg-green-500'}`}
                  style={{ width: `${storageUsagePercentage}%` }}
                ></div>
              </div>
              <p className="text-xs text-gray-500 mt-1">{storageUsagePercentage}%</p>
            </div>

            {/* Upload Quota */}
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700">Monthly Upload</span>
                <span className="text-sm text-gray-600">
                  {userProfile ? formatFileSize(userProfile.upload_quota_used) : '0 B'} / {userProfile ? formatFileSize(userProfile.upload_quota_limit) : '0 B'}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-3">
                <div 
                  className={`h-3 rounded-full transition-all duration-300 ${uploadQuotaPercentage > 85 ? 'bg-red-500' : 'bg-blue-500'}`}
                  style={{ width: `${uploadQuotaPercentage}%` }}
                ></div>
              </div>
              <p className="text-xs text-gray-500 mt-1">{uploadQuotaPercentage}%</p>
            </div>

            {/* Download Quota */}
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700">Monthly Download</span>
                <span className="text-sm text-gray-600">
                  {userProfile ? formatFileSize(userProfile.download_quota_used) : '0 B'} / {userProfile ? formatFileSize(userProfile.download_quota_limit) : '0 B'}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-3">
                <div 
                  className={`h-3 rounded-full transition-all duration-300 ${downloadQuotaPercentage > 85 ? 'bg-red-500' : 'bg-purple-500'}`}
                  style={{ width: `${downloadQuotaPercentage}%` }}
                ></div>
              </div>
              <p className="text-xs text-gray-500 mt-1">{downloadQuotaPercentage}%</p>
            </div>
          </div>

          {showQuotaDetails && (
            <div className="mt-6 pt-6 border-t border-gray-200">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-gray-600">Quota Reset:</span>
                  <p className="font-medium">{quotaStatus ? formatDate(quotaStatus.next_reset) : 'Unknown'}</p>
                </div>
                <div>
                  <span className="text-gray-600">Violations:</span>
                  <p className={`font-medium ${userProfile && userProfile.violations_count > 0 ? 'text-yellow-600' : 'text-green-600'}`}>
                    {userProfile?.violations_count || 0}
                  </p>
                </div>
                <div className="col-span-2">
                  <span className="text-gray-600">Last Activity:</span>
                  <p className="font-medium">{userProfile ? formatDate(userProfile.last_activity) : 'Never'}</p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Tier Management */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <h2 className="text-xl font-bold text-gray-900 mb-6">Storage Tier</h2>

          <div className="text-center mb-6">
            <div 
              className="inline-flex items-center gap-2 px-4 py-2 rounded-full text-white font-semibold mb-4"
              style={{ backgroundColor: userProfile ? getTierColor(userProfile.tier) : '#9CA3AF' }}
            >
              <Award className="w-5 h-5" />
              {userProfile?.tier || 'Free'}
            </div>
            
            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-center gap-2 text-gray-600">
                <HardDrive className="w-4 h-4" />
                {userProfile ? formatFileSize(userProfile.max_storage) : '0 B'} Storage
              </div>
              <div className="flex items-center justify-center gap-2 text-gray-600">
                <Upload className="w-4 h-4" />
                {userProfile ? formatFileSize(userProfile.upload_quota_limit) : '0 B'} Monthly Upload
              </div>
              <div className="flex items-center justify-center gap-2 text-gray-600">
                <Download className="w-4 h-4" />
                {userProfile ? formatFileSize(userProfile.download_quota_limit) : '0 B'} Monthly Download
              </div>
            </div>
          </div>

          {canUpgradeTier && nextTier && (
            <div className="border-t pt-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Available Upgrades</h3>
              <div 
                className="border border-gray-200 rounded-lg p-4 cursor-pointer hover:border-blue-300 hover:bg-blue-50 transition-colors"
                onClick={() => setShowTierUpgrade(true)}
              >
                <div className="flex justify-between items-center mb-2">
                  <span className="font-semibold" style={{ color: getTierColor(nextTier.name) }}>
                    {nextTier.name}
                  </span>
                  <span className="text-sm font-semibold text-blue-600">
                    {nextTier.monthly_cost ? `$${nextTier.monthly_cost}/month` : 'Free'}
                  </span>
                </div>
                <p className="text-sm text-gray-600 mb-2">{nextTier.description}</p>
                <div className="text-xs text-gray-500">
                  {formatFileSize(nextTier.max_storage)} • {formatFileSize(nextTier.upload_quota)} upload • {formatFileSize(nextTier.download_quota)} download
                </div>
                <div className="flex items-center justify-end mt-2">
                  <ChevronRight className="w-4 h-4 text-gray-400" />
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Contribution Panel */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-bold text-gray-900">Storage Contribution</h2>
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${contributionStatus?.active ? 'bg-green-500' : 'bg-gray-400'}`}></span>
              <span className="text-sm text-gray-600">
                {contributionStatus?.active ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>

          {contributionStatus?.active ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-gray-600">Contributing:</span>
                  <p className="font-semibold">{formatFileSize(contributionStatus.contributed_amount)}</p>
                </div>
                <div>
                  <span className="text-gray-600">Verified:</span>
                  <p className="font-semibold">{formatFileSize(contributionStatus.verified_amount)}</p>
                </div>
                <div className="col-span-2">
                  <span className="text-gray-600">Last Verification:</span>
                  <p className="font-semibold">{formatDate(contributionStatus.last_verification)}</p>
                </div>
              </div>
              
              <div>
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium text-gray-700">Verification Progress</span>
                  <span className="text-sm text-gray-600">
                    {contributionStatus.contributed_amount > 0 ? 
                      Math.round((contributionStatus.verified_amount / contributionStatus.contributed_amount) * 100) : 0}%
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    className="h-2 bg-green-500 rounded-full transition-all duration-300"
                    style={{ 
                      width: `${contributionStatus.contributed_amount > 0 ? 
                        (contributionStatus.verified_amount / contributionStatus.contributed_amount) * 100 : 0}%` 
                    }}
                  ></div>
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center">
              <div className="mb-4">
                <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-2" />
                <p className="text-gray-600 mb-2">Storage contribution is currently inactive.</p>
                <p className="text-sm text-gray-500">
                  Enable contribution to earn additional storage space and improve your reputation score.
                </p>
              </div>
              
              <button
                onClick={() => setShowContributionSetup(true)}
                disabled={!userProfile?.can_contribute}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed mx-auto"
              >
                <Plus className="w-4 h-4" />
                Set Up Contribution
              </button>
              
              {!userProfile?.can_contribute && (
                <div className="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="w-4 h-4 text-yellow-600" />
                    <span className="text-sm text-yellow-800">
                      Your reputation score is too low to contribute storage.
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Recent Transactions */}
        <div className="bg-white rounded-xl shadow-sm p-6 border border-gray-200">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-bold text-gray-900">Recent Transactions</h2>
            <button className="text-sm text-blue-600 hover:text-blue-700">
              View All
            </button>
          </div>

          {transactions.length > 0 ? (
            <div className="space-y-4">
              {transactions.slice(0, 5).map((transaction) => (
                <div key={transaction.transaction_id} className="flex items-center gap-4 p-3 border border-gray-100 rounded-lg">
                  <div className="p-2 bg-gray-100 rounded-lg">
                    {getTransactionIcon(transaction.transaction_type)}
                  </div>
                  <div className="flex-1">
                    <p className="font-medium text-gray-900">{transaction.description}</p>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <span>{formatDate(transaction.timestamp)}</span>
                      <span className={getStatusColor(transaction.status)}>
                        {transaction.status}
                      </span>
                    </div>
                  </div>
                  <div className="text-sm font-semibold text-gray-900">
                    {formatFileSize(transaction.amount)}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8">
              <FileText className="w-12 h-12 text-gray-400 mx-auto mb-2" />
              <p className="text-gray-600">No recent transactions</p>
            </div>
          )}
        </div>
      </div>

      {/* Modals/Dialogs would go here */}
      {/* TODO: Implement ContributionSetupDialog, TierUpgradeDialog, etc. */}
    </div>
  )
}

export default Economy