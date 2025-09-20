// Types for file deduplication system
export interface DeduplicationResult {
  source_path: string
  target_path: string
  was_deduplicated: boolean
  space_saved: number
  duplicate_of?: string
  content_hash: ContentHash
}

export interface ContentHash {
  hash: string
  size: number
  file_type?: string
}

export interface DuplicateFile {
  path: string
  name: string
  size: number
  hash: string
  lastModified: Date
}

export interface DuplicateGroup {
  hash: string
  files: DuplicateFile[]
  totalSize: number
  potentialSavings: number
}

export interface DuplicateComparisonProps {
  originalFile: DuplicateFile
  duplicateFiles: DuplicateFile[]
  onResolve: (action: DuplicateAction, selectedFiles?: string[]) => void
  onCancel: () => void
}

export interface DuplicateAction {
  type: 'keep_original' | 'keep_selected' | 'keep_all' | 'deduplicate'
  selectedFiles?: string[]
}

export interface DuplicateResolutionPolicy {
  auto_deduplicate: boolean
  always_prompt: boolean
  prefer_newest: boolean
  prefer_largest: boolean
}

export interface StorageStats {
  total_files: number
  total_references: number
  unreferenced_count: number
  space_saved: number
}

export interface GarbageCollectionResult {
  deleted_files: number
  space_freed: number
  cleaned_entries: number
  duration: number
  errors: string[]
}
