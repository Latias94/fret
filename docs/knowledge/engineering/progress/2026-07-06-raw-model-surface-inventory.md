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

- Plot/chart demos such as `plot_demo`, `stems_demo`, `histogram_demo`, `chart_multi_axis_demo`.
  They allocate model graphs that are consumed by plot/chart component APIs rather than ordinary
  view-local app state.
  - Follow-up: `plot_declarative_demo.rs` now uses `LinePlotPanelBinding` as the app-facing tracer
    bullet. Keep explicit raw `LinePlotPanelProps` in advanced/linked/overlay plot demos until
    each plot family has an equivalent binding contract.
- Custom effect demos such as `custom_effect_v2_*`. They expose effect parameter models and reset
  groups; these need a dedicated parameter/control-surface design before deletion.
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

Likely next cleanup slices:

- `api_workbench_lite_demo.rs`: tests show it is app-facing and should stay on `LocalState` /
  `LocalStateTxn`; keep it as the regression template for first-contact app authoring.
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
