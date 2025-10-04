// Icon utilities and mappings

import React from 'react'
import type { IconProps } from './types'
import {
  DocumentIcon,
  PDFIcon,
  WordIcon,
  PowerPointIcon,
  AIStatusIcon,
  HealthIcon,
  ConfidenceIcon,
  CompareIcon,
  GenerateIcon,
  AnalyzeIcon,
  SearchIcon,
  SettingsIcon,
  WorkspaceIcon,
  SpinnerIcon,
  PulseIcon,
  UserIcon,
  CollaborationIcon,
  ChevronDownIcon,
  AlertIcon,
  ColumnsIcon,
  LayersIcon,
  FilterIcon,
  XIcon,
  LinkIcon,
  MessageCircleIcon,
  FileTextIcon,
  LoaderIcon,
  SendIcon,
  CpuIcon,
  ZapIcon,
  LayoutIcon,
  EyeIcon,
  Share2Icon,
  AlertTriangleIcon,
  RefreshCcwIcon,
  AlertCircleIcon,
  HeartIcon,
  TargetIcon,
  LightBulbIcon,
  ArrowRightIcon,
  BookOpenIcon,
  FolderIcon,
  TrendingUpIcon,
  TrendingDownIcon,
  MinusIcon,
  GitCompareIcon,
  FilePlusIcon,
  EditIcon,
  PaletteIcon,
  InfoIcon,
  HistoryIcon,
  GitBranchIcon,
  CheckCircleIcon,
  LightbulbIcon,
  SparklesIcon,
  CheckIcon,
  DownloadIcon,
} from './index'

// Utility function to get document type icon component
export const getDocumentTypeIcon = (mimeType: string): React.FC<IconProps> => {
  if (mimeType.includes('pdf')) return PDFIcon
  if (mimeType.includes('word') || mimeType.includes('document')) return WordIcon
  if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) return PowerPointIcon
  return DocumentIcon
}

// Export all icons for easy access
export const Icons = {
  // Document types
  Document: DocumentIcon,
  PDF: PDFIcon,
  Word: WordIcon,
  PowerPoint: PowerPointIcon,

  // Status indicators
  AIStatus: AIStatusIcon,
  Health: HealthIcon,
  Confidence: ConfidenceIcon,

  // Actions
  Compare: CompareIcon,
  Generate: GenerateIcon,
  Analyze: AnalyzeIcon,

  // Navigation
  Search: SearchIcon,
  Settings: SettingsIcon,
  Workspace: WorkspaceIcon,

  // Loading
  Spinner: SpinnerIcon,
  Pulse: PulseIcon,
  Loader: LoaderIcon,

  // Collaboration
  User: UserIcon,
  Collaboration: CollaborationIcon,

  // Navigation elements
  ChevronDown: ChevronDownIcon,

  // Common UI elements
  Alert: AlertIcon,
  Columns: ColumnsIcon,
  Layers: LayersIcon,
  Filter: FilterIcon,
  X: XIcon,
  Link: LinkIcon,
  MessageCircle: MessageCircleIcon,
  FileText: FileTextIcon,
  Send: SendIcon,
  Cpu: CpuIcon,
  Zap: ZapIcon,
  Layout: LayoutIcon,
  Eye: EyeIcon,
  Share2: Share2Icon,
  AlertTriangle: AlertTriangleIcon,
  RefreshCcw: RefreshCcwIcon,
  AlertCircle: AlertCircleIcon,
  Heart: HeartIcon,
  Target: TargetIcon,
  LightBulb: LightBulbIcon,
  ArrowRight: ArrowRightIcon,
  BookOpen: BookOpenIcon,
  Folder: FolderIcon,
  TrendingUp: TrendingUpIcon,
  TrendingDown: TrendingDownIcon,
  Minus: MinusIcon,
  GitCompare: GitCompareIcon,
  FilePlus: FilePlusIcon,
  Edit: EditIcon,
  Palette: PaletteIcon,
  Info: InfoIcon,
  History: HistoryIcon,
  GitBranch: GitBranchIcon,
  CheckCircle: CheckCircleIcon,
  Lightbulb: LightbulbIcon,
  Sparkles: SparklesIcon,
  Check: CheckIcon,
  Download: DownloadIcon,

  // Utility
  getDocumentTypeIcon,
}

export default Icons
