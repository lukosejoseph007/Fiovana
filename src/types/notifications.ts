// src/types/notifications.ts
// TypeScript types for notification system

export interface Notification {
  id: string
  type: 'info' | 'success' | 'warning' | 'error' | 'conflict'
  title: string
  message: string
  timestamp: number
  duration?: number // milliseconds to show notification (optional)
  action?: {
    label: string
    onClick: () => void
  }
  metadata?: Record<string, unknown>
}

export interface FileChangeNotification extends Notification {
  type: 'info' | 'success' | 'warning' | 'error'
  filePath: string
  eventType: 'created' | 'modified' | 'deleted' | 'renamed' | 'moved'
  size?: number
  isDirectory: boolean
}

export interface ConflictNotification extends Notification {
  type: 'conflict'
  conflictType:
    | 'external-modification'
    | 'external-deletion'
    | 'external-creation'
    | 'content'
    | 'timestamp'
  filePath: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  resolutionRequired: boolean
  externalTimestamp?: number
  applicationTimestamp?: number
  externalHash?: string
  applicationHash?: string
}

export interface SecurityNotification extends Notification {
  type: 'warning' | 'error'
  securityLevel: 'low' | 'medium' | 'high' | 'critical'
  operation: string
  path: string
  reason: string
}

export interface NotificationPreferences {
  fileChanges: boolean
  conflicts: boolean
  security: boolean
  soundEnabled: boolean
  duration: number // default duration in ms
}

export interface NotificationState {
  notifications: Notification[]
  unreadCount: number
  preferences: NotificationPreferences
  isEnabled: boolean
}
