# Fret Launch + App Surface (Fearless Refactor v1) TODO

This checklist is scoped to the launch/public-surface refactor only.

Companion docs:

- Design: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/DESIGN.md`
- Export inventory: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
- Config inventory: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`
- Surface audit: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`
- Crate audit (L1): `docs/workstreams/crate-audits/fret-launch.l1.md`
- Milestones: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/MILESTONES.md`

## Setup / Inventory

- [x] Inventory every root export in `crates/fret-launch/src/lib.rs`.
- [x] Group those exports into:
  - [x] stable public contract
  - [x] transitional public surface
  - [x] compatibility/internal-plumbing path that should stop being part of the default public story
- [x] Inventory which lower-level launch types are referenced directly from `ecosystem/fret`.
- [x] Inventory which lower-level launch types are re-exported through `crates/fret-framework` bundles.

## Surface contract decisions

- [x] Decide whether `pub mod runner` should remain public in `crates/fret-launch`.
- [x] Decide the intended long-term advanced entry recommendation:
  - [ ] `FnDriver` only
  - [x] `FnDriver` recommended, `WinitAppDriver` compatibility-only
- [x] Write down a deprecation/migration posture for `WinitAppDriver` if we choose a single-path model.
- [x] Decide which host-integration helper types are currently contract-worthy enough to keep at crate root.
- [x] Decide whether `crates/fret-framework::launch` should remain a full mirror of `fret_launch::*`.
- [x] Keep compatibility-only `WinitAppDriver` off `crates/fret-framework::launch`; callers that still need it depend on `fret-launch` directly.

## Config curation

- [x] Partition `WinitRunnerConfig` conceptually into:
  - [x] app/window defaults
  - [x] render/backend tuning
  - [x] streaming/media tuning
  - [x] platform/web specifics
- [x] Decide which config fields should be documented as advanced-only.
- [x] Decide whether the future direction is:
  - [ ] nested config groups
  - [x] helper builders only
  - [ ] new public wrapper type

## `fret` facade alignment

- [x] Verify that `fret::App`, `UiAppDriver`, and `UiAppBuilder` expose the minimum lower-layer concepts needed for common apps.
- [x] Identify where current docs/examples force users to learn runner-centric concepts too early.
- [x] Map which advanced seams should remain first-class on `fret`:
  - [x] `configure(...)`
  - [x] `on_gpu_ready(...)`
  - [x] `install_custom_effects(...)`
  - [x] window create/close hooks
  - [x] engine-frame customization
- [x] Add a high-level `FnDriver` bootstrap escape hatch (`fret_bootstrap::BootstrapBuilder::new_fn(...)`, `fret_bootstrap::BootstrapBuilder::new_fn_with_hooks(...)`, `fret::run_native_with_fn_driver(...)`, `fret::run_native_with_fn_driver_with_hooks(...)`, `fret::run_native_with_configured_fn_driver(...)`).

## Docs / Examples

- [x] Update launch-facing docs so they describe three paths consistently:
  - [x] `fret` for app authors
  - [x] `fret-framework` for manual assembly
  - [x] `fret-launch` for advanced integration
- [x] Migrate representative launch examples from direct `WinitAppDriver` impls to `FnDriver` implementation paths, and rename `FnDriver`-backed helper entrypoints to `build_fn_driver()` and return concrete `FnDriver<...>` helper types where that keeps the advanced posture explicit (`chart_demo`, `bars_demo`, `error_bars_demo`, `area_demo`, `candlestick_demo`, `grouped_bars_demo`, `stacked_bars_demo`, `horizontal_bars_demo`, `histogram_demo`, `plot_demo`, `plot_image_demo`, `heatmap_demo`, `histogram2d_demo`, `inf_lines_demo`, `linked_cursor_demo`, `category_line_demo`, `stairs_demo`, `stems_demo`, `tags_demo`, `plot3d_demo`, `date_picker_demo`, `datatable_demo`, `form_demo`, `simple_todo_demo`, `plot_stress_demo`, `canvas_datagrid_stress_demo`, `sonner_demo`, `table_demo`, `table_stress_demo`, `virtual_list_stress_demo`, `ime_smoke_demo`, `docking_demo`, `container_queries_docking_demo`, `workspace_shell_demo`, `docking_arbitration_demo`, `node_graph_legacy_demo`, `node_graph_domain_demo`, `gizmo3d_demo`, `components_gallery`, `custom_effect_v2_identity_web_demo`, `custom_effect_v2_glass_chrome_web_demo`, `custom_effect_v3_web_demo`, `first_frame_smoke_demo`, `effects_demo`, `external_texture_imports_web_demo`).
- [x] Migrate the remaining heavy direct `WinitAppDriver` example impls (`components_gallery`) after their local harness glue is split into reviewable free-hook helpers.
- [x] Add one minimal advanced example that shows the supported launch escape hatch without exposing unnecessary internals.
- [x] Cross-link this workstream from any relevant builder/onboarding docs if the final surface changes.
- [x] Add a focused guardrail so `fret` crate-root helper regressions and README drift are caught early.
- [x] Add a focused guardrail so `fret-launch` crate-root exports stay curated and specialized helpers remain namespaced.
- [x] Add an explicit root-surface snapshot guardrail for `crates/fret-launch/src/lib.rs` so future export creep becomes diffable.
- [x] Add a focused guardrail so remaining direct `WinitAppDriver` example impls cannot introduce hooks outside current `FnDriver` coverage without an explicit review.
- [x] Add a focused guardrail so `fret-framework::launch` stays a curated manual-assembly facade and does not re-export compatibility-only launch traits.

## Validation gates

- [x] `cargo nextest run -p fret-launch`
- [x] `cargo nextest run -p fret-framework`
- [x] `cargo nextest run -p fret`
- [x] `python tools/check_layering.py`
- [x] `python tools/gate_fret_builder_only_surface.py`
- [x] `python tools/gate_fret_launch_surface_contract.py`
- [x] `python tools/gate_fret_launch_root_surface_snapshot.py`
- [x] `python tools/gate_fret_framework_launch_surface.py`
- [x] `python tools/gate_fn_driver_example_naming.py`
- [x] `python tools/gate_winit_driver_example_hook_coverage.py`

## Rollout notes

- [x] Land documentation and export classification before removing or hiding launch exports.
- [x] Prefer staged de-emphasis + migration docs before hard API removal.
- [x] Treat the current remaining direct `WinitAppDriver` examples as migration debt, not as evidence of missing `FnDriver` hooks.
- [x] Keep host-integration use cases working throughout the refactor.
- [x] Coordinate any `fret` top-level API changes with `docs/workstreams/app-entry-builder-v1/`.
