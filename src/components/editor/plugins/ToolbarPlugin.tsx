import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { useCallback, useEffect, useState } from 'react'
import {
  $getSelection,
  $isRangeSelection,
  FORMAT_TEXT_COMMAND,
  UNDO_COMMAND,
  REDO_COMMAND,
  CAN_UNDO_COMMAND,
  CAN_REDO_COMMAND,
} from 'lexical'
import { $setBlocksType } from '@lexical/selection'
import { $createHeadingNode, $isHeadingNode, HeadingTagType } from '@lexical/rich-text'
import { $wrapNodes } from '@lexical/selection'
import { $createQuoteNode } from '@lexical/rich-text'
import {
  INSERT_ORDERED_LIST_COMMAND,
  INSERT_UNORDERED_LIST_COMMAND,
  REMOVE_LIST_COMMAND,
  $isListNode,
  ListNode,
} from '@lexical/list'
import { $getNearestNodeOfType, mergeRegister } from '@lexical/utils'

function ToolbarPlugin(): React.JSX.Element {
  const [editor] = useLexicalComposerContext()
  const [canUndo, setCanUndo] = useState(false)
  const [canRedo, setCanRedo] = useState(false)
  const [isBold, setIsBold] = useState(false)
  const [isItalic, setIsItalic] = useState(false)
  const [isUnderline, setIsUnderline] = useState(false)
  const [isStrikethrough, setIsStrikethrough] = useState(false)
  const [blockType, setBlockType] = useState<string>('paragraph')

  const updateToolbar = useCallback(() => {
    const selection = $getSelection()
    if ($isRangeSelection(selection)) {
      // Update text format
      setIsBold(selection.hasFormat('bold'))
      setIsItalic(selection.hasFormat('italic'))
      setIsUnderline(selection.hasFormat('underline'))
      setIsStrikethrough(selection.hasFormat('strikethrough'))

      // Update block type
      const anchorNode = selection.anchor.getNode()
      const element =
        anchorNode.getKey() === 'root' ? anchorNode : anchorNode.getTopLevelElementOrThrow()
      const elementKey = element.getKey()
      const elementDOM = editor.getElementByKey(elementKey)

      if (elementDOM !== null) {
        if ($isListNode(element)) {
          const parentList = $getNearestNodeOfType(anchorNode, ListNode)
          const type = parentList ? parentList.getTag() : element.getTag()
          setBlockType(type)
        } else {
          const type = $isHeadingNode(element) ? element.getTag() : element.getType()
          setBlockType(type)
        }
      }
    }
  }, [editor])

  useEffect(() => {
    return mergeRegister(
      editor.registerUpdateListener(({ editorState }) => {
        editorState.read(() => {
          updateToolbar()
        })
      }),
      editor.registerCommand(
        CAN_UNDO_COMMAND,
        payload => {
          setCanUndo(payload)
          return false
        },
        1
      ),
      editor.registerCommand(
        CAN_REDO_COMMAND,
        payload => {
          setCanRedo(payload)
          return false
        },
        1
      )
    )
  }, [editor, updateToolbar])

  const formatHeading = (headingSize: HeadingTagType) => {
    if (blockType !== headingSize) {
      editor.update(() => {
        const selection = $getSelection()
        if ($isRangeSelection(selection)) {
          $setBlocksType(selection, () => $createHeadingNode(headingSize))
        }
      })
    }
  }

  const formatParagraph = () => {
    if (blockType !== 'paragraph') {
      editor.update(() => {
        const selection = $getSelection()
        if ($isRangeSelection(selection)) {
          $setBlocksType(selection, () => $createHeadingNode('h1'))
        }
      })
    }
  }

  const formatQuote = () => {
    if (blockType !== 'quote') {
      editor.update(() => {
        const selection = $getSelection()
        if ($isRangeSelection(selection)) {
          $wrapNodes(selection, () => $createQuoteNode())
        }
      })
    }
  }

  const formatBulletList = () => {
    if (blockType !== 'ul') {
      editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)
    } else {
      editor.dispatchCommand(REMOVE_LIST_COMMAND, undefined)
    }
  }

  const formatNumberedList = () => {
    if (blockType !== 'ol') {
      editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)
    } else {
      editor.dispatchCommand(REMOVE_LIST_COMMAND, undefined)
    }
  }

  return (
    <div className="toolbar flex gap-1 p-2 border-b border-gray-200 flex-wrap">
      <button
        disabled={!canUndo}
        onClick={() => {
          editor.dispatchCommand(UNDO_COMMAND, undefined)
        }}
        className="toolbar-item px-3 py-1 rounded hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
        aria-label="Undo"
        title="Undo (Ctrl+Z)"
      >
        ↶
      </button>
      <button
        disabled={!canRedo}
        onClick={() => {
          editor.dispatchCommand(REDO_COMMAND, undefined)
        }}
        className="toolbar-item px-3 py-1 rounded hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
        aria-label="Redo"
        title="Redo (Ctrl+Y)"
      >
        ↷
      </button>
      <div className="divider w-px bg-gray-300 mx-2" />

      {/* Text formatting */}
      <button
        onClick={() => {
          editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold')
        }}
        className={`toolbar-item px-3 py-1 rounded font-bold ${
          isBold ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Format Bold"
        title="Bold (Ctrl+B)"
      >
        B
      </button>
      <button
        onClick={() => {
          editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic')
        }}
        className={`toolbar-item px-3 py-1 rounded italic ${
          isItalic ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Format Italic"
        title="Italic (Ctrl+I)"
      >
        I
      </button>
      <button
        onClick={() => {
          editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'underline')
        }}
        className={`toolbar-item px-3 py-1 rounded underline ${
          isUnderline ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Format Underline"
        title="Underline (Ctrl+U)"
      >
        U
      </button>
      <button
        onClick={() => {
          editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'strikethrough')
        }}
        className={`toolbar-item px-3 py-1 rounded line-through ${
          isStrikethrough ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Format Strikethrough"
        title="Strikethrough"
      >
        S
      </button>
      <div className="divider w-px bg-gray-300 mx-2" />

      {/* Block formatting */}
      <select
        className="toolbar-item px-2 py-1 rounded border border-gray-300"
        value={blockType}
        onChange={e => {
          const value = e.target.value
          if (value === 'paragraph') {
            formatParagraph()
          } else if (
            value === 'h1' ||
            value === 'h2' ||
            value === 'h3' ||
            value === 'h4' ||
            value === 'h5' ||
            value === 'h6'
          ) {
            formatHeading(value as HeadingTagType)
          } else if (value === 'quote') {
            formatQuote()
          }
        }}
      >
        <option value="paragraph">Paragraph</option>
        <option value="h1">Heading 1</option>
        <option value="h2">Heading 2</option>
        <option value="h3">Heading 3</option>
        <option value="h4">Heading 4</option>
        <option value="h5">Heading 5</option>
        <option value="h6">Heading 6</option>
        <option value="quote">Quote</option>
      </select>

      <button
        onClick={formatBulletList}
        className={`toolbar-item px-3 py-1 rounded ${
          blockType === 'ul' ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Bullet List"
        title="Bullet List"
      >
        • List
      </button>
      <button
        onClick={formatNumberedList}
        className={`toolbar-item px-3 py-1 rounded ${
          blockType === 'ol' ? 'bg-blue-100' : 'hover:bg-gray-100'
        }`}
        aria-label="Numbered List"
        title="Numbered List"
      >
        1. List
      </button>
    </div>
  )
}

export default ToolbarPlugin
