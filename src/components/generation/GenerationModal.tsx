import React, { useState, useEffect, useCallback, useMemo } from 'react'
import Modal from '../ui/Modal'
import Button from '../ui/Button'
import Dropdown from '../ui/Dropdown'
import Icon from '../ui/Icon'
import { designTokens } from '../../styles/tokens'
import { templateService } from '../../services/templateService'
import { documentGenerationService } from '../../services/documentGenerationService'
import { documentService } from '../../services/documentService'
import type { ContentTemplate, DocumentGeneration, Document } from '../../types'

export interface GenerationModalProps {
  isOpen: boolean
  onClose: () => void
  sourceDocumentId?: string
  onGenerationComplete?: (generation: DocumentGeneration) => void
}

interface GenerationConfig {
  sourceDocuments: string[]
  templateId: string
  audience: string
  stylePreservation: 'strict' | 'moderate' | 'flexible'
  outputFormat: string
  customParameters: Record<string, unknown>
}

// Wizard steps
type WizardStep = 'source' | 'format' | 'options' | 'preview'

const AUDIENCES = [
  {
    value: 'instructors',
    label: 'Instructors',
    description: 'Teaching professionals and educators with advanced knowledge',
  },
  {
    value: 'students',
    label: 'Students',
    description: 'Learners at various education levels requiring clear explanations',
  },
  {
    value: 'professionals',
    label: 'Professionals',
    description: 'Industry practitioners with domain expertise',
  },
  {
    value: 'managers',
    label: 'Managers',
    description: 'Team leads and decision makers focused on strategic outcomes',
  },
  {
    value: 'technicians',
    label: 'Technicians',
    description: 'Technical staff and operators requiring practical guidance',
  },
  {
    value: 'general',
    label: 'General Audience',
    description: 'Broad audience with varied backgrounds and knowledge levels',
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
    description: 'Maintain exact style and formatting from source documents',
  },
  {
    value: 'moderate',
    label: 'Moderate',
    description: 'Adapt style while preserving key characteristics and tone',
  },
  {
    value: 'flexible',
    label: 'Flexible',
    description: 'Optimize style for target audience and output format',
  },
]

const GenerationModal: React.FC<GenerationModalProps> = ({
  isOpen,
  onClose,
  sourceDocumentId,
  onGenerationComplete,
}) => {
  const [currentStep, setCurrentStep] = useState<WizardStep>('source')
  const [templates, setTemplates] = useState<ContentTemplate[]>([])
  const [availableDocuments, setAvailableDocuments] = useState<Document[]>([])
  const [selectedTemplate, setSelectedTemplate] = useState<ContentTemplate | null>(null)
  const [config, setConfig] = useState<GenerationConfig>({
    sourceDocuments: sourceDocumentId ? [sourceDocumentId] : [],
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

  // Load initial data on mount
  useEffect(() => {
    if (isOpen) {
      loadTemplates()
      loadAvailableDocuments()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen])

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
      } else {
        // No templates available - this is okay, generation can work without templates
        setTemplates([])
        console.info('No templates available - generation will proceed without template')
      }
    } catch (err) {
      console.error('Failed to load templates:', err)
      // Don't show error - template is optional
      setTemplates([])
    }
  }, [config.templateId])

  const loadAvailableDocuments = useCallback(async () => {
    try {
      const response = await documentService.getAllDocuments()
      if (response.success && response.data) {
        setAvailableDocuments(response.data)
      }
    } catch (err) {
      console.error('Failed to load documents:', err)
      setError('Failed to load available documents')
    }
  }, [])

  // Load preview when on preview step
  useEffect(() => {
    if (currentStep === 'preview' && config.sourceDocuments.length > 0) {
      loadPreview()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentStep])

  const loadPreview = useCallback(async () => {
    if (config.sourceDocuments.length === 0) return

    setIsLoadingPreview(true)
    setError(null)
    try {
      const selectedAudience = AUDIENCES.find(a => a.value === config.audience)
      const selectedFormat = OUTPUT_FORMATS.find(f => f.value === config.outputFormat)
      const selectedPreservation = STYLE_PRESERVATION.find(
        s => s.value === config.stylePreservation
      )

      const previewText = `
Generation Preview
==================

Source Documents: ${config.sourceDocuments.length} selected
Template: ${selectedTemplate?.name || 'None (direct generation from source)'}
Target Audience: ${selectedAudience?.label} - ${selectedAudience?.description}
Output Format: ${selectedFormat?.label}
Style Preservation: ${selectedPreservation?.label} - ${selectedPreservation?.description}

The generated document will be created using the selected configuration.
${selectedTemplate ? 'Using template: ' + selectedTemplate.name : 'Generating directly from source documents without a template.'}
Content will be adapted for ${selectedAudience?.label.toLowerCase()} with ${config.stylePreservation} style preservation.
Output will be in ${selectedFormat?.label} format.
      `.trim()

      setPreviewContent(previewText)
    } catch (err) {
      console.error('Failed to load preview:', err)
      setError('Failed to generate preview')
    } finally {
      setIsLoadingPreview(false)
    }
  }, [config, selectedTemplate])

  const handleTemplateChange = useCallback(
    (templateId: string) => {
      const template = templates.find(t => t.id === templateId)
      setSelectedTemplate(template || null)
      setConfig(prev => ({ ...prev, templateId }))
    },
    [templates]
  )

  const handleDocumentToggle = useCallback((documentId: string) => {
    setConfig(prev => ({
      ...prev,
      sourceDocuments: prev.sourceDocuments.includes(documentId)
        ? prev.sourceDocuments.filter(id => id !== documentId)
        : [...prev.sourceDocuments, documentId],
    }))
  }, [])

  const handleNextStep = useCallback(() => {
    const steps: WizardStep[] = ['source', 'format', 'options', 'preview']
    const currentIndex = steps.indexOf(currentStep)
    if (currentIndex < steps.length - 1) {
      setCurrentStep(steps[currentIndex + 1] as WizardStep)
      setError(null)
    }
  }, [currentStep])

  const handlePreviousStep = useCallback(() => {
    const steps: WizardStep[] = ['source', 'format', 'options', 'preview']
    const currentIndex = steps.indexOf(currentStep)
    if (currentIndex > 0) {
      setCurrentStep(steps[currentIndex - 1] as WizardStep)
      setError(null)
    }
  }, [currentStep])

  const handleGenerate = useCallback(async () => {
    if (config.sourceDocuments.length === 0) {
      setError('Please select at least one source document')
      return
    }

    setIsGenerating(true)
    setGenerationProgress(0)
    setError(null)

    try {
      // Step 1: Resolve document ID to file path
      setGenerationProgress(20)
      const primaryDocId = config.sourceDocuments[0]
      if (!primaryDocId) {
        throw new Error('No source document selected')
      }

      // Get the document details to find the actual file path
      const doc = availableDocuments.find(d => d.id === primaryDocId)
      if (!doc || !doc.path) {
        throw new Error(`Could not find file path for document: ${primaryDocId}`)
      }

      const filePath = doc.path

      // Step 2: Read source document content (40% progress)
      setGenerationProgress(40)
      const contentResponse = await documentService.getDocumentContent(filePath)

      if (!contentResponse.success || !contentResponse.data) {
        throw new Error(
          `Failed to read document content: ${contentResponse.error || 'Unknown error'}`
        )
      }

      const sourceContent = contentResponse.data

      // Step 3: Generate document (65% progress)
      setGenerationProgress(65)
      let generationResponse

      if (config.templateId) {
        // Generate from template if one is selected
        generationResponse = await documentGenerationService.generateFromTemplate(
          config.templateId,
          {
            sourceContent,
            audience: config.audience,
            stylePreservation: config.stylePreservation,
            ...config.customParameters,
          },
          config.outputFormat
        )
      } else {
        // Generate directly from source content without template
        const outputFilename = `generated_${Date.now()}.${config.outputFormat}`

        // Extract a reasonable title from the document path or use a default
        const documentTitle =
          primaryDocId
            .split('/')
            .pop()
            ?.replace(/\.[^/.]+$/, '') || 'Generated Document'

        generationResponse = await documentGenerationService.generateFromText(
          documentTitle,
          sourceContent,
          outputFilename,
          config.outputFormat,
          {
            sourceDocument: primaryDocId,
            audience: config.audience,
            stylePreservation: config.stylePreservation,
            ...Object.fromEntries(
              Object.entries(config.customParameters).map(([k, v]) => [k, String(v)])
            ),
          }
        )
      }

      if (!generationResponse.success || !generationResponse.data) {
        throw new Error('Document generation failed')
      }

      // Step 3: Format conversion if needed (75% progress)
      setGenerationProgress(75)
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
  }, [config, onGenerationComplete, onClose])

  const resetState = useCallback(() => {
    setCurrentStep('source')
    setGenerationProgress(0)
    setPreviewContent('')
    setError(null)
    setIsLoadingPreview(false)
    setConfig({
      sourceDocuments: sourceDocumentId ? [sourceDocumentId] : [],
      templateId: '',
      audience: 'general',
      stylePreservation: 'moderate',
      outputFormat: 'docx',
      customParameters: {},
    })
  }, [sourceDocumentId])

  const handleClose = useCallback(() => {
    if (!isGenerating) {
      resetState()
      onClose()
    }
  }, [isGenerating, onClose, resetState])

  // Check if can proceed to next step
  const canProceed = useMemo(() => {
    switch (currentStep) {
      case 'source':
        return config.sourceDocuments.length > 0
      case 'format':
        // Template is optional, only format is required
        return config.outputFormat !== ''
      case 'options':
        return config.audience !== ''
      case 'preview':
        return true
      default:
        return false
    }
  }, [currentStep, config])

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'source':
        return (
          <div style={sectionStyles}>
            <h3 style={stepTitleStyles}>Step 1: Select Source Documents</h3>
            <p style={descriptionStyles}>
              Choose one or more documents to use as the source for generation
            </p>
            <div style={documentListStyles}>
              {availableDocuments.length === 0 ? (
                <div style={emptyStateStyles}>
                  <Icon name="FileText" size={48} color={designTokens.colors.text.tertiary} />
                  <p style={{ marginTop: designTokens.spacing[2] }}>No documents available</p>
                  <p style={{ fontSize: designTokens.typography.fontSize.sm }}>
                    Please import documents to get started
                  </p>
                </div>
              ) : (
                availableDocuments.map(doc => (
                  <div
                    key={doc.id}
                    style={{
                      ...documentItemStyles,
                      ...(config.sourceDocuments.includes(doc.id) ? selectedDocumentStyles : {}),
                    }}
                    onClick={() => handleDocumentToggle(doc.id)}
                  >
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: designTokens.spacing[2],
                      }}
                    >
                      <div
                        style={{
                          ...checkboxStyles,
                          ...(config.sourceDocuments.includes(doc.id) ? checkboxCheckedStyles : {}),
                        }}
                      >
                        {config.sourceDocuments.includes(doc.id) && (
                          <Icon name="Check" size={14} color="white" />
                        )}
                      </div>
                      <div>
                        <div style={documentTitleStyles}>{doc.path}</div>
                        <div style={documentPathStyles}>{doc.id}</div>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
            {config.sourceDocuments.length > 0 && (
              <div style={selectionCountStyles}>
                {config.sourceDocuments.length} document
                {config.sourceDocuments.length > 1 ? 's' : ''} selected
              </div>
            )}
          </div>
        )

      case 'format':
        return (
          <div style={sectionStyles}>
            <h3 style={stepTitleStyles}>Step 2: Choose Output Format</h3>
            <p style={descriptionStyles}>Select the template and output format for your document</p>

            <div style={{ marginTop: designTokens.spacing[4] }}>
              <label style={labelStyles}>Template (Optional)</label>
              {templates.length > 0 ? (
                <>
                  <Dropdown
                    options={templates.map(t => ({
                      value: t.id,
                      label: t.name,
                    }))}
                    value={config.templateId}
                    onChange={handleTemplateChange}
                    placeholder="Select a template (optional)"
                    disabled={isGenerating}
                  />
                  {selectedTemplate && (
                    <p style={descriptionStyles}>{selectedTemplate.description}</p>
                  )}
                </>
              ) : (
                <div style={noTemplatesMessageStyles}>
                  <Icon name="Info" size={16} color={designTokens.colors.text.secondary} />
                  <span style={{ marginLeft: designTokens.spacing[2] }}>
                    No templates available. You can proceed without a template - the system will
                    generate content directly from your source documents.
                  </span>
                </div>
              )}
            </div>

            <div style={{ marginTop: designTokens.spacing[4] }}>
              <label style={labelStyles}>Output Format</label>
              <div style={formatGridStyles}>
                {OUTPUT_FORMATS.map(format => (
                  <div
                    key={format.value}
                    style={{
                      ...formatCardStyles,
                      ...(config.outputFormat === format.value ? selectedFormatCardStyles : {}),
                    }}
                    onClick={() => setConfig(prev => ({ ...prev, outputFormat: format.value }))}
                  >
                    <div style={formatIconStyles}>{format.icon}</div>
                    <div style={formatLabelStyles}>{format.label}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )

      case 'options':
        return (
          <div style={sectionStyles}>
            <h3 style={stepTitleStyles}>Step 3: Configure Generation Options</h3>
            <p style={descriptionStyles}>Customize how the content should be adapted and styled</p>

            <div style={{ marginTop: designTokens.spacing[4] }}>
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

            <div style={{ marginTop: designTokens.spacing[4] }}>
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
          </div>
        )

      case 'preview':
        return (
          <div style={sectionStyles}>
            <h3 style={stepTitleStyles}>Step 4: Preview Generation Parameters</h3>
            <p style={descriptionStyles}>Review your configuration before generating</p>

            <div style={previewContainerStyles}>
              {isLoadingPreview ? (
                <div style={previewLoadingStyles}>
                  <Icon name="Loader" size={24} color={designTokens.colors.text.secondary} />
                  <span style={{ marginLeft: designTokens.spacing[2] }}>Loading preview...</span>
                </div>
              ) : previewContent ? (
                <pre style={previewTextStyles}>{previewContent}</pre>
              ) : (
                <div style={previewLoadingStyles}>Configure settings to see preview</div>
              )}
            </div>
          </div>
        )

      default:
        return null
    }
  }

  // Styles
  const containerStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '500px',
  }

  const sectionStyles: React.CSSProperties = {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[3],
  }

  const stepTitleStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.xl,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    marginBottom: designTokens.spacing[1],
  }

  const labelStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    color: designTokens.colors.text.primary,
    marginBottom: designTokens.spacing[2],
    display: 'block',
  }

  const descriptionStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.secondary,
    marginTop: designTokens.spacing[1],
  }

  const documentListStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: designTokens.spacing[2],
    maxHeight: '300px',
    overflowY: 'auto',
    marginTop: designTokens.spacing[3],
    padding: designTokens.spacing[2],
    backgroundColor: designTokens.colors.surface.secondary,
    borderRadius: designTokens.borderRadius.md,
    border: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const documentItemStyles: React.CSSProperties = {
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.primary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    cursor: 'pointer',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
  }

  const selectedDocumentStyles: React.CSSProperties = {
    borderColor: designTokens.colors.accent.ai,
    backgroundColor: `${designTokens.colors.accent.ai}10`,
  }

  const checkboxStyles: React.CSSProperties = {
    width: '20px',
    height: '20px',
    borderRadius: designTokens.borderRadius.sm,
    border: `2px solid ${designTokens.colors.border.medium}`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  }

  const checkboxCheckedStyles: React.CSSProperties = {
    backgroundColor: designTokens.colors.accent.ai,
    borderColor: designTokens.colors.accent.ai,
  }

  const documentTitleStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.medium,
    color: designTokens.colors.text.primary,
  }

  const documentPathStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.xs,
    color: designTokens.colors.text.tertiary,
    marginTop: designTokens.spacing[1],
  }

  const selectionCountStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.accent.ai,
    fontWeight: designTokens.typography.fontWeight.medium,
    marginTop: designTokens.spacing[2],
  }

  const emptyStateStyles: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: designTokens.spacing[8],
    color: designTokens.colors.text.tertiary,
    textAlign: 'center',
  }

  const noTemplatesMessageStyles: React.CSSProperties = {
    display: 'flex',
    alignItems: 'flex-start',
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    color: designTokens.colors.text.secondary,
    fontSize: designTokens.typography.fontSize.sm,
    lineHeight: '1.5',
  }

  const formatGridStyles: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))',
    gap: designTokens.spacing[3],
    marginTop: designTokens.spacing[2],
  }

  const formatCardStyles: React.CSSProperties = {
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.secondary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    cursor: 'pointer',
    textAlign: 'center',
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
  }

  const selectedFormatCardStyles: React.CSSProperties = {
    borderColor: designTokens.colors.accent.ai,
    backgroundColor: `${designTokens.colors.accent.ai}10`,
  }

  const formatIconStyles: React.CSSProperties = {
    fontSize: '32px',
    marginBottom: designTokens.spacing[2],
  }

  const formatLabelStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.primary,
    fontWeight: designTokens.typography.fontWeight.medium,
  }

  const previewContainerStyles: React.CSSProperties = {
    backgroundColor: designTokens.colors.surface.tertiary,
    border: `1px solid ${designTokens.colors.border.subtle}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[4],
    minHeight: '250px',
    maxHeight: '350px',
    overflowY: 'auto',
    marginTop: designTokens.spacing[3],
  }

  const previewLoadingStyles: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '250px',
    color: designTokens.colors.text.secondary,
    fontSize: designTokens.typography.fontSize.sm,
  }

  const previewTextStyles: React.CSSProperties = {
    fontSize: designTokens.typography.fontSize.sm,
    color: designTokens.colors.text.primary,
    fontFamily: 'monospace',
    whiteSpace: 'pre-wrap',
    lineHeight: '1.6',
  }

  const errorStyles: React.CSSProperties = {
    backgroundColor: `${designTokens.colors.accent.alert}20`,
    border: `1px solid ${designTokens.colors.accent.alert}`,
    borderRadius: designTokens.borderRadius.md,
    padding: designTokens.spacing[3],
    color: designTokens.colors.accent.alert,
    fontSize: designTokens.typography.fontSize.sm,
    marginTop: designTokens.spacing[3],
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
    justifyContent: 'space-between',
    gap: designTokens.spacing[3],
    marginTop: designTokens.spacing[6],
    paddingTop: designTokens.spacing[4],
    borderTop: `1px solid ${designTokens.colors.border.subtle}`,
  }

  const stepIndicatorStyles: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[2],
    marginBottom: designTokens.spacing[4],
  }

  const stepDotStyles = (active: boolean, completed: boolean): React.CSSProperties => ({
    width: '32px',
    height: '32px',
    borderRadius: '50%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: designTokens.typography.fontSize.sm,
    fontWeight: designTokens.typography.fontWeight.semibold,
    backgroundColor: completed
      ? designTokens.colors.confidence.high
      : active
        ? designTokens.colors.accent.ai
        : designTokens.colors.surface.tertiary,
    color: active || completed ? 'white' : designTokens.colors.text.tertiary,
    border: active ? `2px solid ${designTokens.colors.accent.ai}` : 'none',
  })

  const stepLineStyles = (completed: boolean): React.CSSProperties => ({
    flex: 1,
    height: '2px',
    backgroundColor: completed
      ? designTokens.colors.confidence.high
      : designTokens.colors.border.subtle,
  })

  const steps: Array<{ key: WizardStep; label: string; number: number }> = [
    { key: 'source', label: 'Source', number: 1 },
    { key: 'format', label: 'Format', number: 2 },
    { key: 'options', label: 'Options', number: 3 },
    { key: 'preview', label: 'Preview', number: 4 },
  ]

  const currentStepIndex = steps.findIndex(s => s.key === currentStep)

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
        {/* Step Indicator */}
        <div style={stepIndicatorStyles}>
          {steps.map((step, index) => (
            <React.Fragment key={step.key}>
              <div style={stepDotStyles(step.key === currentStep, index < currentStepIndex)}>
                {index < currentStepIndex ? (
                  <Icon name="Check" size={16} color="white" />
                ) : (
                  step.number
                )}
              </div>
              {index < steps.length - 1 && <div style={stepLineStyles(index < currentStepIndex)} />}
            </React.Fragment>
          ))}
        </div>

        {/* Step Content */}
        {renderStepContent()}

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
          <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
            <Button variant="secondary" onClick={handleClose} disabled={isGenerating}>
              Cancel
            </Button>
          </div>
          <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
            {currentStep !== 'source' && (
              <Button variant="secondary" onClick={handlePreviousStep} disabled={isGenerating}>
                Previous
              </Button>
            )}
            {currentStep !== 'preview' ? (
              <Button
                variant="primary"
                onClick={handleNextStep}
                disabled={!canProceed || isGenerating}
              >
                Next
              </Button>
            ) : (
              <Button
                variant="primary"
                onClick={handleGenerate}
                disabled={!canProceed || isGenerating}
                isLoading={isGenerating}
              >
                {isGenerating ? 'Generating...' : 'Generate Document'}
              </Button>
            )}
          </div>
        </div>
      </div>
    </Modal>
  )
}

export default GenerationModal
