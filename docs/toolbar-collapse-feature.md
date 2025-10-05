# Toolbar Collapse Feature

## Overview

The DocumentToolbar now includes a collapsible ribbon feature similar to Microsoft Word, allowing users to maximize document viewing space when needed while keeping essential controls accessible.

## User Experience

### Default State
- Toolbar is **expanded by default** showing all contextual tools
- Three rows visible: Document Info, Tab Navigation, and Contextual Tools

### Collapsed State
- Only two rows visible: Document Info and Tab Navigation
- Contextual tools (Row 3) are hidden
- More vertical space for document content
- Active tab is highlighted with a subtle glow

### Interactions

#### Collapsing the Toolbar
1. Click the chevron button (↓) next to the primary actions
2. Row 3 smoothly animates out
3. Chevron rotates to indicate collapsed state (↓ → ↑)
4. State is saved to localStorage

#### Expanding the Toolbar
**Method 1: Click the Chevron Button**
- Click the chevron button (↑)
- Row 3 smoothly animates in
- Chevron rotates back (↑ → ↓)

**Method 2: Click Any Tab**
- When collapsed, clicking any tab automatically expands the toolbar
- The clicked tab becomes active and its tools are shown
- This provides quick access to specific tool groups

## Visual States

### Expanded (Default)
```
┌─────────────────────────────────────────────────────────────┐
│ [Icon] Document Name [Badges] [Users]              [Close] │ Row 1
├─────────────────────────────────────────────────────────────┤
│ Home│Review│AI│More      [↓][Save][Actions]                │ Row 2
├─────────────────────────────────────────────────────────────┤
│ [Edit] [View] [AI On/Off]                                  │ Row 3
└─────────────────────────────────────────────────────────────┘
```

### Collapsed
```
┌─────────────────────────────────────────────────────────────┐
│ [Icon] Document Name [Badges] [Users]              [Close] │ Row 1
├─────────────────────────────────────────────────────────────┤
│ Home│Review│AI│More      [↑][Save][Actions]                │ Row 2
└─────────────────────────────────────────────────────────────┘
```

Note: Active tab shows subtle blue glow when collapsed

## Technical Implementation

### State Management

```typescript
// Collapse state (persisted to localStorage)
const [isCollapsed, setIsCollapsed] = useState<boolean>(() => {
  const saved = localStorage.getItem(TOOLBAR_COLLAPSED_KEY)
  return saved !== null ? saved === 'true' : defaultCollapsed
})

// Persist on change
useEffect(() => {
  localStorage.setItem(TOOLBAR_COLLAPSED_KEY, isCollapsed.toString())
  onCollapseChange?.(isCollapsed)
}, [isCollapsed, onCollapseChange])
```

### Auto-Expand on Tab Click

```typescript
const handleTabChange = (tab: ToolbarTab) => {
  setActiveTab(tab)
  onTabChange?.(tab)
  // Auto-expand when switching tabs if collapsed
  if (isCollapsed) {
    setIsCollapsed(false)
  }
}
```

### Animation

```typescript
// Row 3 styling
<div style={{
  display: isCollapsed ? 'none' : 'flex',
  opacity: isCollapsed ? 0 : 1,
  maxHeight: isCollapsed ? '0px' : '48px',
  overflow: 'hidden',
  transition: `all ${designTokens.animation.duration.normal} ${designTokens.animation.easing.easeOut}`,
}}>
```

### Visual Feedback

**Active Tab Highlight (Collapsed)**
```typescript
background: activeTab === tab.id
  ? isCollapsed
    ? 'rgba(0, 212, 255, 0.15)'  // Blue glow when collapsed
    : 'rgba(255, 255, 255, 0.1)'  // White when expanded
  : 'transparent'
```

**Chevron Rotation**
```typescript
<Icon
  name="ChevronDown"
  style={{
    transform: isCollapsed ? 'rotate(0deg)' : 'rotate(180deg)',
    transition: `transform ${designTokens.animation.duration.fast}`,
  }}
/>
```

## Component API

### New Props

```typescript
interface DocumentToolbarProps {
  // ... existing props

  // Collapse functionality
  defaultCollapsed?: boolean        // Initial state (default: false)
  onCollapseChange?: (collapsed: boolean) => void  // Callback on state change
}
```

### Usage Example

```typescript
<DocumentToolbar
  // ... other props
  defaultCollapsed={false}
  onCollapseChange={(collapsed) => {
    console.log('Toolbar collapsed:', collapsed)
    // Optionally adjust document layout
  }}
/>
```

## Persistence

### localStorage Key
```typescript
const TOOLBAR_COLLAPSED_KEY = 'fiovana_toolbar_collapsed'
```

### Storage Format
- Value: `'true'` or `'false'` (string)
- Persists across sessions
- User preference is remembered per browser

## Benefits

### 1. **More Reading Space**
- Collapsed toolbar provides ~48px additional vertical space
- Useful for small screens or when focused on content
- Similar to full-screen reading modes

### 2. **Familiar Pattern**
- Matches Microsoft Word/Excel ribbon behavior
- Users intuitively understand the collapse/expand interaction
- Reduces learning curve

### 3. **Persistent Preference**
- User's choice is saved
- Consistent experience across document sessions
- No need to re-collapse every time

### 4. **Quick Access**
- Tab click provides one-click access to any tool group
- No need to find and click expand button
- Efficient workflow

### 5. **Non-Intrusive**
- Essential controls (Save, Close, Tabs) always visible
- Document info always accessible
- Only contextual tools are hidden

## Keyboard Shortcuts (Future Enhancement)

Potential shortcuts for quick toggle:
- `Ctrl/Cmd + Shift + R` - Toggle ribbon collapse
- `F11` - Full screen mode with collapsed toolbar
- `Alt + H/R/A/M` - Jump to Home/Review/AI/More tab (auto-expands)

## Accessibility

### ARIA Labels
```typescript
<button aria-label={isCollapsed ? 'Expand toolbar' : 'Collapse toolbar'}>
  <Icon name="ChevronDown" />
</button>
```

### Keyboard Navigation
- Tab key navigates through all controls
- Enter/Space activates collapse button
- Focus is maintained when collapsing/expanding

### Screen Reader Announcements
- "Toolbar collapsed" / "Toolbar expanded"
- Active tab announced when switching
- Tool groups announced when expanded

## Edge Cases Handled

### 1. **Mid-Collapse Navigation**
If user collapses while on Tab A, then later expands by clicking Tab B:
- ✅ Toolbar expands
- ✅ Tab B becomes active
- ✅ Tab B's tools are shown

### 2. **Unsaved Changes**
Collapsing the toolbar doesn't affect:
- ✅ Save button remains visible
- ✅ Save state indicators visible
- ✅ Auto-save continues working

### 3. **Collaboration Indicators**
When collapsed, Row 1 still shows:
- ✅ Active users
- ✅ Offline sync status
- ✅ Document metadata

### 4. **First-Time Users**
- ✅ Toolbar is expanded by default
- ✅ Tooltip explains collapse button
- ✅ Visual chevron indicates collapse direction

## Testing Checklist

- [ ] Click collapse button → Row 3 hides smoothly
- [ ] Click expand button → Row 3 shows smoothly
- [ ] Click tab when collapsed → Auto-expands and switches tab
- [ ] Refresh page → Collapse state persists
- [ ] Clear localStorage → Defaults to expanded
- [ ] Rapid collapse/expand → Animations don't break
- [ ] Collapse → Switch tabs → Correct tools shown
- [ ] Multiple documents → Each remembers state independently (future)

## Responsive Behavior

### Desktop (>1024px)
- Full collapse/expand functionality
- Smooth animations
- All tabs and tools accessible

### Tablet (768px - 1024px)
- Collapse provides more benefit (limited screen space)
- Touch-friendly collapse button
- Same functionality as desktop

### Mobile (<768px)
- Toolbar may auto-collapse by default (future enhancement)
- Swipe gestures to expand/collapse (future enhancement)
- Simplified tab interface

## Performance Considerations

### Animation Performance
- Uses CSS transitions (GPU-accelerated)
- `display: none` when fully collapsed (removes from layout)
- No JavaScript animation loops
- Smooth 60fps animation

### localStorage
- Minimal writes (only on state change)
- Synchronous read on mount (fast)
- String value (< 10 bytes)

### Re-renders
- Collapse state is local to toolbar
- Parent component not re-rendered
- Only Row 3 re-renders on toggle

## Future Enhancements

### Phase 2: Enhanced Interactions
- [ ] Double-click tab to collapse
- [ ] Pin specific tools to always show
- [ ] Keyboard shortcut to toggle
- [ ] Gesture support on touch devices

### Phase 3: Customization
- [ ] Auto-collapse after inactivity
- [ ] Remember per-document collapse state
- [ ] Floating mini-toolbar when collapsed
- [ ] Customizable collapse button position

### Phase 4: Context Awareness
- [ ] Auto-collapse in full-screen mode
- [ ] Auto-expand when specific actions triggered
- [ ] Collapse presets (minimal, compact, full)
- [ ] Smart collapse based on screen size

## Related Features

Works seamlessly with:
- ✅ Tab switching
- ✅ Save functionality
- ✅ Collaboration indicators
- ✅ Version history
- ✅ Track changes
- ✅ All toolbar actions

## Migration Guide

No breaking changes - feature is backward compatible:

```typescript
// Before (still works)
<DocumentToolbar
  documentName="My Doc"
  homeActions={[...]}
  // ... other props
/>

// After (with collapse control)
<DocumentToolbar
  documentName="My Doc"
  homeActions={[...]}
  defaultCollapsed={false}
  onCollapseChange={(collapsed) => {
    // Optional: React to collapse state
  }}
/>
```

## Conclusion

The toolbar collapse feature provides a familiar, space-saving option that enhances the document editing experience, especially on smaller screens or when users need to focus on content without sacrificing access to essential tools.
