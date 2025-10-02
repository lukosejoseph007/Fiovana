import React, { useMemo } from 'react'
import { Card, Badge } from '../ui'
import { designTokens } from '../../styles/tokens'
import type { VersionInfo } from '../../services/documentEditingService'

interface VersionData {
  content: string
  label: string
  metadata: VersionInfo
}

interface VersionDiffProps {
  oldVersion: VersionData
  newVersion: VersionData
}

interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  content: string
  oldLineNumber?: number
  newLineNumber?: number
}

const VersionDiff: React.FC<VersionDiffProps> = ({ oldVersion, newVersion }) => {
  // Simple line-by-line diff algorithm
  const diffLines = useMemo((): DiffLine[] => {
    const oldLines = oldVersion.content.split('\n')
    const newLines = newVersion.content.split('\n')

    const result: DiffLine[] = []
    let oldIndex = 0
    let newIndex = 0

    while (oldIndex < oldLines.length || newIndex < newLines.length) {
      const oldLine = oldLines[oldIndex]
      const newLine = newLines[newIndex]

      if (oldIndex >= oldLines.length) {
        // Only new lines remaining
        result.push({
          type: 'added',
          content: newLine || '',
          newLineNumber: newIndex + 1,
        })
        newIndex++
      } else if (newIndex >= newLines.length) {
        // Only old lines remaining
        result.push({
          type: 'removed',
          content: oldLine || '',
          oldLineNumber: oldIndex + 1,
        })
        oldIndex++
      } else if (oldLine === newLine) {
        // Lines are the same
        result.push({
          type: 'unchanged',
          content: oldLine || '',
          oldLineNumber: oldIndex + 1,
          newLineNumber: newIndex + 1,
        })
        oldIndex++
        newIndex++
      } else {
        // Lines are different - check if line was modified or added/removed
        const nextOldLine = oldLines[oldIndex + 1]
        const nextNewLine = newLines[newIndex + 1]

        if (nextNewLine === oldLine) {
          // Line was added in new version
          result.push({
            type: 'added',
            content: newLine || '',
            newLineNumber: newIndex + 1,
          })
          newIndex++
        } else if (nextOldLine === newLine) {
          // Line was removed from old version
          result.push({
            type: 'removed',
            content: oldLine || '',
            oldLineNumber: oldIndex + 1,
          })
          oldIndex++
        } else {
          // Lines were modified - show both as removed and added
          result.push({
            type: 'removed',
            content: oldLine || '',
            oldLineNumber: oldIndex + 1,
          })
          result.push({
            type: 'added',
            content: newLine || '',
            newLineNumber: newIndex + 1,
          })
          oldIndex++
          newIndex++
        }
      }
    }

    return result
  }, [oldVersion.content, newVersion.content])

  // Calculate statistics
  const stats = useMemo(() => {
    const added = diffLines.filter(line => line.type === 'added').length
    const removed = diffLines.filter(line => line.type === 'removed').length
    const unchanged = diffLines.filter(line => line.type === 'unchanged').length

    return { added, removed, unchanged, total: diffLines.length }
  }, [diffLines])

  const getLineStyle = (type: DiffLine['type']) => {
    switch (type) {
      case 'added':
        return {
          background: 'rgba(64, 192, 87, 0.1)',
          borderLeft: `3px solid ${designTokens.colors.accent.success}`,
        }
      case 'removed':
        return {
          background: 'rgba(250, 82, 82, 0.1)',
          borderLeft: `3px solid ${designTokens.colors.accent.alert}`,
        }
      default:
        return {
          background: 'transparent',
          borderLeft: '3px solid transparent',
        }
    }
  }

  const getLinePrefix = (type: DiffLine['type']) => {
    switch (type) {
      case 'added':
        return '+ '
      case 'removed':
        return '- '
      default:
        return '  '
    }
  }

  return (
    <div
      style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Diff Header */}
      <div
        style={{
          padding: designTokens.spacing[4],
          borderBottom: `1px solid ${designTokens.colors.border.subtle}`,
          background: designTokens.colors.surface.secondary,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: designTokens.spacing[3],
          }}
        >
          <h3
            style={{
              fontSize: designTokens.typography.fontSize.lg,
              fontWeight: designTokens.typography.fontWeight.semibold,
              color: designTokens.colors.text.primary,
              margin: 0,
            }}
          >
            Version Comparison
          </h3>

          <div style={{ display: 'flex', gap: designTokens.spacing[2] }}>
            <Badge variant="success" style={{ fontSize: designTokens.typography.fontSize.sm }}>
              +{stats.added} added
            </Badge>
            <Badge variant="error" style={{ fontSize: designTokens.typography.fontSize.sm }}>
              -{stats.removed} removed
            </Badge>
            <Badge variant="default" style={{ fontSize: designTokens.typography.fontSize.sm }}>
              {stats.unchanged} unchanged
            </Badge>
          </div>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: designTokens.spacing[4],
            fontSize: designTokens.typography.fontSize.sm,
          }}
        >
          <div>
            <div
              style={{
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[1],
              }}
            >
              Older Version
            </div>
            <div style={{ color: designTokens.colors.text.primary, fontWeight: 600 }}>
              {oldVersion.label}
            </div>
            <div
              style={{
                color: designTokens.colors.text.secondary,
                fontSize: designTokens.typography.fontSize.xs,
                marginTop: designTokens.spacing[1],
              }}
            >
              Size: {(oldVersion.metadata.size / 1024).toFixed(1)} KB • Hash:{' '}
              {oldVersion.metadata.hash.substring(0, 8)}
            </div>
          </div>

          <div>
            <div
              style={{
                color: designTokens.colors.text.secondary,
                marginBottom: designTokens.spacing[1],
              }}
            >
              Newer Version
            </div>
            <div style={{ color: designTokens.colors.text.primary, fontWeight: 600 }}>
              {newVersion.label}
            </div>
            <div
              style={{
                color: designTokens.colors.text.secondary,
                fontSize: designTokens.typography.fontSize.xs,
                marginTop: designTokens.spacing[1],
              }}
            >
              Size: {(newVersion.metadata.size / 1024).toFixed(1)} KB • Hash:{' '}
              {newVersion.metadata.hash.substring(0, 8)}
            </div>
          </div>
        </div>
      </div>

      {/* Unified Diff View */}
      <div
        style={{
          flex: 1,
          overflow: 'auto',
          padding: designTokens.spacing[4],
        }}
      >
        <Card
          variant="default"
          style={{
            padding: 0,
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              fontFamily: designTokens.typography.fonts.mono.join(', '),
              fontSize: designTokens.typography.fontSize.sm,
              lineHeight: '1.6',
            }}
          >
            {diffLines.map((line, index) => (
              <div
                key={index}
                style={{
                  display: 'flex',
                  ...getLineStyle(line.type),
                  padding: `${designTokens.spacing[1]} ${designTokens.spacing[3]}`,
                  transition: 'background 0.15s ease',
                }}
                onMouseEnter={e => {
                  if (line.type !== 'unchanged') {
                    e.currentTarget.style.background =
                      line.type === 'added' ? 'rgba(64, 192, 87, 0.2)' : 'rgba(250, 82, 82, 0.2)'
                  }
                }}
                onMouseLeave={e => {
                  const style = getLineStyle(line.type)
                  e.currentTarget.style.background = style.background
                }}
              >
                {/* Line Numbers */}
                <div
                  style={{
                    minWidth: '80px',
                    display: 'flex',
                    gap: designTokens.spacing[2],
                    color: designTokens.colors.text.secondary,
                    userSelect: 'none',
                    flexShrink: 0,
                  }}
                >
                  <span style={{ width: '35px', textAlign: 'right' }}>
                    {line.oldLineNumber || ''}
                  </span>
                  <span style={{ width: '35px', textAlign: 'right' }}>
                    {line.newLineNumber || ''}
                  </span>
                </div>

                {/* Line Content */}
                <div
                  style={{
                    flex: 1,
                    whiteSpace: 'pre-wrap',
                    wordWrap: 'break-word',
                    color:
                      line.type === 'added'
                        ? designTokens.colors.accent.success
                        : line.type === 'removed'
                          ? designTokens.colors.accent.alert
                          : designTokens.colors.text.primary,
                  }}
                >
                  <span
                    style={{
                      color: designTokens.colors.text.secondary,
                      marginRight: designTokens.spacing[2],
                      userSelect: 'none',
                    }}
                  >
                    {getLinePrefix(line.type)}
                  </span>
                  {line.content || ' '}
                </div>
              </div>
            ))}
          </div>

          {/* Empty state */}
          {diffLines.length === 0 && (
            <div
              style={{
                padding: designTokens.spacing[8],
                textAlign: 'center',
                color: designTokens.colors.text.secondary,
              }}
            >
              <p>No differences found between versions</p>
            </div>
          )}
        </Card>
      </div>
    </div>
  )
}

export default VersionDiff
