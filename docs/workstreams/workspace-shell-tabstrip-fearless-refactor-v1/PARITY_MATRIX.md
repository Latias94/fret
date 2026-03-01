# Workspace Shell TabStrip (Fearless Refactor v1) — Parity Matrix

Legend:

- **Yes**: implemented and considered baseline-stable
- **Partial**: present, but missing key UX details or gates
- **No**: not implemented
- **N/A**: not a goal / not applicable to that reference

| Capability | Zed (`repo-ref/zed`) | dockview (`repo-ref/dockview`) | gpui-component (`repo-ref/gpui-component`) | Fret (current) | Fret (target v1) |
|---|---:|---:|---:|---:|---:|
| Scrollable tabs + keep active in view | Yes | Yes | Yes | Yes | Yes |
| Wheel over tab bar scrolls tabs | Yes (policy + suppress parent scroll) | Yes (scroll container + observers) | Partial | Yes | Yes |
| Overflow dropdown/list | Partial (scroll-first, plus UX around scroll) | Yes | Yes (`menu(true)`) | Partial | Yes |
| Overflow membership detection | Yes | Yes (`OverflowObserver`) | Partial | Partial | Yes |
| Close button (tab) | Yes | Yes | Partial | Yes | Yes |
| Dirty indicator | Yes | N/A | N/A | Yes | Yes |
| Context menu (close/close others/close left/right) | Yes | Partial | Partial | Partial (feature-flagged) | Yes |
| Keyboard nav (roving, APG-like) | Yes | Partial | Partial | Yes | Yes |
| Focus tab strip (command) | Yes | N/A | Partial | Yes (unit) | Yes |
| Toggle tab strip focus (Ctrl+F6) | Yes | N/A | Partial | Yes (unit) | Yes |
| Exit tab strip (Escape → content) | Yes | N/A | Partial | Yes (unit) | Yes |
| Do not steal editor focus on click (editor chrome rule) | Yes | N/A | N/A | Yes | Yes |
| Drag reorder within strip | Yes | Yes | Yes (dock tab panel) | Yes | Yes |
| “Drop after last tab” explicit target | Yes | Yes (header space) | Yes (empty space) | Yes | Yes |
| Cross-pane move (drag to other pane) | Yes | Yes | Yes | Partial | Yes |
| Drag-to-split (edge targets) | Yes | Yes | Partial | Partial | Yes |
| Pinned tabs | Yes | No | No | Partial | Yes |
| Separate pinned row | Yes (optional setting) | No | No | No | Optional |
| Preview tabs (single preview per pane) | Yes | No | No | Yes | Yes (recommended) |
| MRU tab switch (Ctrl+Tab) | Yes | No | No | Partial | Yes |
| Stable automation hooks (`test_id`/selectors) | Yes | Partial (DOM selectors) | Partial | Partial | Yes |

Notes:

- Zed is the reference for **editor semantics** (pinned/preview/MRU/drag-to-split outcomes).
- dockview is the reference for **overflow dropdown/list pipeline** and header-space drop UX.
- gpui-component is a useful intermediate for “GPUI-native” wiring patterns but is not a complete
  editor semantics reference.
