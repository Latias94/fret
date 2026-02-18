# Renderer Effect: Backdrop Warp v1 — TODO Tracker

Status: Draft (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `BWARP-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Design lock

- [~] BWARP-adr-010 Add an ADR for the bounded backdrop warp surface (v1):
      contract shape, deterministic degradation rules, and wasm/mobile constraints.
  - Exit criteria:
    - step parameters are bounded (no open-ended shader sources),
    - step behavior is defined for both `EffectMode::{Backdrop,FilterContent}`,
    - degradation order is explicit and testable,
    - sample counts and quality knobs are bounded and observable.
  - Evidence anchors:
    - `docs/adr/0284-backdrop-warp-effect-step-v1.md`
    - `docs/workstreams/renderer-effect-backdrop-warp-v1.md`

## Contract changes

- [x] BWARP-core-020 Add a new `EffectStep` variant for backdrop warp (v1) in `fret-core`.
  - Notes:
    - Prefer a small enum for warp function selection (bounded vocabulary).
    - Keep chromatic aberration optional and bounded by a small max.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`EffectStep`)
    - `crates/fret-core/src/scene/{validate.rs,fingerprint.rs}` (sanitize + determinism)

## Renderer implementation (wgpu)

- [x] BWARP-wgpu-030 Implement `BackdropWarp` in `fret-render-wgpu`:
      integrate into the backdrop effect pass with bounded pipeline variants.
  - Requirements:
    - strict scissor to effect bounds,
    - uniform-control-flow-safe WGSL (no divergent sampling branches),
    - deterministic degradation when unsupported / budgeted.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (chain compile)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (effect execution)
    - `crates/fret-render-wgpu/src/renderer/pipelines/backdrop_warp.rs` (pipelines + bind groups)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`BACKDROP_WARP_*` shaders)

## Conformance + portability gates

- [x] BWARP-test-040 Add a GPU readback conformance test:
      backdrop warp modifies sampled backdrop pixels deterministically.
  - Evidence anchors:
    - `crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

- [ ] BWARP-perf-050 Add a steady-state perf gate + baseline:
      worst-frame stability under bounded warp + blur chain.
  - Evidence anchors:
    - `tools/perf/*` or `tools/diag-scripts/*` (new gate/script)
    - `docs/workstreams/perf-baselines/*` (baseline JSON)

## Demo validation (optional but recommended)

- [x] BWARP-demo-060 Extend the liquid glass demo to exercise the new step behind a toggle:
      keep fake-glass baseline available and document deterministic degradation.
  - Evidence anchors:
    - `apps/fret-examples/src/liquid_glass_demo.rs` (fake vs warp lenses + inspector toggles)
    - `apps/fret-demo/src/bin/liquid_glass_demo.rs` (entrypoint, if needed)
