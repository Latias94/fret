# Docking TabBar Fearless Refactor v1 (Parity Matrix)

Status legend:

- ✅ implemented + gated
- 🟡 implemented (not gated)
- ❌ missing
- 🧪 experimental / refactor in progress

| Feature | Fret docking TabBar | Fret workspace tab strip | Zed | gpui-component | dockview | VS Code |
|---|---:|---:|---:|---:|---:|---:|
| Drop at end (insert_index == tab_count) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cross-pane tab move | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ |
| Drag-to-split from tab/content | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ |
| Overflow dropdown / menu | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ |
| Auto-scroll while dragging | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ |
| Pinned tabs | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| Preview tabs | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| Keyboard navigation | ❌ | ✅ | ✅ | 🟡 | 🟡 | ✅ |
| Focus restore invariants | ❌ | ✅ | ✅ | 🟡 | 🟡 | ✅ |
| Context menu | 🟡 | 🟡 | ✅ | ❌ | ✅ | ✅ |

Notes:

- Docking TabBar is currently **interaction-gated** (diag scripts + unit tests) for drop resolution,
  overflow click arbitration, and edge auto-scroll while dragging.
- Workspace tab strip has broader “editor” coverage (keyboard + focus restore) because it is a
  focusable UI element in the workspace shells.
