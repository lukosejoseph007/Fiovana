import React, { useState, useEffect, useCallback, useMemo } from 'react'
import Modal from '../ui/Modal'
import Button from '../ui/Button'
import Dropdown from '../ui/Dropdown'
import Badge from '../ui/Badge'
import Progress from '../ui/Progress'
import { designTokens } from '../../styles/tokens'
import { styleAnalysisService } from '../../services/styleAnalysisService'
import { styleTransferService } from '../../services/styleTransferService'
import type { StyleProfile, TransferResult } from '../../types'

export interface StyleTransferProps {
  isOpen: boolean
  onClose: () => void
  documentId: string
  onTransferComplete?: (result: TransferResult) => void
}

type TransferMode = 'conservative' | 'moderate' | 'aggressive'

interface ComparisonData {
  original: string
  transformed: string
  confidence: number
  changes: string[]
}

const TRANSFER_MODES = [
  {
    value: 'conservative' as TransferMode,
    label: 'Conservative',
    description: 'Minimal changes, preserve original voice',
    color: designTokens.colors.accent.success,
  },
  {
    value: 'moderate' as TransferMode,
    label: 'Moderate',
    description: 'Balanced approach, adapt key style elements',
    color: designTokens.colors.accent.ai,
  },
  {
    value: 'aggressive' as TransferMode,
    label: 'Aggressive',
    description: 'Maximum transformation, full style adoption',
    color: designTokens.colors.accent.warning,
  },
]

const StyleTransfer: React.FC<StyleTransferProps> = ({
  isOpen,
  onClose,
  documentId,
  onTransferComplete,
}) => {
  const [styleProfiles, setStyleProfiles] = useState<StyleProfile[]>([])
  const [selectedProfile, setSelectedProfile] = useState<StyleProfile | null>(null)
  const [transferMode, setTransferMode] = useState<TransferMode>('moderate')
  const [isLearning, setIsLearning] = useState(false)
  const [learningProgress, setLearningProgress] = useState(0)
  const [isTransferring, setIsTransferring] = useState(false)
  const [transferProgress, setTransferProgress] = useState(0)
  const [comparisonData, setComparisonData] = useState<ComparisonData | null>(null)
  const [isLoadingPreview, setIsLoadingPreview] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'select' | 'compare'>('select')

  // Define loadStyleProfiles first
  const loadStyleProfiles = useCallback(async () => {
    try {
      const response = await styleAnalysisService.listStyleProfiles()
      if (response.success && response.data) {
        const profiles = Array.isArray(response.data) ? response.data : []
        setStyleProfiles(profiles)
        if (profiles.length > 0 && !selectedProfile) {
          setSelectedProfile(profiles[0] || null)
        }
      } else {
        // No profiles available, set empty array
        setStyleProfiles([])
      }
    } catch (err) {
      console.error('Failed to load style profiles:', err)
      setStyleProfiles([])
      setError('Failed to load style profiles')
    }
  }, [selectedProfile])

  // Load style profiles on mount
  useEffect(() => {
    if (isOpen) {
      loadStyleProfiles()
    }
  }, [isOpen, loadStyleProfiles])

  const loadPreview = useCallback(async () => {
    if (!selectedProfile || !documentId) return

    setIsLoadingPreview(true)
    setError(null)

    try {
      const response = await styleTransferService.previewStyleTransfer(
        documentId,
        selectedProfile.id
      )

      if (response.success && response.data) {
        // Simulate comparison data (real implementation would come from backend)
        setComparisonData({
          original: 'Original text with current style...',
          transformed: 'Transformed text with new style applied...',
          confidence: 0.85,
          changes: [
            'Vocabulary adjustment: 5 terms replaced',
            'Sentence structure: 3 sentences restructured',
            'Tone adjustment: Formality increased',
            'Technical terminology: Added 2 domain-specific terms',
          ],
        })
      }
    } catch (err) {
      console.error('Failed to load preview:', err)
      setError('Failed to load preview')
    } finally {
      setIsLoadingPreview(false)
    }
  }, [selectedProfile, documentId])

  // Load preview when profile or mode changes
  useEffect(() => {
    if (selectedProfile && documentId && activeTab === 'compare') {
      loadPreview()
    }
  }, [selectedProfile, transferMode, activeTab, documentId, loadPreview])

  const handleProfileChange = useCallback(
    (profileId: string) => {
      const profile = styleProfiles.find(p => p.id === profileId)
      setSelectedProfile(profile || null)
      setComparisonData(null)
      setError(null)
    },
    [styleProfiles]
  )

  const handleLearnStyle = useCallback(async () => {
    if (!documentId) return

    setIsLearning(true)
    setLearningProgress(0)
    setError(null)

    try {
      // Simulate learning progress
      for (let i = 0; i <= 100; i += 20) {
        setLearningProgress(i)
        await new Promise(resolve => setTimeout(resolve, 300))
      }

      const profileName = `Style from Document ${Date.now()}`
      const response = await styleAnalysisService.createStyleProfile(documentId, profileName)

      if (response.success && response.data) {
        await loadStyleProfiles()
        setSelectedProfile(response.data)
        setLearningProgress(100)
      } else {
        throw new Error('Failed to create style profile')
      }
    } catch (err) {
      console.error('Style learning failed:', err)
      setError(err instanceof Error ? err.message : 'Style learning failed')
    } finally {
      setIsLearning(false)
    }
  }, [documentId, loadStyleProfiles])

  const handleClose = useCallback(() => {
    if (!isTransferring && !isLearning) {
      setComparisonData(null)
      setError(null)
      setTransferProgress(0)
      setLearningProgress(0)
      setActiveTab('select')
      onClose()
    }
  }, [isTransferring, isLearning, onClose])

  const handleTransfer = useCallback(async () => {
    if (!selectedProfile || !documentId) {
      setError('Please select a style profile')
      return
    }

    setIsTransferring(true)
    setTransferProgress(0)
    setError(null)

    try {
      // Simulate transfer progress
      for (let i = 0; i <= 75; i += 25) {
        setTransferProgress(i)
        await new Promise(resolve => setTimeout(resolve, 400))
      }

      const response = await styleTransferService.applyStyleProfile(
        documentId,
        selectedProfile.id,
        {
          mode: transferMode,
        }
      )

      setTransferProgress(100)

      if (response.success && response.data) {
        if (onTransferComplete) {
          onTransferComplete(response.data)
        }

        // Close modal after brief delay
        setTimeout(() => {
          handleClose()
        }, 500)
      } else {
        throw new Error('Style transfer failed')
      }
    } catch (err) {
      console.error('Transfer failed:', err)
      setError(err instanceof Error ? err.message : 'Transfer failed')
    } finally {
      setIsTransferring(false)
    }
  }, [selectedProfile, documentId, transferMode, onTransferComplete, handleClose])

  const selectedModeConfig = useMemo(
    () => TRANSFER_MODES.find(m => m.value === transferMode),
    [transferMode]
  )

  // Styles
  const containerStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[6],
    minHeight: '600px',
  }

  const tabsContainerStyles: React.CSSProperties = {
    display: 'flex',
    gap: designTokens.spacing[2],
    borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
    marginBottom: designTokens.spacing[4],
  }

  const tabStyles = (isActive: boolean): React.CSSProperties => ({
    padding: `${designTokens.spacing[3]} ${designTokens.spacing[4]}`,
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: isActive
      ? designTokens.typography.fontWeight.semibold
      : designTokens.typography.fontWeight.normal,
    color: isActive ? designTokens.colors.accent.ai : designTokens.colors.text.secondary,
    borderBottom: isActive ? `2px solid ${designTokens.colors.accent.ai}` : 'none',
    cursor: 'pointer',
    background: 'none',
    border: 'none',
    outline: 'none',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
  })

  const sectionStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[3],
  }

  const labelStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
  }

  const descriptionStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    marginTop: designTokens.spacing[1],
  }

  const profileCardStyles: React.CSSProperties = {
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[4],
  }

  const profileHeaderStyles: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: designTokens.spacing[3],
  }

  const profileNameStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.base,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
  }

  const profileFeaturesStyles: React.CSSProperties = {
    display: 'flex',
    flexWrap: 'wrap',
    gap: designTokens.spacing[2],
    marginTop: designTokens.spacing[3],
  }

  const modeGridStyles: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: designTokens.spacing[3],
  }

  const modeCardStyles = (isSelected: boolean, color: string): React.CSSProperties => ({
    backgroundColor: isSelected ? `${color}10` : designTokens.colors.surface.secondary,
    border: `2px solid ${isSelected ? color : designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[4],
    cursor: 'pointer',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    textAlign: 'center',
  })

  const modeLabelStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    marginBottom: designTokens.spacing[2],
  }

  const modeDescStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.xs,
    color: designTokens.colors.text.secondary,
  }

  const comparisonContainerStyles: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: designTokens.spacing[4],
    marginTop: designTokens.spacing[4],
  }

  const comparisonPanelStyles: React.CSSProperties = {
    backgroundColor: designTokens.colors.surface.tertiary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[4],
    minHeight: '200px',
  }

  const comparisonTitleStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    marginBottom: designTokens.spacing[3],
  }

  const comparisonContentStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    lineHeight: designTokens.typography.lineHeight.relaxed,
  }

  const changesListStyles: React.CSSProperties = {
    marginTop: designTokens.spacing[4],
    padding: designTokens.spacing[4],
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
  }

  const changeItemStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    marginBottom: designTokens.spacing[2],
    paddingLeft: designTokens.spacing[3],
  }

  const errorStyles: React.CSSProperties = {
    backgroundColor: `${designTokens.colors.accent.alert}20`,
    border: `1px solid ${designTokens.colors.accent.alert}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[3],
    color: designTokens.colors.accent.alert,
    fontSize: designTokens.typography.fontSize.sm,
  }

  const actionsStyles: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    marginTop: designTokens.spacing[6],
    paddingTop: designTokens.spacing[4],
    borderTop: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const leftActionsStyles: React.CSSProperties = {
    display: 'flex',
    gap: designTokens.spacing[2],
  }

  const rightActionsStyles: React.CSSProperties = {
    display: 'flex',
    gap: designTokens.spacing[3],
  }

  const loadingOverlayStyles: React.CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: `${designTokens.colors.surface.primary}80`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    borderRadius: designTokens.borderRadius.md,
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      size="xl"
      title="Style Transfer"
      closeOnOverlayClick={!isTransferring && !isLearning}
      closeOnEscape={!isTransferring && !isLearning}
    >
      <div style={containerStyles}>
        {/* Tabs */}
        <div style={tabsContainerStyles}>
          <button
            style={tabStyles(activeTab === 'select')}
            onClick={() => setActiveTab('select')}
            disabled={isTransferring || isLearning}
          >
            Select Style
          </button>
          <button
            style={tabStyles(activeTab === 'compare')}
            onClick={() => setActiveTab('compare')}
            disabled={isTransferring || isLearning || !selectedProfile}
          >
            Compare & Preview
          </button>
        </div>

        {/* Select Style Tab */}
        {activeTab === 'select' && (
          <>
            {/* Style Profile Selection */}
            <div style={sectionStyles}>
              <label style={labelStyles}>Style Profile</label>
              <Dropdown
                options={(Array.isArray(styleProfiles) ? styleProfiles : []).map(p => ({
                  value: p.id,
                  label: p.name,
                }))}
                value={selectedProfile?.id || ''}
                onChange={handleProfileChange}
                placeholder="Select a style profile"
                disabled={isTransferring || isLearning}
              />
              <p style={descriptionStyles}>
                Choose a style profile to apply to your document, or learn a new style from existing
                content.
              </p>
            </div>

            {/* Selected Profile Details */}
            {selectedProfile && (
              <div style={profileCardStyles}>
                <div style={profileHeaderStyles}>
                  <div style={profileNameStyles}>{selectedProfile.name}</div>
                  <Badge variant="ai">
                    {`${Math.round(selectedProfile.confidence * 100)}% confidence`}
                  </Badge>
                </div>
                <p style={descriptionStyles}>Source: {selectedProfile.source}</p>

                {/* Style Features */}
                {selectedProfile.features.length > 0 && (
                  <div style={profileFeaturesStyles}>
                    {selectedProfile.features.slice(0, 6).map((feature, idx) => (
                      <Badge key={idx} variant="status">
                        {feature.type}
                      </Badge>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Transfer Mode Selection */}
            <div style={sectionStyles}>
              <label style={labelStyles}>Transfer Mode</label>
              <div style={modeGridStyles}>
                {TRANSFER_MODES.map(mode => (
                  <div
                    key={mode.value}
                    style={modeCardStyles(transferMode === mode.value, mode.color)}
                    onClick={() => !isTransferring && !isLearning && setTransferMode(mode.value)}
                  >
                    <div style={modeLabelStyles}>{mode.label}</div>
                    <div style={modeDescStyles}>{mode.description}</div>
                  </div>
                ))}
              </div>
              {selectedModeConfig && (
                <p style={descriptionStyles}>{selectedModeConfig.description}</p>
              )}
            </div>

            {/* Learning Progress */}
            {isLearning && (
              <div style={sectionStyles}>
                <label style={labelStyles}>Learning Style...</label>
                <Progress value={learningProgress} variant="ai" showPercentage />
                <p style={descriptionStyles}>
                  Analyzing document patterns and extracting style characteristics...
                </p>
              </div>
            )}
          </>
        )}

        {/* Compare & Preview Tab */}
        {activeTab === 'compare' && (
          <>
            {isLoadingPreview ? (
              <div style={{ ...comparisonPanelStyles, position: 'relative', minHeight: '300px' }}>
                <div style={loadingOverlayStyles}>Loading preview...</div>
              </div>
            ) : comparisonData ? (
              <>
                {/* Before/After Comparison */}
                <div style={comparisonContainerStyles}>
                  <div style={comparisonPanelStyles}>
                    <div style={comparisonTitleStyles}>Original</div>
                    <div style={comparisonContentStyles}>{comparisonData.original}</div>
                  </div>
                  <div style={comparisonPanelStyles}>
                    <div style={comparisonTitleStyles}>
                      Transformed
                      <Badge variant="ai" style={{ marginLeft: designTokens.spacing[2] }}>
                        {`${Math.round(comparisonData.confidence * 100)}% confidence`}
                      </Badge>
                    </div>
                    <div style={comparisonContentStyles}>{comparisonData.transformed}</div>
                  </div>
                </div>

                {/* Change Summary */}
                <div style={changesListStyles}>
                  <label style={labelStyles}>Style Changes Applied</label>
                  {comparisonData.changes.map((change, idx) => (
                    <div key={idx} style={changeItemStyles}>
                      • {change}
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <div style={comparisonPanelStyles}>
                <p style={descriptionStyles}>
                  Select a style profile and mode to preview changes...
                </p>
              </div>
            )}

            {/* Transfer Progress */}
            {isTransferring && (
              <div style={sectionStyles}>
                <label style={labelStyles}>Applying Style...</label>
                <Progress value={transferProgress} variant="ai" showPercentage />
                <p style={descriptionStyles}>
                  {transferProgress < 100
                    ? 'Transforming document with selected style...'
                    : 'Complete!'}
                </p>
              </div>
            )}
          </>
        )}

        {/* Error Display */}
        {error && <div style={errorStyles}>{error}</div>}

        {/* Actions */}
        <div style={actionsStyles}>
          <div style={leftActionsStyles}>
            <Button
              variant="secondary"
              onClick={handleLearnStyle}
              disabled={!documentId || isTransferring || isLearning}
              isLoading={isLearning}
            >
              {isLearning ? 'Learning...' : 'Learn Style'}
            </Button>
          </div>
          <div style={rightActionsStyles}>
            <Button
              variant="secondary"
              onClick={handleClose}
              disabled={isTransferring || isLearning}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleTransfer}
              disabled={!selectedProfile || !documentId || isTransferring || isLearning}
              isLoading={isTransferring}
            >
              {isTransferring ? 'Applying...' : 'Apply Style'}
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  )
}

export default StyleTransfer
