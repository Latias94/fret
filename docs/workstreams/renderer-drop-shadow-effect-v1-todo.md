# Renderer Effect: Drop Shadow v1 — TODO Tracker

Status: Done (ADR + contract + wgpu implementation + conformance + perf baseline)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `DSHADOW-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Design lock

- [x] DSHADOW-adr-010 Add an ADR for the bounded drop shadow step (v1):
      contract shape, degradation rules, and portability constraints.
  - Evidence anchors:
    - `docs/adr/0286-drop-shadow-effect-step-v1.md`
    - `docs/workstreams/renderer-drop-shadow-effect-v1.md`

## Contract changes

- [x] DSHADOW-core-020 Add `EffectStep::DropShadowV1` (or equivalent) to `fret-core`.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`EffectStep`)
    - `crates/fret-core/src/scene/{validate.rs,fingerprint.rs}`

## Renderer implementation (wgpu)

- [x] DSHADOW-wgpu-030 Implement the step in `fret-render-wgpu`’s filter-content effect pipeline:
      intermediate reuse + bounded blur + deterministic degradation.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (effect execution)
    - `crates/fret-render-wgpu/src/renderer/pipelines/*` (shadow pipeline)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (shadow eval)

## Conformance + perf

- [x] DSHADOW-test-040 Add a GPU readback conformance test.
  - Evidence anchors:
    - `crates/fret-render-wgpu/tests/effect_drop_shadow_v1_conformance.rs`

- [x] DSHADOW-perf-050 Add a perf gate + baseline for shadow-heavy scenes.
  - Evidence anchors:
    - `tools/diag-scripts/drop-shadow-v1-steady.json`
    - `apps/fret-examples/src/drop_shadow_demo.rs`
    - `apps/fret-demo/src/bin/drop_shadow_demo.rs`
    - `docs/workstreams/perf-baselines/policies/drop-shadow-v1-steady.v1.json`
    - `docs/workstreams/perf-baselines/drop-shadow-v1-steady.windows-rtx4090.v1.json`
    - `tools/perf/diag_drop_shadow_v1_gate.ps1`
