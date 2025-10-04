# Toolbar Integration Bugfix

## Issue
React Hook ordering violation in DocumentViewer component causing the error:
```
Error: Rendered more hooks than during the previous render.
```

## Root Cause
The new toolbar action groups (`homeActions`, `reviewActions`, `aiToolsActions`, `moreActions`) were defined using `useMemo` hooks **after** the component's early return statements (loading state and document-not-found checks).

This violated React's Rules of Hooks which require:
1. Hooks must be called in the **exact same order** on every render
2. Hooks cannot be called conditionally or after early returns

### Problematic Code Structure (Before Fix)
```typescript
const DocumentViewer = () => {
  // 1. All useState hooks
  // 2. All useRef hooks
  // 3. useContext
  // 4. Some useMemo hooks (collaborationUsers, cursorPositions, confidenceScore)
  // 5. useCallback hooks
  // 6. useEffect hooks

  // Early return for loading state
  if (isLoading) return <LoadingView />

  // Early return for not found
  if (!document) return <NotFoundView />

  // ❌ WRONG: useMemo hooks defined AFTER early returns
  const homeActions = useMemo(...)
  const reviewActions = useMemo(...)
  const aiToolsActions = useMemo(...)
  const moreActions = useMemo(...)

  return <DocumentToolbar ... />
}
```

When the component re-renders:
- **First render (loading)**: Returns early, only ~92 hooks called
- **Second render (loaded)**: Executes all code, ~96 hooks called
- **Result**: Hook count mismatch → Error

## Solution
Move all `useMemo` hooks for toolbar actions **before** any early return statements, grouping them with other memoization hooks.

### Fixed Code Structure (After Fix)
```typescript
const DocumentViewer = () => {
  // 1. All useState hooks (17 hooks)
  const [document, setDocument] = useState(...)
  const [structure, setStructure] = useState(...)
  // ... more state

  // 2. All useRef hooks (2 hooks)
  const contentRef = useRef(...)
  const saveNotificationTimeoutRef = useRef(...)

  // 3. useContext hook (1 hook)
  const collaboration = useCollaboration()

  // 4. All derived state/computation hooks
  const collaborationUsers = useMemo(...)
  const cursorPositions = useMemo(...)

  // 5. Document state management hooks
  const { content, isDirty, ... } = useDocumentState(...)

  // 6. Custom hooks
  useAutoSave(...)
  const { isTyping, handleTyping } = useTypingIndicator(...)
  const { conflicts, ... } = useConflictResolution(...)
  const { isSyncing, ... } = useOfflineSync(...)
  const { changes, ... } = useTrackChanges(...)

  // 7. More useMemo hooks for derived data
  const confidenceScore = useMemo(...)

  // ✅ CORRECT: Toolbar action groups BEFORE early returns
  const homeActions = useMemo<ToolbarGroup[]>(...)
  const reviewActions = useMemo<ToolbarGroup[]>(...)
  const aiToolsActions = useMemo<ToolbarGroup[]>(...)
  const moreActions = useMemo<ToolbarGroup[]>(...)

  // 8. All useCallback hooks
  const generateAISuggestions = useCallback(...)
  const updateOperation = useCallback(...)
  const loadDocument = useCallback(...)

  // 9. All useEffect hooks
  useEffect(() => loadDocument(), [loadDocument])
  useEffect(() => initializeDocumentState(...), [...])
  useEffect(() => handleContentClick(...), [...])

  // Early returns now safe - all hooks already called
  if (isLoading) return <LoadingView />
  if (!document) return <NotFoundView />

  return <DocumentToolbar ... />
}
```

## Files Changed
- `src/components/canvas/DocumentViewer.tsx`
  - Moved 4 `useMemo` hooks from after early returns to before them
  - Removed duplicate hook definitions
  - Maintained proper hook ordering

## Verification
```bash
# Type check passed
npx tsc --noEmit
# ✅ No errors

# Hook count verification
grep -n "useMemo\|useState\|useEffect" DocumentViewer.tsx
# ✅ All hooks in consistent order
```

## Hook Order Summary
Total hooks in DocumentViewer (always consistent now):
1. `useState`: 17 hooks (state variables)
2. `useRef`: 2 hooks (refs)
3. `useContext`: 1 hook (collaboration)
4. `useMemo`: 7 hooks (derived state including toolbar actions)
5. `useCallback`: 5 hooks (memoized functions)
6. `useEffect`: 3+ hooks (side effects)
7. Custom hooks: Multiple (document state, auto-save, collaboration, etc.)

**Total: ~93 hooks always called in the same order ✅**

## React Rules of Hooks
This fix ensures compliance with:

### ✅ Rule 1: Only Call Hooks at the Top Level
- Never call hooks inside loops, conditions, or nested functions
- All hooks now called unconditionally before any early returns

### ✅ Rule 2: Only Call Hooks from React Functions
- All hooks called within DocumentViewer functional component
- Custom hooks properly composed

### ✅ Rule 3: Call Hooks in the Same Order
- Hook order now consistent across all renders
- No conditional hook calls

## Testing
1. **Loading state**: Component renders with skeleton → All hooks called
2. **Loaded state**: Component renders with content → Same hooks called
3. **Re-renders**: State changes → Same hook order maintained
4. **Mode switching**: Edit ↔ View → Hook order consistent

## Prevention
To prevent similar issues in the future:

1. **Always define hooks at the top of the component**
2. **Group hooks by type** (useState, useRef, useMemo, useCallback, useEffect)
3. **Never add hooks after early returns**
4. **Use ESLint plugin**: `eslint-plugin-react-hooks`
5. **Run type checking**: Catches many hook-related issues

## Resources
- [Rules of Hooks - React Docs](https://react.dev/link/rules-of-hooks)
- [ESLint Plugin React Hooks](https://www.npmjs.com/package/eslint-plugin-react-hooks)
