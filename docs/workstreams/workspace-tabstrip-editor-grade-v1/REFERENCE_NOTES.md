# Workspace TabStrip (editor-grade) v1 (Reference Notes)

This file is a “where to look” index for reference implementations and what each is best at
teaching us (behavior outcomes, not pixels).

## In-repo references (`repo-ref/`)

### Zed (editor semantics)

Primary reference for editor-grade outcomes:

- pinned tabs and pinned boundary semantics
- preview tab semantics
- focus-neutral close/reorder behaviors
- drop-to-split expectations

Evidence anchor:

- `repo-ref/zed/crates/workspace/src/pane.rs`

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

## External OSS references (not vendored)

These are useful as “behavior checklists” even if we do not vendor the sources:

- VS Code / Monaco: editor tab semantics (preview, dirty indicator, pinned), keyboard + focus rules.
- Eclipse Theia: similar to VS Code tab/workbench behaviors.
- Jupyter Lumino: docking/tab strip behaviors and hit-testing patterns.
- GoldenLayout: docking/tab container UX patterns (split/drop targets).

We should treat these as outcome references and encode the outcomes as:

- unit-tested invariants (kernel),
- `fretboard diag` scripts (full-stack integration),
- and only then visual polish.

