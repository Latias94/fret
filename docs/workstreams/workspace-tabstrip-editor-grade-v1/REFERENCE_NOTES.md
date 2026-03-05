# Workspace TabStrip (editor-grade) v1 (Reference Notes)

This file is a “where to look” index for reference implementations and what each is best at
teaching us (behavior outcomes, not pixels).

See also:

- `GAP_ANALYSIS.md` (prioritized gaps vs references)

## In-repo references (`repo-ref/`)

### Zed (editor semantics)

Primary reference for editor-grade outcomes:

- pinned tabs and pinned boundary semantics
- preview tab semantics
- focus-neutral close/reorder behaviors
- drop-to-split expectations
- end-drop target as a real surface (a flex-grow “tab_bar_drop_target” at the end of the scroll row), which is a good
  reminder to keep close-button hit testing stable so end-drop does not accidentally win clicks when the active tab is
  partially visible.

Evidence anchor:

- `repo-ref/zed/crates/workspace/src/pane.rs`
  - `render_two_row_tab_bar(...)` (separate pinned row mode)
  - `render_tab_bar_drop_target(...)` (`id("tab_bar_drop_target")` flex-grow end-drop surface)
  - `render_pinned_tab_bar_drop_target(...)` (pinned row end-drop surface + pinned boundary styling)
  - `handle_drag_move(...)` (split-zone preview routing; useful for row-suppression expectations)

### gpui-component (wiring shape + end-drop surface)

Good for minimal shapes:

- dock/tab panel wiring (separation between model, view, and hit testing)
- explicit “empty space to drop at end” concept

Evidence anchors:

- `repo-ref/gpui-component/crates/ui/src/dock/tab_panel.rs`
- `repo-ref/gpui-component/crates/ui/src/tab/tab.rs`

### dockview (overflow pipeline + drop surfaces + tests)

Good for:

- overflow as a pipeline (visible set vs overflow list)
- treating “header space” as a drop surface (not just gaps between tabs)
- invariants-first test strategy for tab containers

Evidence anchors:

- `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`
- `repo-ref/dockview/packages/dockview-core/src/__tests__/dockview/components/titlebar/tabsContainer.spec.ts`
- `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/voidContainer.ts`
  - `kind: 'header_space'` overlay events (header space is a first-class drop surface)

## External OSS references (not vendored)

These are useful as “behavior checklists” even if we do not vendor the sources:

- VS Code: editor tab semantics (preview, dirty indicator, pinned), keyboard + focus rules.
  - Anchor (tab control + overflow + close/pin wiring): `src/vs/workbench/browser/parts/editor/editorTabsControl.ts`
  - Anchor (tab model + group semantics): `src/vs/workbench/common/editor/editorGroupModel.ts`
- Monaco: the editor itself; tab semantics are primarily in VS Code (Monaco alone is not a “workspace shell”).
- Eclipse Theia: similar to VS Code tab/workbench behaviors (useful alternative viewpoint on command routing + focus).
- Jupyter Lumino: docking/tab strip behaviors and hit-testing patterns (strong reference for dock layout + drag/drop).
- GoldenLayout: docking/tab container UX patterns (split/drop targets).

We should treat these as outcome references and encode the outcomes as:

- unit-tested invariants (kernel),
- `fretboard diag` scripts (full-stack integration),
- and only then visual polish.
