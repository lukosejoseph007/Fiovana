import React from 'react'
import { Card, Badge, Icon } from '../ui'
import { designTokens } from '../../styles/tokens'

export interface DocumentCardProps {
  id: string
  name: string
  type: string
  size?: number
  modified?: Date
  status?: 'processing' | 'ready' | 'error'
  confidence?: number
  thumbnail?: string
  onClick?: () => void
  onAction?: (action: string) => void
}

const DocumentCard: React.FC<DocumentCardProps> = ({
  id: _id,
  name,
  type,
  size,
  modified,
  status = 'ready',
  confidence,
  thumbnail,
  onClick,
  onAction,
}) => {
  const formatSize = (bytes?: number) => {
    if (!bytes) return 'Unknown size'
    const kb = bytes / 1024
    const mb = kb / 1024
    if (mb >= 1) return `${mb.toFixed(1)} MB`
    return `${kb.toFixed(1)} KB`
  }

  const formatDate = (date?: Date) => {
    if (!date) return 'Unknown date'
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const hours = diff / (1000 * 60 * 60)
    const days = hours / 24

    if (hours < 1) return 'Just now'
    if (hours < 24) return `${Math.floor(hours)}h ago`
    if (days < 7) return `${Math.floor(days)}d ago`
    return date.toLocaleDateString()
  }

  const getStatusColor = () => {
    switch (status) {
      case 'processing':
        return designTokens.colors.accent.ai
      case 'ready':
        return designTokens.colors.confidence.high
      case 'error':
        return designTokens.colors.accent.alert
      default:
        return designTokens.colors.text.tertiary
    }
  }

  const getStatusIcon = () => {
    switch (status) {
      case 'processing':
        return 'Loader'
      case 'ready':
        return 'Check'
      case 'error':
        return 'AlertCircle'
      default:
        return 'Document'
    }
  }

  const getTypeIcon = () => {
    const typeMap: Record<string, string> = {
      pdf: 'FileText',
      doc: 'FileText',
      docx: 'FileText',
      txt: 'FileText',
      md: 'FileText',
      html: 'Code',
      json: 'Code',
      xml: 'Code',
    }
    return typeMap[type.toLowerCase()] || 'Document'
  }

  return (
    <Card
      variant="glass"
      className="document-card"
      style={{
        cursor: onClick ? 'pointer' : 'default',
        transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
        position: 'relative',
        overflow: 'hidden',
      }}
      onClick={onClick}
    >
      <style>
        {`
          .document-card {
            border: 1px solid ${designTokens.colors.border.subtle};
          }

          .document-card:hover {
            border-color: ${designTokens.colors.accent.ai};
            box-shadow: ${designTokens.shadows.lg};
            transform: translateY(-2px);
          }

          .document-card .quick-actions {
            opacity: 0;
            transition: opacity ${designTokens.animation.duration.fast} ${designTokens.animation.easing.easeOut};
          }

          .document-card:hover .quick-actions {
            opacity: 1;
          }

          .document-thumbnail {
            width: 100%;
            height: 120px;
            background: linear-gradient(135deg, ${designTokens.colors.surface.tertiary}, ${designTokens.colors.surface.secondary});
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: ${designTokens.borderRadius.md} ${designTokens.borderRadius.md} 0 0;
            margin: -${designTokens.spacing[4]} -${designTokens.spacing[4]} ${designTokens.spacing[3]} -${designTokens.spacing[4]};
            position: relative;
          }

          .document-thumbnail img {
            width: 100%;
            height: 100%;
            object-fit: cover;
          }
        `}
      </style>

      {/* Thumbnail/Preview */}
      <div className="document-thumbnail">
        {thumbnail ? (
          <img src={thumbnail} alt={name} />
        ) : (
          <Icon name={getTypeIcon() as never} size={48} color={designTokens.colors.text.tertiary} />
        )}

        {/* Status Indicator */}
        <div
          style={{
            position: 'absolute',
            top: designTokens.spacing[2],
            right: designTokens.spacing[2],
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[1],
            padding: `${designTokens.spacing[1]} ${designTokens.spacing[2]}`,
            background: `${designTokens.colors.surface.primary}E6`,
            backdropFilter: 'blur(8px)',
            borderRadius: designTokens.borderRadius.full,
          }}
        >
          <Icon
            name={getStatusIcon() as never}
            size={12}
            color={getStatusColor()}
            style={{
              animation: status === 'processing' ? 'spin 1s linear infinite' : 'none',
            }}
          />
          <span
            style={{
              fontSize: designTokens.typography.fontSize.xs,
              fontWeight: designTokens.typography.fontWeight.medium,
              color: getStatusColor(),
              textTransform: 'capitalize',
            }}
          >
            {status}
          </span>
        </div>

        {/* Quick Actions */}
        <div
          className="quick-actions"
          style={{
            position: 'absolute',
            bottom: designTokens.spacing[2],
            right: designTokens.spacing[2],
            display: 'flex',
            gap: designTokens.spacing[1],
          }}
          onClick={e => e.stopPropagation()}
        >
          <button
            onClick={() => onAction?.('preview')}
            style={{
              padding: designTokens.spacing[1.5],
              background: `${designTokens.colors.surface.primary}E6`,
              backdropFilter: 'blur(8px)',
              border: 'none',
              borderRadius: designTokens.borderRadius.md,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: designTokens.colors.text.secondary,
              transition: `all ${designTokens.animation.duration.fast}`,
            }}
            title="Preview"
          >
            <Icon name="Eye" size={14} />
          </button>
          <button
            onClick={() => onAction?.('analyze')}
            style={{
              padding: designTokens.spacing[1.5],
              background: `${designTokens.colors.surface.primary}E6`,
              backdropFilter: 'blur(8px)',
              border: 'none',
              borderRadius: designTokens.borderRadius.md,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: designTokens.colors.text.secondary,
              transition: `all ${designTokens.animation.duration.fast}`,
            }}
            title="Analyze"
          >
            <Icon name="Search" size={14} />
          </button>
        </div>
      </div>

      {/* Document Info */}
      <div>
        <h4
          style={{
            fontSize: designTokens.typography.fontSize.base,
            fontWeight: designTokens.typography.fontWeight.semibold,
            color: designTokens.colors.text.primary,
            marginBottom: designTokens.spacing[2],
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={name}
        >
          {name}
        </h4>

        {/* Metadata */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: designTokens.spacing[2],
            flexWrap: 'wrap',
            marginBottom: designTokens.spacing[2],
          }}
        >
          <Badge
            variant="default"
            size="sm"
            style={{
              textTransform: 'uppercase',
              fontSize: designTokens.typography.fontSize.xs,
            }}
          >
            {type}
          </Badge>
          <span
            style={{
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
            }}
          >
            {formatSize(size)}
          </span>
          <span
            style={{
              fontSize: designTokens.typography.fontSize.xs,
              color: designTokens.colors.text.tertiary,
            }}
          >
            {formatDate(modified)}
          </span>
        </div>

        {/* Confidence Score */}
        {confidence !== undefined && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: designTokens.spacing[2],
              marginTop: designTokens.spacing[2],
              paddingTop: designTokens.spacing[2],
              borderTop: `1px solid ${designTokens.colors.border.subtle}`,
            }}
          >
            <Icon name="Target" size={12} color={designTokens.colors.accent.ai} />
            <div style={{ flex: 1 }}>
              <div
                style={{
                  height: '4px',
                  background: designTokens.colors.surface.tertiary,
                  borderRadius: designTokens.borderRadius.full,
                  overflow: 'hidden',
                }}
              >
                <div
                  style={{
                    width: `${confidence * 100}%`,
                    height: '100%',
                    background: `linear-gradient(90deg, ${designTokens.colors.accent.ai}, ${designTokens.colors.confidence.high})`,
                    transition: `width ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
                  }}
                />
              </div>
            </div>
            <span
              style={{
                fontSize: designTokens.typography.fontSize.xs,
                fontWeight: designTokens.typography.fontWeight.medium,
                color: designTokens.colors.text.secondary,
              }}
            >
              {Math.round(confidence * 100)}%
            </span>
          </div>
        )}
      </div>

      {/* Spin animation for processing status */}
      <style>
        {`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}
      </style>
    </Card>
  )
}

export default DocumentCard
