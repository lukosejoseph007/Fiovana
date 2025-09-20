import React, { useState, useEffect } from 'react'
import { clsx } from 'clsx'
import { DuplicateResolutionPolicy } from '../types/deduplication'

interface DuplicateResolutionPolicyConfigProps {
  initialPolicy?: Partial<DuplicateResolutionPolicy>
  onPolicyChange: (policy: DuplicateResolutionPolicy) => void
  className?: string
}

const DuplicateResolutionPolicyConfig: React.FC<DuplicateResolutionPolicyConfigProps> = ({
  initialPolicy = {},
  onPolicyChange,
  className,
}) => {
  const [policy, setPolicy] = useState<DuplicateResolutionPolicy>({
    auto_deduplicate: false,
    always_prompt: true,
    prefer_newest: false,
    prefer_largest: false,
    ...initialPolicy,
  })

  const [showAdvanced, setShowAdvanced] = useState(false)
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)

  useEffect(() => {
    setHasUnsavedChanges(
      JSON.stringify(policy) !==
        JSON.stringify({
          auto_deduplicate: false,
          always_prompt: true,
          prefer_newest: false,
          prefer_largest: false,
          ...initialPolicy,
        })
    )
  }, [policy, initialPolicy])

  const handlePolicyUpdate = (updates: Partial<DuplicateResolutionPolicy>) => {
    const newPolicy = { ...policy, ...updates }

    // Handle mutual exclusions
    if (updates.auto_deduplicate && newPolicy.always_prompt) {
      newPolicy.always_prompt = false
    } else if (updates.always_prompt && newPolicy.auto_deduplicate) {
      newPolicy.auto_deduplicate = false
    }

    setPolicy(newPolicy)
  }

  const handleSave = () => {
    onPolicyChange(policy)
    setHasUnsavedChanges(false)
  }

  const handleReset = () => {
    const resetPolicy = {
      auto_deduplicate: false,
      always_prompt: true,
      prefer_newest: false,
      prefer_largest: false,
      ...initialPolicy,
    }
    setPolicy(resetPolicy)
    setHasUnsavedChanges(false)
  }

  const getRecommendationLevel = () => {
    if (policy.auto_deduplicate && !policy.always_prompt) {
      return {
        level: 'aggressive',
        color: 'red',
        description: 'Automatic deduplication without prompts - fastest but least control',
      }
    } else if (!policy.auto_deduplicate && policy.always_prompt) {
      return {
        level: 'conservative',
        color: 'blue',
        description: 'Always ask before making changes - safest but requires more interaction',
      }
    } else {
      return {
        level: 'balanced',
        color: 'green',
        description: 'Automatic deduplication with prompts for conflicts - good balance',
      }
    }
  }

  const recommendation = getRecommendationLevel()

  return (
    <div className={clsx('bg-white border border-gray-200 rounded-lg overflow-hidden', className)}>
      {/* Header */}
      <div className="bg-gray-50 px-6 py-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900">Duplicate Resolution Policy</h3>
          <div className="flex items-center space-x-2">
            {hasUnsavedChanges && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-amber-100 text-amber-800">
                Unsaved Changes
              </span>
            )}
            <span
              className={clsx(
                'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
                {
                  'bg-red-100 text-red-800': recommendation.level === 'aggressive',
                  'bg-blue-100 text-blue-800': recommendation.level === 'conservative',
                  'bg-green-100 text-green-800': recommendation.level === 'balanced',
                }
              )}
            >
              {recommendation.level.charAt(0).toUpperCase() + recommendation.level.slice(1)}
            </span>
          </div>
        </div>
        <p className="text-sm text-gray-600 mt-1">{recommendation.description}</p>
      </div>

      <div className="p-6">
        {/* Basic Settings */}
        <div className="space-y-6">
          <div>
            <h4 className="text-base font-medium text-gray-900 mb-4">
              Duplicate Handling Behavior
            </h4>

            <div className="space-y-4">
              {/* Auto Deduplicate */}
              <PolicyToggle
                id="auto_deduplicate"
                title="Automatic Deduplication"
                description="Automatically create hard links for duplicate files without prompting"
                enabled={policy.auto_deduplicate}
                onChange={enabled => handlePolicyUpdate({ auto_deduplicate: enabled })}
                icon="auto"
                warning={
                  policy.auto_deduplicate ? 'Files will be automatically deduplicated' : undefined
                }
              />

              {/* Always Prompt */}
              <PolicyToggle
                id="always_prompt"
                title="Always Ask Before Changes"
                description="Show a confirmation dialog before making any duplicate resolution changes"
                enabled={policy.always_prompt}
                onChange={enabled => handlePolicyUpdate({ always_prompt: enabled })}
                icon="prompt"
                recommended={!policy.auto_deduplicate}
              />
            </div>
          </div>

          {/* Advanced Settings */}
          <div className="border-t border-gray-200 pt-6">
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center space-x-2 text-sm text-blue-600 hover:text-blue-800 transition-colors mb-4"
            >
              <svg
                className={clsx('w-4 h-4 transform transition-transform', {
                  'rotate-90': showAdvanced,
                })}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
              <span>{showAdvanced ? 'Hide' : 'Show'} Advanced Settings</span>
            </button>

            {showAdvanced && (
              <div className="space-y-4">
                <h4 className="text-base font-medium text-gray-900 mb-4">
                  Conflict Resolution Preferences
                </h4>

                <PolicyToggle
                  id="prefer_newest"
                  title="Prefer Newer Files"
                  description="When automatically resolving conflicts, prefer files with more recent modification dates"
                  enabled={policy.prefer_newest}
                  onChange={enabled => handlePolicyUpdate({ prefer_newest: enabled })}
                  icon="clock"
                  conflictsWith={
                    policy.prefer_largest ? "Conflicts with 'Prefer Larger Files'" : undefined
                  }
                />

                <PolicyToggle
                  id="prefer_largest"
                  title="Prefer Larger Files"
                  description="When automatically resolving conflicts, prefer files with larger file sizes"
                  enabled={policy.prefer_largest}
                  onChange={enabled => handlePolicyUpdate({ prefer_largest: enabled })}
                  icon="size"
                  conflictsWith={
                    policy.prefer_newest ? "Conflicts with 'Prefer Newer Files'" : undefined
                  }
                />

                {/* Conflict Warning */}
                {policy.prefer_newest && policy.prefer_largest && (
                  <div className="bg-amber-50 border border-amber-200 rounded-lg p-4">
                    <div className="flex items-start space-x-3">
                      <svg
                        className="w-5 h-5 text-amber-600 mt-0.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 15.5c-.77.833.192 2.5 1.732 2.5z"
                        />
                      </svg>
                      <div>
                        <p className="text-sm font-medium text-amber-800">
                          Conflicting Preferences
                        </p>
                        <p className="text-sm text-amber-700 mt-1">
                          Both "prefer newest" and "prefer largest" are enabled. The system will
                          prioritize newer files first, then larger files as a tiebreaker.
                        </p>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Policy Preview */}
          <div className="border-t border-gray-200 pt-6">
            <h4 className="text-base font-medium text-gray-900 mb-4">Policy Summary</h4>
            <div className="bg-gray-50 rounded-lg p-4">
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">When duplicates are found:</span>
                  <span className="font-medium">
                    {policy.always_prompt
                      ? 'Show dialog'
                      : policy.auto_deduplicate
                        ? 'Auto-deduplicate'
                        : 'Skip'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Conflict resolution:</span>
                  <span className="font-medium">
                    {policy.prefer_newest && policy.prefer_largest
                      ? 'Newest, then largest'
                      : policy.prefer_newest
                        ? 'Prefer newest'
                        : policy.prefer_largest
                          ? 'Prefer largest'
                          : 'Manual selection'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">User interaction:</span>
                  <span className="font-medium">
                    {policy.always_prompt ? 'Always required' : 'Minimal'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center justify-between mt-8 pt-6 border-t border-gray-200">
          <button
            onClick={handleReset}
            className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
            disabled={!hasUnsavedChanges}
          >
            Reset to Default
          </button>

          <div className="flex space-x-3">
            <button
              onClick={handleSave}
              disabled={!hasUnsavedChanges}
              className={clsx('px-4 py-2 rounded-md font-medium transition-colors', {
                'bg-blue-600 text-white hover:bg-blue-700': hasUnsavedChanges,
                'bg-gray-300 text-gray-500 cursor-not-allowed': !hasUnsavedChanges,
              })}
            >
              Save Policy
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

interface PolicyToggleProps {
  id: string
  title: string
  description: string
  enabled: boolean
  onChange: (enabled: boolean) => void
  icon: 'auto' | 'prompt' | 'clock' | 'size'
  warning?: string
  recommended?: boolean
  conflictsWith?: string
}

const PolicyToggle: React.FC<PolicyToggleProps> = ({
  id,
  title,
  description,
  enabled,
  onChange,
  icon,
  warning,
  recommended,
  conflictsWith,
}) => {
  const IconComponent = () => {
    switch (icon) {
      case 'auto':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13 10V3L4 14h7v7l9-11h-7z"
            />
          </svg>
        )
      case 'prompt':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
            />
          </svg>
        )
      case 'clock':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        )
      case 'size':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z"
            />
          </svg>
        )
    }
  }

  return (
    <div
      className={clsx('border rounded-lg p-4 transition-colors', {
        'border-blue-200 bg-blue-50': enabled,
        'border-gray-200': !enabled,
      })}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-3 flex-1">
          <div
            className={clsx('p-2 rounded-lg transition-colors', {
              'bg-blue-100 text-blue-600': enabled,
              'bg-gray-100 text-gray-600': !enabled,
            })}
          >
            <IconComponent />
          </div>

          <div className="flex-1">
            <div className="flex items-center space-x-2">
              <label htmlFor={id} className="font-medium text-gray-900 cursor-pointer">
                {title}
              </label>
              {recommended && (
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  Recommended
                </span>
              )}
            </div>
            <p className="text-sm text-gray-600 mt-1">{description}</p>

            {warning && enabled && (
              <div className="mt-2 text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded px-2 py-1">
                ⚠️ {warning}
              </div>
            )}

            {conflictsWith && enabled && (
              <div className="mt-2 text-xs text-red-700 bg-red-50 border border-red-200 rounded px-2 py-1">
                ⚠️ {conflictsWith}
              </div>
            )}
          </div>
        </div>

        <div className="ml-4">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              id={id}
              checked={enabled}
              onChange={e => onChange(e.target.checked)}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
          </label>
        </div>
      </div>
    </div>
  )
}

export default DuplicateResolutionPolicyConfig
