import React, { useState, useCallback, useMemo, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { designTokens } from '../../styles/tokens'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Progress from '../ui/Progress'
import Modal from '../ui/Modal'
import { documentGenerationService } from '../../services/documentGenerationService'
import { formatConversionService } from '../../services/formatConversionService'

export interface BatchManagerProps {
  className?: string
  style?: React.CSSProperties
  onClose?: () => void
  onBatchComplete?: (results: BatchOperationResult[]) => void
}

export interface BatchFile {
  id: string
  path: string
  name: string
  size: number
  type: string
  status: 'pending' | 'processing' | 'completed' | 'error'
  progress: number
  error?: string
  result?: unknown
}

export interface BatchOperation {
  id: string
  type: 'generate' | 'convert' | 'analyze' | 'compare' | 'index' | 'validate'
  status: 'idle' | 'running' | 'paused' | 'completed' | 'error'
  files: BatchFile[]
  parameters: Record<string, unknown>
  startedAt?: Date
  completedAt?: Date
  totalProgress: number
}

export interface BatchOperationResult {
  fileId: string
  filePath: string
  success: boolean
  result?: unknown
  error?: string
  duration: number
}

const OPERATION_TYPES = [
  {
    value: 'validate',
    label: 'Validate Files',
    icon: 'Shield',
    description: 'Check file integrity and format',
  },
  {
    value: 'index',
    label: 'Index Documents',
    icon: 'Database',
    description: 'Add files to search index',
  },
  {
    value: 'analyze',
    label: 'Analyze Content',
    icon: 'Search',
    description: 'Analyze document structure and content',
  },
  {
    value: 'convert',
    label: 'Convert Format',
    icon: 'RefreshCw',
    description: 'Convert to different format',
  },
  {
    value: 'generate',
    label: 'Generate Outputs',
    icon: 'FilePlus',
    description: 'Generate derivative content',
  },
] as const

const BatchManager: React.FC<BatchManagerProps> = ({
  className = '',
  style,
  onClose,
  onBatchComplete,
}) => {
  const [operation, setOperation] = useState<BatchOperation>({
    id: `batch-${Date.now()}`,
    type: 'validate',
    status: 'idle',
    files: [],
    parameters: {},
    totalProgress: 0,
  })

  const [selectedOperationType, setSelectedOperationType] = useState<string>('validate')
  const [isDragging, setIsDragging] = useState(false)
  const [showResults, setShowResults] = useState(false)
  const [results, setResults] = useState<BatchOperationResult[]>([])
  const [isPaused, setIsPaused] = useState(false)
  const dropZoneRef = useRef<HTMLDivElement>(null)

  // Handle file selection via dialog
  const handleSelectFiles = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Documents',
            extensions: ['pdf', 'docx', 'doc', 'txt', 'md', 'html', 'pptx'],
          },
        ],
      })

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected]
        const newFiles: BatchFile[] = paths.map(path => ({
          id: `file-${Date.now()}-${Math.random().toString(36).substring(7)}`,
          path,
          name: path.split('/').pop() || path,
          size: 0, // Will be populated on processing
          type: path.split('.').pop() || 'unknown',
          status: 'pending' as const,
          progress: 0,
        }))

        setOperation(prev => ({
          ...prev,
          files: [...prev.files, ...newFiles],
        }))
      }
    } catch (error) {
      console.error('Failed to select files:', error)
    }
  }, [])

  // Handle drag and drop
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.currentTarget === dropZoneRef.current) {
      setIsDragging(false)
    }
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)

    try {
      const items = Array.from(e.dataTransfer.items)
      const filePaths: string[] = []

      for (const item of items) {
        if (item.kind === 'file') {
          const file = item.getAsFile()
          if (file) {
            // In Tauri, we need to get the path differently
            // For now, we'll use the file name as a placeholder
            filePaths.push(file.name)
          }
        }
      }

      if (filePaths.length > 0) {
        const newFiles: BatchFile[] = filePaths.map(path => ({
          id: `file-${Date.now()}-${Math.random().toString(36).substring(7)}`,
          path,
          name: path.split('/').pop() || path,
          size: 0,
          type: path.split('.').pop() || 'unknown',
          status: 'pending' as const,
          progress: 0,
        }))

        setOperation(prev => ({
          ...prev,
          files: [...prev.files, ...newFiles],
        }))
      }
    } catch (error) {
      console.error('Failed to process dropped files:', error)
    }
  }, [])

  // Remove file from batch
  const handleRemoveFile = useCallback((fileId: string) => {
    setOperation(prev => ({
      ...prev,
      files: prev.files.filter(f => f.id !== fileId),
    }))
  }, [])

  // Clear all files
  const handleClearAll = useCallback(() => {
    setOperation(prev => ({
      ...prev,
      files: [],
      status: 'idle',
      totalProgress: 0,
    }))
    setResults([])
  }, [])

  // Process single file
  const processFile = useCallback(
    async (file: BatchFile): Promise<BatchOperationResult> => {
      const startTime = Date.now()

      try {
        let result: unknown = null

        switch (operation.type) {
          case 'validate':
            result = await invoke('validate_file_comprehensive', { filePath: file.path })
            break

          case 'index':
            result = await invoke('index_document', { filePath: file.path })
            break

          case 'analyze':
            result = await invoke('analyze_document_structure', { filePath: file.path })
            break

          case 'convert':
            if (operation.parameters.targetFormat) {
              result = await formatConversionService.convertDocument(
                file.path,
                operation.parameters.targetFormat as string,
                {
                  preserveFormatting: true,
                  includeImages: true,
                }
              )
            }
            break

          case 'generate':
            if (operation.parameters.outputFormat) {
              result = await documentGenerationService.generateFromPrompt(
                `Process ${file.name}`,
                operation.parameters.outputFormat as string,
                {}
              )
            }
            break

          default:
            throw new Error(`Unknown operation type: ${operation.type}`)
        }

        return {
          fileId: file.id,
          filePath: file.path,
          success: true,
          result,
          duration: Date.now() - startTime,
        }
      } catch (error) {
        return {
          fileId: file.id,
          filePath: file.path,
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error',
          duration: Date.now() - startTime,
        }
      }
    },
    [operation.type, operation.parameters]
  )

  // Start batch processing
  const handleStartBatch = useCallback(async () => {
    if (operation.files.length === 0) return

    setOperation(prev => ({
      ...prev,
      status: 'running',
      startedAt: new Date(),
      totalProgress: 0,
    }))

    const operationResults: BatchOperationResult[] = []
    const totalFiles = operation.files.length

    for (let i = 0; i < operation.files.length; i++) {
      if (isPaused) {
        setOperation(prev => ({ ...prev, status: 'paused' }))
        break
      }

      const file = operation.files[i]
      if (!file) continue

      // Update file status to processing
      setOperation(prev => ({
        ...prev,
        files: prev.files.map(f =>
          f.id === file.id ? { ...f, status: 'processing' as const, progress: 0 } : f
        ),
      }))

      // Process file
      const result = await processFile(file)
      operationResults.push(result)

      // Update file status
      setOperation(prev => ({
        ...prev,
        files: prev.files.map(f =>
          f.id === file.id
            ? {
                ...f,
                status: result.success ? ('completed' as const) : ('error' as const),
                progress: 100,
                error: result.error,
                result: result.result,
              }
            : f
        ),
        totalProgress: Math.round(((i + 1) / totalFiles) * 100),
      }))

      // Simulate progress for visual feedback
      for (let progress = 0; progress <= 100; progress += 20) {
        setOperation(prev => ({
          ...prev,
          files: prev.files.map(f =>
            f.id === file.id && f.status === 'processing' ? { ...f, progress } : f
          ),
        }))
        await new Promise(resolve => setTimeout(resolve, 100))
      }
    }

    setResults(operationResults)
    setOperation(prev => ({
      ...prev,
      status: 'completed',
      completedAt: new Date(),
      totalProgress: 100,
    }))

    setShowResults(true)
    onBatchComplete?.(operationResults)
  }, [operation.files, isPaused, processFile, onBatchComplete])

  // Pause processing
  const handlePause = useCallback(() => {
    setIsPaused(true)
    setOperation(prev => ({ ...prev, status: 'paused' }))
  }, [])

  // Resume processing
  const handleResume = useCallback(() => {
    setIsPaused(false)
    handleStartBatch()
  }, [handleStartBatch])

  // Retry failed files
  const handleRetryFailed = useCallback(() => {
    setOperation(prev => ({
      ...prev,
      files: prev.files.map(f =>
        f.status === 'error'
          ? { ...f, status: 'pending' as const, progress: 0, error: undefined }
          : f
      ),
      status: 'idle',
    }))
  }, [])

  // Calculate statistics
  const stats = useMemo(() => {
    const total = operation.files.length
    const completed = operation.files.filter(f => f.status === 'completed').length
    const failed = operation.files.filter(f => f.status === 'error').length
    const processing = operation.files.filter(f => f.status === 'processing').length
    const pending = operation.files.filter(f => f.status === 'pending').length

    return { total, completed, failed, processing, pending }
  }, [operation.files])

  // Format time
  const formatTime = useCallback((ms: number) => {
    if (ms < 1000) return `${ms}ms`
    if (ms < 60000) return `${Math.round(ms / 1000)}s`
    return `${Math.round(ms / 60000)}m ${Math.round((ms % 60000) / 1000)}s`
  }, [])

  // Get operation icon
  const getFileIcon = useCallback((type: string) => {
    switch (type.toLowerCase()) {
      case 'pdf':
        return 'FileText'
      case 'docx':
      case 'doc':
        return 'FileText'
      case 'txt':
      case 'md':
        return 'FileText'
      case 'html':
        return 'Code'
      case 'pptx':
        return 'Presentation'
      default:
        return 'File'
    }
  }, [])

  const getStatusColor = useCallback((status: BatchFile['status']) => {
    switch (status) {
      case 'completed':
        return designTokens.colors.confidence.high
      case 'error':
        return designTokens.colors.accent.alert
      case 'processing':
        return designTokens.colors.accent.ai
      default:
        return designTokens.colors.text.tertiary
    }
  }, [])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      padding: designTokens.spacing[6],
      maxWidth: '900px',
      margin: '0 auto',
      ...style,
    }),
    [style]
  )

  const headerStyles = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: designTokens.spacing[4],
  }

  const dropZoneStyles = {
    border: `2px dashed ${isDragging ? designTokens.colors.accent.ai : designTokens.colors.border.medium}`,
    borderRadius: designTokens.borderRadius.lg,
    padding: designTokens.spacing[8],
    textAlign: 'center' as const,
    backgroundColor: isDragging
      ? `${designTokens.colors.accent.ai}10`
      : designTokens.colors.surface.secondary,
    transition: `all ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut}`,
    cursor: 'pointer',
    marginBottom: designTokens.spacing[4],
  }

  const fileListStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[2],
    maxHeight: '400px',
    overflowY: 'auto' as const,
    marginBottom: designTokens.spacing[4],
  }

  const fileItemStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.primary,
    borderRadius: designTokens.borderRadius.sm,
    border: `1px solid ${designTokens.colors.border.medium}`,
  }

  const statsCardStyles = {
    flex: 1,
    padding: designTokens.spacing[3],
    textAlign: 'center' as const,
  }

  return (
    <div className={`fiovana-batch-manager ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={headerStyles}>
        <div>
          <h2
            style={{
              fontSize: designTokens.typography.fontSize['2xl'],
              fontWeight: designTokens.typography.fontWeight.bold,
              color: designTokens.colors.text.primary,
              marginBottom: designTokens.spacing[1],
            }}
          >
            Batch Operations
          </h2>
          <p
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
            }}
          >
            Process multiple files simultaneously
          </p>
        </div>
        {onClose && (
          <Button variant="ghost" size="sm" onClick={onClose}>
            <Icon name="X" size={16} />
          </Button>
        )}
      </div>

      {/* Operation Type Selection */}
      <Card style={{ padding: designTokens.spacing[4], marginBottom: designTokens.spacing[4] }}>
        <div
          style={{
            fontSize: designTokens.typography.fontSize.sm,
            fontWeight: designTokens.typography.fontWeight.medium,
            marginBottom: designTokens.spacing[3],
            color: designTokens.colors.text.primary,
          }}
        >
          Operation Type
        </div>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
            gap: designTokens.spacing[2],
          }}
        >
          {OPERATION_TYPES.map(opType => (
            <Button
              key={opType.value}
              variant={selectedOperationType === opType.value ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => {
                setSelectedOperationType(opType.value)
                setOperation(prev => ({ ...prev, type: opType.value as BatchOperation['type'] }))
              }}
              disabled={operation.status === 'running'}
              leftIcon={<Icon name={opType.icon as never} size={16} />}
              style={{ justifyContent: 'flex-start' }}
            >
              <div style={{ textAlign: 'left', flex: 1 }}>
                <div
                  style={{
                    fontSize: designTokens.typography.fontSize.xs,
                    fontWeight: designTokens.typography.fontWeight.medium,
                  }}
                >
                  {opType.label}
                </div>
              </div>
            </Button>
          ))}
        </div>
      </Card>

      {/* File Selection / Drop Zone */}
      {operation.files.length === 0 && (
        <div
          ref={dropZoneRef}
          style={dropZoneStyles}
          onDragEnter={handleDragEnter}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={handleSelectFiles}
        >
          <Icon
            name={isDragging ? 'Download' : 'Zap'}
            size={48}
            color={isDragging ? designTokens.colors.accent.ai : designTokens.colors.text.tertiary}
            style={{ marginBottom: designTokens.spacing[3] }}
          />
          <div
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: designTokens.colors.text.primary,
              marginBottom: designTokens.spacing[2],
            }}
          >
            {isDragging ? 'Drop files here' : 'Drag & drop files or click to browse'}
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.tertiary,
            }}
          >
            Supported: PDF, DOCX, TXT, MD, HTML, PPTX
          </div>
        </div>
      )}

      {/* File List */}
      {operation.files.length > 0 && (
        <>
          {/* Statistics */}
          <div
            style={{
              display: 'flex',
              gap: designTokens.spacing[3],
              marginBottom: designTokens.spacing[4],
            }}
          >
            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize['2xl'],
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {stats.total}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Total Files
              </div>
            </Card>

            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize['2xl'],
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.confidence.high,
                }}
              >
                {stats.completed}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Completed
              </div>
            </Card>

            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize['2xl'],
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.accent.alert,
                }}
              >
                {stats.failed}
              </div>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                }}
              >
                Failed
              </div>
            </Card>
          </div>

          {/* Overall Progress */}
          <Card style={{ padding: designTokens.spacing[4], marginBottom: designTokens.spacing[4] }}>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: designTokens.spacing[2],
              }}
            >
              <span
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  fontWeight: designTokens.typography.fontWeight.medium,
                  color: designTokens.colors.text.primary,
                }}
              >
                Overall Progress
              </span>
              <Badge
                variant={
                  operation.status === 'completed'
                    ? 'success'
                    : operation.status === 'error'
                      ? 'error'
                      : 'default'
                }
              >
                {operation.status}
              </Badge>
            </div>
            <Progress value={operation.totalProgress} variant="ai" animated showPercentage />
          </Card>

          {/* Files */}
          <div style={fileListStyles}>
            {operation.files.map(file => (
              <div key={file.id} style={fileItemStyles}>
                <Icon
                  name={getFileIcon(file.type) as never}
                  size={20}
                  color={getStatusColor(file.status)}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.sm,
                      fontWeight: designTokens.typography.fontWeight.medium,
                      color: designTokens.colors.text.primary,
                      marginBottom: designTokens.spacing[1],
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {file.name}
                  </div>
                  {file.status === 'processing' && (
                    <Progress value={file.progress} size="sm" animated variant="ai" />
                  )}
                  {file.error && (
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.accent.alert,
                      }}
                    >
                      Error: {file.error}
                    </div>
                  )}
                </div>
                <Badge variant="default" size="sm">
                  {file.status}
                </Badge>
                {file.status === 'pending' && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleRemoveFile(file.id)}
                    style={{ minWidth: 'auto', padding: designTokens.spacing[1] }}
                  >
                    <Icon name="X" size={14} />
                  </Button>
                )}
              </div>
            ))}
          </div>

          {/* Add More Files Button */}
          <Button
            variant="secondary"
            size="sm"
            onClick={handleSelectFiles}
            disabled={operation.status === 'running'}
            leftIcon={<Icon name="FilePlus" size={16} />}
            style={{ marginBottom: designTokens.spacing[4] }}
          >
            Add More Files
          </Button>
        </>
      )}

      {/* Actions */}
      <div style={{ display: 'flex', gap: designTokens.spacing[2], justifyContent: 'flex-end' }}>
        {operation.files.length > 0 && operation.status === 'idle' && (
          <>
            <Button variant="ghost" onClick={handleClearAll}>
              Clear All
            </Button>
            <Button
              variant="primary"
              onClick={handleStartBatch}
              leftIcon={<Icon name="ArrowRight" size={16} />}
            >
              Start Batch
            </Button>
          </>
        )}

        {operation.status === 'running' && (
          <Button
            variant="secondary"
            onClick={handlePause}
            leftIcon={<Icon name="Pulse" size={16} />}
          >
            Pause
          </Button>
        )}

        {operation.status === 'paused' && (
          <Button
            variant="primary"
            onClick={handleResume}
            leftIcon={<Icon name="ArrowRight" size={16} />}
          >
            Resume
          </Button>
        )}

        {operation.status === 'completed' && (
          <>
            {stats.failed > 0 && (
              <Button
                variant="secondary"
                onClick={handleRetryFailed}
                leftIcon={<Icon name="RefreshCcw" size={16} />}
              >
                Retry Failed
              </Button>
            )}
            <Button
              variant="primary"
              onClick={() => setShowResults(true)}
              leftIcon={<Icon name="TrendingUp" size={16} />}
            >
              View Results
            </Button>
          </>
        )}
      </div>

      {/* Results Modal */}
      <Modal
        isOpen={showResults}
        onClose={() => setShowResults(false)}
        title="Batch Operation Results"
        size="lg"
      >
        <div style={{ padding: designTokens.spacing[4] }}>
          <div
            style={{
              marginBottom: designTokens.spacing[4],
              display: 'flex',
              gap: designTokens.spacing[4],
            }}
          >
            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xl,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.confidence.high,
                }}
              >
                {results.filter(r => r.success).length}
              </div>
              <div style={{ fontSize: designTokens.typography.fontSize.sm }}>Successful</div>
            </Card>
            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xl,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.accent.alert,
                }}
              >
                {results.filter(r => !r.success).length}
              </div>
              <div style={{ fontSize: designTokens.typography.fontSize.sm }}>Failed</div>
            </Card>
            <Card style={statsCardStyles}>
              <div
                style={{
                  fontSize: designTokens.typography.fontSize.xl,
                  fontWeight: designTokens.typography.fontWeight.bold,
                  color: designTokens.colors.text.primary,
                }}
              >
                {formatTime(
                  results.reduce((sum, r) => sum + r.duration, 0) / (results.length || 1)
                )}
              </div>
              <div style={{ fontSize: designTokens.typography.fontSize.sm }}>Avg Duration</div>
            </Card>
          </div>

          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: designTokens.spacing[2],
              maxHeight: '400px',
              overflowY: 'auto',
            }}
          >
            {results.map((result, index) => (
              <Card key={index} style={{ padding: designTokens.spacing[3] }}>
                <div
                  style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}
                >
                  <Icon
                    name={result.success ? 'CheckCircle' : 'AlertCircle'}
                    size={20}
                    color={
                      result.success
                        ? designTokens.colors.confidence.high
                        : designTokens.colors.accent.alert
                    }
                  />
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.sm,
                        fontWeight: designTokens.typography.fontWeight.medium,
                        marginBottom: designTokens.spacing[1],
                      }}
                    >
                      {result.filePath.split('/').pop() || result.filePath}
                    </div>
                    {result.error && (
                      <div
                        style={{
                          fontSize: designTokens.typography.fontSize.xs,
                          color: designTokens.colors.accent.alert,
                        }}
                      >
                        {result.error}
                      </div>
                    )}
                    <div
                      style={{
                        fontSize: designTokens.typography.fontSize.xs,
                        color: designTokens.colors.text.tertiary,
                      }}
                    >
                      Duration: {formatTime(result.duration)}
                    </div>
                  </div>
                  <Badge variant={result.success ? 'success' : 'error'}>
                    {result.success ? 'Success' : 'Failed'}
                  </Badge>
                </div>
              </Card>
            ))}
          </div>

          <div
            style={{
              marginTop: designTokens.spacing[4],
              display: 'flex',
              justifyContent: 'flex-end',
              gap: designTokens.spacing[2],
            }}
          >
            <Button variant="ghost" onClick={() => setShowResults(false)}>
              Close
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}

export default React.memo(BatchManager)
