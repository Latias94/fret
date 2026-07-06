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
    examples. Keep `drag_demo`, `inf_lines_demo`, and `linked_cursor_demo` on explicit raw props
    until drag feedback and linked plot registration move to named binding/coordinator APIs.
  - Follow-up: `chart_declarative_demo.rs` now uses `ChartCanvasPanelBinding`, so the default
    FretApp chart example no longer imports `fret_runtime::Model` or wires
    `ChartCanvasPanelProps::engine` directly. Keep explicit raw chart panel props in stress,
    linked, multi-grid, and output-model demos until those advanced contracts are named.
- Custom effect demos such as `custom_effect_v2_*`. They expose effect parameter models and reset
  groups; these need a dedicated parameter/control-surface design before deletion.
  - Follow-up: `custom_effect_v2_web_demo.rs`, `custom_effect_v2_identity_web_demo.rs`,
    `custom_effect_v2_lut_web_demo.rs`, and `custom_effect_v2_glass_chrome_web_demo.rs` now keep
    direct reset/toggle model writes behind local owner helpers. Keep model allocation in the
    function-driver setup path for now; design a shared parameter binding only after the duplicated
    shape proves worth exposing as a public app-facing abstraction.
- `apps/fret-ui-gallery/src/driver/*`. These are gallery runtime drivers and not first-contact app
  authoring examples.
- `apps/fret-ui-gallery/src/ui/snippets/ai/canvas_world_layer_spike.rs`. This is a large spike with
  canvas/node-graph interaction state and should be handled as a dedicated workstream.
- `imui_editor_proof_demo/*`. This is an editor-grade retained proof lane with explicit model
  graphs; migrate only with editor-state public contracts, not by mechanical LocalState rewrites.
- `workspace_shell_demo/*`. Audited after the initial inventory: it is application-level workspace
  shell state, so the shared model graph remains. The follow-up cleanup routes writes through
  demo-local owner helpers instead of scattering raw `models_mut().update(...)` calls.
- `components_gallery.rs`, `virtual_list_stress_demo.rs`, and `editor_notes_demo.rs`. Audited and
  cleaned after the initial inventory: each keeps intentional shared models but routes writes
  through local owner helpers with source gates.
- `genui_demo.rs`. Audited and cleaned after the initial inventory: it remains an advanced GenUI
  runtime/reference surface with shared runtime models, while raw model reads/writes are routed
  through local owner helpers.
- `api_workbench_lite_demo.rs`. Audited and cleaned after the initial inventory: it remains a
  first-contact LocalState/query/mutation app example, while the necessary mutation/query
  `ModelStore` access is routed through a local `ApiWorkbenchModelOwner` and source-gated.

Likely next cleanup slices:

- Custom-effect parameter binding contracts after the V2 web variants have the same owner boundary
  and duplicated parameter shapes are clearer.
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
