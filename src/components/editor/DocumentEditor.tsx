import React, { useEffect, useState } from 'react'
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
import { $getRoot } from 'lexical'
import HistoryPlugin from './plugins/HistoryPlugin'
import AutoLinkPlugin from './plugins/AutoLinkPlugin'
import ToolbarPlugin from './plugins/ToolbarPlugin'

interface DocumentEditorProps {
  initialContent?: string
  readOnly?: boolean
  onChange?: (content: string) => void
  className?: string
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

function DocumentEditor({
  initialContent = '',
  readOnly = false,
  onChange,
  className = '',
}: DocumentEditorProps): React.JSX.Element {
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
          </div>
        </div>
      </LexicalComposer>
    </div>
  )
}

export default DocumentEditor
