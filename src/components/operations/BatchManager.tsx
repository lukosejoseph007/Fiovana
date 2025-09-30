import React, { useState, useCallback, useMemo, useEffect } from 'react'
import { designTokens } from '../../styles/tokens'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Badge from '../ui/Badge'
import Icon from '../ui/Icon'
import Progress from '../ui/Progress'
import Modal from '../ui/Modal'
import { documentGenerationService } from '../../services/documentGenerationService'
import { formatConversionService } from '../../services/formatConversionService'
import { documentService } from '../../services/documentService'

export interface BatchManagerProps {
  className?: string
  style?: React.CSSProperties
  onClose?: () => void
  onBatchComplete?: (results: BatchOperationResult[]) => void
}

export interface BatchOperation {
  id: string
  type: 'generate' | 'convert' | 'analyze' | 'compare'
  status: 'pending' | 'running' | 'completed' | 'error' | 'cancelled'
  progress: number
  documentId?: string
  documentName?: string
  parameters: Record<string, unknown>
  result?: unknown
  error?: string
  createdAt: Date
  startedAt?: Date
  completedAt?: Date
  estimatedDuration?: number
  actualDuration?: number
}

export interface BatchOperationResult {
  operation: BatchOperation
  success: boolean
  result?: unknown
  error?: string
  duration: number
}

export interface BatchQueue {
  id: string
  name: string
  operations: BatchOperation[]
  status: 'idle' | 'running' | 'paused' | 'completed' | 'error'
  totalOperations: number
  completedOperations: number
  failedOperations: number
  progress: number
  createdAt: Date
  startedAt?: Date
  completedAt?: Date
}

const BatchManager: React.FC<BatchManagerProps> = ({
  className = '',
  style,
  onClose,
  onBatchComplete,
}) => {
  const [queue, setQueue] = useState<BatchQueue>({
    id: `batch-${Date.now()}`,
    name: 'Batch Operation',
    operations: [],
    status: 'idle',
    totalOperations: 0,
    completedOperations: 0,
    failedOperations: 0,
    progress: 0,
    createdAt: new Date(),
  })

  const [showResults, setShowResults] = useState(false)
  const [results, setResults] = useState<BatchOperationResult[]>([])
  const [retryEnabled, setRetryEnabled] = useState(true)
  const [autoDownload, setAutoDownload] = useState(false)

  // Add operation to queue
  const addOperation = useCallback((operation: Omit<BatchOperation, 'id' | 'createdAt'>) => {
    const newOperation: BatchOperation = {
      ...operation,
      id: `op-${Date.now()}-${Math.random().toString(36).substring(7)}`,
      createdAt: new Date(),
    }

    setQueue(prev => ({
      ...prev,
      operations: [...prev.operations, newOperation],
      totalOperations: prev.totalOperations + 1,
    }))

    return newOperation.id
  }, [])

  // Remove operation from queue (future use for manual queue management)
  // const removeOperation = useCallback((operationId: string) => {
  //   setQueue(prev => ({
  //     ...prev,
  //     operations: prev.operations.filter(op => op.id !== operationId),
  //     totalOperations: prev.totalOperations - 1,
  //   }))
  // }, [])

  // Cancel operation
  const cancelOperation = useCallback((operationId: string) => {
    setQueue(prev => ({
      ...prev,
      operations: prev.operations.map(op =>
        op.id === operationId ? { ...op, status: 'cancelled' as const } : op
      ),
    }))
  }, [])

  // Retry failed operations
  const retryFailedOperations = useCallback(() => {
    setQueue(prev => ({
      ...prev,
      operations: prev.operations.map(op =>
        op.status === 'error' ? { ...op, status: 'pending' as const, error: undefined } : op
      ),
      failedOperations: 0,
    }))
  }, [])

  // Clear completed operations
  const clearCompletedOperations = useCallback(() => {
    setQueue(prev => ({
      ...prev,
      operations: prev.operations.filter(op => op.status !== 'completed'),
      completedOperations: 0,
    }))
  }, [])

  // Execute single operation
  const executeOperation = useCallback(
    async (operation: BatchOperation): Promise<BatchOperationResult> => {
      const startTime = Date.now()

      try {
        // Update operation status to running in queue
        setQueue(prev => ({
          ...prev,
          operations: prev.operations.map(op =>
            op.id === operation.id
              ? { ...op, status: 'running' as const, startedAt: new Date() }
              : op
          ),
        }))

        let result: unknown = null

        switch (operation.type) {
          case 'generate':
            if (operation.parameters.templateId) {
              const generateResult = await documentGenerationService.generateFromTemplate(
                operation.parameters.templateId as string,
                operation.parameters
              )
              result = generateResult.data
            } else if (operation.parameters.prompt) {
              const generateResult = await documentGenerationService.generateFromPrompt(
                operation.parameters.prompt as string,
                operation.parameters.format as string,
                operation.parameters
              )
              result = generateResult.data
            }
            break

          case 'convert':
            if (operation.documentId && operation.parameters.targetFormat) {
              const convertResult = await formatConversionService.convertDocument(
                operation.documentId,
                operation.parameters.targetFormat as string,
                {
                  preserveFormatting: (operation.parameters.preserveFormatting as boolean) ?? true,
                  includeImages: (operation.parameters.includeImages as boolean) ?? true,
                  customMapping: operation.parameters.customMapping as
                    | Record<string, string>
                    | undefined,
                  quality:
                    (operation.parameters.quality as 'low' | 'medium' | 'high' | undefined) ??
                    'high',
                }
              )
              result = convertResult.data
            }
            break

          case 'analyze':
            if (operation.documentId) {
              const analyzeResult = await documentService.analyzeDocument(operation.documentId)
              result = analyzeResult.data
            }
            break

          case 'compare':
            if (operation.parameters.baselineId && operation.parameters.updatedId) {
              const compareResult = await documentService.compareDocuments(
                operation.parameters.baselineId as string,
                operation.parameters.updatedId as string
              )
              result = compareResult.data
            }
            break

          default:
            throw new Error(`Unknown operation type: ${operation.type}`)
        }

        const duration = Date.now() - startTime

        // Auto-download if enabled and result is a document
        if (autoDownload && result && typeof result === 'object' && 'documentId' in result) {
          // Trigger download (implementation depends on backend API)
          console.log('Auto-downloading result:', result)
        }

        return {
          operation,
          success: true,
          result,
          duration,
        }
      } catch (error) {
        const duration = Date.now() - startTime

        return {
          operation,
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error',
          duration,
        }
      }
    },
    [autoDownload]
  )

  // Process queue
  const processQueue = useCallback(async () => {
    if (queue.status === 'running') return

    setQueue(prev => ({ ...prev, status: 'running', startedAt: new Date() }))

    const pendingOperations = queue.operations.filter(op => op.status === 'pending')
    const operationResults: BatchOperationResult[] = []

    for (const operation of pendingOperations) {
      // Check if queue was paused or cancelled
      if (queue.status === 'paused') {
        break
      }

      const result = await executeOperation(operation)
      operationResults.push(result)

      setQueue(prev => {
        const updatedOperations = prev.operations.map(op =>
          op.id === operation.id
            ? {
                ...op,
                status: result.success ? ('completed' as const) : ('error' as const),
                progress: 100,
                result: result.result,
                error: result.error,
                completedAt: new Date(),
                actualDuration: result.duration,
              }
            : op
        )

        const completed = updatedOperations.filter(op => op.status === 'completed').length
        const failed = updatedOperations.filter(op => op.status === 'error').length
        const total = updatedOperations.length

        return {
          ...prev,
          operations: updatedOperations,
          completedOperations: completed,
          failedOperations: failed,
          progress: total > 0 ? ((completed + failed) / total) * 100 : 0,
        }
      })

      // Retry logic
      if (!result.success && retryEnabled && operation.parameters.retryCount !== undefined) {
        const retryCount = (operation.parameters.retryCount as number) || 0
        if (retryCount < 3) {
          // Retry up to 3 times
          console.log(`Retrying operation ${operation.id} (attempt ${retryCount + 1}/3)`)
          addOperation({
            ...operation,
            status: 'pending',
            parameters: { ...operation.parameters, retryCount: retryCount + 1 },
          })
        }
      }
    }

    setResults(prev => [...prev, ...operationResults])

    setQueue(prev => ({
      ...prev,
      status: 'completed',
      completedAt: new Date(),
      progress: 100,
    }))

    setShowResults(true)
    onBatchComplete?.(operationResults)
  }, [queue, executeOperation, retryEnabled, addOperation, onBatchComplete])

  // Pause queue processing
  const pauseQueue = useCallback(() => {
    setQueue(prev => ({ ...prev, status: 'paused' }))
  }, [])

  // Resume queue processing
  const resumeQueue = useCallback(() => {
    setQueue(prev => ({ ...prev, status: 'running' }))
    processQueue()
  }, [processQueue])

  // Cancel all operations
  const cancelAllOperations = useCallback(() => {
    setQueue(prev => ({
      ...prev,
      operations: prev.operations.map(op =>
        op.status === 'pending' || op.status === 'running'
          ? { ...op, status: 'cancelled' as const }
          : op
      ),
      status: 'idle',
    }))
  }, [])

  // Calculate estimated time remaining
  const estimatedTimeRemaining = useMemo(() => {
    const pendingOperations = queue.operations.filter(op => op.status === 'pending')
    const avgDuration =
      queue.operations
        .filter(op => op.actualDuration)
        .reduce((sum, op) => sum + (op.actualDuration || 0), 0) /
        queue.operations.filter(op => op.actualDuration).length || 30000 // Default 30s

    return pendingOperations.length * avgDuration
  }, [queue.operations])

  // Format time
  const formatTime = useCallback((ms: number) => {
    if (ms < 1000) return `${ms}ms`
    if (ms < 60000) return `${Math.round(ms / 1000)}s`
    return `${Math.round(ms / 60000)}m ${Math.round((ms % 60000) / 1000)}s`
  }, [])

  // Auto-start processing when operations are added
  useEffect(() => {
    if (queue.operations.some(op => op.status === 'pending') && queue.status === 'idle') {
      processQueue()
    }
  }, [queue.operations, queue.status, processQueue])

  // Memoized styles
  const containerStyles = useMemo(
    () => ({
      padding: designTokens.spacing[6],
      maxWidth: '800px',
      margin: '0 auto',
      ...style,
    }),
    [style]
  )

  const queueHeaderStyles = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: designTokens.spacing[4],
  }

  const queueStatsStyles = {
    display: 'flex',
    gap: designTokens.spacing[4],
    marginBottom: designTokens.spacing[4],
  }

  const statCardStyles = {
    flex: 1,
    padding: designTokens.spacing[3],
    textAlign: 'center' as const,
  }

  const operationListStyles = {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: designTokens.spacing[2],
    maxHeight: '400px',
    overflowY: 'auto' as const,
    padding: designTokens.spacing[2],
    backgroundColor: designTokens.colors.surface.secondary,
    borderRadius: designTokens.borderRadius.md,
  }

  const operationItemStyles = {
    display: 'flex',
    alignItems: 'center',
    gap: designTokens.spacing[3],
    padding: designTokens.spacing[3],
    backgroundColor: designTokens.colors.surface.primary,
    borderRadius: designTokens.borderRadius.sm,
    border: `1px solid ${designTokens.colors.border.medium}`,
  }

  const actionsStyles = {
    display: 'flex',
    gap: designTokens.spacing[2],
    marginTop: designTokens.spacing[4],
    justifyContent: 'flex-end',
  }

  const getOperationIcon = useCallback((type: BatchOperation['type']) => {
    switch (type) {
      case 'generate':
        return 'file-plus'
      case 'convert':
        return 'refresh-cw'
      case 'analyze':
        return 'search'
      case 'compare':
        return 'git-compare'
      default:
        return 'file'
    }
  }, [])

  const getStatusColor = useCallback((status: BatchOperation['status']) => {
    switch (status) {
      case 'completed':
        return designTokens.colors.confidence.high
      case 'error':
        return designTokens.colors.accent.alert
      case 'running':
        return designTokens.colors.accent.ai
      case 'cancelled':
        return designTokens.colors.text.tertiary
      default:
        return designTokens.colors.text.secondary
    }
  }, [])

  return (
    <div className={`proxemic-batch-manager ${className}`} style={containerStyles}>
      {/* Header */}
      <div style={queueHeaderStyles}>
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
            Manage multiple document operations in a single queue
          </p>
        </div>
        {onClose && (
          <Button variant="ghost" size="sm" onClick={onClose}>
            <Icon name="X" size={16} />
          </Button>
        )}
      </div>

      {/* Queue Stats */}
      <div style={queueStatsStyles}>
        <Card style={statCardStyles}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize['2xl'],
              fontWeight: designTokens.typography.fontWeight.bold,
              color: designTokens.colors.text.primary,
            }}
          >
            {queue.totalOperations}
          </div>
          <div
            style={{
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.secondary,
            }}
          >
            Total Operations
          </div>
        </Card>

        <Card style={statCardStyles}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize['2xl'],
              fontWeight: designTokens.typography.fontWeight.bold,
              color: designTokens.colors.confidence.high,
            }}
          >
            {queue.completedOperations}
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

        <Card style={statCardStyles}>
          <div
            style={{
              fontSize: designTokens.typography.fontSize['2xl'],
              fontWeight: designTokens.typography.fontWeight.bold,
              color: designTokens.colors.accent.alert,
            }}
          >
            {queue.failedOperations}
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
          <Badge variant={queue.status === 'completed' ? 'success' : 'default'}>
            {queue.status}
          </Badge>
        </div>
        <Progress value={queue.progress} variant="ai" animated showPercentage />
        {queue.status === 'running' && estimatedTimeRemaining > 0 && (
          <div
            style={{
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
              marginTop: designTokens.spacing[2],
            }}
          >
            Est. time remaining: {formatTime(estimatedTimeRemaining)}
          </div>
        )}
      </Card>

      {/* Operations List */}
      <div style={operationListStyles}>
        {queue.operations.length === 0 ? (
          <div
            style={{
              textAlign: 'center',
              padding: designTokens.spacing[6],
              color: designTokens.colors.text.tertiary,
            }}
          >
            <Icon
              name="FileText"
              size={48}
              color={designTokens.colors.text.tertiary}
              style={{ marginBottom: designTokens.spacing[2] }}
            />
            <div>No operations in queue</div>
          </div>
        ) : (
          queue.operations.map(operation => (
            <div key={operation.id} style={operationItemStyles}>
              <Icon
                name={getOperationIcon(operation.type) as never}
                size={20}
                color={getStatusColor(operation.status)}
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
                  {operation.documentName || operation.type}
                </div>
                {operation.status === 'running' && (
                  <Progress value={operation.progress} size="sm" animated variant="ai" />
                )}
                {operation.error && (
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      color: designTokens.colors.accent.alert,
                    }}
                  >
                    Error: {operation.error}
                  </div>
                )}
                {operation.actualDuration && (
                  <div
                    style={{
                      fontSize: designTokens.typography.fontSize.xs,
                      color: designTokens.colors.text.tertiary,
                    }}
                  >
                    Duration: {formatTime(operation.actualDuration)}
                  </div>
                )}
              </div>
              <Badge variant="default" size="sm">
                {operation.status}
              </Badge>
              {(operation.status === 'pending' || operation.status === 'running') && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => cancelOperation(operation.id)}
                  style={{ minWidth: 'auto', padding: designTokens.spacing[1] }}
                >
                  <Icon name="X" size={14} />
                </Button>
              )}
            </div>
          ))
        )}
      </div>

      {/* Options */}
      <Card style={{ padding: designTokens.spacing[3], marginTop: designTokens.spacing[4] }}>
        <div
          style={{
            display: 'flex',
            gap: designTokens.spacing[4],
            alignItems: 'center',
          }}
        >
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.primary,
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={retryEnabled}
              onChange={e => setRetryEnabled(e.target.checked)}
              style={{ cursor: 'pointer' }}
            />
            Auto-retry failed operations
          </label>
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              fontSize: designTokens.typography.fontSize.sm,
              color: designTokens.colors.text.primary,
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={autoDownload}
              onChange={e => setAutoDownload(e.target.checked)}
              style={{ cursor: 'pointer' }}
            />
            Auto-download completed results
          </label>
        </div>
      </Card>

      {/* Actions */}
      <div style={actionsStyles}>
        {queue.failedOperations > 0 && (
          <Button variant="secondary" onClick={retryFailedOperations}>
            <Icon
              name="RefreshCcw"
              size={16}
              style={{ marginRight: designTokens.spacing[2], display: 'inline-block' }}
            />
            Retry Failed
          </Button>
        )}
        {queue.completedOperations > 0 && (
          <Button variant="ghost" onClick={clearCompletedOperations}>
            Clear Completed
          </Button>
        )}
        {queue.status === 'running' && (
          <Button variant="secondary" onClick={pauseQueue}>
            <Icon
              name="AlertCircle"
              size={16}
              style={{ marginRight: designTokens.spacing[2], display: 'inline-block' }}
            />
            Pause
          </Button>
        )}
        {queue.status === 'paused' && (
          <Button variant="primary" onClick={resumeQueue}>
            <Icon
              name="ArrowRight"
              size={16}
              style={{ marginRight: designTokens.spacing[2], display: 'inline-block' }}
            />
            Resume
          </Button>
        )}
        {(queue.status === 'running' || queue.status === 'paused') && (
          <Button variant="ghost" onClick={cancelAllOperations}>
            Cancel All
          </Button>
        )}
        {queue.status === 'completed' && (
          <Button variant="primary" onClick={() => setShowResults(true)}>
            <Icon
              name="TrendingUp"
              size={16}
              style={{ marginRight: designTokens.spacing[2], display: 'inline-block' }}
            />
            View Results
          </Button>
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
            <Card style={statCardStyles}>
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
            <Card style={statCardStyles}>
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
            <Card style={statCardStyles}>
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
                    name={result.success ? 'Health' : 'AlertCircle'}
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
                      {result.operation.documentName || result.operation.type}
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
            <Button variant="primary" onClick={() => console.log('Download results:', results)}>
              <Icon
                name="Share2"
                size={16}
                style={{ marginRight: designTokens.spacing[2], display: 'inline-block' }}
              />
              Download Summary
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}

export default React.memo(BatchManager)
