# Workspace TabStrip (editor-grade) v1 — Parity Matrix

Legend:

- ✅ implemented + gated (unit tests and/or `fretboard diag` scripts)
- 🟡 implemented but gaps / not gated
- ❌ missing

| Feature / invariant | Workspace TabStrip (Fret) | Docking TabBar (Fret) | Zed | dockview | gpui-component |
|---|---:|---:|---:|---:|---:|
| Select tab | 🟡 | 🟡 | ✅ | ✅ | ✅ |
| Close tab (button) | ✅ | 🟡 | ✅ | ✅ | ✅ |
| Dirty indicator | 🟡 | ❌ | ✅ | 🟡 | 🟡 |
| Preview tab semantics | 🟡 | ❌ | ✅ | ❌ | ❌ |
| Pinned tabs + boundary | 🟡 | ❌ | ✅ | ❌ | ❌ |
| Reorder within strip | ✅ (smoke) | 🟡 | ✅ | ✅ | ✅ |
| Cross-pane move | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| End-drop surface (`insert_index == tab_count`) | ✅ | 🟡 | ✅ | ✅ | ✅ |
| Header-space drop surface | 🟡 | 🟡 | ✅ | ✅ | 🟡 |
| Overflow detection | 🟡 | 🟡 | ✅ | ✅ | 🟡 |
| Overflow menu/list | ✅ (unit) | 🟡 | ✅ | ✅ | ❌ |
| Canonical index mapping under overflow | 🟡 | ❌ | ✅ | ✅ | 🟡 |
| Scroll-to-active | 🟡 | 🟡 | ✅ | 🟡 | 🟡 |
| Edge auto-scroll during drag | 🟡 | 🟡 | ✅ | 🟡 | 🟡 |
| Drag-to-split integration | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| Keyboard roving focus | 🟡 | ❌ | ✅ | 🟡 | 🟡 |
| Focus tab strip (command) | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Focus restore after close/move | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Diagnostics gates (scripts) | ✅ | 🟡 | n/a | n/a | n/a |

Notes:

- “Implemented” is not enough; the goal is **implemented + gated** so refactors are fearless.
- Parity targets differ by layer:
  - workspace/editor: Zed semantics
  - docking: dockview drop-surface + overflow pipeline concepts
