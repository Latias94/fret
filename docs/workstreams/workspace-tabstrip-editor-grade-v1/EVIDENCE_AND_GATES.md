# Workspace TabStrip (editor-grade) v1 — Evidence & Gates

Goal: keep the tab strip refactor **fearless** by locking outcomes behind unit tests and
`fretboard diag` scripts (bounded artifacts, invariants-first).

## Evidence anchors (current)

- `ecosystem/fret-workspace/src/tab_strip/mod.rs` (workspace tab strip implementation)
- `ecosystem/fret-workspace/src/tab_strip/kernel.rs` (drop target computation)
- `ecosystem/fret-workspace/src/tab_strip/geometry.rs` (tab rect collection / hit testing helpers)
- `ecosystem/fret-workspace/src/tab_strip/utils.rs` (canonical end-drop resolution)
- `ecosystem/fret-workspace/tests/tab_strip_pointer_down_does_not_steal_focus.rs` (focus stability)
- `ecosystem/fret-workspace/tests/tab_strip_focus_restore_after_close_command.rs` (close focus restore)

## M1 gates (must exist before/while refactoring)

### Unit tests (kernel-level)

- `insert_index` is canonical (full list index space).
- end-drop surface resolves to `insert_index == tab_count`.
- pinned boundary prevents crossing on reorder (unless pin/unpin intent).
- overflow mapping preserves canonical order.

Location recommendation:

- `ecosystem/fret-workspace/src/tab_strip/kernel.rs` tests (pure-ish math)
- `ecosystem/fret-workspace/tests/*` for integration-level invariants (focus, command dispatch)

### Diagnostics scripts (full-stack)

Add promoted scripts under `tools/diag-scripts/workspace/**`:

1) Reorder within strip (invariants-first)
   - drag first tab to end and assert:
     - tab order updated
     - active tab unchanged unless moved

2) Close behaviors (smoke)
   - close active via button and assert deterministic next selection
   - middle-click close (if enabled by policy)

3) Overflow (resize) (invariants-first)
   - shrink tab strip width until overflow occurs
   - open overflow menu, activate a hidden tab, assert it becomes active and scrolls into view
   - promoted script:
     - `workspace-shell-demo-tab-overflow-activate-hidden-smoke`
   - drag-to-end reorder while overflowed:
     - `workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke`

4) Drag-to-split preview (bridge)
   - promoted scripts:
     - `workspace-shell-demo-tab-drag-to-split-right`
     - `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot`
   - notes:
     - prefer `drag_pointer_until` over large raw pixel deltas to avoid runner/window-size drift
     - the screenshot script may use `release_on_success: false` to keep the preview visible for
       `capture_screenshot` before releasing
   - replace screenshot-only gating with an invariants-based gate once the preview snapshot surface is stable

## Tooling checklist

- `python3 tools/check_diag_scripts_registry.py` passes after script registration/promotions.
- Avoid raw `bundle.json` by default; prefer schema2 + sidecars and `diag query/slice`.
