# Renderer Effect: Drop Shadow v1 — Milestones

Status: Done (conformance + perf baseline landed)

Tracking files:

- `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1-todo.md`
- `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1-milestones.md`

## Progress (living)

- M0: done (ADR drafted).
- M1: done (core contract + wgpu implementation + WebGPU shader validation).
- M2: done (conformance + perf baseline + gate script).

Progress record:

- Date: 2026-02-18
- Status: Landed (conformance + perf baseline + gate script)
- Evidence anchors:
  - `docs/adr/0286-drop-shadow-effect-step-v1.md`
  - `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1-todo.md`
  - `crates/fret-render-wgpu/tests/effect_drop_shadow_v1_conformance.rs`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
  - `tools/diag-scripts/drop-shadow-v1-steady.json`
  - `apps/fret-examples/src/drop_shadow_demo.rs`
  - `docs/workstreams/perf-baselines/drop-shadow-v1-steady.windows-rtx4090.v1.json`
  - `tools/perf/diag_drop_shadow_v1_gate.ps1`

## M0 — ADR lock + bounded parameters

Exit criteria:

- ADR defines the v1 surface (single layer, bounded blur, solid color).
- Degradation is deterministic and explicitly testable.

## M1 — wgpu implementation + portability validation

Exit criteria:

- Step is implemented in the filter-content pipeline.
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.

## M2 — Conformance + perf gates

Exit criteria:

- GPU readback conformance exists.
- Perf gate exists with a checked-in baseline.
