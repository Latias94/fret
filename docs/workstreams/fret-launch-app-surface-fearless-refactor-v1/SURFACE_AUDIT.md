# Fret Launch + App Surface (Fearless Refactor v1) 闁?Surface Audit

Status: Post-example migration audit

Scope:

- `crates/fret-launch`
- `ecosystem/fret`
- `ecosystem/fret-bootstrap`
- `crates/fret-framework::launch`

This note answers the practical question behind the workstream: after curating docs and migrating
representative examples to `FnDriver` implementation paths, is the current launch/app surface
reasonable for general-purpose applications while still preserving editor-grade customization?

## Verdict

Short answer: **yes, mostly**.

The current surface already has the right capability split:

- `fret` is now a credible desktop-first, general-purpose app-author surface.
- `fret-launch` still exposes enough seams for host-integrated / editor-grade applications.
- `fret-bootstrap` is the correct bridge for callers who want bootstrap defaults without dropping all
  the way down to raw launch wiring.

The main remaining debt is **surface curation**, not missing power.

Compared with GPUI/Zed-style expectations, Fret is no longer blocked on extensibility. The core
question is now how aggressively to shrink and classify public exports, not whether the framework
can support advanced apps.

## Evidence-backed assessment

### 1) `fret` is sufficient for common apps

For a typical desktop app author, the relevant mental model is already close to:

- app
- main window
- root UI/view
- optional app-level install hooks

Evidence:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`

Why this is enough:

- `fret::App::new(...).window(...).ui(...)` is the recommended short path.
- `fret::App::new(...).window(...).ui_with_hooks(...)` keeps advanced driver hooks on that same
  builder path.
- The builder chain is now the only `fret` app-author entry story, which removes first-contact
  ambiguity at the crate root.
- `UiAppBuilder` still exposes real extension points without forcing app authors to start from
  `fret-launch`.

### 2) The advanced seams are still first-class on `fret`

The `fret` facade already exposes the advanced hooks that matter for non-trivial products:

- `App::{ui_with_hooks, view_with_hooks::<V>}`
- `configure(...)`
- `on_gpu_ready(...)`
- `install_custom_effects(...)`
- window create/created/before-close hooks
- engine-frame recording
- viewport input and global command hooks

Evidence:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

This means advanced users can stay on `fret` longer than before, and only drop to
`fret-bootstrap` / `fret-launch` when they truly need runner-owned behavior.

### 3) `fret-launch` is strong enough for editor-grade integration

`fret-launch` still carries the seams that matter for serious host integration:

- host-provided GPU init (`WgpuInit`)
- per-window create specs
- event / command / render contexts
- viewport input
- docking ops
- engine-frame hooks
- accessibility hooks
- imported viewport / external texture interop

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/src/runner/common/fn_driver.rs`

This is enough to support the same class of outcomes expected from editor-like shells, even though
the shape differs from GPUI.

### 4) The migration from trait-driver examples materially reduced ambiguity

Representative examples now route their runtime path through `FnDriver`, including:

- chart/plot demos
- form/table/date picker demos
- stress demos
- docking demos
- workspace shell
- node graph legacy demo

Evidence:

- `apps/fret-examples/src/`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`

This matters because examples define the real public story more strongly than prose docs do.

## GPUI comparison

### What is now comparable

Fret now supports the same broad layering outcome:

- simple app-author entry path
- advanced integration escape hatch
- editor-grade multi-window / docking / viewport customization

### What is intentionally different

Fret闁炽儲鐛?advanced runtime posture is still explicitly **function-pointer / hook based** (`FnDriver`),
not a direct GPUI-style closure runtime.

That difference is acceptable because it buys a few things Fret explicitly values:

- hotpatch-friendly function boundaries
- clearer separation between mechanism hooks and app/bootstrap defaults
- easier preservation of native/web runner wiring as an explicit layer

### Current conclusion

Fret no longer lacks extensibility relative to the intended target. What it still lacks is a tighter
curation story around which names are the stable/default ones.

## Recent closure

### `fret-framework::launch` is now a curated subset

`crates/fret-framework/src/lib.rs` now re-exports the core manual-assembly launch contract instead
of mirroring the full `fret_launch::*` surface.

Included surface (high level):

- `FnDriver` / `FnDriverHooks`
- core runner contexts without compatibility-only `WinitAppDriver`
- `WinitRunnerConfig`, `WgpuInit`, and window-spec types
- top-level app entry wiring (`WinitAppBuilder`, `run_app*`, wasm handle entrypoints)

Left on `fret-launch` directly:

- specialized media helpers
- imported viewport / external texture interop helpers
- other launch-root exports that are real but not part of the compact framework facade

Evidence:

- `crates/fret-framework/src/lib.rs`

Interpretation:

- manual assembly keeps a compact umbrella path,
- compatibility-only trait usage now requires an explicit `fret-launch` dependency,
- specialized integrations still have a direct escape hatch,
- the previous full-mirror hazard is now closed in this worktree.

### Specialized interop/media helpers now live under explicit modules

The `fret-launch` public surface now distinguishes between:

- core launch/builder/driver contracts kept at crate root,
- specialized interop helpers under `imported_viewport_target` / `native_external_import`,
- platform media helpers under `media`,
- shared-allocation interop under `shared_allocation`.

That is a meaningful curation improvement because advanced helpers remain public without pretending
to be part of the first-contact crate-root story.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`

## Remaining hazards

### H1) `WinitRunnerConfig` is stable but still over-broad

The launch config is still a single object that mixes:

- app/window defaults
- backend tuning
- streaming/media tuning
- platform/web specifics

Evidence:

- `crates/fret-launch/src/runner/common/config.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`

Interpretation:

- keep it stable for now,
- prefer helper-layer curation over a breaking shape split.

### H2) `WinitAppDriver` remains public as compatibility surface

This is now well documented, but it is still real public API.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`

Interpretation:

- acceptable short term,
- the remaining de-emphasis blocker is caller migration and naming posture, not a missing `FnDriver` hook gap.
- current remaining direct example impls only use hooks already covered by `FnDriver`.

Additional evidence:

- `apps/fret-examples/src/chart_multi_axis_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v3_web_demo.rs`
- `apps/fret-examples/src/first_frame_smoke_demo.rs`
- `apps/fret-examples/src/effects_demo.rs`
- `apps/fret-examples/src/external_texture_imports_web_demo.rs`
- `apps/fret-examples/src/datatable_demo.rs`
- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/ime_smoke_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/src/plot_stress_demo.rs`
- `apps/fret-examples/src/table_demo.rs`
- `apps/fret-examples/src/table_stress_demo.rs`
- `apps/fret-examples/src/virtual_list_stress_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/canvas_datagrid_stress_demo.rs`
- `apps/fret-examples/src/docking_demo.rs`
- `apps/fret-examples/src/container_queries_docking_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-demo-web/src/wasm.rs`
- `tools/gate_winit_driver_example_hook_coverage.py`

Interpretation update:

- the latest remaining web custom-effect helpers now teach `FnDriver` directly, so the
  advanced helper posture is tighter than the raw `impl WinitAppDriver` inventory alone
  suggests.
- remaining in-file direct trait impls cluster around multi-window/hot-reload/accessibility
  or engine-frame customization examples; that is migration debt and review surface, not a
  demonstrated `FnDriver` capability gap.
- the low-risk batch (`first_frame_smoke_demo`, `effects_demo`, `external_texture_imports_web_demo`) further reduces the remaining direct `WinitAppDriver` inventory to 18 at this checkpoint, without requiring any new `FnDriver` hook surface.
- the current single-window UI batch (`datatable_demo`, `date_picker_demo`, `form_demo`, `ime_smoke_demo`, `sonner_demo`) reduces that inventory again to 13, reinforcing that the remaining blockers are composition complexity and migration effort, not launch-hook insufficiency.
- the latest medium-complexity desktop batch (`plot_stress_demo`, `table_demo`, `table_stress_demo`, `virtual_list_stress_demo`, `workspace_shell_demo`) reduces that inventory again to 8, leaving only the heaviest docking/gallery/gizmo/node-graph examples on direct `WinitAppDriver` posture.
- `canvas_datagrid_stress_demo` also migrates cleanly to pure free hooks, reducing that inventory again to 7 and further supporting the conclusion that `FnDriver` is not blocked on additional stress/perf hook surface.
- `docking_demo` shows the remaining gap was not in `FnDriverHooks` at all, but in the facade posture: once `fret::run_native_with_configured_fn_driver(...)` exists for preconfigured `.with_init(...)` drivers, docking orchestration also migrates cleanly and the inventory drops again to 6.
- `container_queries_docking_demo` confirms that the same helper is reusable rather than one-off: container-query-aware docking also migrates cleanly and the inventory drops again to 5, still without any new `FnDriverHooks`.
- `docking_arbitration_demo` closes the next multi-window docking case too: viewport input, dock-op arbitration, floating-window lifecycle, and dev-state export/import all move to the existing free-hook surface, reducing the remaining direct `WinitAppDriver` inventory again to 4 without adding any new `FnDriverHooks`.
- `node_graph_legacy_demo` closes the retained node-graph reference path as well: model/global propagation, command routing, persistence debounce, diagnostics interception, and retained/declarative render selection all fit the existing free-hook surface, reducing the remaining direct `WinitAppDriver` inventory again to 3 without adding any new `FnDriverHooks`.
- `node_graph_domain_demo` closes the domain/runtime-oriented node-graph path too: model/global propagation, command routing, persistence debounce, and retained canvas rendering all fit the existing free-hook surface, reducing the remaining direct `WinitAppDriver` inventory again to 2 without adding any new `FnDriverHooks`.
- `gizmo3d_demo` closes the viewport-tool / engine-frame / 3D overlay path too: init hooks, command routing, global-change redraw arbitration, viewport input routing, engine-frame recording, and retained HUD rendering all fit the existing free-hook surface, reducing the remaining direct `WinitAppDriver` inventory again to 1 without adding any new `FnDriverHooks`.
- the current remaining direct-example inventory is now limited to `components_gallery`.

### H3) Specialized launch modules still need classification discipline

Several specialized interop/media/render-target exports are valid, but they should stay
classification-driven and should not expand casually even now that they live under explicit
submodules.

Evidence:

- `crates/fret-launch/src/lib.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`

## Recommended next steps

1. **Keep `fret-framework::launch` curated**
   - Outcome: future additions require explicit facade-level justification; specialized helpers and compatibility-only traits stay on direct `fret-launch` dependencies.
   - Gate: `cargo check -p fret-framework --all-features`, `python tools/check_layering.py`, `python tools/gate_fret_framework_launch_surface.py`

2. **Lean into helper-layer config curation**
   - Outcome: keep `WinitRunnerConfig` stable, but expose more app-facing config helpers through
     `fret` / `fret-bootstrap` instead of teaching the full config object by default.
   - Gate: `cargo nextest run -p fret -p fret-bootstrap`

3. **Define a sunset bar for `WinitAppDriver` based on caller inventory**
   - Outcome: track which remaining callers still opt into the compatibility trait, since the current example inventory no longer points to a hook-coverage gap.
   - Gate: docs update + `python tools/gate_winit_driver_example_hook_coverage.py`.

4. **Keep specialized launch modules classification-driven**
   - Outcome: future specialized additions land under explicit modules and must still be tagged as stable / specialized / transitional.
   - Gate: export inventory stays current.

## Final judgment

For the original question 闁?闁炽儲绗卻 the launcher/internal surface reasonable, and can users of the
public `fret` facade extend/customize enough like Zed/GPUI while still being suitable for general
apps?闁?闁?the current answer is:

- **General-purpose apps:** yes
- **Advanced/editor-grade customization:** yes
- **Main remaining problem:** curation / naming / export discipline, not capability

That is a good place to be before publication.

## Gates run for this audit phase

- `cargo fmt -p fret-examples`
- `cargo check -p fret-examples -p fret-demo-web --all-targets`
- `cargo nextest run -p fret-examples`
- `cargo fmt -p fret-framework`
- `cargo nextest run -p fret-framework --no-tests pass`
- `python tools/check_layering.py`

## Evidence anchors

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-framework/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`
