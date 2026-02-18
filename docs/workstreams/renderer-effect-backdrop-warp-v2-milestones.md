# Renderer Effect: Backdrop Warp v2 — Milestones

Status: Draft (workstream tracker)

Tracking files:

- `docs/workstreams/renderer-effect-backdrop-warp-v2.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v2-todo.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v2-milestones.md`

## Progress (living)

- M0: not started (ADR lock).
- M1: not started (core contract + wgpu implementation).
- M2: not started (conformance + steady-state perf baseline).
- M3: not started (demo validation + adoption notes).

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

## M3 — Demo validation + adoption notes

Exit criteria:

- `liquid_glass_demo` can exercise:
  - fake-glass baseline (blur + color adjust),
  - procedural v1 warp,
  - and image-driven v2 warp,
  - with visible toggles and documented fallbacks.

Evidence anchors:

- `apps/fret-examples/src/liquid_glass_demo.rs`

