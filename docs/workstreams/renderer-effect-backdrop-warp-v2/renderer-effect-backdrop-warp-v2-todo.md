# Renderer Effect: Backdrop Warp v2 — TODO Tracker

Status: Landed (wgpu default renderer; conformance + perf baseline recorded)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `BWARP2-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Design lock

- [x] BWARP2-adr-010 Add an ADR for the bounded backdrop warp v2 surface:
      image-driven warp field source, decoding semantics, deterministic degradation rules, and
      wasm/mobile constraints.
  - Exit criteria:
    - v2 parameters are bounded (no open-ended shader sources),
    - warp map decoding is explicit and testable,
    - behavior is defined for both `EffectMode::{Backdrop,FilterContent}`,
    - degradation order is explicit and testable,
    - sample counts and quality knobs are bounded and observable.
  - Evidence anchors:
    - `docs/adr/0285-backdrop-warp-effect-step-v2-texture-field.md`
    - `docs/workstreams/renderer-effect-backdrop-warp-v2/renderer-effect-backdrop-warp-v2.md`

## Contract changes

- [x] BWARP2-core-020 Add a new `EffectStep` variant for backdrop warp v2 in `fret-core`.
  - Notes:
    - Prefer `ImageId + UvRect + ImageSamplingHint` (portable, bounded).
    - Keep decoding vocabulary small (e.g. `WarpMapEncodingV1`).
    - Keep chromatic aberration optional and bounded by a small max.
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs` (`EffectStep::BackdropWarpV2`, `BackdropWarpV2`, `BackdropWarpFieldV2`)
    - `crates/fret-core/src/scene/fingerprint.rs` (`mix_scene_op` for `EffectStep::BackdropWarpV2`)
    - `crates/fret-core/src/scene/validate.rs` (`EffectStep::BackdropWarpV2` validation)

## Renderer implementation (wgpu)

- [x] BWARP2-wgpu-030 Implement `BackdropWarpV2` in `fret-render-wgpu`:
      integrate into the backdrop effect pass with bounded pipeline variants and one extra sampled
      image binding (warp field).
  - Requirements:
    - strict scissor to effect bounds,
    - uniform-control-flow-safe WGSL (no divergent sampling branches),
    - deterministic degradation when unsupported / budgeted.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (`EffectStep::BackdropWarpV2` -> `RenderPlanPass::BackdropWarp`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/backdrop_warp.rs` (warp-field image pipelines + bind group layouts)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`BACKDROP_WARP_*` WGSL; WebGPU-uniformity-safe sampling)

## Conformance + portability gates

- [x] BWARP2-test-040 Add a GPU readback conformance test:
      image-driven warp modifies sampled backdrop pixels deterministically (and degrades
      deterministically when the warp field is missing).
  - Evidence anchors:
    - `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

- [x] BWARP2-perf-050 Add a steady-state perf gate + baseline for v2:
      worst-frame stability under map-driven warp + blur chain.
  - Notes:
    - Keep sample counts bounded and observe pipeline key growth via existing renderer counters.
  - Evidence anchors:
    - `tools/diag-scripts/liquid-glass-backdrop-warp-v2-steady.json`
    - `docs/workstreams/perf-baselines/policies/liquid-glass-backdrop-warp-v2-steady.v1.json`
    - `docs/workstreams/perf-baselines/liquid-glass-backdrop-warp-v2-steady.windows-rtx4090.v1.json`
    - `tools/perf/diag_liquid_glass_backdrop_warp_v2_gate.py`

## Demo validation (optional but recommended)

- [x] BWARP2-demo-060 Update `liquid_glass_demo` to exercise v2 behind a toggle:
      include a dense background stage so distortion is visually obvious, and keep the fake-glass
      baseline available.
  - Evidence anchors:
    - `apps/fret-examples/src/liquid_glass_demo.rs`

- [ ] BWARP2-demo-070 Improve “human-visible” demo quality (no contract changes):
      reduce UI chrome occlusion (HUD/inspector), add a more diagnostic backdrop stage (high-frequency
      shapes + text), and enable bounded dither by default to avoid obvious 8-bit banding in blur/adjust chains.
  - Notes:
    - Keep scripted perf/conformance anchors stable (`test_id`), even if the HUD becomes hideable.
    - Treat `BackdropWarpV1` as a procedural baseline; v2 is the recommended “real” authoring surface.
