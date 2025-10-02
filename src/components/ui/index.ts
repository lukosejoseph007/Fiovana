// Core UI Components for Proxemic
// Based on the refined visual architecture with enterprise-grade functionality

export { default as Button } from './Button'
export type { ButtonProps } from './Button'

export { default as Input } from './Input'
export type { InputProps } from './Input'

export { default as Card, CardHeader, CardBody, CardFooter } from './Card'
export type { CardProps } from './Card'

export { default as Modal } from './Modal'
export type { ModalProps } from './Modal'

export { default as Tooltip } from './Tooltip'
export type { TooltipProps } from './Tooltip'

export { default as Progress, ConfidenceProgress, HealthProgress, AIProgress } from './Progress'
export type { ProgressProps } from './Progress'

export { default as Badge, StatusBadge, ConfidenceBadge, AIStatusBadge } from './Badge'
export type { BadgeProps } from './Badge'

export { default as Dropdown } from './Dropdown'
export type { DropdownProps, DropdownOption } from './Dropdown'

export { default as Tabs, TabList, Tab, TabPanels, TabPanel } from './Tabs'
export type { TabsProps, TabListProps, TabProps, TabPanelsProps, TabPanelProps } from './Tabs'

export { default as Avatar, AvatarGroup } from './Avatar'
export type { AvatarProps, AvatarGroupProps } from './Avatar'

export { Icon } from './Icon'
export type { IconComponentProps } from './Icon'

export { default as EmptyState } from './EmptyState'
export type { EmptyStateProps, EmptyStateAction } from './EmptyState'

export { default as LoadingState, Skeleton, ProgressCard } from './LoadingState'
export type { LoadingStateProps, SkeletonProps, ProgressCardProps } from './LoadingState'

// Enhanced Loading States System
export {
  TopProgressLine,
  AIThinkingIndicator,
  Skeleton as SkeletonNew,
  DocumentSkeleton,
  ChatSkeleton,
  CardSkeleton,
  ListSkeleton,
  OperationProgressTracker,
  LongOperationProgress,
} from './LoadingStates'
export type {
  TopProgressLineProps,
  AIThinkingIndicatorProps,
  SkeletonProps as SkeletonNewProps,
  OperationProgress,
  OperationProgressTrackerProps,
  LongOperationProgressProps,
} from './LoadingStates'

// Offline Support Components
export { OfflineIndicator } from './OfflineIndicator'
export type { OfflineIndicatorProps } from './OfflineIndicator'

// Re-export icon assets for convenience
export { Icons } from '../../assets/icons/utils'
export type { IconProps } from '../../assets/icons/types'

// Design tokens for consistent styling
export { designTokens } from '../../styles/tokens'
