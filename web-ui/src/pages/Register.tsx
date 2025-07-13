import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import toast from 'react-hot-toast'
import { EyeIcon, EyeSlashIcon, KeyIcon } from '@heroicons/react/24/outline'

import { useAuthStore } from '@/stores/authStore'
import { RegisterForm } from '@/types'
import { Spinner } from '@/components/ui/Spinner'

export function Register() {
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [generateKey, setGenerateKey] = useState(true)
  const navigate = useNavigate()
  const { register: registerUser, isLoading } = useAuthStore()

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<RegisterForm>()

  const password = watch('password')

  const generateKeyPair = () => {
    // In a real implementation, you would generate an actual cryptographic key pair
    // For demo purposes, we'll generate a simple hex string
    const publicKey = Array.from(crypto.getRandomValues(new Uint8Array(32)))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('')
    
    return publicKey
  }

  const onSubmit = async (data: RegisterForm) => {
    try {
      const publicKey = generateKey ? generateKeyPair() : data.public_key
      
      await registerUser(data.email, data.password, publicKey)
      
      navigate('/dashboard', { replace: true })
      toast.success('Account created successfully!')
    } catch (error) {
      // Error is already handled in the store
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-secondary-50 dark:bg-secondary-900 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div>
          <div className="mx-auto h-12 w-12 bg-primary-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-xl">D</span>
          </div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-secondary-900 dark:text-white">
            Create your account
          </h2>
          <p className="mt-2 text-center text-sm text-secondary-600 dark:text-secondary-400">
            Or{' '}
            <Link
              to="/login"
              className="font-medium text-primary-600 hover:text-primary-500"
            >
              sign in to your existing account
            </Link>
          </p>
        </div>
        
        <form className="mt-8 space-y-6" onSubmit={handleSubmit(onSubmit)}>
          <div className="space-y-4">
            <div>
              <label htmlFor="email" className="label">
                Email address
              </label>
              <input
                {...register('email', {
                  required: 'Email is required',
                  pattern: {
                    value: /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i,
                    message: 'Invalid email address',
                  },
                })}
                type="email"
                autoComplete="email"
                className="input"
                placeholder="Enter your email"
              />
              {errors.email && (
                <p className="mt-1 text-sm text-danger-600">{errors.email.message}</p>
              )}
            </div>
            
            <div>
              <label htmlFor="password" className="label">
                Password
              </label>
              <div className="relative">
                <input
                  {...register('password', {
                    required: 'Password is required',
                    minLength: {
                      value: 8,
                      message: 'Password must be at least 8 characters',
                    },
                    pattern: {
                      value: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/,
                      message: 'Password must contain uppercase, lowercase, number and special character',
                    },
                  })}
                  type={showPassword ? 'text' : 'password'}
                  autoComplete="new-password"
                  className="input pr-10"
                  placeholder="Create a strong password"
                />
                <button
                  type="button"
                  className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  onClick={() => setShowPassword(!showPassword)}
                >
                  {showPassword ? (
                    <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                  ) : (
                    <EyeIcon className="h-5 w-5 text-secondary-400" />
                  )}
                </button>
              </div>
              {errors.password && (
                <p className="mt-1 text-sm text-danger-600">{errors.password.message}</p>
              )}
            </div>
            
            <div>
              <label htmlFor="confirmPassword" className="label">
                Confirm password
              </label>
              <div className="relative">
                <input
                  {...register('confirmPassword', {
                    required: 'Please confirm your password',
                    validate: (value) =>
                      value === password || 'Passwords do not match',
                  })}
                  type={showConfirmPassword ? 'text' : 'password'}
                  autoComplete="new-password"
                  className="input pr-10"
                  placeholder="Confirm your password"
                />
                <button
                  type="button"
                  className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                >
                  {showConfirmPassword ? (
                    <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                  ) : (
                    <EyeIcon className="h-5 w-5 text-secondary-400" />
                  )}
                </button>
              </div>
              {errors.confirmPassword && (
                <p className="mt-1 text-sm text-danger-600">{errors.confirmPassword.message}</p>
              )}
            </div>

            {/* Encryption Key Section */}
            <div className="border-t border-secondary-200 dark:border-secondary-700 pt-4">
              <div className="flex items-center mb-3">
                <KeyIcon className="h-5 w-5 text-secondary-400 mr-2" />
                <span className="label mb-0">Encryption Key</span>
              </div>
              
              <div className="space-y-3">
                <div className="flex items-center">
                  <input
                    id="generate-key"
                    type="radio"
                    checked={generateKey}
                    onChange={() => setGenerateKey(true)}
                    className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-secondary-300"
                  />
                  <label htmlFor="generate-key" className="ml-2 block text-sm text-secondary-900 dark:text-secondary-300">
                    Generate key automatically (recommended)
                  </label>
                </div>
                
                <div className="flex items-center">
                  <input
                    id="provide-key"
                    type="radio"
                    checked={!generateKey}
                    onChange={() => setGenerateKey(false)}
                    className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-secondary-300"
                  />
                  <label htmlFor="provide-key" className="ml-2 block text-sm text-secondary-900 dark:text-secondary-300">
                    Provide my own public key
                  </label>
                </div>
                
                {!generateKey && (
                  <div className="mt-2">
                    <input
                      {...register('public_key', {
                        required: !generateKey ? 'Public key is required' : false,
                        minLength: {
                          value: 64,
                          message: 'Public key must be at least 64 characters',
                        },
                      })}
                      type="text"
                      className="input font-mono text-xs"
                      placeholder="Enter your public key (hex format)"
                    />
                    {errors.public_key && (
                      <p className="mt-1 text-sm text-danger-600">{errors.public_key.message}</p>
                    )}
                  </div>
                )}
              </div>
              
              <p className="mt-2 text-xs text-secondary-500 dark:text-secondary-400">
                Your encryption key is used to secure your files. Keep it safe!
              </p>
            </div>
          </div>

          <div className="flex items-center">
            <input
              id="accept-terms"
              name="accept-terms"
              type="checkbox"
              required
              className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-secondary-300 rounded"
            />
            <label htmlFor="accept-terms" className="ml-2 block text-sm text-secondary-900 dark:text-secondary-300">
              I accept the{' '}
              <a href="#" className="text-primary-600 hover:text-primary-500">
                Terms of Service
              </a>{' '}
              and{' '}
              <a href="#" className="text-primary-600 hover:text-primary-500">
                Privacy Policy
              </a>
            </label>
          </div>

          <div>
            <button
              type="submit"
              disabled={isLoading}
              className="btn-primary w-full flex justify-center py-3"
            >
              {isLoading ? (
                <>
                  <Spinner size="sm" className="mr-2" />
                  Creating account...
                </>
              ) : (
                'Create account'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}