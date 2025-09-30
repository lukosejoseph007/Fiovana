import React, { useState, useEffect, useCallback } from 'react'
import Modal from '../ui/Modal'
import Button from '../ui/Button'
import Dropdown from '../ui/Dropdown'
import { designTokens } from '../../styles/tokens'
import { templateService } from '../../services/templateService'
import { documentGenerationService } from '../../services/documentGenerationService'
import { contentAdaptationService } from '../../services/contentAdaptationService'
import type { ContentTemplate, DocumentGeneration } from '../../types'

export interface GenerationModalProps {
  isOpen: boolean
  onClose: () => void
  sourceDocumentId?: string
  onGenerationComplete?: (generation: DocumentGeneration) => void
}

interface GenerationConfig {
  templateId: string
  audience: string
  stylePreservation: 'strict' | 'moderate' | 'flexible'
  outputFormat: string
  customParameters: Record<string, unknown>
}

const AUDIENCES = [
  {
    value: 'instructors',
    label: 'Instructors',
    description: 'Teaching professionals and educators',
  },
  { value: 'students', label: 'Students', description: 'Learners at various education levels' },
  { value: 'professionals', label: 'Professionals', description: 'Industry practitioners' },
  { value: 'managers', label: 'Managers', description: 'Team leads and decision makers' },
  { value: 'technicians', label: 'Technicians', description: 'Technical staff and operators' },
  {
    value: 'general',
    label: 'General Audience',
    description: 'Broad audience with varied backgrounds',
  },
]

const OUTPUT_FORMATS = [
  { value: 'docx', label: 'Word Document (.docx)', icon: '📄' },
  { value: 'pptx', label: 'PowerPoint (.pptx)', icon: '📊' },
  { value: 'pdf', label: 'PDF Document (.pdf)', icon: '📕' },
  { value: 'html', label: 'HTML Page (.html)', icon: '🌐' },
  { value: 'markdown', label: 'Markdown (.md)', icon: '📝' },
  { value: 'txt', label: 'Plain Text (.txt)', icon: '📃' },
]

const STYLE_PRESERVATION = [
  {
    value: 'strict',
    label: 'Strict',
    description: 'Maintain exact style and formatting from source',
  },
  {
    value: 'moderate',
    label: 'Moderate',
    description: 'Adapt style while preserving key characteristics',
  },
  {
    value: 'flexible',
    label: 'Flexible',
    description: 'Optimize style for target audience and format',
  },
]

const GenerationModal: React.FC<GenerationModalProps> = ({
  isOpen,
  onClose,
  sourceDocumentId,
  onGenerationComplete,
}) => {
  const [templates, setTemplates] = useState<ContentTemplate[]>([])
  const [selectedTemplate, setSelectedTemplate] = useState<ContentTemplate | null>(null)
  const [config, setConfig] = useState<GenerationConfig>({
    templateId: '',
    audience: 'general',
    stylePreservation: 'moderate',
    outputFormat: 'docx',
    customParameters: {},
  })
  const [isGenerating, setIsGenerating] = useState(false)
  const [generationProgress, setGenerationProgress] = useState(0)
  const [previewContent, setPreviewContent] = useState<string>('')
  const [isLoadingPreview, setIsLoadingPreview] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load templates on mount
  useEffect(() => {
    loadTemplates()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const loadTemplates = useCallback(async () => {
    try {
      const response = await templateService.listTemplates()
      if (response.success && response.data && response.data.length > 0) {
        setTemplates(response.data)
        if (!config.templateId) {
          const firstTemplate = response.data[0]
          if (firstTemplate) {
            setConfig(prev => ({ ...prev, templateId: firstTemplate.id }))
            setSelectedTemplate(firstTemplate)
          }
        }
      }
    } catch (err) {
      console.error('Failed to load templates:', err)
      setError('Failed to load templates')
    }
  }, [config.templateId])

  // Load preview when config changes
  useEffect(() => {
    if (config.templateId && sourceDocumentId) {
      loadPreview()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config, sourceDocumentId])

  const loadPreview = useCallback(async () => {
    if (!config.templateId || !sourceDocumentId) return

    setIsLoadingPreview(true)
    try {
      // Simulate preview generation (real implementation would call backend)
      await new Promise(resolve => setTimeout(resolve, 800))
      setPreviewContent('Preview: Document will be generated with selected configuration...')
    } catch (err) {
      console.error('Failed to load preview:', err)
    } finally {
      setIsLoadingPreview(false)
    }
  }, [config, sourceDocumentId])

  const handleTemplateChange = useCallback(
    (templateId: string) => {
      const template = templates.find(t => t.id === templateId)
      setSelectedTemplate(template || null)
      setConfig(prev => ({ ...prev, templateId }))
    },
    [templates]
  )

  const handleGenerate = useCallback(async () => {
    if (!config.templateId || !sourceDocumentId) {
      setError('Please select a template and source document')
      return
    }

    setIsGenerating(true)
    setGenerationProgress(0)
    setError(null)

    try {
      // Step 1: Adapt content for audience (25% progress)
      setGenerationProgress(25)
      const adaptationResponse = await contentAdaptationService.adaptContentForAudience(
        sourceDocumentId,
        config.audience,
        {
          stylePreservation: config.stylePreservation,
          customParameters: config.customParameters,
        }
      )

      if (!adaptationResponse.success) {
        throw new Error('Content adaptation failed')
      }

      // Step 2: Generate from template (50% progress)
      setGenerationProgress(50)
      const generationResponse = await documentGenerationService.generateFromTemplate(
        config.templateId,
        {
          sourceDocumentId,
          audience: config.audience,
          stylePreservation: config.stylePreservation,
          ...config.customParameters,
        }
      )

      if (!generationResponse.success || !generationResponse.data) {
        throw new Error('Document generation failed')
      }

      // Step 3: Convert to target format (75% progress)
      setGenerationProgress(75)
      // Note: Format conversion would be done at generation time with the backend
      // DocumentGeneration doesn't have an 'id' field - it's the generated content itself
      // The backend would handle format conversion during generation
      if (config.outputFormat !== 'docx') {
        // Simulate format conversion delay
        await new Promise(resolve => setTimeout(resolve, 500))
      }

      // Complete
      setGenerationProgress(100)

      if (onGenerationComplete && generationResponse.data) {
        onGenerationComplete(generationResponse.data)
      }

      // Close modal after brief delay
      setTimeout(() => {
        onClose()
        resetState()
      }, 500)
    } catch (err) {
      console.error('Generation failed:', err)
      setError(err instanceof Error ? err.message : 'Generation failed')
    } finally {
      setIsGenerating(false)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config, sourceDocumentId, onGenerationComplete, onClose])

  const resetState = useCallback(() => {
    setGenerationProgress(0)
    setPreviewContent('')
    setError(null)
    setIsLoadingPreview(false)
  }, [])

  const handleClose = useCallback(() => {
    if (!isGenerating) {
      resetState()
      onClose()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isGenerating, onClose])

  const containerStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[6],
    minHeight: '500px',
  }

  const sectionStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[3],
  }

  const labelStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    marginBottom: designTokens.spacing[1],
  }

  const descriptionStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    marginTop: designTokens.spacing[1],
  }

  const previewContainerStyles: React.CSSProperties = {
    backgroundColor: designTokens.colors.surface.tertiary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[4],
    minHeight: '200px',
    maxHeight: '300px',
    overflowY: 'auto',
    position: 'relative',
  }

  const previewLoadingStyles: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '200px',
    color: designTokens.colors.text.secondary,
    fontSize: designTokens.typography.fontSize.sm,
  }

  const errorStyles: React.CSSProperties = {
    backgroundColor: `${designTokens.colors.accent.alert}20`,
    border: `1px solid ${designTokens.colors.accent.alert}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[3],
    color: designTokens.colors.accent.alert,
    fontSize: designTokens.typography.fontSize.sm,
  }

  const progressContainerStyles: React.CSSProperties = {
    marginTop: designTokens.spacing[4],
  }

  const progressBarStyles: React.CSSProperties = {
    width: '100%',
    height: '4px',
    backgroundColor: designTokens.colors.surface.tertiary,
    borderRadius: designTokens.borderRadius.full,
    overflow: 'hidden',
    marginBottom: designTokens.spacing[2],
  }

  const progressFillStyles: React.CSSProperties = {
    height: '100%',
    backgroundColor: designTokens.colors.accent.ai,
    width: `${generationProgress}%`,
    transition: `width ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
  }

  const progressTextStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    textAlign: 'center',
  }

  const actionsStyles: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: designTokens.spacing[3],
    marginTop: designTokens.spacing[6],
    paddingTop: designTokens.spacing[4],
    borderTop: `1px solid ${designTokens.colors.border.subtle}`,
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      size="xl"
      title="Generate Document"
      closeOnOverlayClick={!isGenerating}
      closeOnEscape={!isGenerating}
    >
      <div style={containerStyles}>
        {/* Template Selection */}
        <div style={sectionStyles}>
          <label style={labelStyles}>Template</label>
          <Dropdown
            options={templates.map(t => ({
              value: t.id,
              label: t.name,
            }))}
            value={config.templateId}
            onChange={handleTemplateChange}
            placeholder="Select a template"
            disabled={isGenerating}
          />
          {selectedTemplate && <p style={descriptionStyles}>{selectedTemplate.description}</p>}
        </div>

        {/* Audience Selection */}
        <div style={sectionStyles}>
          <label style={labelStyles}>Target Audience</label>
          <Dropdown
            options={AUDIENCES.map(a => ({
              value: a.value,
              label: a.label,
            }))}
            value={config.audience}
            onChange={(value: string) => setConfig(prev => ({ ...prev, audience: value }))}
            placeholder="Select target audience"
            disabled={isGenerating}
          />
          {AUDIENCES.find(a => a.value === config.audience) && (
            <p style={descriptionStyles}>
              {AUDIENCES.find(a => a.value === config.audience)?.description}
            </p>
          )}
        </div>

        {/* Style Preservation */}
        <div style={sectionStyles}>
          <label style={labelStyles}>Style Preservation</label>
          <Dropdown
            options={STYLE_PRESERVATION.map(s => ({
              value: s.value,
              label: s.label,
            }))}
            value={config.stylePreservation}
            onChange={(value: string) =>
              setConfig(prev => ({
                ...prev,
                stylePreservation: value as 'strict' | 'moderate' | 'flexible',
              }))
            }
            placeholder="Select style preservation level"
            disabled={isGenerating}
          />
          {STYLE_PRESERVATION.find(s => s.value === config.stylePreservation) && (
            <p style={descriptionStyles}>
              {STYLE_PRESERVATION.find(s => s.value === config.stylePreservation)?.description}
            </p>
          )}
        </div>

        {/* Output Format */}
        <div style={sectionStyles}>
          <label style={labelStyles}>Output Format</label>
          <Dropdown
            options={OUTPUT_FORMATS.map(f => ({
              value: f.value,
              label: `${f.icon} ${f.label}`,
            }))}
            value={config.outputFormat}
            onChange={(value: string) => setConfig(prev => ({ ...prev, outputFormat: value }))}
            placeholder="Select output format"
            disabled={isGenerating}
          />
        </div>

        {/* Preview Section */}
        <div style={sectionStyles}>
          <label style={labelStyles}>Preview</label>
          <div style={previewContainerStyles}>
            {isLoadingPreview ? (
              <div style={previewLoadingStyles}>Loading preview...</div>
            ) : previewContent ? (
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                }}
              >
                {previewContent}
              </div>
            ) : (
              <div style={previewLoadingStyles}>Configure settings to see preview</div>
            )}
          </div>
        </div>

        {/* Error Display */}
        {error && <div style={errorStyles}>{error}</div>}

        {/* Progress Indicator */}
        {isGenerating && (
          <div style={progressContainerStyles}>
            <div style={progressBarStyles}>
              <div style={progressFillStyles} />
            </div>
            <div style={progressTextStyles}>
              {generationProgress < 100 ? `Generating... ${generationProgress}%` : 'Complete!'}
            </div>
          </div>
        )}

        {/* Actions */}
        <div style={actionsStyles}>
          <Button variant="secondary" onClick={handleClose} disabled={isGenerating}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleGenerate}
            disabled={!config.templateId || !sourceDocumentId || isGenerating}
            isLoading={isGenerating}
          >
            {isGenerating ? 'Generating...' : 'Generate Document'}
          </Button>
        </div>
      </div>
    </Modal>
  )
}

export default GenerationModal
