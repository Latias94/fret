# Renderer Effect: Backdrop Warp v1 — Milestones

Status: Draft (workstream tracker)

Tracking files:

- `docs/workstreams/renderer-effect-backdrop-warp-v1.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v1-todo.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v1-milestones.md`

## Progress (living)

- M0: in progress (ADR is present but still marked Draft).
- M1: done (wgpu implementation + WebGPU shader validation).
- M2: in progress (conformance landed; perf baseline still pending).
- M3: in progress (demo now exercises the step; adoption notes pending).

## M0 — ADR lock + bounded vocabulary

Exit criteria:

- ADR defines the v1 contract surface (bounded parameters + deterministic degradation).
- The step’s sample-count impact is explicit, and the degradation order is testable.

Evidence anchors:

- `docs/adr/0284-backdrop-warp-effect-step-v1.md`
- `docs/workstreams/renderer-effect-backdrop-warp-v1.md`

## M1 — wgpu implementation + WebGPU shader validation

Exit criteria:

- `fret-render-wgpu` implements the step under `EffectMode::Backdrop`.
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.
- Degradation behavior is deterministic (explicit counters or stable behavior under budget).

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (effect execution)
- `crates/fret-render-wgpu/src/renderer/shaders/*` (warp sampling)

## M2 — Conformance + perf baselines

Exit criteria:

- GPU readback conformance exists and runs in CI-friendly mode.
- A steady-state perf gate exists with a checked-in baseline.

Evidence anchors:

- `crates/fret-render-wgpu/tests/*_conformance.rs` (new test)
- `docs/workstreams/perf-baselines/*` (baseline)

## M3 — Demo validation + adoption notes

Exit criteria:

- `liquid_glass_demo` can exercise:
  - fake-glass baseline (blur + color adjust),
  - and true warp (bounded, capability-gated),
  - with visible toggles and documented fallbacks.

Evidence anchors:

- `apps/fret-examples/src/liquid_glass_demo.rs`
