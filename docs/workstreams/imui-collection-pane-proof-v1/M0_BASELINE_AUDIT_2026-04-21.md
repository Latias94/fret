# M0 Baseline Audit - 2026-04-21

Status: active baseline note

Purpose: justify why `imui-collection-pane-proof-v1` is a new narrow follow-on instead of a
reopened umbrella folder or another broad "imgui parity" lane.

## Findings

### 1) The immediate stack already has the relevant helper seams

Current evidence shows that the missing gap is not "we cannot express this at all".
The repo already has:

- keyed collection helpers,
- `ImUiMultiSelectState<K>` and model-backed multi-select rows,
- `child_region[_with_options]`,
- editor adapters,
- and shell composition surfaces.

This means the next question is proof breadth and helper sufficiency, not runtime existence.

Primary evidence:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`

### 2) The current first-party proofs are still fragmented for this specific gap

`imui_editor_proof_demo` and `workspace_shell_demo` are both valuable, but together they still do
not settle:

- asset-grid/file-browser grade multi-select breadth,
- marquee / box-select bridging,
- or `BeginChild()`-scale pane composition over the current `child_region` seam.

Primary evidence:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

### 3) The shell-helper question is already closed for now

The repo already decided not to promote a higher-level workbench shell helper yet.
So the next collection/pane proof should use the current workspace starter set and example-local
assembly, not reopen helper promotion by accident.

Primary evidence:

- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`

### 4) Multi-window parity is a different active lane

The runner/backend-owned multi-window work remains active in
`docking-multiwindow-imgui-parity`.
This lane should not try to absorb overlap/follow/Wayland/mixed-DPI work.

Primary evidence:

- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`

### 5) The umbrella already told us exactly when to split this lane

The latest P0 parity status explicitly says:

- keep the umbrella as the status recorder,
- do not start another larger parent workstream,
- and only start a new P0 follow-on when the next slice is implementation-heavy around a narrow
  topic such as collection/pane proof breadth.

Primary evidence:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`

## Decision

Create a new narrow active lane:

- `docs/workstreams/imui-collection-pane-proof-v1/`

This lane owns:

- collection-first proof breadth,
- pane-first proof breadth,
- and only the helper decisions justified by those proofs.

This lane does not own:

- runtime widening,
- shell-helper promotion,
- key ownership,
- or runner/backend multi-window parity.

## Immediate execution consequence

Use the current baseline proof pair as the first-open repro surface:

1. `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. `cargo run -p fret-demo --bin workspace_shell_demo`

Keep the first gate floor narrow and existing:

1. `cargo nextest run -p fret-examples --lib imui_editor_proof_non_raw_helpers_prefer_typed_return_signatures imui_editor_proof_authoring_immediate_column_uses_official_editor_adapters imui_editor_proof_keeps_app_owned_sortable_and_dock_helpers_explicit --no-fail-fast`
2. `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
3. `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
4. `cargo nextest run -p fret-imui`

Only after the proof roster is frozen should this lane add its own dedicated source-policy gate or
new dedicated proof demo.
