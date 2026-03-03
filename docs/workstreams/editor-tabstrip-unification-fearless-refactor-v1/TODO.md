# Editor TabStrip Unification Fearless Refactor v1 (TODO)

Goal: converge `workspace` tabstrip + `docking` tab bar behaviors onto shared mechanism vocabulary
(`fret-ui-headless`) while keeping policy in ecosystem layers.

Rolling log:
- `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/LOG.md`

This workstream is intentionally scoped to “editor-grade tab UX”:
- overflow behavior (membership, dropdown list, close affordances)
- drag & drop targets (tab halves, end-drop surfaces, overflow button exclusion)
- scroll/visibility guarantees (active tab stays visible)
- keyboard/focus semantics (editor expectations)

## Scope

- [x] Write a cross-reference parity matrix:
  - `repo-ref/zed` (editor pane tab bar, pinned/unpinned, scroll handle)
  - `repo-ref/dockview` (overflow list membership + dropdown behaviors)
  - Fret: `ecosystem/fret-workspace` + `ecosystem/fret-docking`
-   Output: `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/PARITY_MATRIX.md`
- [ ] Normalize terminology in code and docs:
  - “tabs viewport”, “header space”, “overflow control”, “end-drop surface”
- [ ] Decide overflow dropdown policy:
  - list overflowed-only vs overflowed+active (current docking policy)
  - include close buttons in overflow list (dockview has tests for this)
- [x] Enable overflow dropdown close parity (workspace + docking):
  - Close affordance is visible in overflow menu rows.
  - Clicking close dispatches close without implicitly activating.
  - Evidence: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` + workspace/docking tests.
- [ ] Add diag script coverage:
  - [x] Workspace: overflow dropdown close does not activate:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-close-does-not-activate.json`
  - [x] Workspace: close button does not activate inactive tab:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-activate.json`
  - [x] Docking: overflow dropdown open + select row activates:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-select-row-1-activates.json`
  - [x] Docking: close button does not activate inactive tab:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-close-button-does-not-activate.json`
  - [x] Workspace: close button does not start a tab drag:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-start-drag.json`
    - Predicate: `workspace_tab_strip_drag_active_is active=false pane_id=\"pane-a\"`
  - [x] Docking: close button does not start a dock drag:
    - Gate is embedded in `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-close-button-does-not-activate.json`
    - Predicate: `dock_drag_active_is active=false` after a small pointer move while pressed
  - [x] Select-from-dropdown keeps active visible (explicit assert / evidence):
    - Docking gate: `dock_tab_strip_active_visible_is visible=true` after selecting a row.
      - Script: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-select-row-1-activates.json`
    - Workspace gate: `workspace_tab_strip_active_visible_is visible=true pane_id="pane-a"` after selecting a row.
      - Script: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
    - Shared plumbing evidence:
      - Runtime snapshot: `crates/fret-runtime/src/interaction_diagnostics.rs`
      - Workspace publisher: `ecosystem/fret-workspace/src/tab_strip/mod.rs`
      - Bundle snapshot mapping: `ecosystem/fret-bootstrap/src/ui_diagnostics/workspace_diagnostics.rs`
      - Predicates + script runner wiring: `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`
      - Protocol predicate variants: `crates/fret-diag-protocol/src/lib.rs`
  - [x] Drag end-drop on overflow header space resolves canonical insert_index:
    - Workspace gate: drag first tab to end-drop while overflowed.
      - Script: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`
      - End-drop surface: `workspace-shell-pane-pane-a-tab-strip.drop_end`
      - Overflow precondition: `workspace_tab_strip_active_overflow_is overflow=true pane_id="pane-a"`
      - Verified: 2026-03-02 (PASS run_id=1772422873574)
  - [x] Cross-pane move-to-end drops into target pane:
    - Workspace gate: drag a tab from pane A onto pane B end-drop.
      - Script: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-cross-pane-move-to-end.json`
      - Target surface: `workspace-shell-pane-pane-b-tab-strip.drop_end`
      - Verified: 2026-03-02 (PASS run_id=1772422901186)
  - [x] In-pane reorder can land on the explicit end-drop surface:
    - Workspace gate: drag first tab to end (non-overflow).
      - Script: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
      - Verified: 2026-03-02 (PASS run_id=1772422846249)
- [x] Add minimal unit tests where headless helpers are used by adapters.

## Non-goals

- Component styling/token design beyond necessary hit rects and affordances.
- Replacing Fret’s retained tree with declarative rebuild (tracked elsewhere).
- Full APG/a11y closure (tracked in a11y workstreams); only tab-relevant keyboard/focus here.

## References

- Existing workstreams:
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/`
  - `docs/workstreams/docking-tabbar-fearless-refactor-v1/`
- Headless mechanism helpers:
  - `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`
  - `ecosystem/fret-ui-headless/src/tab_strip_overflow.rs`
