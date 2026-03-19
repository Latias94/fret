# Fret Launch + App Surface (Fearless Refactor v1) 闂?Milestones

This workstream is staged to keep the launch stack landable while tightening public contracts.

## M0 闂?Audit captured + documentation aligned

**Outcome**

- A dedicated workstream folder exists with design, milestones, and TODO documents.
- The current surface split (`fret`, `fret-framework`, `fret-launch`) is documented clearly.
- Known hazards are recorded before code changes begin.

**Gates**

- Links from `docs/README.md` point to this folder.
- The design doc includes evidence anchors for all major claims.

## M1 闂?Export inventory and contract classification

**Outcome**

- Every root export from `crates/fret-launch/src/lib.rs` is classified as:
  - stable public,
  - transitional public,
  - internal plumbing.
- `fret` wrappers/re-exports are mapped to the minimum lower-level launch surface they require.

**Evidence anchors**

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`
- `ecosystem/fret/src/lib.rs`

**Gates**

- A reviewable export table exists in the implementation PR or linked audit note.
- No new launch exports are added without classification.

## M2 闂?Single advanced driver recommendation

**Outcome**

- `FnDriver` becomes the clearly documented advanced driver path.
- Any remaining `WinitAppDriver`-only requirements are either:
  - moved into `FnDriverHooks`, or
  - explicitly justified as compatibility surface.
- `fret-framework::launch` stops re-exporting compatibility-only `WinitAppDriver`.

**Evidence anchors**

- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- `apps/fret-examples/src/chart_demo.rs`
- `apps/fret-examples/src/bars_demo.rs`
- `apps/fret-examples/src/error_bars_demo.rs`

**Gates**

- `cargo nextest run -p fret-launch`
- Any touched docs/examples build or type-check if compile-checked in the relevant crate.
- Representative advanced examples prefer `FnDriver` over bespoke `WinitAppDriver` impls.
- Recent web custom-effect helpers (`custom_effect_v2_identity_web_demo`, `custom_effect_v2_glass_chrome_web_demo`, `custom_effect_v3_web_demo`) now also expose explicit `build_fn_driver()` entrypoints.
  This is launch-surface evidence for advanced/manual web harnesses, not default app-lane
  authoring guidance.
- Recent low-risk demos (`first_frame_smoke_demo`, `effects_demo`, `external_texture_imports_web_demo`) now also run through `FnDriver`-backed entrypoints.
- Single-window UI demos (`datatable_demo`, `date_picker_demo`, `form_demo`, `ime_smoke_demo`, `sonner_demo`) now also use pure `FnDriver` hooks instead of keeping local compat impls.
- Medium-complexity desktop demos (`plot_stress_demo`, `table_demo`, `table_stress_demo`, `virtual_list_stress_demo`, `workspace_shell_demo`) now also run through pure `FnDriver` hook assembly, reducing the remaining direct `WinitAppDriver` example inventory to eight without expanding launch surface area.
- `canvas_datagrid_stress_demo` now also uses pure free `FnDriver` hooks, reducing the remaining direct `WinitAppDriver` example inventory to seven while keeping stress/perf reporting inside the existing hook set.
- `fret::advanced::run_native_with_configured_fn_driver(...)` now covers the preconfigured-`FnDriver` posture (including `.with_init(...)`) without falling back to compat naming, and `docking_demo` uses it to reduce the remaining direct `WinitAppDriver` example inventory to six.
- `container_queries_docking_demo` now follows the same configured-`FnDriver` posture, reducing the remaining direct `WinitAppDriver` example inventory to five without introducing any new docking/runtime hooks.
- `docking_arbitration_demo` now follows the same configured-`FnDriver` posture too, proving that multi-window docking arbitration, dev-state wiring, and floating-window lifecycle hooks still fit the existing free-hook surface while reducing the remaining direct `WinitAppDriver` example inventory to four.
- `node_graph_legacy_demo` now follows the same pure free-hook `FnDriver` posture on the retained node-graph reference path too, reducing the remaining direct `WinitAppDriver` example inventory to three without expanding launch surface area.
- `node_graph_domain_demo` now follows the same pure free-hook `FnDriver` posture on the domain/runtime-oriented node-graph path too, reducing the remaining direct `WinitAppDriver` example inventory to two without expanding launch surface area.
- `gizmo3d_demo` now follows the same pure free-hook `FnDriver` posture on the viewport-tool / engine-frame / 3D overlay path too, reducing the remaining direct `WinitAppDriver` example inventory to one without expanding launch surface area.
- `components_gallery` now follows the same pure free-hook `FnDriver` posture on the accessibility / semantics / diagnostics-heavy gallery path too, reducing the remaining direct `WinitAppDriver` example inventory to zero without expanding launch surface area.
- The examples-side direct `WinitAppDriver` inventory is now zero; any future direct impl should be treated as an explicit, reviewed regression from the preferred `FnDriver` posture.
- `python tools/gate_fret_launch_root_surface_snapshot.py`
- `python tools/gate_fret_framework_launch_surface.py`
- `python tools/gate_fn_driver_example_naming.py`
- `python tools/gate_winit_driver_example_hook_coverage.py`

## M3 闂?Config curation without capability loss

**Outcome**

- Launch configuration is documented in app-facing vs backend-heavy groups.
- Beginner-facing docs stop teaching low-level tuning by default.
- Advanced host-integration knobs remain reachable.

**Evidence anchors**

- `crates/fret-launch/src/runner/common/config.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`

**Gates**

- `cargo nextest run -p fret`
- No regression in examples that depend on GPU init customization or window-create hooks.

## M4 闂?Cross-surface docs and naming closure

**Outcome**

- `fret`, `fret-framework`, and `fret-launch` each have a distinct one-line role statement.
- The app-author path and integration path are both documented with minimal ambiguity.
- Docs stop implying that large internal launch namespaces are stable by accident.

**Evidence anchors**

- `docs/README.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `crates/fret-framework/src/lib.rs`
- `crates/fret-launch/README.md`
- `tools/gate_fret_builder_only_surface.py`
- `tools/gate_fret_framework_launch_surface.py`
- `tools/gate_fn_driver_example_naming.py`

**Gates**

- `cargo nextest run -p fret -p fret-launch -p fret-framework`
- `python tools/check_layering.py`
- `python tools/gate_fret_builder_only_surface.py`
- `python tools/gate_fret_framework_launch_surface.py`
- `python tools/gate_fn_driver_example_naming.py`

## M5 闂?Optional follow-up: web/high-level symmetry

**Outcome**

- We make an explicit decision on whether `fret` should expose a peer high-level web entry surface.
- If not, docs say so clearly; if yes, a separate workstream owns the design.

**Constraint**

- This is not required to land the core launch/public-surface cleanup.

**Gates**

- Decision recorded in docs with a clear scope boundary.
