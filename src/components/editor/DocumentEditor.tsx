import React, { useEffect, useState, useRef, useCallback } from 'react'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { AutoFocusPlugin } from '@lexical/react/LexicalAutoFocusPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { LinkPlugin } from '@lexical/react/LexicalLinkPlugin'
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListItemNode, ListNode } from '@lexical/list'
import { CodeNode, CodeHighlightNode } from '@lexical/code'
import { LinkNode, AutoLinkNode } from '@lexical/link'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  $convertFromMarkdownString,
  $convertToMarkdownString,
  TRANSFORMERS,
} from '@lexical/markdown'
import { $getRoot, $getSelection, $isRangeSelection } from 'lexical'
import HistoryPlugin from './plugins/HistoryPlugin'
import AutoLinkPlugin from './plugins/AutoLinkPlugin'
import ToolbarPlugin from './plugins/ToolbarPlugin'
import { AITextMenu } from './AITextMenu'
import { AIOperationModal } from './AIOperationModal'
import { useAITextOperations } from '../../hooks/useAITextOperations'
import { TextOperation } from '../../services/textOperationService'

interface DocumentEditorProps {
  initialContent?: string
  readOnly?: boolean
  onChange?: (content: string) => void
  className?: string
  documentId?: string
  documentTitle?: string
}

const editorTheme = {
  paragraph: 'editor-paragraph',
  quote: 'editor-quote',
  heading: {
    h1: 'editor-heading-h1',
    h2: 'editor-heading-h2',
    h3: 'editor-heading-h3',
    h4: 'editor-heading-h4',
    h5: 'editor-heading-h5',
    h6: 'editor-heading-h6',
  },
  list: {
    ol: 'editor-list-ol',
    ul: 'editor-list-ul',
    listitem: 'editor-listitem',
  },
  link: 'editor-link',
  text: {
    bold: 'editor-text-bold',
    italic: 'editor-text-italic',
    underline: 'editor-text-underline',
    strikethrough: 'editor-text-strikethrough',
    code: 'editor-text-code',
  },
  code: 'editor-code',
}

function onError(error: Error): void {
  console.error('Lexical Editor Error:', error)
}

// Plugin to load initial markdown content
function InitialContentPlugin({ initialContent }: { initialContent: string }) {
  const [editor] = useLexicalComposerContext()
  const [isInitialized, setIsInitialized] = useState(false)

  useEffect(() => {
    // Only load initial content ONCE when component mounts
    if (initialContent && !isInitialized) {
      console.log('Loading initial content (ONE TIME ONLY):', initialContent.substring(0, 50))
      editor.update(() => {
        // Clear any existing content first
        const root = $getRoot()
        root.clear()

        // Convert markdown to Lexical format
        $convertFromMarkdownString(initialContent, TRANSFORMERS)

        // Force LTR direction on root
        root.setDirection('ltr')
      })
      setIsInitialized(true)
    }
  }, [editor, initialContent, isInitialized])

  return null
}

// Plugin to capture content changes
function OnChangePlugin({ onChange }: { onChange?: (content: string) => void }) {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    if (!onChange) return

    return editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const markdown = $convertToMarkdownString(TRANSFORMERS)
        onChange(markdown)
      })
    })
  }, [editor, onChange])

  return null
}

// Plugin to force LTR direction at all times
function ForceDirectionPlugin() {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    return editor.registerUpdateListener(() => {
      editor.update(() => {
        const root = $getRoot()
        if (root.getDirection() !== 'ltr') {
          root.setDirection('ltr')
        }
      })
    })
  }, [editor])

  return null
}

// Plugin to track text selection and show AI menu
function AITextMenuPlugin({
  documentId,
  documentTitle,
  onContentChange,
}: {
  documentId?: string
  documentTitle?: string
  onContentChange?: () => void
}) {
  const [editor] = useLexicalComposerContext()
  const [showMenu, setShowMenu] = useState(false)
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 })
  const [selectedText, setSelectedText] = useState('')
  const [showModal, setShowModal] = useState(false)
  const { isLoading, result, error, execute, reset } = useAITextOperations()
  const [lastOperation, setLastOperation] = useState<TextOperation | null>(null)

  useEffect(() => {
    return editor.registerUpdateListener(() => {
      editor.getEditorState().read(() => {
        const selection = $getSelection()

        if (!$isRangeSelection(selection)) {
          setShowMenu(false)
          return
        }

        const text = selection.getTextContent()

        if (!text || text.length < 3) {
          setShowMenu(false)
          return
        }

        // Get bounding rect for menu positioning
        const nativeSelection = window.getSelection()
        if (nativeSelection && nativeSelection.rangeCount > 0) {
          const range = nativeSelection.getRangeAt(0)
          const rect = range.getBoundingClientRect()

          // Calculate smart positioning that keeps menu in viewport
          const menuWidth = 320 // Approximate menu width
          const menuHeight = 500 // Approximate menu height (max)
          const viewportWidth = window.innerWidth
          const viewportHeight = window.innerHeight
          const margin = 20 // Minimum margin from viewport edges

          let x = rect.left + rect.width / 2
          let y = rect.bottom + 10

          // Adjust horizontal position to keep menu in viewport
          if (x + menuWidth / 2 > viewportWidth - margin) {
            x = viewportWidth - menuWidth / 2 - margin
          } else if (x - menuWidth / 2 < margin) {
            x = menuWidth / 2 + margin
          }

          // Adjust vertical position - use viewport coordinates (not page coordinates)
          const spaceBelow = viewportHeight - rect.bottom
          const spaceAbove = rect.top

          if (spaceBelow < menuHeight && spaceAbove > spaceBelow) {
            // Not enough space below and more space above - position above selection
            y = rect.top - 10
          } else {
            // Position below selection
            y = rect.bottom + 10
          }

          // Ensure menu stays within viewport bounds (using viewport coordinates)
          if (y < margin) {
            // Too close to top - push down
            y = margin
          } else if (y + menuHeight > viewportHeight - margin) {
            // Too close to bottom - push up
            y = viewportHeight - menuHeight - margin
          }

          // Ensure y is never less than minimum margin from top
          y = Math.max(margin, y)

          setMenuPosition({ x, y })
          setSelectedText(text)
          setShowMenu(true)
        }
      })
    })
  }, [editor])

  const handleOperationSelect = useCallback(
    async (operation: TextOperation) => {
      setShowMenu(false)
      setShowModal(true)
      setLastOperation(operation)

      try {
        await execute(selectedText, operation, {
          document_id: documentId,
          document_title: documentTitle,
        })
      } catch (err) {
        console.error('Error executing AI operation:', err)
      }
    },
    [selectedText, documentId, documentTitle, execute]
  )

  const handleAccept = useCallback(() => {
    if (!result) return

    editor.update(() => {
      const selection = $getSelection()
      if ($isRangeSelection(selection)) {
        // Delete the selected text first
        selection.removeText()

        // Convert markdown to Lexical nodes
        $convertFromMarkdownString(result.result, TRANSFORMERS)

        onContentChange?.()
      }
    })

    setShowModal(false)
    reset()
  }, [editor, result, reset, onContentChange])

  const handleReject = useCallback(() => {
    setShowModal(false)
    reset()
  }, [reset])

  const handleRetry = useCallback(() => {
    if (lastOperation) {
      execute(selectedText, lastOperation, {
        document_id: documentId,
        document_title: documentTitle,
      })
    }
  }, [lastOperation, selectedText, documentId, documentTitle, execute])

  return (
    <>
      {showMenu && (
        <AITextMenu
          position={menuPosition}
          selectedText={selectedText}
          onOperationSelect={handleOperationSelect}
          onClose={() => setShowMenu(false)}
        />
      )}
      <AIOperationModal
        isOpen={showModal}
        onClose={() => setShowModal(false)}
        isLoading={isLoading}
        result={result}
        error={error}
        onAccept={handleAccept}
        onReject={handleReject}
        onRetry={handleRetry}
      />
    </>
  )
}

function DocumentEditor({
  initialContent = '',
  readOnly = false,
  onChange,
  className = '',
  documentId,
  documentTitle,
}: DocumentEditorProps): React.JSX.Element {
  const contentChangedRef = useRef(false)

  const handleContentChange = useCallback(() => {
    contentChangedRef.current = true
  }, [])

  const initialConfig = {
    namespace: 'DocumentEditor',
    theme: editorTheme,
    onError,
    editable: !readOnly,
    editorState: undefined, // Let Lexical handle initial state
    nodes: [
      HeadingNode,
      QuoteNode,
      ListNode,
      ListItemNode,
      CodeNode,
      CodeHighlightNode,
      LinkNode,
      AutoLinkNode,
    ],
  }

  // Debug: Check if text is being reversed
  useEffect(() => {
    console.log('DocumentEditor mounted with initialContent length:', initialContent?.length || 0)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <div className={`document-editor-container ${className}`} dir="ltr">
      <LexicalComposer initialConfig={initialConfig}>
        <div className="editor-shell" dir="ltr">
          {!readOnly && <ToolbarPlugin />}
          <div className="editor-container" dir="ltr">
            <RichTextPlugin
              contentEditable={
                <ContentEditable
                  className={`editor-input ${readOnly ? 'cursor-default' : ''}`}
                  aria-placeholder="Start typing your document content..."
                  placeholder={
                    <div className="editor-placeholder" dir="ltr">
                      Start typing your document content...
                    </div>
                  }
                  dir="ltr"
                  style={{ direction: 'ltr', textAlign: 'left', unicodeBidi: 'normal' }}
                />
              }
              ErrorBoundary={LexicalErrorBoundary}
            />
            <InitialContentPlugin initialContent={initialContent} />
            <OnChangePlugin onChange={onChange} />
            <ForceDirectionPlugin />
            <HistoryPlugin />
            <AutoFocusPlugin />
            <ListPlugin />
            <LinkPlugin />
            <AutoLinkPlugin />
            {!readOnly && (
              <AITextMenuPlugin
                documentId={documentId}
                documentTitle={documentTitle}
                onContentChange={handleContentChange}
              />
            )}
          </div>
        </div>
      </LexicalComposer>
    </div>
  )
}

export default DocumentEditor
