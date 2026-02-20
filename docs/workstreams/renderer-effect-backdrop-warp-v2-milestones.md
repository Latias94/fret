# Renderer Effect: Backdrop Warp v2 — Milestones

Status: Landed (wgpu default renderer; conformance + perf baseline recorded)

Tracking files:

- `docs/workstreams/renderer-effect-backdrop-warp-v2.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v2-todo.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v2-milestones.md`

## Progress (living)

- M0: done (ADR drafted + bounded vocabulary).
- M1: done (core contract + wgpu implementation + WebGPU shader validation).
- M2: done (conformance + steady-state perf baseline).
- M3: done (demo validation + adoption notes).

Progress record (M0/M1):

- Date: 2026-02-18
- Status: Landed (conformance + perf + demo validated)
- Evidence anchors:
  - `docs/adr/0285-backdrop-warp-effect-step-v2-texture-field.md`
  - `crates/fret-core/src/scene/mod.rs` (`EffectStep::BackdropWarpV2`, `BackdropWarpV2`)
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (`EffectStep::BackdropWarpV2` compile path)
  - `crates/fret-render-wgpu/src/renderer/pipelines/backdrop_warp.rs` (warp-field image pipeline variants)
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`BACKDROP_WARP_*` WGSL)
  - Gates:
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
    - `cargo test -p fret-render-wgpu --test effect_backdrop_warp_v2_conformance`

Progress record (M2):

- Date: 2026-02-18
- Status: Baseline recorded (v2 steady script)
- Evidence anchors:
  - `tools/diag-scripts/liquid-glass-backdrop-warp-v2-steady.json`
  - `docs/workstreams/perf-baselines/policies/liquid-glass-backdrop-warp-v2-steady.v1.json`
  - `docs/workstreams/perf-baselines/liquid-glass-backdrop-warp-v2-steady.windows-rtx4090.v1.json`

## M0 — ADR lock + bounded vocabulary

Exit criteria:

- ADR defines the v2 contract surface (image warp field + decoding + deterministic degradation).
- The step’s sample-count impact is explicit, and the degradation order is testable.

Evidence anchors:

- `docs/adr/0285-backdrop-warp-effect-step-v2-texture-field.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v2.md`

## M1 — Core contract + wgpu implementation + WebGPU shader validation

Exit criteria:

- `fret-core` exposes the v2 step with sanitize/fingerprint determinism.
- `fret-render-wgpu` implements the step under `EffectMode::Backdrop`.
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.

Evidence anchors:

- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-render-wgpu/src/renderer/*`

## M2 — Conformance + perf baselines

Exit criteria:

- GPU readback conformance exists and runs in CI-friendly mode.
- A steady-state perf gate exists with a checked-in baseline.

Evidence anchors:

- `crates/fret-render-wgpu/tests/*_conformance.rs`
- `docs/workstreams/perf-baselines/*`
  - Script: `tools/diag-scripts/liquid-glass-backdrop-warp-v2-steady.json`
  - Seed policy: `docs/workstreams/perf-baselines/policies/liquid-glass-backdrop-warp-v2-steady.v1.json`
  - Baseline: `docs/workstreams/perf-baselines/liquid-glass-backdrop-warp-v2-steady.windows-rtx4090.v1.json`

## M3 — Demo validation + adoption notes

Exit criteria:

- `liquid_glass_demo` can exercise:
  - fake-glass baseline (blur + color adjust),
  - procedural v1 warp,
  - and image-driven v2 warp,
  - with visible toggles and documented fallbacks.
  - and stable `test_id` anchors for scripted perf baselines.

Evidence anchors:

- `apps/fret-examples/src/liquid_glass_demo.rs`
  - Stage HUD layout: ensures toggles remain visible while preserving backdrop visibility.
  - Script anchors: `liquid-glass-switch-show-{fake,warp-v1,warp-v2}`.
- Perf gate:
  - `tools/perf/diag_liquid_glass_backdrop_warp_v2_gate.py`
