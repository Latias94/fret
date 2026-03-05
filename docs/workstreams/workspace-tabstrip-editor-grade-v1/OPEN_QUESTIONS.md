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
- Keep `workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke` as a non-screenshot guardrail:
  it asserts that the right-edge split preview does not activate until the pointer leaves the tab strip row.

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
- For hover-driven edge preview routing, prefer `set_cursor_in_window_logical` (runner cursor override) plus a tiny
  `pointer_move` nudge to keep the cursor **inside** the window while still triggering hover updates.
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

## Q9: Should “pane content focus target” be a required seam for editor shells?

Why it matters:
- Pointer interactions in chrome are intentionally focus-neutral; focus may enter the tab strip via
  non-command paths (tests, automation, or future focus traversal rules).
- `workspace.pane.focus_content` / `Ctrl+F6` toggle should still have a deterministic “exit” target.

Options:
1) Require shells to register a focusable “content target” per pane (recommended).
2) Keep it best-effort and allow `focus_content` to no-op if no return target exists.

Recommendation (v1):
- Prefer **(1)** for editor shells: register a focusable content target per pane using
  `WorkspacePaneContentFocusTarget`.
- Treat missing registration as a soft failure (no hard panic), but gate the behavior in unit
  tests so it does not regress.

Gates:
- `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_fallbacks_to_registered_pane_content.rs`.

## Q10: Which upstream should be treated as the “primary” reference for tab strip outcomes (Zed vs dockview vs gpui-component)?

Why it matters:
- If we pick the wrong primary reference, we either overfit to DOM/CSS details (dockview) or miss overflow/drop-surface
  invariants (Zed).
- The choice affects which gaps are considered “P0” and which behaviors must be locked down first.

Recommendation:
- Treat **Zed** as the primary reference for **editor semantics** (pinned/preview/close policies + focus invariants).
- Treat **dockview** as the primary reference for **overflow + drop-surface mechanics + tests** (void/header drop surface,
  overflow pipeline discipline).
- Treat **gpui-component** as a reference for **wiring patterns**, not for final editor-grade behavior.

Status:
- This reference split is already captured in `REFERENCE_NOTES.md` and `GAP_ANALYSIS.md`; keep it explicit so “fearless
  refactors” do not degrade editor outcomes while chasing dockview-style mechanics.

## Q11: What is the contract for “close button click vs tab activation vs tab drag” under pointer jitter?

Why it matters:
- Close affordances are tiny and often adjacent to drag affordances.
- In overflowed strips, the active tab width can change when the close button becomes visible, which can make the close
  hit-test unstable (partially visible tab, end-drop surface overlaps, etc.).
- We want editor-grade semantics: a close click should be focus-neutral and should not accidentally activate another tab
  or start a drag session.

Contract (v1 recommendation):
- On left button down (no modifiers), if the pointer is in the close affordance hit region:
  - capture the pointer and arm a `close_press` state (per-pointer),
  - clear any in-scope tab DnD state for that pointer,
  - stop propagation so the tab pressable does not arm activation or drag.
- On pointer up, if the pointer moved within a small slop (e.g. 8px in window space), dispatch close and return
  `SkipActivate` (do not activate a different tab as a side-effect).
- Keep the active tab **fully visible** (not just partially visible) when not dragging, so the close target remains
  reachable and diagnostics scripts remain stable under minor chrome width changes.

Gate:
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-start-drag.json`

## Q12: If we add a separate pinned row, how does it interact with overflow and “header space”?

Why it matters:
- Zed’s two-row mode changes the *user mental model* of where “drop at end” lives:
  - pinned row has its own end-drop surface,
  - unpinned row has its own end-drop surface,
  - the “boundary” becomes a visual separator instead of a single in-row drop target.
- If we keep a single overflow menu and a single scroll handle, we can accidentally make pinned
  row drops feel inconsistent (e.g. header space resolves to the wrong canonical insert index).

Options:
1) Pinned row never overflows: cap pinned count or shrink pinned tabs aggressively.
2) Pinned row is independently scrollable (Zed-style), but uses the same overflow menu policy as unpinned (complex).
3) Pinned row is independently scrollable, and pinned row has **no overflow menu** (recommended for v1).

Recommendation (v1):
- Prefer **(3)**:
  - Keep pinned row scroll as “best effort” (`overflow_x_scroll`) with **no overflow menu**.
  - Keep the unpinned row as the only place with overflow menu + scroll buttons.
  - Ensure both rows expose explicit end-drop surfaces (flex-grow header space) so drop semantics
    stay stable under future button clusters.

Gates:
- Add a pinned-row diag script that asserts:
  - end-drop in pinned row resolves `insert_index == pinned_count`,
  - end-drop in unpinned row resolves `insert_index == tab_count`,
  - header-space does not route into the overflow control surface.

## Q13: Why do tab context menus fail to open in `diag --launch` runs (right click)?

Observed:
- `click(button="right")` on a workspace tab does not reliably open the tab context menu in `diag --launch` runs.
  Scripts can time out waiting for menu items like `Pin Tab`.

Intended outcome:
- Determine whether the fix belongs in the diagnostics harness (event synthesis), the context-menu
  trigger wiring (pointer region), or the workspace tab strip (pointer propagation / capture).

Evidence anchors:
- Script: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-left-keeps-pinned-smoke.json`
- Workspace tab pointer policy: `ecosystem/fret-workspace/src/tab_strip/interaction.rs`
- Context menu open policy: `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`
- shadcn ContextMenu wrapper: `ecosystem/fret-ui-shadcn/src/context_menu.rs`
