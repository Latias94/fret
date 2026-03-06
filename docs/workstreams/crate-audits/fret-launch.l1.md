# Crate audit (L1) — `fret-launch`

Focus: public surface hygiene, extension thresholds, and whether the current launch layer still
needs fearless contraction before publication.

## Crate

- Name: `fret-launch`
- Path: `crates/fret-launch`
- Owners / adjacent crates: `fret-framework`, `fret-bootstrap`, `fret`, `fret-runner-winit`, `fret-runner-web`
- Current “layer”: advanced launch / runner integration facade

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
- The remaining maintainability risk is not “wrong dependencies”, but **surface drift** and large
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

- Failure mode: docs say “prefer `FnDriver`”, but examples and helper signatures still teach
  `impl WinitAppDriver` as the visible type boundary.
- Why this matters: this is not a capability gap, but it weakens the de-emphasis story and makes a
  future hard contraction more expensive.
- Existing gates: launch/example coverage in the current launch surface workstream plus `tools/gate_fn_driver_example_naming.py` for representative `FnDriver`-backed helpers and concrete `FnDriver<...>` return types.
- Missing gate to add: a narrower follow-up check for non-helper direct `FnDriver::new(...)` call sites if we decide those should also standardize on named helper surfaces.

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

### H3) There is still no explicit public-surface snapshot gate for `fret-launch`

- Failure mode: future root exports or re-export growth slips in without classification.
- Why this matters: `fret-launch` intentionally sits on many specialized seams, so surface drift is
  easier here than in contract crates.
- Existing gates: export inventory docs and review discipline.
- Missing gate to add: a narrow root-surface guardrail or snapshot for `crates/fret-launch/src/lib.rs`.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
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
- The main unresolved problem is therefore not “can it extend enough?” but “can we keep the naming
  and default story tight enough that advanced escape hatches do not become the accidental default?”

Evidence anchors:

- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`

## 8) Recommended refactor steps (small, gated)

1. Add a root-surface guardrail for `crates/fret-launch/src/lib.rs`
   - Outcome: export creep becomes reviewable and classification-driven.
   - Gate: a small Python/PowerShell guardrail plus the existing export inventory docs.

2. Keep `fret-framework::launch` free of compatibility-only launch traits
   - Outcome: manual assembly stays curated; callers that truly need `WinitAppDriver` depend on
     `fret-launch` directly instead of inheriting compat posture by accident.
   - Gate: `python tools/gate_fret_framework_launch_surface.py`, `cargo check -p fret-framework --all-features`.

3. Make the advanced example story name `FnDriver` more directly
   - Outcome: compatibility trait remains public, but stops feeling like the recommended mental model.
   - Gate: `cargo nextest run -p fret-examples`.

4. Continue helper-layer config curation instead of reshaping `WinitRunnerConfig` directly
   - Outcome: keep power where it is, but move more app-facing convenience up to `fret` / `fret-bootstrap`.
   - Gate: `cargo nextest run -p fret -p fret-bootstrap`.

5. Decide whether native handle parity is actually needed before adding more launch API
   - Outcome: avoid speculative surface growth while keeping one real future gap visible.
   - Gate: decision note + a concrete embedding use case.

## 9) Open questions / decisions needed

- What exact criteria would let the direct `fret-launch` `WinitAppDriver` surface shrink further in a future cycle?
- Do we want a native equivalent of `WebRunnerHandle`, or is `WinitAppBuilder` + host-owned event
  loop enough for the intended embedding stories?
- Should representative examples keep returning `impl WinitAppDriver`, or should we standardize on
  a more explicit `FnDriver`-named boundary for new advanced examples?

## Current judgment

- **Is the launcher crate external surface still too open?** Not critically. The major “accidental
  public plumbing” problem has already been reduced.
- **Is extension/customization capability insufficient?** Not for the current target. General apps
  and editor-grade apps both have enough room.
- **What still needs fearless closure?** Naming, docs/example posture, config curation, and an
  explicit guardrail against root-surface drift.
