# Workspace TabStrip (editor-grade) v1 — Open questions

This file captures decisions that affect long-term behavior and scriptability. Prefer answering
these with contracts + gates rather than ad-hoc implementation tweaks.

## Q1: Should “drag to split” be allowed while the pointer is still in the tab strip row?

Why it matters:
- Users expect “tab strip row” drags to mean reorder / move-to-pane.
- Allowing split zones in the same row tends to cause accidental splits near the pane edges.

Recommendation:
- **No**. Treat the tab strip row as a “center-only” zone for split preview purposes.
- Require the pointer to leave the tab strip row (into the pane content bounds) before split zones
  can activate.

Gate:
- Keep `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot` (screenshot) and add
  an invariants-based split gate once preview routing is stable.

Status:
- Implemented (best-effort) in workspace pane drop-zone arbitration so split zones do not latch
  while the pointer is still over the tab strip row.

## Q2: Should diagnostics scripts rely on pixel deltas for editor-grade drag interactions?

Why it matters:
- `set_window_inner_size` is best-effort; actual window bounds can differ across runners/OS.
- Large deltas can overshoot window bounds, preventing edge-margin logic from triggering.

Recommendation:
- Prefer `drag_to` when a stable target exists.
- Prefer `drag_pointer_until` for “find the edge preview” style gates.
- Only use raw `pointer_move` deltas for short, local motions.

## Q3: Where should “keep drag session position fresh” live long-term?

Today:
- The tab strip interaction layer updates `DragSession::position` defensively during drag moves.

Recommendation:
- Treat this as a **mechanism** concern. Long-term, move “drag session position tracking” into the
  DnD/runtime layer (so non-tab surfaces don’t depend on tab-strip-local pointer capture).
- Until then, keep the current workaround and gate it via scripts.

## Q4: What is the contract between “roving focus” and “tab cycling”?

Why it matters:
- Users expect arrow keys in the tab strip to move focus *visually* (in-order).
- Editors often bind `workspace.tab.next/prev` to cycle MRU (or at least not be strictly in-order).

Recommendation (default for v1):
- **ArrowLeft/ArrowRight** in the focused tab strip: in-order roving focus + **automatic activation**
  (APG Tabs-style).
- `workspace.tab.next/prev` commands: keep delegating to `WorkspaceTabs` `cycle_mode` (default MRU),
  and document this as a workspace policy surface (not a `fret-ui` runtime behavior).

Gate:
- Unit: `ecosystem/fret-workspace/tests/tab_strip_keyboard_roving_arrow_activates_tab.rs`
  (focus moves + `workspace.tab.activate.<id>` is dispatched).

## Q5: Should focus restore after close rely on “temporarily focusable” non-active tabs?

Why it matters:
- Our tab strip uses a roving focus policy where only the active tab is focusable.
- When closing the active tab, the predicted next tab is typically **not** focusable until the
  selection has updated (next frame).

Options:
1) Temporarily make the predicted next tab focusable in the same frame (render-time policy).
2) Defer focus restore to a timer tick (post-command, after selection update).

Recommendation:
- Prefer **(2)** for v1: keep the roving focus policy simple and deterministic, and use a
  best-effort deferred focus attempt (timer + retries) to restore focus within the strip.

Gate:
- Keep `ecosystem/fret-workspace/tests/tab_strip_focus_restore_after_close_command.rs` green and
  add a diagnostics script if this becomes flaky across runners.

## Q6: What is the contract for “entering the tab strip” from pane content?

Why it matters:
- Pointer-down intentionally does **not** steal focus from the active editor surface (editor-grade).
- Keyboard users still need a deterministic way to move focus into the tab strip to use roving
  navigation, close/pin via menus, etc.

Options:
1) Add a pane-scoped command (e.g. `workspace.pane.focus_tab_strip`) that can be invoked anywhere
   inside the pane subtree.
2) Use focus traversal (`focus.next/previous`) and rely on tab order (fragile when panes contain
   many focusables).
3) Introduce a new mechanism surface (e.g. “focus handles”) to query/restore focus (bigger contract).

Recommendation (v1):
- Prefer **(1)**. Implement `workspace.pane.focus_tab_strip` as a policy-level command that focuses
  the active tab in the pane's `WorkspaceTabStrip`.
- Keep “exit tab strip back to editor surface” as an app-owned decision for now (depends on editor
  widget focus semantics).
- Default keybinding: bind `Ctrl+F6` to `workspace.pane.toggle_tab_strip_focus` (can be overridden
  via keymap layering).

Gate:
- Unit: `ecosystem/fret-workspace/tests/pane_focus_tab_strip_command_focuses_active_tab.rs`.

## Q7: What is the contract for “exiting the tab strip” back to pane content?

Why it matters:
- Keyboard users need a deterministic way to get back to the editor/content surface after using
  roving navigation in the tab strip.
- We cannot rely on pointer-driven focus transfer because pointer-down is intentionally focus-neutral.

Options:
1) `Escape` exits the tab strip and restores the focus target that was focused before entering.
2) `ArrowDown` exits (like some editor shells), `Escape` stays within chrome.
3) Use generic focus traversal (`focus.next/previous`) and accept non-determinism.

Recommendation (v1):
- Prefer **(1)**: `Escape` dispatches `workspace.pane.focus_content`.
- Implement `workspace.pane.focus_content` as a **shell policy** in `WorkspaceCommandScope` that
  restores the last focus target observed before `workspace.pane.focus_tab_strip` was invoked.
- If no prior focus target is known, fall back to a pane-registered “content focus target”
  (best-effort). See `WorkspacePaneContentFocusTarget`.

Gate:
- Unit: focus `outside` → `workspace.pane.focus_tab_strip` → `workspace.pane.focus_content`
  restores focus to `outside`.
- Unit: if the tab strip is focused but no return target was recorded,
  `workspace.pane.focus_content` falls back to the registered pane content target:
  `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_fallbacks_to_registered_pane_content.rs`.

## Q8: Should `Ctrl+F6` be a one-way “focus tab strip” or a toggle?

Why it matters:
- A one-way command requires users to learn a separate “exit” gesture.
- Editors commonly use a “cycle focus” / “toggle focus” chord that is symmetric.

Recommendation (v1):
- Bind `Ctrl+F6` to `workspace.pane.toggle_tab_strip_focus`.
- Keep `workspace.pane.focus_tab_strip` and `workspace.pane.focus_content` as explicit, scriptable
  commands (useful for automation and future command palette integration).

Gates:
- `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_toggles_between_content_and_tab_strip.rs`
- `ecosystem/fret-workspace/tests/workspace_commands_default_keybindings_include_ctrl_f6_toggle.rs`
