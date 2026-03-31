# imui authoring vocabulary closure v1 - TODO

Status: Historical tracker (partially superseded by `docs/workstreams/imui-stack-fearless-refactor-v2/`)

Status note (2026-03-31): read this file as the narrow historical gap board that existed before the
v2 baseline audit. The current execution board now lives in
`docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`.
All gap headings and "required outcomes" below are historical records of what this lane used to
track; they are not current repo gap claims anymore. The final shipped state for this lane is
captured in `docs/workstreams/imui-authoring-vocabulary-closure-v1/CLOSEOUT_AUDIT_2026-03-31.md`.

Last updated: 2026-03-31.

This tracker is workstream-local.
It recorded the remaining high-frequency immediate authoring vocabulary gaps that existed after the
stack reset, editor-grade helper closure, sortable recipe closure, and ghost closeouts.

## Fearless refactor rule

This lane does not require compatibility shims.

If a new helper becomes the correct canonical surface:

- migrate in-tree call sites,
- keep at most a tiny convenience wrapper when it is truly non-overlapping,
- otherwise delete the superseded helper.

## Closed elsewhere - do not reopen in this lane

These questions are already closed or intentionally owned elsewhere:

- stack reset / alias cleanup
- editor composites
- tooltip helper
- tree/collapsing helper
- typed drag/drop seam
- sortable/reorder recipe
- same-window drag-preview ghost
- generic cross-window ghost baseline
- shell-aware docking ghost choreography
- transparent moving-window payload overlap diagnostics

Primary evidence:

- `docs/workstreams/imui-stack-fearless-refactor-v1/`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/`
- `docs/workstreams/imui-sortable-recipe-v1/`
- `docs/workstreams/imui-drag-preview-ghost-v1/`
- `docs/workstreams/imui-cross-window-ghost-v1/`
- `docs/workstreams/imui-shell-ghost-choreography-v1/`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/`

## Historical tracker

### P0 - `selectable` family in `fret-ui-kit::imui`

Owner:

- `ecosystem/fret-ui-kit::imui`

Required outcomes:

- generic selected/unselected row helper for dense immediate lists,
- full-row hit box and stable identity,
- explicit disabled state,
- explicit popup-close policy instead of hard-coded menu semantics,
- response surface suitable for list selection and popup item activation.

Suggested gates:

- unit or smoke test for full-row hit behavior and stable id derivation,
- popup integration test for close-on-activate vs stay-open policy,
- proof/demo call site using the helper outside menus.

Status:

- Completed

Current slice landed:

- canonical `selectable(...)` / `selectable_with_options(...)` surface in
  `ecosystem/fret-ui-kit::imui`
- `SelectableOptions` owner split and default semantics role
- `combo_model_with_options(...)` reusing `selectable` rows for popup options
- focused regression artifacts:
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_selectable_smoke.rs`

### P0 - generic `begin_combo` / `combo` family in `fret-ui-kit::imui`

Owner:

- `ecosystem/fret-ui-kit::imui`

Required outcomes:

- preview + popup immediate helper with canonical open/close flow,
- body can host `selectable(...)` rows or small custom row content,
- focus restore and dismissal remain explicit and testable,
- current narrow `select_model` overlap is resolved instead of duplicated.

Cross-lane note:

- do not rebuild shadcn/Base UI `Select` or `Combobox` part surfaces here,
- reuse shared substrate where possible,
- keep this helper immediate-authoring shaped.

Suggested gates:

- open/close + focus restore test,
- selection commit test using `selectable(...)` rows,
- proof/demo showing a dense editor picker that no longer uses ad hoc popup radio wiring.

Status:

- Completed

Current slice landed:

- canonical `combo(...)` / `combo_with_options(...)` surface in `ecosystem/fret-ui-kit::imui`
- toggle-on-trigger-press popup flow instead of ad hoc trigger wiring per call site
- old `select_model` naming removed; model-backed convenience now lives on the combo path as
  `combo_model(...)` / `combo_model_with_options(...)`
- focused regression artifacts:
  - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_combo_smoke.rs`
  - `ecosystem/fret-imui/src/tests/models.rs`
    - `combo_popup_escape_closes_and_restores_trigger_focus`
    - `combo_can_commit_selection_with_selectable_rows`

### P1 - immediate table/columns wrapper

Owner:

- `ecosystem/fret-ui-kit::imui`

Required outcomes:

- header + row + column authoring vocabulary,
- clear distinction from the existing layout `grid(...)` helper,
- suitable for inspector-like or results-table surfaces without becoming a spreadsheet/data-grid.

Suggested gates:

- width negotiation / header-body alignment test,
- scrolling or clipping proof where table rows remain aligned.

Status:

- Completed

Current slice landed:

- canonical `table(...)` / `table_with_options(...)` immediate surface in
  `ecosystem/fret-ui-kit::imui`
- bounded `TableColumn` / `TableOptions` / `TableRowOptions` vocabulary instead of reusing the
  heavier declarative table surface
- focused regression artifacts:
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`
  - `ecosystem/fret-imui/src/tests/composition.rs`
- proof/demo call site:
  - `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`

### P1 - generic list clipper / virtualized row helper

Owner:

- `ecosystem/fret-ui-kit::imui`

Required outcomes:

- stable keyed visible-range authoring for large lists,
- reusable outside `PropertyGridVirtualized`,
- compatible with selection rows and combo/table bodies.

Suggested gates:

- visible-range test with deterministic row count and viewport height,
- proof/demo with a large list using the clipper surface.

Status:

- Completed

Current slice landed:

- canonical `virtual_list(...)` / `virtual_list_with_options(...)` immediate surface in
  `ecosystem/fret-ui-kit::imui`
- stable keyed virtualized row submission on top of the runtime `virtual_list_keyed_with_layout`
  substrate instead of another editor-only composite
- focused regression artifacts:
  - `ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_virtual_list_smoke.rs`
  - `ecosystem/fret-imui/src/tests/composition.rs`
- proof/demo call site:
  - `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`

### P2 - `separator_text` and small hand-feel helpers

Owner:

- `ecosystem/fret-ui-kit::imui`

Required outcomes:

- close the remaining section-label chrome gap after P0/P1 helpers settle,
- keep the scope narrow and avoid reopening style-stack mirroring.

Suggested gates:

- screenshot or geometry-backed smoke proof,
- one real proof/demo usage replacing ad hoc label + separator composition.

Status:

- Completed

Current slice landed:

- canonical `separator_text(...)` / `separator_text_with_options(...)` immediate surface in
  `ecosystem/fret-ui-kit::imui`
- narrow section-label chrome helper without reopening style-stack or window-flag mirroring
- focused regression artifacts:
  - `ecosystem/fret-ui-kit/src/imui/separator_text_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_separator_text_smoke.rs`
  - `ecosystem/fret-imui/src/tests/composition.rs`
- proof/demo call site:
  - `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`

## Explicit non-goals for this tracker

Do not pull these items into this workstream unless new evidence proves the owner split wrong:

- docking tab bars
- workspace shell chrome
- broad `WindowFlags` mirroring
- global style stacks
- editor-specific composites already covered by `fret-ui-editor::imui`
- shadcn/Base UI part-surface redesign

## Exit criteria

Before calling this lane closed, verify:

1. each accepted helper has one canonical public surface,
2. overlapping older helpers were removed or made tiny conveniences,
3. at least one real immediate proof surface uses each new helper,
4. the owner split still matches `imui` vs editor vs shell boundaries.

Current status:

- All planned P0/P1/P2 slices in this workstream are now landed.
