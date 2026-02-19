Status: Landed (wgpu default renderer; conformance + perf baseline recorded)

This workstream defines and lands a **bounded, portable “backdrop warp v2”** effect step that
enables *asset-driven* liquid-glass style visuals:

- refraction-like background displacement driven by a displacement/normal-map image,
- optional chromatic aberration,
- deterministic degradation on wasm/mobile,
- without opening an unbounded “user-provided shader” contract.

This is a renderer/mechanism surface. Recipes (token sets, assets, hover intents, animation
policies, etc.) remain policy in ecosystem crates.

## Why this exists (v2 motivation)

`BackdropWarpV1` proves a portable, bounded **procedural** displacement surface. In practice,
design systems often require a more “organic” lens: the displacement field is authored as an
asset (usually a normal map or displacement map) and applied to a lens shape.

Typical “real liquid glass” implementations (e.g. shader demos) rely on:

- sampling a *backdrop* texture,
- sampling a *warp field* texture (normal/displacement),
- displacing the backdrop UVs,
- optionally adding chromatic aberration and noise,
- and compositing children on top.

Without a bounded contract surface, ecosystem authors must approximate with multiple passes or
special-case postprocess pipelines, which:

- breaks batching and increases intermediate churn,
- is difficult to keep portable to WebGPU/WGSL uniformity constraints,
- and is hard to lock down with conformance + perf gates.

## Non-goals (v2)

- No general “custom WGSL fragment shader” plugin surface.
- No open-ended shader graph / texture graph system.
- No promise of a 1:1 match to any specific platform aesthetic (e.g. iOS).
  - The mechanism surface provides the *primitives*; ecosystem provides the recipes.

## Proposed contract surface (v2)

Add a new effect step variant to `fret-core::scene::EffectStep`:

- `EffectStep::BackdropWarpV2(BackdropWarpV2)`

Where `BackdropWarpV2` extends v1 with a **bounded warp field source**:

- `field: BackdropWarpFieldV2`
  - `Procedural` (uses `base` as the portable fallback field)
  - `ImageDisplacementMap { image: ImageId, uv: UvRect, sampling: ImageSamplingHint, encoding: WarpMapEncodingV1 }`

Notes:

- `ImageId` is already the portable image handle used by other paint/mask surfaces.
- `encoding` is a small enum that defines how to interpret sampled RGBA into a displacement vector.
- The maximum displacement (in px) remains bounded by the contract (clamped).

The step is only meaningful under `EffectMode::Backdrop` (sampling already-rendered backdrop).
Under `FilterContent`, renderers deterministically degrade by skipping the step (no-op).

## Semantics (v2)

Given a pixel in the effect bounds:

1. Compute `uv_local` (effect-local UV, derived from pixel position + `uv` mapping).
2. Evaluate a displacement vector `d_px`:
   - Procedural: same as v1.
   - Image map:
     - sample the warp map at `uv_local` (bounded, clamped),
     - decode into a vector in `[-1, 1]` (or other explicit decoding),
     - scale by `strength_px` (clamped by contract).
3. Convert `d_px` into backdrop UV units (`d_uv`) and sample backdrop at `uv_backdrop + d_uv`.
4. If chromatic aberration is enabled, sample R/G/B from bounded additional offsets.
5. Composite children on top.

Deterministic degradation rules (ordered):

1. If `EffectMode::FilterContent`: skip the warp step (no-op) and preserve the rest of the chain.
2. If the image warp field is unavailable (missing image, unsupported sampling, budgets):
   - degrade to `Procedural` with the same strength clamp (or skip if `strength_px == 0`).
3. Under budget pressure:
   - disable chromatic aberration first,
   - then clamp/scale down displacement strength,
   - then skip the warp step entirely (fall back to fake-glass chain).

## WebGPU / WGSL portability notes

WGSL uniformity rules require texture sampling and derivatives to be in uniform control flow.
Implementation must be branchless (or use uniform branches) around sampling:

- always compute a displacement and always sample,
- use `clamp`/`select` instead of divergent `if` branches around sampling,
- keep sample counts bounded and pipeline variants explicit.

## Gates (required)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- A GPU readback conformance test that proves:
  - the image-driven warp changes backdrop pixels deterministically,
  - degradation is deterministic when the warp map is missing/unsupported,
  - chromatic aberration (if enabled) affects channel separation in a controlled way.
- A steady-state perf gate with a checked-in baseline.

## Tracking

- TODOs: `docs/workstreams/renderer-effect-backdrop-warp-v2-todo.md`
- Milestones: `docs/workstreams/renderer-effect-backdrop-warp-v2-milestones.md`

Related ADR:

- `docs/adr/0285-backdrop-warp-effect-step-v2-texture-field.md`

Related (v1):

- Workstream: `docs/workstreams/renderer-effect-backdrop-warp-v1.md`
- ADR: `docs/adr/0284-backdrop-warp-effect-step-v1.md`

## Demo UX notes (practical)

`apps/fret-examples/src/liquid_glass_demo.rs` is intentionally treated as an *observability surface*:

- Keep the toggle `test_id`s stable so `tools/diag-scripts/liquid-glass-backdrop-warp-v2-steady.json` remains robust.
- Prefer a small “stage HUD” (always visible) over a large centered panel that occludes the backdrop.
- Default the inspector to off; scripted perf baselines should not depend on inspector layout.
- Keep the default lens visibility stable (fake + v1 visible, v2 hidden) so the v2 perf script can
  deterministically toggle into the “v2 only” steady state.
