# Crate audit (L1) 閳?`fret-launch`

Focus: public surface hygiene, extension thresholds, and whether the current launch layer still
needs fearless contraction before publication.

## Crate

- Name: `fret-launch`
- Path: `crates/fret-launch`
- Owners / adjacent crates: `fret-framework`, `fret-bootstrap`, `fret`, `fret-runner-winit`, `fret-runner-web`
- Current 閳ユ笓ayer閳? advanced launch / runner integration facade

## 1) Purpose (what this crate *is*)

- `fret-launch` is the explicit advanced integration layer above runtime/render/platform crates.
- It is the place where host event-loop ownership, window creation, GPU initialization, advanced
  render-target interop, and platform-specific media helpers are intentionally allowed to meet.
- This crate is not the first-contact app-author surface; that role belongs to `fret`, with
  `fret-framework` acting as the curated manual-assembly facade.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`
- `crates/fret-framework/src/lib.rs`
- `ecosystem/fret/src/lib.rs`

## 2) Current public contract surface

The crate-root surface is materially healthier than the earlier L0 audit:

- core launch contracts stay at crate root (`FnDriver`, `FnDriverHooks`, `WinitRunnerConfig`,
  `WinitAppBuilder`, `run_app`, `run_app_with_event_loop`, window geometry/spec types, runner
  contexts, `WgpuInit`),
- specialized integrations live under explicit modules (`imported_viewport_target`,
  `native_external_import`, `media`, `shared_allocation`),
- `WinitAppDriver` remains public on `fret-launch`, but explicitly as a compatibility surface rather
  than the recommended advanced model; it no longer rides the curated `fret-framework::launch`
  facade.

This is already a usable long-term split. The remaining issue is **curation debt**, not a missing
escape hatch.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`

## 3) Extension thresholds (what is enough, and when callers should drop lower)

### Stay on `fret`

The current `fret` facade is already sufficient for many general-purpose and editor-grade apps when
you need:

- app/view builder entry (`App::new(...).window(...).ui(...)`, `view::<V>()`),
- driver-side hooks for event/command/model/global changes,
- window create/created/before-close hooks,
- viewport input and engine-frame customization,
- launch config tweaks via `UiAppBuilder::configure(...)`,
- GPU-ready and custom-effect installation on the builder path.

Evidence anchors:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`

### Drop to `fret-launch`

Callers should intentionally drop to `fret-launch` when they need capabilities that are **runner- or
host-owned**, including:

- event-loop ownership / injection (`WinitAppBuilder::with_event_loop`,
  `with_event_loop_builder_hook`),
- host-provided GPU setup (`WgpuInit::Provided`, `WgpuInit::Factory`),
- explicit web-runner handle control (`WebRunnerHandle`, `run_app_with_handle`),
- specialized imported viewport / external texture / shared-allocation / media interop,
- full low-level accessibility or runner contract coverage beyond the curated `fret::UiAppDriver`
  wrapper surface.

This threshold looks reasonable: advanced host integration is available, but it does not pollute the
default app-author story.

Evidence anchors:

- `crates/fret-launch/src/runner/desktop/runner/run.rs`
- `crates/fret-launch/src/runner/common/config.rs`
- `crates/fret-launch/src/lib.rs`
- `ecosystem/fret/src/lib.rs`

## 4) Dependency posture

- Heavy backend/platform coupling is expected here and is architecturally acceptable.
- The remaining maintainability risk is not 閳ユ辅rong dependencies閳? but **surface drift** and large
  desktop-runner implementation files.
- Current file-size hotspots still sit in desktop runner internals (`app_handler.rs`, `effects.rs`,
  `window.rs`), which makes the public/root surface discipline even more important.

Evidence anchors:

- `crates/fret-launch/Cargo.toml`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- `crates/fret-launch/src/runner/desktop/runner/window.rs`

## 5) Module ownership map (internal seams)

- Core advanced driver contract
  - Files: `crates/fret-launch/src/runner/common/fn_driver.rs`, `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- Launch configuration and geometry
  - Files: `crates/fret-launch/src/runner/common/config.rs`, `crates/fret-launch/src/runner/common/window_create_spec.rs`, `crates/fret-launch/src/runner/common/window_geometry.rs`
- Native launch/builder wiring
  - Files: `crates/fret-launch/src/runner/desktop/runner/run.rs`, `crates/fret-launch/src/runner/desktop/runner/mod.rs`
- Desktop runner internals
  - Files: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`, `crates/fret-launch/src/runner/desktop/runner/window.rs`, `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- Web runner / handle path
  - Files: `crates/fret-launch/src/runner/web/mod.rs`, `crates/fret-launch/src/runner/web/render_loop.rs`
- Specialized interop/media
  - Files: `crates/fret-launch/src/lib.rs`, `crates/fret-launch/src/runner/windows_mf_video.rs`

## 6) Refactor hazards (what can regress easily)

### H1) `WinitAppDriver` still shapes the mental model more than intended

- Failure mode: docs say 閳ユ笡refer `FnDriver`閳? but examples and helper signatures still teach
  `impl WinitAppDriver` as the visible type boundary.
- Why this matters: this is not a capability gap, but it weakens the de-emphasis story and makes a
  future hard contraction more expensive.
- Existing gates: launch/example coverage in the current launch surface workstream plus `tools/gate_fn_driver_example_naming.py` for representative `FnDriver`-backed helpers and concrete `FnDriver<...>` return types, and `tools/gate_winit_driver_example_hook_coverage.py` to keep remaining direct trait examples inside current `FnDriver` hook coverage.
- Updated reading: remaining direct `WinitAppDriver` examples now look like migration debt, not evidence of a missing advanced hook.

Evidence anchors:

- `apps/fret-examples/src/plot_demo.rs`
- `apps/fret-examples/src/bars_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`

### H2) `WinitRunnerConfig` is still too raw and too broad

- Failure mode: advanced users must learn one large config object mixing app defaults, backend
  tuning, streaming/media settings, and platform-specific knobs.
- Why this matters: the crate has enough power, but the affordance is still lower-level than it
  needs to be.
- Existing gates: documentation partitioning in the launch-surface workstream.
- Missing gate to add: helper-level compile tests/docs for the preferred config helper story in
  `fret` / `fret-bootstrap`.

Evidence anchors:

- `crates/fret-launch/src/runner/common/config.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`

### H3) Root-surface drift is now guarded, but classification discipline still matters

- Failure mode: future root exports or re-export growth could still bypass the intended classification story if the snapshot is updated casually.
- Why this matters: `fret-launch` intentionally sits on many specialized seams, so every root-surface addition should still justify why it belongs at crate root instead of under a specialized module.
- Existing gates: `tools/gate_fret_launch_surface_contract.py`, `tools/gate_fret_launch_root_surface_snapshot.py`, export inventory docs, and review discipline.
- Remaining work: keep the snapshot aligned with explicit export classification instead of treating it as a mechanical allowlist.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `tools/gate_fret_launch_surface_contract.py`
- `tools/gate_fret_launch_root_surface_snapshot.py`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`

### H4) Native/web control asymmetry is still an open design edge

- Failure mode: embedders may eventually want a native-side control/handle story comparable to the
  web-side `WebRunnerHandle`.
- Why this matters: there is no in-tree evidence that this is needed *today*, so this should not
  force premature API expansion, but it is one of the few plausible future capability gaps.
- Existing gates: none.
- Missing gate to add: decision first, API second.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/src/runner/desktop/runner/run.rs`
- `apps/fret-demo-web/src/wasm.rs`

## 7) Code quality findings (Rust / API hygiene)

- `FnDriver` is already close to the intended long-term advanced posture: its hook surface covers
  initialization, GPU readiness, hot reload, event/render, command/global changes, model/global
  changes, window lifecycle, viewport input, docking, engine-frame customization, and accessibility.
- Relative to GPUI, the difference is primarily **posture**, not missing power: GPUI hides more of
  the platform/runner layer behind app/platform injection, while Fret chooses to keep launch as an
  explicit crate. That is acceptable because Fret wants runner ownership, hotpatch-friendly
  function-pointer seams, and specialized interop/media escape hatches to remain first-class.
- The main unresolved problem is therefore not "can it extend enough?" but "can we keep the naming
  and default story tight enough that advanced escape hatches do not become the accidental default?"
- After migrating `custom_effect_v2_identity_web_demo`, `custom_effect_v2_glass_chrome_web_demo`,
  and `custom_effect_v3_web_demo` to explicit `build_fn_driver()` helpers, the remaining
  example-local `impl WinitAppDriver` bodies are better understood as internal migration debt
  or complex harness glue (multi-window, hot reload, accessibility, engine-frame hooks), not
  as evidence that `FnDriver` lacks extension headroom.
- The same holds for the low-risk batch (`first_frame_smoke_demo`, `effects_demo`,
  `external_texture_imports_web_demo`): even smoke/profiling/external-texture demos can move to
  `FnDriver` without exposing additional launch hooks.
- The same also holds for the single-window UI batch (`datatable_demo`, `date_picker_demo`,
  `form_demo`, `ime_smoke_demo`, `sonner_demo`): local hot-reload/command/model/global-change
  wiring migrates cleanly to free `FnDriver` hooks, so the remaining direct impl inventory is no
  longer a meaningful argument for keeping compat posture as the default story.
- The same now holds for the medium-complexity desktop batch (`plot_stress_demo`, `table_demo`,
  `table_stress_demo`, `virtual_list_stress_demo`, `workspace_shell_demo`): stress/perf harnesses
  and workspace-shell command orchestration also migrate to pure free hooks, so the remaining
  direct impl inventory is down to eight and concentrated in genuinely heavier docking/gallery/
  gizmo/node-graph examples rather than a missing launch primitive.
- `canvas_datagrid_stress_demo` reinforces the same point from the retained-canvas stress side:
  GPU prep, perf snapshot reporting, model/global propagation, and close semantics all migrate to
  the existing free-hook surface, taking the remaining direct impl inventory down to seven.
- `docking_demo` tightens the conclusion further: the only blocker there was an app-facing helper
  gap for preconfigured `.with_init(...)` `FnDriver` values, not a missing hook. Adding
  `fret::run_native_with_configured_fn_driver(...)` closes that posture gap and reduces the
  remaining direct impl inventory to six.
- `container_queries_docking_demo` confirms the helper is generic enough for another docking-heavy
  example with container-query behavior, reducing the remaining direct impl inventory again to five
  without expanding the hook matrix.
- `docking_arbitration_demo` closes the next multi-window docking arbitration harness too:
  viewport input, dock-op arbitration, floating-window lifecycle, and dev-state export/import all
  fit the existing free-hook surface, reducing the remaining direct impl inventory again to four.
- `node_graph_legacy_demo` closes the retained node-graph reference harness too: model/global
  propagation, command routing, persistence debounce, diagnostics interception, and the
  retained/declarative render split all fit the existing free-hook surface, reducing the remaining
  direct impl inventory again to three.
- `node_graph_domain_demo` closes the domain/runtime-oriented node-graph harness too: model/global
  propagation, command routing, persistence debounce, and retained canvas rendering all fit the
  existing free-hook surface, reducing the remaining direct impl inventory again to two.
- `gizmo3d_demo` closes the viewport-tool / engine-frame / 3D overlay harness too: init hooks,
  command routing, global-change redraw arbitration, viewport input routing, engine-frame
  recording, and retained HUD rendering all fit the existing free-hook surface, reducing the
  remaining direct impl inventory again to one.
- The current remaining direct example impl inventory is now limited to `components_gallery`.

Evidence anchors:

- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-launch/src/runner/common/winit_app_driver.rs`
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
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `ecosystem/fret/src/lib.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`

## 8) Recommended refactor steps (small, gated)

1. Keep the root-surface guardrails for `crates/fret-launch/src/lib.rs` classification-driven
   - Outcome: export creep stays reviewable and every snapshot change still carries a surface-classification reason.
   - Gate: `python tools/gate_fret_launch_surface_contract.py`, `python tools/gate_fret_launch_root_surface_snapshot.py`.

2. Keep `fret-framework::launch` free of compatibility-only launch traits
   - Outcome: manual assembly stays curated; callers that truly need `WinitAppDriver` depend on
     `fret-launch` directly instead of inheriting compat posture by accident.
   - Gate: `python tools/gate_fret_framework_launch_surface.py`, `cargo check -p fret-framework --all-features`.

3. Make the advanced example story name `FnDriver` more directly
   - Outcome: compatibility trait remains public, but stops feeling like the recommended mental model.
   - Gate: `cargo nextest run -p fret-examples`, `python tools/gate_winit_driver_example_hook_coverage.py`.

4. Continue helper-layer config curation instead of reshaping `WinitRunnerConfig` directly
   - Outcome: keep power where it is, but move more app-facing convenience up to `fret` / `fret-bootstrap`.
   - Gate: `cargo nextest run -p fret -p fret-bootstrap`.

5. Decide whether native handle parity is actually needed before adding more launch API
   - Outcome: avoid speculative surface growth while keeping one real future gap visible.
   - Gate: decision note + a concrete embedding use case.

## 9) Open questions / decisions needed

- What exact caller-inventory criteria would let the direct `fret-launch` `WinitAppDriver` surface shrink further in a future cycle, now that example hook coverage is no longer the blocker?
- Do we want a native equivalent of `WebRunnerHandle`, or is `WinitAppBuilder` + host-owned event
  loop enough for the intended embedding stories?
- Should representative examples keep returning `impl WinitAppDriver`, or should we standardize on
  a more explicit `FnDriver`-named boundary for new advanced examples?

## Current judgment

- **Is the launcher crate external surface still too open?** Not critically. The major 閳ユ竵ccidental
  public plumbing閳?problem has already been reduced.
- **Is extension/customization capability insufficient?** Not for the current target. General apps
  and editor-grade apps both have enough room.
- **What still needs fearless closure?** Naming, docs/example posture, config curation, and disciplined caller migration; not more advanced hook surface.
