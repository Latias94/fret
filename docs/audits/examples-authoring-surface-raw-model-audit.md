# Examples Authoring Surface Raw-Model Audit

Date: 2026-03-18  
Scope: `apps/fret-examples` raw `Model<T>` usage after the LocalState-first default-path cleanup

This note records which remaining raw `Model<T>` usages in `apps/fret-examples` are still
intentional, which ones belong to low-level harness surfaces, and which app-facing examples remain
reasonable future cleanup candidates.

This is not a progress tracker. It is an audit baseline so future refactors do not conflate:

- default app-lane authoring ergonomics,
- advanced/manual-runtime interop,
- shared retained runtime state owned by framework-level widgets.

## Executive Summary

The LocalState-first cleanup for canonical app-path examples is effectively complete.

Evidence:

- `hello_counter_demo`
- `query_demo`
- `query_async_tokio_demo`
- `simple_todo_demo`
- `todo_demo`
- source-policy gate in `apps/fret-examples/src/lib.rs`

What remains in `apps/fret-examples` is not one homogeneous cleanup bucket.

There are three distinct categories:

1. Intentional retained/shared runtime state.
2. Intentional low-level/manual-runtime harness state.
3. App-facing examples that still look like ordinary control state and remain valid future
   LocalState-first migration candidates.

The key conclusion is simple:

- raw `Model<T>` is no longer the default authoring path for applications,
- but raw `Model<T>` is still a correct mechanism boundary for retained widgets, driver-owned
  windows, explicit `UiTree` harnesses, and shared state crossing render/event/runtime boundaries.

## Category A: Intentional Shared Retained Runtime State

These examples use raw `Model<T>` because the state is not just "local form state". It is owned by
retained subsystems or shared runtime objects with separate update/render lifecycles.

### A1) Plot demos

Representative files:

- `apps/fret-examples/src/plot_demo.rs`
- `apps/fret-examples/src/area_demo.rs`
- `apps/fret-examples/src/bars_demo.rs`
- `apps/fret-examples/src/candlestick_demo.rs`
- `apps/fret-examples/src/drag_demo.rs`
- `apps/fret-examples/src/error_bars_demo.rs`
- `apps/fret-examples/src/grouped_bars_demo.rs`
- `apps/fret-examples/src/heatmap_demo.rs`
- `apps/fret-examples/src/histogram_demo.rs`
- `apps/fret-examples/src/histogram2d_demo.rs`
- `apps/fret-examples/src/inf_lines_demo.rs`
- `apps/fret-examples/src/linked_cursor_demo.rs`
- `apps/fret-examples/src/plot_image_demo.rs`
- `apps/fret-examples/src/shaded_demo.rs`
- `apps/fret-examples/src/stacked_bars_demo.rs`
- `apps/fret-examples/src/stairs_demo.rs`
- `apps/fret-examples/src/stems_demo.rs`
- `apps/fret-examples/src/tags_demo.rs`

Reason:

- the examples hold retained plot models such as `LinePlotModel`, `BarsPlotModel`, `PlotState`,
  and `PlotOutput`,
- these values are shared with retained plot canvases and runtime-owned output channels,
- they are not ordinary render-loop-owned "local controls".

Evidence:

- `apps/fret-examples/src/plot_demo.rs`
- `apps/fret-examples/src/linked_cursor_demo.rs`

Audit judgment:

- keep raw `Model<T>` here unless the retained plot layer itself changes ownership semantics.

### A2) Retained table/data-grid state

Representative files:

- `apps/fret-examples/src/table_demo.rs`
- `apps/fret-examples/src/table_stress_demo.rs`
- `apps/fret-examples/src/datatable_demo.rs`
- `apps/fret-examples/src/canvas_datagrid_stress_demo.rs`

Reason:

- these examples hold shared `TableState`, menu-open state, output handles, or revision counters
  consumed by retained table/data-grid infrastructure,
- the state participates in command/event handlers and retained widget internals, not only in the
  render function.

Evidence:

- `apps/fret-examples/src/table_demo.rs`
- `apps/fret-examples/src/datatable_demo.rs`

Audit judgment:

- raw `Model<T>` remains acceptable here,
- future cleanup should happen at the retained table/data-grid surface, not by forcing example-local
  `LocalState<T>` wrappers on top.

### A3) Node-graph / chart / workspace retained domains

Representative files:

- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/chart_declarative_demo.rs`
- `apps/fret-examples/src/echarts_demo.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Reason:

- these examples keep shared graph stores, chart engines, docking/workspace state, or retained
  editor-grade runtime models,
- the state is intentionally wider than a single render pass and often shared with domain helpers,
  overlays, or integration hooks.

Evidence:

- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Audit judgment:

- keep raw `Model<T>` unless the underlying domain runtime is redesigned.

## Category B: Intentional Low-Level / Manual-Runtime Harness State

These examples are not default app-lane ergonomics examples. They are explicit low-level harnesses,
manual `UiTree` drivers, or runtime/driver demonstrations.

### B1) Manual `UiTree` + driver harnesses

Representative files:

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-examples/src/ime_smoke_demo.rs`

Reason:

- these are mostly manual `UiTree<App>` demos built around driver hooks, retained overlays, or
  explicit root rendering,
- the state is stored outside the render function and then read from explicit render/event
  pipelines,
- many of them still use raw `selector(...)` because the dependencies are manual `Model<T>` handles,
  not render-owned `LocalState<T>`.

Evidence:

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`

Audit judgment:

- raw `Model<T>` is not automatically wrong here,
- these examples should not be used as the source of truth for the default app authoring path.
- `apps/fret-examples/src/form_demo.rs` is now the first proof that a manual `UiTree` harness can
  still use grouped `AppUi` authoring via `fret::advanced::view::render_root_with_app_ui(...)`
  plus explicit `LocalState::from_model(...)` bridges.

### B2) Window/runtime interop harnesses

Representative files:

- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_materials_demo.rs`
- `apps/fret-examples/src/window_hit_test_probe_demo.rs`
- `apps/fret-examples/src/external_texture_imports_demo.rs`
- `apps/fret-examples/src/external_texture_imports_web_demo.rs`
- `apps/fret-examples/src/external_video_imports_avf_demo.rs`
- `apps/fret-examples/src/external_video_imports_mf_demo.rs`
- `apps/fret-examples/src/virtual_list_stress_demo.rs`
- `apps/fret-examples/src/plot_stress_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`

Reason:

- the state is owned by driver/window/runtime orchestration, not by the default `View::render`
  local-state surface,
- these examples intentionally demonstrate command hooks, visibility toggles, external import
  plumbing, diagnostics, stress loops, or inter-window/runtime behavior.

Evidence:

- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/external_texture_imports_web_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`

Audit judgment:

- keep raw `Model<T>` where driver ownership is part of the demo's purpose.

## Category C: Future Cleanup Candidates

These examples are still low-level/manual in implementation style, but their state shape is mostly
"ordinary control state". If we want to keep reducing ceremony outside the canonical todo/query
path, these are the best next candidates.

### C1) Control-panel demos with mostly local form/control state

Candidate files:

- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-examples/src/ime_smoke_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`

Why these are candidates:

- most state slots are booleans, strings, selected values, open flags, and small inspector values,
- they are conceptually close to ordinary app-lane local state,
- unlike retained plot/table/node-graph state, these examples do not fundamentally require shared
  retained graph ownership for every control model.

Why they are not P0:

- they currently sit on manual `UiTree` / driver surfaces,
- the right migration path is now to reuse the landed
  `fret::advanced::view::render_root_with_app_ui(...)` bridge rather than doing one-off ad hoc
  rewrites.

### C2) Web custom-effect inspector harnesses

Candidate files:

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`

Why these are candidates:

- most controls are ordinary inspector knobs,
- they still have duplicated `selector(...)` dependency/read boilerplate,
- the family is structurally repetitive and could likely share a thinner helper for manual
  `UiTree` demos.

Why they are not P0:

- they are explicitly low-level WebGPU/Web driver harnesses,
- the right fix is likely a manual-harness ergonomics helper, not forcing app-lane `LocalState<T>`
  into every driver-owned state bag.

## What Should Stay Locked

The canonical default app examples must remain LocalState-first.

Current locked examples:

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`

Current enforcement:

- `authoring_surface_policy_tests::canonical_default_app_examples_stay_local_state_first`
  in `apps/fret-examples/src/lib.rs`

## Recommended Next Step

If we keep pushing ergonomics, the next workstream should not be "replace every raw `Model<T>` in
examples with `LocalState<T>`".

It should be one of these:

1. Reuse the landed `render_root_with_app_ui(...)` bridge on the next clean control-panel/manual
   demos (`date_picker_demo`, `sonner_demo`, `components_gallery`) and only reopen helper design if
   those migrations reveal a real missing seam.
2. Pick one candidate family with repetitive inspector state (for example the `custom_effect_v2_*`
   web demos) and converge that family on one shared pattern.
3. Leave retained plot/table/node-graph/chart examples alone until the retained runtime surfaces
   themselves change.

The important boundary is:

- default app authoring should stay narrow and LocalState-first,
- advanced/manual harnesses can stay raw where raw ownership is the actual mechanism boundary.
