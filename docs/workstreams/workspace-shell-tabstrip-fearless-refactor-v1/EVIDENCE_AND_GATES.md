# Workspace Shell TabStrip (Fearless Refactor v1) — Evidence & Gates

This document turns milestones into concrete, reviewable evidence and regression protection.

## Stable `test_id` conventions (proposal)

We should keep selectors stable across refactors. Suggested shapes:

- Root:
  - `{root}` (via `WorkspaceTabStrip::test_id_root`)
- Per-tab:
  - tab trigger: `{tab_prefix}-{tab_id}` (via `WorkspaceTabStrip::tab_test_id_prefix`)
  - tab chrome container: `{tab_prefix}-{tab_id}.chrome`
  - close button: `{tab_prefix}-{tab_id}.close`
  - dirty indicator: `{tab_prefix}-{tab_id}.dirty`
- Overflow:
  - overflow button: `{root}.overflow_button`
  - overflow entry: `{root}.overflow_entry.{tab_id}`
  - overflow entry close (future): `{root}.overflow_entry.{tab_id}.close`
- Drop targets:
  - end-of-strip: `{root}.drop_end`
  - pinned boundary: `{root}.drop_pinned_boundary`
  - pinned row border (if separate row, TODO): `{root}.drop_pinned_row`
- Drag-to-split (workspace panes):
  - drop preview overlay: `workspace-pane-{pane_id}.drop_preview.{zone}`
    - `{zone}`: `left | right | up | down | center`

Notes:

- Prefer `{tab_id}` over indices to keep automation stable under reorder.
- When `{tab_id}` contains slashes/spaces, normalize (e.g. replace non-alnum with `_`).

## Test gates (unit/integration)

### Core state invariants (pure logic)

Add tests close to the kernel/module (or in `ecosystem/fret-workspace/tests/` if kept there):

- Reorder intent correctness matrix:
  - given rects + pointer positions, compute `(target_id, insertion_side)` deterministically.
- “Drop end” target:
  - dropping in empty space produces “insert at end” intent.
- Pinned boundary:
  - pin/unpin updates `pinned_tab_count` and preserves active tab.
- Edge auto-scroll:
  - pointer near left/right edges produces deterministic scroll deltas.
  - prefer shared helper coverage: `ecosystem/fret-dnd/src/scroll.rs` (`compute_autoscroll_x/y`)
- Preview tab:
  - open previewable item replaces existing preview tab slot.
- MRU:
  - toggling MRU between two most recent remains stable under close/reorder.

### UI wiring gates (runtime behavior)

Prefer nextest tests for “hard” behaviors that do not require real rendering:

- Focus stability:
  - pointer down on tab does not steal focus from an existing focus target.
- Focus transfer:
  - `workspace.pane.focus_tab_strip` focuses the active tab in the focused pane.
  - `workspace.pane.focus_tab_strip` works when focus starts outside the pane subtree (shell scope).
  - `workspace.pane.focus_content` restores the pre-tabstrip focus target after keyboard use of the strip.
  - Default keybinding suggestion: `Ctrl+F6` bound to `workspace.pane.toggle_tab_strip_focus`
    (apps can override via keymap layering).
  - Roving keyboard navigation:
    - arrow keys move roving focus and activate correct tab.

## Diag gates (interaction-heavy)

For drag/drop and overflow UX, scripted `fretboard-dev diag` gates are preferred:

### Script gates (current)

- Suite:
  - `diag-hardening-smoke-workspace` (promoted via `tools/diag-scripts/index.json`)
- Workspace shell demo:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-cross-pane-move-to-end.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-text-during-dock-drag-stability.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json`
    (gates `source_kind=pointer` for `workspace.tab.close.doc-a-0`)
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-mru-fallback-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`

### U9 workspace owner and diagnostics closeout (2026-07-12)

The model-command path has one window owner. `WorkspaceWorkbench` applies tab/pane/dirty-close
transactions before retained UI dispatch; `WorkspaceCommandScope` remains the widget owner for
focus-only commands and publishes the tab-strip/content focus lane used by Workbench. The
app-facing `WorkspaceApp` receives `PostFrameUiFocusLifecycle` and driver-trace finalization through
`UiAppDriver`; UI Gallery's explicit advanced driver calls the same
`PostFrameUiFocusLifecycle` and `record_driver_handled_command_dispatch` contracts directly. This
keeps deferred focus ordering, original command source, window scope, `handled_by_driver`, and typed
`CommandDispatchOutcomeV1` evidence aligned.

Modal authorization is derived from the live retained tree. `UiAppCommandBeforeUiContext` carries
the FIFO-preserved source plus `UiTree` input-barrier membership without exposing the raw tree to
the policy owner. Workbench accepts only direct pointer/keyboard model actions proven inside the
active barrier; shortcut, programmatic, missing, stale, underlay, focus-only, and window-close
routes remain blocked or delegated to modal-aware retained UI dispatch. Regression anchors are
`active_input_barrier_scope_accepts_modal_and_higher_layer_elements_only`,
`repeated_same_command_preserves_fifo_sources_across_app_hook_cleanup`, and
`modal_snapshot_defers_external_workspace_commands_but_allows_active_ui_actions`.

Last-tab behavior is policy, not a changed workspace default:

- `WorkspaceLastTabClosePolicy::AllowEmptyPane` remains the default general-purpose behavior.
- UI Gallery opts into `WorkspaceLastTabClosePolicy::PreserveLastTab` because its active route must
  always project from a live workspace tab.
- The policy covers close, close-by-id, explicit-pane close, and dirty-close replay; a protected
  final close leaves layout and router projection unchanged while the trace records a handled no-op
  (`handled_by_driver=true`, `applied=false`).

Passing suite artifacts:

| Gate | Result | Session / representative run | Artifact root |
| --- | --- | --- | --- |
| `ui-gallery-workspace-shell` | 4/4 passed | `1783801365683-65846` / command run `1783801375847` | `target/fret-diag/u9-ui-gallery-workspace-shell-final-20260712` |
| `workspace-shell-app-facing` | 10/10 passed | `1783801123285-47715` / final `1783801150929` | `target/fret-diag/u9-workspace-shell-app-facing-final-rerun-20260712` |

Exact rerun commands:

```bash
cargo build -p fretboard-dev
cargo build -p fret-demo --bin workspace_shell_demo --release
cargo build -p fret-ui-gallery --release
target/debug/fretboard-dev diag suite workspace-shell-app-facing \
  --dir target/fret-diag/u9-workspace-shell-app-facing-final-rerun-20260712 \
  --session-auto --timeout-ms 240000 --launch -- target/release/workspace_shell_demo
target/debug/fretboard-dev diag suite ui-gallery-workspace-shell \
  --dir target/fret-diag/u9-ui-gallery-workspace-shell-final-20260712 \
  --session-auto --timeout-ms 240000 --launch -- target/release/fret-ui-gallery
```

The corrected 1280 px overflow/reveal matrix also passed independently: activate hidden
`1783785518551`, close without activation `1783785462415`, reorder `1783785525133`, held-edge
autoscroll `1783785831206`, keyboard End reveal `1783785899905`, and Home restore
`1783785903189`. These scripts use a valid center viewport above the fixed-rail minimum and stable
overflow selectors rather than interacting with offscreen tab bounds.

The matrix artifacts remain under `target/fret-diag-gpui-u9-overflow-matrix-20260711` (activate,
reorder), `target/fret-diag-gpui-u9-overflow-close-trace-adjacent-20260711` (close),
`target/fret-diag-gpui-u9-autoscroll-edge-held-20260711` (autoscroll), and
`target/fret-diag-gpui-u9-keyboard-reveal-overflow-setup-20260711` (End/Home reveal).

The Gallery suite proves shared chrome plus its direct Workbench owner/driver trace. It is not a
replacement for `workspace-shell-app-facing`, which exercises the real `WorkspaceApp` frame,
split/move, dirty-close, focus, keyboard, semantics, and diagnostics chain.

Full U9 Rust gates passed. The final post-Clippy package reruns and the screenshot fallback
regression are listed separately so a combined run is not mistaken for evidence captured after a
later focused cleanup:

| Package / feature set | Result | Nextest run |
| --- | --- | --- |
| `fret-runtime + fret-diag-protocol + fret-ui --features diagnostics` | 1694/1694 | `08b56e09-6ad7-443a-9347-d79a965c8aaa` |
| `fret-bootstrap --features ui-app-driver,diagnostics + fret-workspace` | 343/343 | `2c609791-3139-4865-bdfe-394c4b786648` |
| `fret --features workspace + fret-examples` | 450/450 | `611a908d-32f9-40ce-b2b1-309ae011fe0d` |
| `fret-bootstrap --features ui-app-driver,diagnostics` (post-Clippy) | 207/207 | `e8c22844-475c-4294-bdf1-e038a2e1adff` |
| `fret --features workspace` (post-Clippy) | 194/194 | `15cd1955-5fb7-4e4c-ad89-25c93abbbd59` |
| `fret-ui-shadcn` | 1937/1937 (1 configured skip) | `7c55223b-da53-4138-b3ef-6a3ef8d8387f` |
| `fret-ui-gallery` | 851/851 (3 configured skips; 1 slow) | `1e97210a-7124-4abb-8ab4-0bf7110b6e56` |
| `fret-launch --features diag-screenshots` | 97/97 | `c1d137ed-393c-4c09-97d8-84a69575507d` |

Clippy passed with warnings denied for the affected default package set, `fret-ui` diagnostics,
`fret-bootstrap` diagnostics, `fret` workspace, and `fret-launch` `diag-screenshots` feature
combinations.

Standalone drag-time visual gate:

- `tools/check_workspace_tab_drag_visual_stability.sh`
  - runs the promoted stability script with a fixed frame delta
  - requires the target tab bounds to converge
  - uses `--check-pixels-unchanged workspace-shell-pane-pane-b-tab-strip` for selector-scoped
    decoded-pixel evidence
  - result: passed in session `1783801403552-66087`, run `1783801403865`, under
    `target/fret-diag/workspace-tab-drag-visual-stability-check`
  - `script.result.json` reports `stage=passed`; `check.pixels_unchanged.json` resolves frames 85
    and 214 to the same selector-scoped hash, `0xeb75ed91b5342007`

### Determinism knobs

When adding scripts, prefer:

- fixed frame delta (`meta.env_defaults.FRET_DIAG_FIXED_FRAME_DELTA_MS=16`) for animation stability
- stable `test_id` targeting over pixel coordinates where possible

## Evidence anchors (what reviewers should look at)

For each milestone PR, include 1–3 anchors:

- key functions (kernel ops / adapter wiring)
- tests / diag script IDs
- demo surface (UI Gallery page and/or `fretboard-dev dev` command)

Current anchors:

- Workspace tab strip adapter: `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- Tab strip interaction kernel (WIP): `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
- Shared tab reveal kernel and oversized-range idempotence gate:
  `ecosystem/fret-ui-headless/src/tab_strip_scroll.rs`
- Workspace model-command owner and last-tab policy:
  `ecosystem/fret-workspace/src/workbench.rs`
- UI Gallery Workbench installation and owner-first dispatch:
  `apps/fret-ui-gallery/src/driver/{window_bootstrap.rs,runtime_driver.rs}`
- Shared advanced-driver focus and command-trace helpers:
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Real `WorkspaceApp` owner-first composition:
  `ecosystem/fret/src/workspace.rs`
- Focus transfer and shell-scope gate:
  `ecosystem/fret-workspace/tests/workspace_command_scope_focus_tab_strip_from_outside_pane.rs`
- Exit tab strip gate: `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_restores_previous_focus.rs`
- Toggle focus gate: `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_toggles_between_content_and_tab_strip.rs`
- Toggle (multi-pane) gate: `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_multi_pane_returns_to_last_non_tabstrip_focus.rs`

Reference anchors:

- Zed pinned/preview/drop targets: `repo-ref/zed/crates/workspace/src/pane.rs`
- dockview overflow list pipeline:
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabs.ts`
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`
