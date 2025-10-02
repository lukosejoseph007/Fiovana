import React, { useState, useEffect, useCallback } from 'react'
import { Card, Button, Icon, Badge, Tooltip } from '../ui'
import { designTokens } from '../../styles/tokens'
import { documentEditingService } from '../../services'
import type { VersionInfo } from '../../services/documentEditingService'
import VersionDiff from './VersionDiff'

interface VersionHistoryProps {
  documentId: string
  currentContent: string
  onClose: () => void
  onRestore?: (content: string) => void
}

const VersionHistory: React.FC<VersionHistoryProps> = ({
  documentId,
  currentContent: _currentContent,
  onClose,
  onRestore,
}) => {
  const [versions, setVersions] = useState<VersionInfo[]>([])
  const [selectedVersion, setSelectedVersion] = useState<VersionInfo | null>(null)
  const [compareVersion, setCompareVersion] = useState<VersionInfo | null>(null)
  const [versionContent, setVersionContent] = useState<string>('')
  const [compareContent, setCompareContent] = useState<string>('')
  const [isLoading, setIsLoading] = useState(true)
  const [isRestoring, setIsRestoring] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<'list' | 'view' | 'compare'>('list')

  // Load version history
  const loadVersions = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await documentEditingService.getDocumentVersions(documentId)

      if (response.success && response.data) {
        setVersions(response.data)
      } else {
        setError(response.error || 'Failed to load version history')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load versions')
    } finally {
      setIsLoading(false)
    }
  }, [documentId])

  // Load version content
  const loadVersionContent = useCallback(
    async (versionId: string) => {
      try {
        const response = await documentEditingService.restoreDocumentVersion(documentId, versionId)

        if (response.success && response.data) {
          return response.data
        } else {
          throw new Error(response.error || 'Failed to load version content')
        }
      } catch (err) {
        console.error('Failed to load version content:', err)
        return ''
      }
    },
    [documentId]
  )

  // Handle version selection for viewing
  const handleViewVersion = useCallback(
    async (version: VersionInfo) => {
      setSelectedVersion(version)
      setViewMode('view')
      const content = await loadVersionContent(version.versionId)
      setVersionContent(content)
    },
    [loadVersionContent]
  )

  // Handle version comparison
  const handleCompareVersion = useCallback(
    async (version: VersionInfo) => {
      if (!selectedVersion) {
        // First selection - set as base for comparison
        setSelectedVersion(version)
        const content = await loadVersionContent(version.versionId)
        setVersionContent(content)
      } else {
        // Second selection - compare with first
        setCompareVersion(version)
        setViewMode('compare')
        const content = await loadVersionContent(version.versionId)
        setCompareContent(content)
      }
    },
    [selectedVersion, loadVersionContent]
  )

  // Handle version restore
  const handleRestore = useCallback(
    async (version: VersionInfo) => {
      const confirmRestore = window.confirm(
        `Restore document to version from ${new Date(version.createdAt).toLocaleString()}?\n\nThis will replace the current content. The current version will be backed up.`
      )

      if (!confirmRestore) return

      setIsRestoring(true)

      try {
        const response = await documentEditingService.restoreDocumentVersion(
          documentId,
          version.versionId
        )

        if (response.success && response.data) {
          onRestore?.(response.data)
          onClose()
        } else {
          setError(response.error || 'Failed to restore version')
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to restore version')
      } finally {
        setIsRestoring(false)
      }
    },
    [documentId, onRestore, onClose]
  )

  // Format file size
  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  // Format relative time
  const formatRelativeTime = (dateString: string): string => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins} min${diffMins > 1 ? 's' : ''} ago`
    if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`
    if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`
    return date.toLocaleDateString()
  }

  useEffect(() => {
    loadVersions()
  }, [loadVersions])

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: 'rgba(0, 0, 0, 0.8)',
        backdropFilter: 'blur(8px)',
        zIndex: designTokens.zIndex.modal,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: designTokens.spacing[6],
      }}
      onClick={onClose}
    >
      <Card
        variant="elevated"
        style={{
          width: '90vw',
          maxWidth: viewMode === 'compare' ? '1400px' : '1000px',
          height: '85vh',
          display: 'flex',
          flexDirection: 'column',
          background: designTokens.colors.surface.primary,
          overflow: 'hidden',
        }}
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            padding: designTokens.spacing[6],
            borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[3] }}>
            <Icon name="Document" size={24} style={{ color: designTokens.colors.accent.ai }} />
            <div>
              <h2
                style={{
                  fontSize: designTokens.typography.fontSize.xl,
                  fontWeight: designTokens.typography.fontWeight.semibold,
                  color: designTokens.colors.text.primary,
                  margin: 0,
                }}
              >
                Version History
              </h2>
              <p
                style={{
                  fontSize: designTokens.typography.fontSize.sm,
                  color: designTokens.colors.text.secondary,
                  margin: `${designTokens.spacing[1]} 0 0 0`,
                }}
              >
                {versions.length} version{versions.length !== 1 ? 's' : ''} available
              </p>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: designTokens.spacing[2] }}>
            {viewMode !== 'list' && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setViewMode('list')
                  setSelectedVersion(null)
                  setCompareVersion(null)
                }}
              >
                <Icon name="ArrowRight" size={16} style={{ transform: 'rotate(180deg)' }} />
                Back to List
              </Button>
            )}

            <Tooltip content="Close">
              <Button variant="ghost" size="sm" onClick={onClose}>
                <Icon name="X" size={20} />
              </Button>
            </Tooltip>
          </div>
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflow: 'hidden', display: 'flex' }}>
          {viewMode === 'list' ? (
            // Version List View
            <div style={{ flex: 1, overflow: 'auto', padding: designTokens.spacing[6] }}>
              {isLoading ? (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    height: '100%',
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  <Icon name="Spinner" size={32} />
                  <span style={{ marginLeft: designTokens.spacing[3] }}>Loading versions...</span>
                </div>
              ) : error ? (
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    height: '100%',
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  <Icon name="X" size={48} style={{ color: designTokens.colors.accent.alert }} />
                  <p style={{ marginTop: designTokens.spacing[4] }}>{error}</p>
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={loadVersions}
                    style={{ marginTop: designTokens.spacing[4] }}
                  >
                    Retry
                  </Button>
                </div>
              ) : versions.length === 0 ? (
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    height: '100%',
                    color: designTokens.colors.text.secondary,
                  }}
                >
                  <Icon name="Document" size={48} />
                  <p style={{ marginTop: designTokens.spacing[4] }}>No versions available yet</p>
                  <p
                    style={{
                      fontSize: designTokens.typography.fontSize.sm,
                      marginTop: designTokens.spacing[2],
                    }}
                  >
                    Versions are created automatically when you save the document
                  </p>
                </div>
              ) : (
                <div
                  style={{ display: 'flex', flexDirection: 'column', gap: designTokens.spacing[3] }}
                >
                  {versions.map((version, index) => (
                    <Card
                      key={version.versionId}
                      variant="default"
                      style={{
                        padding: designTokens.spacing[4],
                        border: `1px solid ${designTokens.colors.border.subtle}`,
                        transition: 'all 0.2s ease',
                        cursor: 'pointer',
                      }}
                      onMouseEnter={e => {
                        e.currentTarget.style.borderColor = designTokens.colors.accent.ai
                        e.currentTarget.style.transform = 'translateY(-2px)'
                      }}
                      onMouseLeave={e => {
                        e.currentTarget.style.borderColor = designTokens.colors.border.subtle
                        e.currentTarget.style.transform = 'translateY(0)'
                      }}
                    >
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          gap: designTokens.spacing[4],
                        }}
                      >
                        {/* Version Info */}
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: designTokens.spacing[3],
                              marginBottom: designTokens.spacing[2],
                            }}
                          >
                            <Badge
                              variant={index === 0 ? 'success' : 'default'}
                              style={{ fontSize: designTokens.typography.fontSize.xs }}
                            >
                              {index === 0 ? 'Latest' : `v${versions.length - index}`}
                            </Badge>
                            <span
                              style={{
                                fontSize: designTokens.typography.fontSize.sm,
                                color: designTokens.colors.text.secondary,
                              }}
                            >
                              {formatRelativeTime(version.createdAt)}
                            </span>
                          </div>

                          <div
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: designTokens.spacing[4],
                              fontSize: designTokens.typography.fontSize.sm,
                            }}
                          >
                            <span style={{ color: designTokens.colors.text.secondary }}>
                              📅 {new Date(version.createdAt).toLocaleString()}
                            </span>
                            <span style={{ color: designTokens.colors.text.secondary }}>
                              📦 {formatSize(version.size)}
                            </span>
                            <Tooltip content={`Hash: ${version.hash}`}>
                              <span
                                style={{
                                  color: designTokens.colors.text.secondary,
                                  fontFamily: 'monospace',
                                }}
                              >
                                🔑 {version.hash.substring(0, 8)}...
                              </span>
                            </Tooltip>
                          </div>
                        </div>

                        {/* Actions */}
                        <div
                          style={{ display: 'flex', gap: designTokens.spacing[2], flexShrink: 0 }}
                        >
                          <Tooltip content="View this version">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleViewVersion(version)}
                            >
                              <Icon name="Document" size={16} />
                              View
                            </Button>
                          </Tooltip>

                          <Tooltip content="Compare with current">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleCompareVersion(version)}
                            >
                              <Icon name="Compare" size={16} />
                              Compare
                            </Button>
                          </Tooltip>

                          <Tooltip content="Restore this version">
                            <Button
                              variant="primary"
                              size="sm"
                              onClick={() => handleRestore(version)}
                              disabled={isRestoring}
                              style={{ minWidth: '90px' }}
                            >
                              <Icon name={isRestoring ? 'Spinner' : 'Generate'} size={16} />
                              {isRestoring ? 'Restoring...' : 'Restore'}
                            </Button>
                          </Tooltip>
                        </div>
                      </div>
                    </Card>
                  ))}
                </div>
              )}
            </div>
          ) : viewMode === 'view' && selectedVersion ? (
            // Single Version View
            <div style={{ flex: 1, overflow: 'auto', padding: designTokens.spacing[6] }}>
              <div style={{ marginBottom: designTokens.spacing[6] }}>
                <h3
                  style={{
                    fontSize: designTokens.typography.fontSize.lg,
                    fontWeight: designTokens.typography.fontWeight.semibold,
                    color: designTokens.colors.text.primary,
                    margin: 0,
                    marginBottom: designTokens.spacing[3],
                  }}
                >
                  Version from {new Date(selectedVersion.createdAt).toLocaleString()}
                </h3>
                <div
                  style={{
                    display: 'flex',
                    gap: designTokens.spacing[4],
                    fontSize: designTokens.typography.fontSize.sm,
                  }}
                >
                  <Badge variant="default">Size: {formatSize(selectedVersion.size)}</Badge>
                  <Badge variant="default">Hash: {selectedVersion.hash.substring(0, 16)}...</Badge>
                </div>
              </div>

              <Card variant="default" style={{ padding: designTokens.spacing[4] }}>
                <pre
                  style={{
                    whiteSpace: 'pre-wrap',
                    wordWrap: 'break-word',
                    fontFamily: designTokens.typography.fonts.mono.join(', '),
                    fontSize: designTokens.typography.fontSize.sm,
                    lineHeight: designTokens.typography.lineHeight.relaxed,
                    color: designTokens.colors.text.primary,
                    margin: 0,
                  }}
                >
                  {versionContent || 'Loading content...'}
                </pre>
              </Card>

              <div
                style={{
                  marginTop: designTokens.spacing[4],
                  display: 'flex',
                  gap: designTokens.spacing[3],
                }}
              >
                <Button
                  variant="primary"
                  onClick={() => handleRestore(selectedVersion)}
                  disabled={isRestoring}
                >
                  <Icon name={isRestoring ? 'Spinner' : 'Generate'} size={16} />
                  {isRestoring ? 'Restoring...' : 'Restore This Version'}
                </Button>
              </div>
            </div>
          ) : viewMode === 'compare' && selectedVersion && compareVersion ? (
            // Comparison View
            <VersionDiff
              oldVersion={{
                content: versionContent,
                label: `Version from ${new Date(selectedVersion.createdAt).toLocaleString()}`,
                metadata: selectedVersion,
              }}
              newVersion={{
                content: compareContent,
                label: `Version from ${new Date(compareVersion.createdAt).toLocaleString()}`,
                metadata: compareVersion,
              }}
            />
          ) : null}
        </div>
      </Card>
    </div>
  )
}

export default VersionHistory
