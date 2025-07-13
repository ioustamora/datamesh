import React, { useState } from 'react'
import { useEconomyStore } from '../stores/economyStore'
import { formatFileSize } from '../utils/api'
import { X, HardDrive, CheckCircle, AlertCircle } from 'lucide-react'

interface ContributionSetupDialogProps {
  isOpen: boolean
  onClose: () => void
}

export const ContributionSetupDialog: React.FC<ContributionSetupDialogProps> = ({
  isOpen,
  onClose,
}) => {
  const { startContribution, loading } = useEconomyStore()
  const [step, setStep] = useState(1)
  const [formData, setFormData] = useState({
    storagePath: '',
    amount: 4,
    unit: 'GB' as 'GB' | 'TB',
  })
  const [agreedToTerms, setAgreedToTerms] = useState(false)

  if (!isOpen) return null

  const getAmountInBytes = () => {
    const multiplier = formData.unit === 'TB' ? 1099511627776 : 1073741824
    return formData.amount * multiplier
  }

  const handleSubmit = async () => {
    try {
      await startContribution({
        storage_path: formData.storagePath,
        amount: getAmountInBytes(),
      })
      onClose()
      // Reset form
      setStep(1)
      setFormData({ storagePath: '', amount: 4, unit: 'GB' })
      setAgreedToTerms(false)
    } catch (error) {
      console.error('Failed to start contribution:', error)
    }
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-bold text-gray-900">Set Up Storage Contribution</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {step === 1 && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Storage Path
              </label>
              <input
                type="text"
                value={formData.storagePath}
                onChange={(e) => setFormData({ ...formData, storagePath: e.target.value })}
                placeholder="Enter the path to storage directory"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Choose a directory with sufficient free space.
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Contribution Size
              </label>
              <div className="flex gap-2">
                <input
                  type="number"
                  min="1"
                  max={formData.unit === 'TB' ? 10 : 1000}
                  value={formData.amount}
                  onChange={(e) => setFormData({ ...formData, amount: parseInt(e.target.value) || 1 })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <select
                  value={formData.unit}
                  onChange={(e) => setFormData({ ...formData, unit: e.target.value as 'GB' | 'TB' })}
                  className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="GB">GB</option>
                  <option value="TB">TB</option>
                </select>
              </div>
              <p className="text-xs text-gray-500 mt-1">
                Minimum: 1 GB, Maximum: {formData.unit === 'TB' ? '10 TB' : '1000 GB'}
              </p>
            </div>

            <div className="bg-blue-50 p-4 rounded-lg">
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-blue-700">Contribution Ratio (4:1)</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-blue-600">You contribute: {formatFileSize(getAmountInBytes())}</span>
                <span className="text-green-600">You earn: {formatFileSize(Math.floor(getAmountInBytes() / 4))}</span>
              </div>
              <p className="text-xs text-blue-600 mt-2">
                For every 4 GB you contribute, you earn 1 GB of storage quota.
              </p>
            </div>

            <div className="flex justify-end gap-3 pt-4">
              <button
                onClick={onClose}
                className="px-4 py-2 text-gray-600 hover:text-gray-800"
              >
                Cancel
              </button>
              <button
                onClick={() => setStep(2)}
                disabled={!formData.storagePath || formData.amount < 1}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <div className="text-center">
              <HardDrive className="w-12 h-12 text-blue-600 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Review Configuration</h3>
            </div>

            <div className="bg-gray-50 p-4 rounded-lg space-y-3">
              <div className="flex justify-between">
                <span className="text-gray-600">Storage Path:</span>
                <span className="font-medium">{formData.storagePath}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Contribution Amount:</span>
                <span className="font-medium">{formatFileSize(getAmountInBytes())}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Storage Quota Earned:</span>
                <span className="font-medium text-green-600">{formatFileSize(Math.floor(getAmountInBytes() / 4))}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Verification Schedule:</span>
                <span className="font-medium">Daily random challenges</span>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <input
                type="checkbox"
                id="terms"
                checked={agreedToTerms}
                onChange={(e) => setAgreedToTerms(e.target.checked)}
                className="mt-1"
              />
              <label htmlFor="terms" className="text-sm text-gray-600">
                I agree to the Storage Contribution Terms and understand the verification requirements
              </label>
            </div>

            <div className="flex justify-end gap-3 pt-4">
              <button
                onClick={() => setStep(1)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800"
              >
                Back
              </button>
              <button
                onClick={handleSubmit}
                disabled={!agreedToTerms || loading.contributing}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                {loading.contributing && (
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                )}
                {loading.contributing ? 'Starting...' : 'Start Contributing'}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default ContributionSetupDialog