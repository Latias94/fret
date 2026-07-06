---
type: Work Progress
title: Raw model surface inventory after IMUI cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-raw-model-inventory
tags: fret,ui-framework,public-surface,raw-model,inventory
---

# Summary

After the IMUI LocalState cleanup, `apps/fret-examples-imui/src` no longer contains raw LocalState
bridge imports or call-site patterns such as `.model()`, `.update_in(...)`, `.set_in(...)`,
`models_mut()`, `layout_value_in(...)`, or `paint_value_in(...)`.

The remaining public-surface raw model pressure is broader than LocalState. Most hits are direct
`Model<T>` graphs or `models_mut()` calls in demos that intentionally prove lower-level mechanisms:
plot/chart state, custom effect parameter graphs, gallery drivers, retained editor proof state, and
AI/gallery snippet state. These should not be blanket-replaced with `LocalState` without a
component-specific public surface.

# Current Classification

Keep raw/shared-model mechanisms for now:

- Plot/chart demos such as `chart_multi_axis_demo`.
  They allocate model graphs that are consumed by plot/chart component APIs rather than ordinary
  view-local app state.
  - Follow-up: `plot_declarative_demo.rs` now uses `LinePlotPanelBinding` as the app-facing tracer
    bullet. `plot_demo.rs` also stores `LinePlotPanelBinding` and reads event output through
    `LinePlotPanelBinding::output_untracked(...)`, so the manual harness no longer exposes the
    line plot's raw state/output models. Keep explicit raw `LinePlotPanelProps` in
    advanced/linked/overlay plot demos until each plot family has an equivalent binding contract.
  - Follow-up: `histogram_demo.rs` now uses `HistogramPlotPanelBinding`; `fret-plot` shares the
    state/output machinery through a private binding core instead of copying per-family output
    reads. Keep explicit raw histogram props in component tests and advanced composition paths.
  - Follow-up: `stems_demo.rs` now uses `StemsPlotPanelBinding`, following the same private binding
    core. Keep explicit raw stems props in component tests and advanced composition paths.
  - Follow-up: `error_bars_demo.rs` now uses `ErrorBarsPlotPanelBinding`. The plot binding wrapper
    generation has moved behind a private macro so new app-facing plot family bindings stay named
    and family-specific without copying the raw model/state/output glue.
  - Follow-up: `grouped_bars_demo.rs` and `stacked_bars_demo.rs` now use
    `BarsPlotPanelBinding`, sharing one app-facing surface over `BarsPlotModel` while keeping raw
    `BarsPlotPanelProps` available for advanced composition.
  - Follow-up: `area_demo.rs`, `shaded_demo.rs`, `candlestick_demo.rs`, `heatmap_demo.rs`, and
    `histogram2d_demo.rs` now use family-specific panel bindings. Keep raw line/area props in
    `drag_demo`, `inf_lines_demo`, `tags_demo`, `plot_image_demo`, and `linked_cursor_demo` until
    their overlay/state/linkage contracts are explicitly named.
  - Follow-up: `stairs_demo.rs` now uses `LinePlotPanelBinding`; step mode stays a normal
    declarative props option layered on top of the default app-facing binding rather than a reason
    to expose raw state/output models.
  - Follow-up: `plot_stress_demo.rs` remains a maintainer/perf harness because it mutates plot
    bounds from the driver loop, but its raw plot/animation models now live behind
    `PlotStressModelOwner` instead of scattered `app.models()` reads and writes.
  - Follow-up: `tags_demo.rs` and `plot_image_demo.rs` now use `LinePlotPanelBinding` for advanced
    state-owned overlays. The binding surface accepts initial `PlotState`, exposes closure-based
    state reads/writes, and keeps raw `Model<PlotState>` / `Model<PlotOutput>` hidden from the app
    examples. `drag_demo.rs` also uses the binding for event-time drag output reads and state
    feedback writes, so the drag overlay example no longer stores raw plot state/output models.
    `linked_cursor_demo.rs` now registers binding-backed plots through
    `LinkedPlotGroup::push_binding(...)` instead of hand-wiring `LinkedPlotMember { state, output }`.
    `inf_lines_demo.rs` now uses `LinePlotPanelBinding::new_with_state(...)` for multi-axis
    reference-line overlays and reads query output through `output_untracked(...)`.
  - Follow-up: `chart_declarative_demo.rs`, the manual-harness `chart_demo.rs`, `bars_demo.rs`,
    `category_line_demo.rs`, and `horizontal_bars_demo.rs` now use `ChartCanvasPanelBinding`. The
    binding owns the default chart engine/output models and exposes `output_untracked(...)` for
    event-time tooltip logging, so these examples no longer import `fret_runtime::Model` or wire
    `ChartCanvasPanelProps::{engine, output_model}` directly. `echarts_demo.rs` also uses the
    binding for adapter smoke charts, so it no longer imports `fret_runtime::Model` or wires
    `ChartCanvasPanelProps::engine` directly. `echarts_multi_grid_demo.rs` uses
    `ChartCanvasMultiGridBinding` for a shared chart engine, per-grid panels, and an overlay-only
    panel; its remaining `fret_runtime` import is a manual runner/bootstrap seam. Keep explicit raw
    chart panel props in stress, linked, and intentionally shared output-model demos until those
    advanced contracts are named.
  - Follow-up: `tools/check_surface_policy.py` now explicitly scans and classifies
    `echarts_demo.rs`, `echarts_multi_grid_demo.rs`, `chart_multi_axis_demo.rs`, and
    `chart_stress_demo.rs` with owners, allowed raw seams, and category-specific retirement
    conditions. The `echarts_demo.rs` record has already shrunk after the binding migration; it no
    longer allows the `fret_runtime` raw seam. The `echarts_multi_grid_demo.rs` record now points at
    `ChartCanvasMultiGridBinding` while preserving its runner-level seam classification. This keeps
    advanced chart exceptions in the same source-policy system as plot, workspace, and harness
    exceptions rather than relying only on ad hoc demo tests.
- Table/data-grid demos.
  - Follow-up: `datatable_demo.rs` now uses `LocalState<shadcn::DataTableViewOutput>` and
    `app.local_state(...)` for the shadcn `DataTable` output handle. The retained table source gate
    has been refreshed from the old `layout_read_ref_in(...)` / `advanced::prelude::LocalState`
    markers to the current app-facing `LocalState` API and now forbids raw
    `Model<DataTableViewOutput>` plumbing in this example.
  - Follow-up: `DataGridCanvas::output_model(...)` now accepts the dedicated
    `IntoDataGridCanvasOutputModel` bridge, and `fret::app::LocalState<DataGridCanvasOutput>`
    implements it. `canvas_datagrid_stress_demo.rs` uses `LocalState` for grid telemetry output
    while preserving raw stress-control models for variable sizing, clamping, and revision state.
  - Follow-up: `table_stress_demo.rs` remains a retained table/perf harness, but keyboard command
    writes for sorting, role/global filters, clearing filters, and `items_revision` now route
    through the demo-local `TableStressModelOwner` helper. Its source-surface test forbids direct
    `models_mut().update(...)`, generic/update-any variants, and UFCS `ModelStore` bypasses in the
    production demo source.
  - Keep raw model seams in lower-level retained table, data-grid, stress, or canvas-grid surfaces
    only when their component contract still names the shared retained state explicitly.
- Custom effect demos such as `custom_effect_v2_*`. They expose effect parameter models and reset
  groups; these need a dedicated parameter/control-surface design before deletion.
  - Follow-up: `custom_effect_v2_web_demo.rs`, `custom_effect_v2_identity_web_demo.rs`,
    `custom_effect_v2_lut_web_demo.rs`, and `custom_effect_v2_glass_chrome_web_demo.rs` now keep
    direct reset/toggle model writes behind the shared private `custom_effect_v2_web_owner.rs`
    helper. Follow-up tightening introduced `CustomEffectV2WebControlBinding`: the duplicated
    show/enabled/mode/quality/sampling/UV/debug controls now live in a shared binding, and the four
    demos route reset/toggle through `toggle_surface_in(...)` / `reset_controls_in(...)` instead of
    naming the private `CustomEffectV2WebModelOwner`. Variant-specific parameter models still stay
    in each demo behind `CustomEffectV2WebVariantReset`. Follow-up tightening also moved raw
    `EffectParamsV1 { vec4s: ... }` literals behind explicit `CustomEffectV2ParamSlot` /
    `CustomEffectV2ParamPack` calls, so the shader ABI ordering is named and source-gated without
    prematurely introducing a full dynamic parameter-schema system. A later scalar-control cleanup
    moved variant-specific slider state behind `CustomEffectV2ScalarControl`, so those demos no
    longer expose raw `Model<Vec<f32>>` fields, direct scalar `models_mut().insert(vec![...])`
    allocation, or variant reset writes through `reset.set_model(&self.foo, ...)`.
  - Follow-up: those four web variants are now included in `tools/check_surface_policy.py` scan
    roots and classified as advanced/manual surfaces with explicit raw seams, owner, and retirement
    condition. The same gate now rejects direct reset/toggle writes through
    `models_mut().update(...)` or UFCS `ModelStore::update(...)` outside the shared private binding
    helper, and rejects legacy owner exposure in the demo files. Treat
    `tools/gate_examples_source_tree_policy.py` as a broader drift report until its existing
    baseline failures are resolved.
- `apps/fret-ui-gallery/src/driver/*`. These are gallery runtime drivers and not first-contact app
  authoring examples.
- `apps/fret-ui-gallery/src/ui/snippets/ai/canvas_world_layer_spike.rs`. This is a large spike with
  canvas/node-graph interaction state and should be handled as a dedicated workstream.
- `imui_editor_proof_demo/*`. This is an editor-grade retained proof lane with explicit model
  graphs; migrate only with editor-state public contracts, not by mechanical LocalState rewrites.
  Follow-up cleanup started with `collection/`: app-side command, context-menu, asset-grid, and
  rename-start writes now route through `ProofCollectionModelOwner`, and the legacy
  `proof_collection_begin_inline_rename_in_app(...)` free helper was deleted. A second cleanup
  routed `asset_ref.rs` and `editor_text_assist.rs` shared string writes through
  `EditorProofModelOwner`. A final editor-surface cleanup routed Material/Advanced numeric reset
  and edit-outcome writes through `EditorProofModelOwner`, leaving no direct
  `models_mut().update(...)` / UFCS `ModelStore::update(...)` hits under
  `imui_editor_proof_demo/*`. The collection keyboard action helpers now route delete, duplicate,
  begin-rename, select-all, and navigation writes through `ProofCollectionModelOwner` instead of
  direct `UiActionHostExt::update_model(...)`. Inline rename outcome actions now also route commit,
  invalid, and cancel writes through `ProofCollectionModelOwner`, leaving focus restoration/redraw
  scheduling at the action boundary. Browser-scope context-menu anchor publication, zoom extent,
  and inline-rename pending-focus consumption now also route through `ProofCollectionModelOwner`.
  Box-select runtime session state now routes through a runtime-local box-select model owner, and
  threshold/clear selection publication routes through `ProofCollectionModelOwner::apply_navigation(...)`.
  There are currently no `UiActionHostExt::update_model(...)`, direct `models_mut().update(...)`,
  UFCS `ModelStore::update(...)`, or `update_any(...)` production hits under
  `imui_editor_proof_demo/*`.
- `workspace_shell_demo/*`. Audited after the initial inventory: it is application-level workspace
  shell state, so the shared model graph remains. The follow-up cleanup routes writes through
  demo-local owner helpers instead of scattering raw `models_mut().update(...)` calls. Follow-up
  tightening upgraded the driver from generic free helpers to `WorkspaceShellModelOwner`.
- `components_gallery.rs`, `virtual_list_stress_demo.rs`, and `editor_notes_demo.rs`. Audited and
  cleaned after the initial inventory: each keeps intentional shared models but routes writes
  through local owner helpers with source gates. Follow-up tightening upgraded
  `components_gallery.rs` from generic free helpers to `ComponentsGalleryModelOwner`,
  `virtual_list_stress_demo.rs` from free helper functions to a named
  `VirtualListStressModelOwner`, and its source gate now forbids direct/generic/update-any and UFCS
  `ModelStore` bypasses in production source. The same tightening upgraded
  `editor_notes_demo.rs` from free host helper functions to `EditorNotesModelOwner`.
- `external_texture_imports_demo.rs`, `external_texture_imports_web_demo.rs`, and the platform
  `external_video_imports_*` demos. Audited and cleaned after the wasm `ui-assets` feature fix:
  they remain low-level external import harnesses, while the shared visibility toggle write now
  routes through the private `ExternalImportsModelOwner` helper instead of duplicated event-handler
  `models_mut().update(...)` calls.
- `embedded_viewport_demo.rs`. Audited and cleaned after the cookbook utility-window owner pass: it
  remains a Tier A embedded viewport interop demo, while its startup `last_input` readout write now
  routes through the demo-local `EmbeddedViewportDemoModelOwner` helper. Its source-surface test now
  forbids direct `models_mut().update(...)` calls in the demo source.
- `docking_arbitration_demo.rs`. Audited and cleaned after the table stress owner pass: it remains
  an advanced docking/overlay arbitration harness, while drop-mask control, synthetic pointer debug,
  and viewport-input diagnostic writes now route through the demo-local
  `DockingArbitrationModelOwner` helper. Its source-surface test now forbids direct/generic/update-any
  and UFCS `ModelStore` bypasses in production demo source.
- `apps/fret-cookbook/examples/external_texture_import_basics.rs`. Audited and cleaned after the
  external imports owner pass: it remains an advanced/manual interop cookbook example, while its
  engine-frame target metric writes now route through the demo-local
  `ExternalTextureImportBasicsModelOwner` helper instead of teaching three direct
  `models_mut().update(...)` calls in `record_engine_frame(...)`.
- `apps/fret-cookbook/examples/embedded_viewport_basics.rs`. Audited and cleaned after the
  cookbook external texture pass: it remains an advanced/manual embedded viewport interop example,
  while viewport-input diagnostic writes now route through the demo-local
  `EmbeddedViewportBasicsModelOwner` helper instead of teaching five direct
  `models_mut().update(...)` calls in `on_viewport_input(...)`.
- `apps/fret-cookbook/examples/utility_window_materials_windows.rs`. Audited and cleaned after the
  cookbook embedded viewport pass: it remains an advanced/manual utility-window materials interop
  example, while command status writes now route through the demo-local
  `UtilityWindowMaterialsModelOwner` helper. `apps/fret-cookbook/examples` currently has no direct
  `models_mut().update(...)` calls outside source-test marker strings.
- `genui_demo.rs`. Audited and cleaned after the initial inventory: it remains an advanced GenUI
  runtime/reference surface with shared runtime models, while raw model reads/writes are routed
  through local owner helpers. Follow-up tightening upgraded app/host free helpers to
  `GenUiModelOwner` and forbids direct read/update bypasses in production source.
- `api_workbench_lite_demo.rs`. Audited and cleaned after the initial inventory: it remains a
  first-contact LocalState/query/mutation app example, while the necessary mutation/query
  `ModelStore` access is routed through a local `ApiWorkbenchModelOwner` and source-gated.
- `apps/fret-demo/src/bin/hotpatch_smoke_demo.rs`. Audited after the IMUI collection host-write
  cleanup: it remains a dev-only hotpatch maintainer smoke harness, while event/command counter and
  debug writes now route through `HotpatchSmokeModelOwner` instead of direct
  `app.models_mut().update(...)` calls. A source-surface regression test keeps those write paths
  behind the owner while allowing the hotpatch harness to keep explicit model allocation/read seams.

Likely next cleanup slices:

- Common custom-effect numeric-control cleanup if `uv_span()` should join the scalar-control binding
  path, or a typed effect schema if defaults, diagnostics labels, and parameter ABI metadata need to
  be shared across more than these four demos.
- Plot/chart family-specific bindings for remaining first-contact demos once their output/state
  contracts are named explicitly.

# Verification

- `git status --short --branch` on latest `main`
- `rg` scan over `apps/fret-examples-imui/src` for raw LocalState bridge patterns returned no
  matches.
- `rg` scan over first-party examples/gallery showed remaining raw model pressure is concentrated
  in plot/chart, custom-effect, gallery-driver, editor-proof, and workspace-shell areas.

# Follow-Up

- Before another code slice, choose between:
  - a consumer-facing cleanup (`components_gallery.rs` or `workspace_shell_demo/*`), or
  - a contract-level design slice for plot/chart/custom-effect model binding APIs.
- If continuing broad raw-model shrinkage, add source gates per chosen surface so `models_mut()`
  does not regrow in app-facing examples after migration.
