# Toolbar Collapse - Visual Guide

## Overview
The toolbar collapse feature provides Word-like ribbon functionality, giving users more document viewing space.

## Visual States

### 1. Expanded State (Default)

```
┌────────────────────────────────────────────────────────────────────┐
│                         ROW 1: DOCUMENT INFO                       │
│  📄 Document Name    [73% confidence]  [Technical]  👥 [●]  [×]   │
├────────────────────────────────────────────────────────────────────┤
│                    ROW 2: TAB NAVIGATION & ACTIONS                 │
│  Home │ Review │ AI Tools │ More          ↓  💾 Save  ⋮ Actions  │
│  ════                                                              │
├────────────────────────────────────────────────────────────────────┤
│                    ROW 3: CONTEXTUAL TOOLS (Home)                  │
│  📝 Edit | 👁️ View     ✨ AI On                                 │
└────────────────────────────────────────────────────────────────────┘
                    Document Content Area
```

**Height**: ~132px total
- Row 1: 44px
- Row 2: 40px
- Row 3: 48px

**Behavior**:
- All tools visible
- Active tab (Home) underlined with cyan
- Tools grouped with dividers
- Contextual tools change with tab selection

---

### 2. Collapsed State (Space-Saving)

```
┌────────────────────────────────────────────────────────────────────┐
│                         ROW 1: DOCUMENT INFO                       │
│  📄 Document Name    [73% confidence]  [Technical]  👥 [●]  [×]   │
├────────────────────────────────────────────────────────────────────┤
│                    ROW 2: TAB NAVIGATION & ACTIONS                 │
│  Home │ Review │ AI Tools │ More          ↑  💾 Save  ⋮ Actions  │
│  ════ (subtle blue glow)                                          │
└────────────────────────────────────────────────────────────────────┘
                    Document Content Area
                   (+48px more vertical space)
```

**Height**: ~84px total
- Row 1: 44px
- Row 2: 40px
- Row 3: Hidden (0px)

**Visual Changes**:
- ↓ becomes ↑ (chevron rotates 180°)
- Active tab has subtle blue glow: `rgba(0, 212, 255, 0.15)`
- Row 3 hidden with smooth animation
- Save button and actions still visible

---

## Interaction Flow

### Scenario 1: Manual Collapse

```
Initial State (Expanded)
        ↓ [User clicks chevron]

Animation (200ms)
┌─ Row 3 opacity: 1 → 0
├─ Row 3 maxHeight: 48px → 0px
├─ Chevron rotation: 180° → 0°
└─ localStorage: 'false' → 'true'

Final State (Collapsed)
```

### Scenario 2: Tab Click When Collapsed

```
Collapsed State
        ↓ [User clicks "Review" tab]

Auto-Expand Animation (200ms)
┌─ Row 3 opacity: 0 → 1
├─ Row 3 maxHeight: 0px → 48px
├─ Active tab: Home → Review
├─ Contextual tools: Edit/View → Versions/Comments/Track
├─ Chevron rotation: 0° → 180°
└─ localStorage: 'true' → 'false'

Final State (Expanded, Review Tab Active)
```

### Scenario 3: Manual Expand

```
Collapsed State
        ↓ [User clicks chevron]

Animation (200ms)
┌─ Row 3 opacity: 0 → 1
├─ Row 3 maxHeight: 0px → 48px
├─ Active tab: Unchanged
├─ Contextual tools: Show current tab's tools
├─ Chevron rotation: 0° → 180°
└─ localStorage: 'true' → 'false'

Final State (Expanded, Same Tab Active)
```

---

## Tab States in Collapsed Mode

### Active Tab (e.g., "Home")
```
┌─────────┐
│  Home   │ ← Cyan underline (2px)
│  ════   │ ← Blue glow background
└─────────┘
```
- Background: `rgba(0, 212, 255, 0.15)`
- Border-bottom: `2px solid #00d4ff`
- Color: White (`#ffffff`)
- Tooltip: "Click to expand Home tools"

### Inactive Tab (e.g., "Review")
```
┌─────────┐
│ Review  │ ← No underline
│         │ ← Transparent background
└─────────┘
```
- Background: `transparent`
- Border-bottom: `2px solid transparent`
- Color: Gray (`#a8a8a8`)
- Hover: Light gray background

---

## Button States

### Collapse Button (Expanded State)
```
┌───┐
│ ↓ │  ← Chevron pointing down
└───┘
```
- Icon: `ChevronDown`
- Transform: `rotate(180deg)`
- Tooltip: "Collapse Toolbar"

### Collapse Button (Collapsed State)
```
┌───┐
│ ↑ │  ← Chevron pointing up
└───┘
```
- Icon: `ChevronDown`
- Transform: `rotate(0deg)`
- Tooltip: "Expand Toolbar"

---

## Contextual Tools by Tab

### Home Tab (Expanded)
```
┌────────────────────────────────────┐
│  📝 Edit │ 👁️ View     ✨ AI On  │
│   Mode              Assistance     │
└────────────────────────────────────┘
```

### Review Tab (Expanded)
```
┌───────────────────────────────────────────────────────────────┐
│  📜 Versions │ ⚖️ Compare     💬 Comments │ 🔀 Track │ ✅ Review │
│   Versions                     Collaboration                   │
└───────────────────────────────────────────────────────────────┘
```

### AI Tools Tab (Expanded)
```
┌───────────────────────────────────────────────────┐
│  🔍 Analyze │ ⚡ Deep     ✨ Generate │ 💡 Suggest │
│   AI Analysis               AI Generation          │
└───────────────────────────────────────────────────┘
```

### More Tab (Expanded)
```
┌──────────────────────────────┐
│  ✏️ Rename │ ⬇️ Download     │
│   Document Actions           │
└──────────────────────────────┘
```

---

## Animation Details

### Collapse Animation
```css
transition: all 200ms cubic-bezier(0, 0, 0.2, 1)

Properties animating:
├─ opacity: 1 → 0
├─ maxHeight: 48px → 0px
├─ padding: 8px 24px → 0px 24px (implicit)
└─ display: flex → none (after animation)
```

### Chevron Rotation
```css
transition: transform 150ms cubic-bezier(0, 0, 0.2, 1)

States:
├─ Expanded: rotate(180deg) → Points down (▼)
└─ Collapsed: rotate(0deg)  → Points up (▲)
```

### Tab Background (Collapsed)
```css
transition: all 150ms cubic-bezier(0, 0, 0.2, 1)

Active Tab:
├─ Background: transparent → rgba(0, 212, 255, 0.15)
└─ Creates subtle blue glow effect
```

---

## Responsive Behavior

### Desktop (>1024px)
- Full 3-row layout when expanded
- 2-row layout when collapsed
- All animations smooth 60fps

### Tablet (768px - 1024px)
- Same behavior as desktop
- Touch-friendly collapse button (larger tap target)
- Collapse provides more benefit due to limited screen space

### Mobile (<768px)
- May default to collapsed (future enhancement)
- Swipe gestures to toggle (future enhancement)
- Simplified tab labels

---

## Color Palette

### Toolbar Background
```
Row 1: linear-gradient(
  to bottom,
  rgba(26, 26, 30, 0.95),  ← Top
  rgba(22, 22, 26, 0.98)   ← Bottom
)

Row 2: Same as Row 1

Row 3: rgba(10, 10, 11, 0.4)  ← Slightly darker
```

### Borders
```
Row 1-2 separator: 1px solid rgba(255, 255, 255, 0.08)
Row 2-3 separator: 1px solid rgba(255, 255, 255, 0.05)
Group dividers:    1px solid rgba(255, 255, 255, 0.1)
```

### Active Elements
```
Active tab underline: #00d4ff (Cyan)
Active tab bg (collapsed): rgba(0, 212, 255, 0.15)
Active tab bg (expanded): rgba(255, 255, 255, 0.1)
Hover background: rgba(255, 255, 255, 0.05)
```

---

## Spacing & Dimensions

### Toolbar Rows
```
Row 1 (Document Info):
├─ Height: 44px
├─ Padding: 8px 24px
└─ Items: 12px gap

Row 2 (Tab Navigation):
├─ Height: 40px
├─ Padding: 0px 24px 8px
└─ Tabs: 4px gap

Row 3 (Contextual Tools):
├─ Height: 48px (when expanded)
├─ Padding: 8px 24px
└─ Groups: 12px gap, Actions: 4px gap
```

### Button Sizes
```
Tab buttons:
├─ Padding: 4px 12px
├─ Font: 14px (sm)
└─ Border-radius: 6px 6px 0 0

Action buttons:
├─ Padding: 6px 12px (with label)
├─ Padding: 6px (icon only)
├─ Font: 14px (sm)
└─ Border-radius: 6px

Collapse button:
├─ Padding: 6px
├─ Icon size: 14px
└─ Border-radius: 6px
```

---

## User Feedback

### Visual Cues
1. **Chevron Direction**: Indicates current state
   - ↓ = Expanded (can collapse)
   - ↑ = Collapsed (can expand)

2. **Active Tab Highlight**: Shows which tools are available
   - Blue glow when collapsed
   - White background when expanded

3. **Smooth Animations**: Provides continuity
   - 200ms for row collapse/expand
   - 150ms for chevron rotation
   - Eased timing function

4. **Tooltips**: Explain actions
   - "Collapse Toolbar" / "Expand Toolbar"
   - "Click to expand [Tab] tools" (when collapsed)

### Interaction Feedback
1. **Hover States**: All buttons have hover feedback
2. **Focus States**: Keyboard navigation visible
3. **Active States**: Pressed button shows feedback
4. **Loading States**: Save button shows spinner when saving

---

## Accessibility Features

### Screen Reader Announcements
```
Collapse: "Toolbar collapsed. Tools hidden."
Expand:   "Toolbar expanded. Tools visible."
Tab change: "Review tab active. Showing review tools."
```

### Keyboard Navigation
```
Tab → Focus next element
Shift+Tab → Focus previous element
Enter/Space → Activate focused button
Arrow keys → Navigate between tabs (future)
```

### ARIA Attributes
```html
<button
  aria-label="Collapse toolbar"
  aria-expanded="true"
  aria-controls="toolbar-content"
>
  ↓
</button>
```

---

## Conclusion

The collapse feature provides a clean, familiar way to maximize document viewing space while maintaining access to essential controls. The implementation follows Microsoft Word's design patterns, making it immediately intuitive for users.
