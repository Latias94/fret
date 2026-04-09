# Workspace TabStrip (editor-grade) v1 — Parity Matrix

Legend:

- ✅ implemented + gated (unit tests and/or `fretboard-dev diag` scripts)
- 🟡 implemented but gaps / not gated
- ❌ missing

| Feature / invariant | Workspace TabStrip (Fret) | Docking TabBar (Fret) | Zed | dockview | gpui-component |
|---|---:|---:|---:|---:|---:|
| Select tab | ✅ (gate) | 🟡 | ✅ | ✅ | ✅ |
| Close tab (button) | ✅ | 🟡 | ✅ | ✅ | ✅ |
| Dirty indicator | 🟡 | ❌ | ✅ | 🟡 | 🟡 |
| Preview tab semantics | ✅ (unit) | ❌ | ✅ | ❌ | ❌ |
| Pinned tabs + boundary | ✅ (unit + gate) | ❌ | ✅ | ❌ | ❌ |
| Reorder within strip | ✅ (smoke) | 🟡 | ✅ | ✅ | ✅ |
| Cross-pane move | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| End-drop surface (`insert_index == tab_count`) | ✅ | 🟡 | ✅ | ✅ | ✅ |
| Header-space drop surface | ✅ (unit) | 🟡 | ✅ | ✅ | 🟡 |
| Overflow detection | ✅ (unit) | 🟡 | ✅ | ✅ | 🟡 |
| Overflow menu/list | ✅ (unit + gate) | 🟡 | ✅ | ✅ | ❌ |
| Canonical index mapping under overflow | ✅ (smoke) | ❌ | ✅ | ✅ | 🟡 |
| Scroll-to-active | ✅ (gate) | 🟡 | ✅ | 🟡 | 🟡 |
| Edge auto-scroll during drag | 🟡 | 🟡 | ✅ | 🟡 | 🟡 |
| Drag-to-split integration | ✅ | 🟡 | ✅ | ✅ | 🟡 |
| Keyboard roving focus | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Focus tab strip (command) | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Toggle tab strip focus (Ctrl+F6) | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Exit tab strip (Escape → content) | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Exit tab strip fallback (pane content target) | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Focus restore after close/move | ✅ (unit) | ❌ | ✅ | 🟡 | 🟡 |
| Diagnostics gates (scripts) | ✅ | 🟡 | n/a | n/a | n/a |

Notes:

- “Implemented” is not enough; the goal is **implemented + gated** so refactors are fearless.
- Parity targets differ by layer:
  - workspace/editor: Zed semantics
  - docking: dockview drop-surface + overflow pipeline concepts
